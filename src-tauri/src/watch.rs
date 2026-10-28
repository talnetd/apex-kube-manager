use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use kube::Api;
use kube::runtime::watcher::{self, Event};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::batch::v1::Job;
use k8s_openapi::api::core::v1::{Event as K8sEvent, Node, Pod};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::error::Result;
use crate::kubernetes::{self, ClusterEventInfo, DeploymentInfo, JobInfo, NodeInfo, PodInfo};
use crate::triage::{self, Verdict};

/// How often a watch re-evaluates time-window verdicts (Pending > 5 min,
/// OOMKilled within 1h, ...) for objects whose spec/status didn't change.
const VERDICT_RECHECK: std::time::Duration = std::time::Duration::from_secs(30);

/// Objects whose verdict can change with time alone, so a watch re-evaluates
/// them on a tick and emits only those whose verdict changed. Healthy and
/// neutral verdicts never worsen without an API change (which arrives as a
/// watch event), so they aren't kept: the set stays small on big clusters.
struct Recheck<T> {
    items: HashMap<String, (T, Verdict)>,
}

impl<T: Clone> Recheck<T> {
    fn new() -> Self {
        Self { items: HashMap::new() }
    }

    fn observe(&mut self, key: String, obj: &T, verdict: &Verdict) {
        if verdict.badness() < 5 {
            self.items.insert(key, (obj.clone(), verdict.clone()));
        } else {
            self.items.remove(&key);
        }
    }

    fn forget(&mut self, key: &str) {
        self.items.remove(key);
    }

    /// Relist: the watcher re-sends every live object, so start over (this
    /// also drops objects deleted while the stream was down).
    fn clear(&mut self) {
        self.items.clear();
    }

    /// Re-evaluate everything kept; return the objects whose verdict changed.
    fn due(
        &mut self,
        now: k8s_openapi::jiff::Timestamp,
        rule: impl Fn(&T, k8s_openapi::jiff::Timestamp) -> Verdict,
    ) -> Vec<(T, Verdict)> {
        let mut changed = Vec::new();
        self.items.retain(|_, (obj, old)| {
            let new = rule(obj, now);
            if new == *old {
                return true;
            }
            changed.push((obj.clone(), new.clone()));
            *old = new;
            old.badness() < 5
        });
        changed
    }
}

fn object_key(meta: &k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta) -> String {
    format!(
        "{}/{}",
        meta.namespace.as_deref().unwrap_or(""),
        meta.name.as_deref().unwrap_or("")
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodWatchEvent {
    pub event_type: String,
    pub pod: PodInfo,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentWatchEvent {
    pub event_type: String,
    pub deployment: DeploymentInfo,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobWatchEvent {
    pub event_type: String,
    pub job: JobInfo,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeWatchEvent {
    pub event_type: String,
    pub node: NodeInfo,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterEventWatchEvent {
    pub event_type: String,
    pub event: ClusterEventInfo,
    pub timestamp: String,
}

struct WatchHandle {
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    #[allow(dead_code)]
    task: JoinHandle<()>,
}

pub struct WatchManager {
    watchers: Arc<RwLock<HashMap<String, WatchHandle>>>,
}

impl WatchManager {
    pub fn new() -> Self {
        Self {
            watchers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_pod_watch(
        &self,
        app: AppHandle,
        namespace: Option<String>,
    ) -> Result<String> {
        let watch_id = Uuid::new_v4().to_string();
        let client = kubernetes::create_client().await?;

        let pods: Api<Pod> = match &namespace {
            Some(ns) => Api::namespaced(client, ns),
            None => Api::all(client),
        };

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();

        let watch_id_clone = watch_id.clone();
        let task = tokio::spawn(async move {
            let watcher_stream = watcher::watcher(pods, watcher::Config::default());
            futures::pin_mut!(watcher_stream);
            let event_name = format!("pod-watch-{}", watch_id_clone);
            let emit = |event_type: &str, pod: PodInfo| {
                let watch_event = PodWatchEvent {
                    event_type: event_type.to_string(),
                    pod,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                if let Err(e) = app.emit(&event_name, &watch_event) {
                    tracing::error!("Failed to emit pod watch event: {}", e);
                }
            };
            // Time-window verdicts age without any API change; re-check them
            // here so the list and Pulse agree (one source of truth: Rust).
            let mut recheck: Recheck<Pod> = Recheck::new();
            let mut tick = tokio::time::interval(VERDICT_RECHECK);
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        tracing::info!("Pod watch {} received shutdown signal", watch_id_clone);
                        break;
                    }
                    _ = tick.tick() => {
                        let now = k8s_openapi::jiff::Timestamp::now();
                        for (pod, verdict) in recheck.due(now, triage::pod_verdict) {
                            if let Some(mut info) = kubernetes::pod_to_info(&pod) {
                                info.verdict = verdict;
                                emit("applied", info);
                            }
                        }
                    }
                    event = watcher_stream.try_next() => {
                        match event {
                            Ok(Some(Event::Apply(pod))) | Ok(Some(Event::InitApply(pod))) => {
                                if let Some(info) = kubernetes::pod_to_info(&pod) {
                                    recheck.observe(object_key(&pod.metadata), &pod, &info.verdict);
                                    emit("applied", info);
                                }
                            }
                            Ok(Some(Event::Delete(pod))) => {
                                recheck.forget(&object_key(&pod.metadata));
                                if let Some(info) = kubernetes::pod_to_info(&pod) {
                                    emit("deleted", info);
                                }
                            }
                            Ok(Some(Event::Init)) => {
                                recheck.clear();
                                tracing::debug!("Pod watch {} initialized", watch_id_clone);
                            }
                            Ok(Some(Event::InitDone)) => {
                                tracing::debug!("Pod watch {} initial sync done", watch_id_clone);
                            }
                            Ok(None) => {
                                tracing::info!("Pod watch {} stream ended", watch_id_clone);
                                break;
                            }
                            Err(e) => {
                                tracing::warn!("Pod watch {} error (will retry): {}", watch_id_clone, e);
                                // watcher auto-recovers, continue the loop
                            }
                        }
                    }
                }
            }
        });

        self.watchers.write().await.insert(watch_id.clone(), WatchHandle {
            shutdown_tx,
            task,
        });

        tracing::info!("Started pod watch {} for namespace {:?}", watch_id, namespace);
        Ok(watch_id)
    }

    pub async fn start_deployment_watch(
        &self,
        app: AppHandle,
        namespace: Option<String>,
    ) -> Result<String> {
        let watch_id = Uuid::new_v4().to_string();
        let client = kubernetes::create_client().await?;

        let deployments: Api<Deployment> = match &namespace {
            Some(ns) => Api::namespaced(client, ns),
            None => Api::all(client),
        };

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();

        let watch_id_clone = watch_id.clone();
        let task = tokio::spawn(async move {
            let watcher_stream = watcher::watcher(deployments, watcher::Config::default());
            futures::pin_mut!(watcher_stream);
            let event_name = format!("deployment-watch-{}", watch_id_clone);
            let emit = |event_type: &str, deployment: DeploymentInfo| {
                let watch_event = DeploymentWatchEvent {
                    event_type: event_type.to_string(),
                    deployment,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                let _ = app.emit(&event_name, &watch_event);
            };
            // Same 30s verdict re-check as the pod watch (grace windows).
            let mut recheck: Recheck<Deployment> = Recheck::new();
            let mut tick = tokio::time::interval(VERDICT_RECHECK);
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        tracing::info!("Deployment watch {} received shutdown signal", watch_id_clone);
                        break;
                    }
                    _ = tick.tick() => {
                        let now = k8s_openapi::jiff::Timestamp::now();
                        for (deploy, verdict) in recheck.due(now, triage::deployment_verdict) {
                            if let Some(mut info) = kubernetes::deployment_to_info(&deploy) {
                                info.verdict = verdict;
                                emit("applied", info);
                            }
                        }
                    }
                    event = watcher_stream.try_next() => {
                        match event {
                            Ok(Some(Event::Apply(deploy))) | Ok(Some(Event::InitApply(deploy))) => {
                                if let Some(info) = kubernetes::deployment_to_info(&deploy) {
                                    recheck.observe(object_key(&deploy.metadata), &deploy, &info.verdict);
                                    emit("applied", info);
                                }
                            }
                            Ok(Some(Event::Delete(deploy))) => {
                                recheck.forget(&object_key(&deploy.metadata));
                                if let Some(info) = kubernetes::deployment_to_info(&deploy) {
                                    emit("deleted", info);
                                }
                            }
                            Ok(Some(Event::Init)) => recheck.clear(),
                            Ok(Some(Event::InitDone)) => {}
                            Ok(None) => break,
                            Err(e) => {
                                tracing::warn!("Deployment watch {} error (will retry): {}", watch_id_clone, e);
                            }
                        }
                    }
                }
            }
        });

        self.watchers.write().await.insert(watch_id.clone(), WatchHandle { shutdown_tx, task });
        tracing::info!("Started deployment watch {} for namespace {:?}", watch_id, namespace);
        Ok(watch_id)
    }

    pub async fn start_job_watch(
        &self,
        app: AppHandle,
        namespace: Option<String>,
    ) -> Result<String> {
        let watch_id = Uuid::new_v4().to_string();
        let client = kubernetes::create_client().await?;

        let jobs: Api<Job> = match &namespace {
            Some(ns) => Api::namespaced(client, ns),
            None => Api::all(client),
        };

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();

        let watch_id_clone = watch_id.clone();
        let task = tokio::spawn(async move {
            let watcher_stream = watcher::watcher(jobs, watcher::Config::default());
            futures::pin_mut!(watcher_stream);

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        tracing::info!("Job watch {} received shutdown signal", watch_id_clone);
                        break;
                    }
                    event = watcher_stream.try_next() => {
                        match event {
                            Ok(Some(Event::Apply(job))) | Ok(Some(Event::InitApply(job))) => {
                                if let Some(info) = kubernetes::job_to_info(&job) {
                                    let watch_event = JobWatchEvent {
                                        event_type: "applied".to_string(),
                                        job: info,
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                    };
                                    let event_name = format!("job-watch-{}", watch_id_clone);
                                    let _ = app.emit(&event_name, &watch_event);
                                }
                            }
                            Ok(Some(Event::Delete(job))) => {
                                if let Some(info) = kubernetes::job_to_info(&job) {
                                    let watch_event = JobWatchEvent {
                                        event_type: "deleted".to_string(),
                                        job: info,
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                    };
                                    let event_name = format!("job-watch-{}", watch_id_clone);
                                    let _ = app.emit(&event_name, &watch_event);
                                }
                            }
                            Ok(Some(Event::Init)) | Ok(Some(Event::InitDone)) => {}
                            Ok(None) => break,
                            Err(e) => {
                                tracing::warn!("Job watch {} error (will retry): {}", watch_id_clone, e);
                            }
                        }
                    }
                }
            }
        });

        self.watchers.write().await.insert(watch_id.clone(), WatchHandle { shutdown_tx, task });
        tracing::info!("Started job watch {} for namespace {:?}", watch_id, namespace);
        Ok(watch_id)
    }

    pub async fn start_node_watch(
        &self,
        app: AppHandle,
    ) -> Result<String> {
        let watch_id = Uuid::new_v4().to_string();
        let client = kubernetes::create_client().await?;

        // Nodes are cluster-scoped
        let nodes: Api<Node> = Api::all(client);

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();

        let watch_id_clone = watch_id.clone();
        let task = tokio::spawn(async move {
            let watcher_stream = watcher::watcher(nodes, watcher::Config::default());
            futures::pin_mut!(watcher_stream);

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        tracing::info!("Node watch {} received shutdown signal", watch_id_clone);
                        break;
                    }
                    event = watcher_stream.try_next() => {
                        match event {
                            Ok(Some(Event::Apply(node))) | Ok(Some(Event::InitApply(node))) => {
                                if let Some(info) = kubernetes::node_to_info(&node) {
                                    let watch_event = NodeWatchEvent {
                                        event_type: "applied".to_string(),
                                        node: info,
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                    };
                                    let event_name = format!("node-watch-{}", watch_id_clone);
                                    let _ = app.emit(&event_name, &watch_event);
                                }
                            }
                            Ok(Some(Event::Delete(node))) => {
                                if let Some(info) = kubernetes::node_to_info(&node) {
                                    let watch_event = NodeWatchEvent {
                                        event_type: "deleted".to_string(),
                                        node: info,
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                    };
                                    let event_name = format!("node-watch-{}", watch_id_clone);
                                    let _ = app.emit(&event_name, &watch_event);
                                }
                            }
                            Ok(Some(Event::Init)) | Ok(Some(Event::InitDone)) => {}
                            Ok(None) => break,
                            Err(e) => {
                                tracing::warn!("Node watch {} error (will retry): {}", watch_id_clone, e);
                            }
                        }
                    }
                }
            }
        });

        self.watchers.write().await.insert(watch_id.clone(), WatchHandle { shutdown_tx, task });
        tracing::info!("Started node watch {}", watch_id);
        Ok(watch_id)
    }

    pub async fn start_event_watch(
        &self,
        app: AppHandle,
        namespace: Option<String>,
    ) -> Result<String> {
        let watch_id = Uuid::new_v4().to_string();
        let client = kubernetes::create_client().await?;

        let events: Api<K8sEvent> = match &namespace {
            Some(ns) => Api::namespaced(client, ns),
            None => Api::all(client),
        };

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();

        let watch_id_clone = watch_id.clone();
        let task = tokio::spawn(async move {
            let watcher_stream = watcher::watcher(events, watcher::Config::default());
            futures::pin_mut!(watcher_stream);

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        tracing::info!("Event watch {} received shutdown signal", watch_id_clone);
                        break;
                    }
                    event = watcher_stream.try_next() => {
                        match event {
                            Ok(Some(Event::Apply(k8s_event))) | Ok(Some(Event::InitApply(k8s_event))) => {
                                let info = kubernetes::event_to_info(&k8s_event);
                                let watch_event = ClusterEventWatchEvent {
                                    event_type: "applied".to_string(),
                                    event: info,
                                    timestamp: chrono::Utc::now().to_rfc3339(),
                                };
                                let event_name = format!("cluster-event-watch-{}", watch_id_clone);
                                let _ = app.emit(&event_name, &watch_event);
                            }
                            Ok(Some(Event::Delete(k8s_event))) => {
                                let info = kubernetes::event_to_info(&k8s_event);
                                let watch_event = ClusterEventWatchEvent {
                                    event_type: "deleted".to_string(),
                                    event: info,
                                    timestamp: chrono::Utc::now().to_rfc3339(),
                                };
                                let event_name = format!("cluster-event-watch-{}", watch_id_clone);
                                let _ = app.emit(&event_name, &watch_event);
                            }
                            Ok(Some(Event::Init)) | Ok(Some(Event::InitDone)) => {}
                            Ok(None) => break,
                            Err(e) => {
                                tracing::warn!("Event watch {} error (will retry): {}", watch_id_clone, e);
                            }
                        }
                    }
                }
            }
        });

        self.watchers.write().await.insert(watch_id.clone(), WatchHandle { shutdown_tx, task });
        tracing::info!("Started event watch {} for namespace {:?}", watch_id, namespace);
        Ok(watch_id)
    }

    pub async fn stop_watch(&self, watch_id: &str) -> Result<()> {
        let mut watchers = self.watchers.write().await;
        if let Some(handle) = watchers.remove(watch_id) {
            let _ = handle.shutdown_tx.send(());
            tracing::info!("Stopped watch {}", watch_id);
        }
        Ok(())
    }

    pub async fn stop_all(&self) {
        let mut watchers = self.watchers.write().await;
        for (id, handle) in watchers.drain() {
            let _ = handle.shutdown_tx.send(());
            tracing::info!("Stopped watch {}", id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use k8s_openapi::jiff::Timestamp;
    use std::time::Duration;

    fn pending_pod(created: &str) -> Pod {
        serde_json::from_value(serde_json::json!({
            "apiVersion": "v1", "kind": "Pod",
            "metadata": { "name": "stuck", "namespace": "payments", "creationTimestamp": created },
            "spec": { "containers": [{ "name": "app", "image": "x" }] },
            "status": { "phase": "Pending" }
        }))
        .unwrap()
    }

    /// Review finding 1: a Pending pod gets no further watch events, but its
    /// verdict must still move from progressing to warn after 5 minutes.
    #[test]
    fn recheck_emits_only_verdicts_that_aged() {
        let created: Timestamp = "2026-10-28T12:00:00Z".parse().unwrap();
        let pod = pending_pod("2026-10-28T12:00:00Z");
        let mut r: Recheck<Pod> = Recheck::new();
        let v0 = triage::pod_verdict(&pod, created);
        assert_eq!(v0.reason, "Pending");
        assert_eq!(v0.tone, Some(triage::Tone::Progressing));
        r.observe(object_key(&pod.metadata), &pod, &v0);

        // 2 minutes in: nothing changed, nothing emitted.
        assert!(r.due(created + Duration::from_secs(120), triage::pod_verdict).is_empty());

        // 6 minutes in: one emit, now a warning.
        let t6 = created + Duration::from_secs(360);
        let changed = r.due(t6, triage::pod_verdict);
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].1.status, triage::Status::Warn);
        assert_eq!(changed[0].1.rule_id, "TRI-POD-008");

        // Same verdict on the next tick: no repeat emit.
        assert!(r.due(t6 + Duration::from_secs(30), triage::pod_verdict).is_empty());
    }

    #[test]
    fn healthy_objects_are_not_kept_and_deletes_are_forgotten() {
        let pod = pending_pod("2026-10-28T12:00:00Z");
        let mut r: Recheck<Pod> = Recheck::new();
        let key = object_key(&pod.metadata);
        r.observe(key.clone(), &pod, &Verdict::ok("TRI-POD-000", "Running"));
        assert!(r.items.is_empty());
        r.observe(key.clone(), &pod, &Verdict::progressing("TRI-POD-011", "Pending"));
        assert_eq!(r.items.len(), 1);
        r.forget(&key);
        assert!(r.items.is_empty());
    }

    /// Review finding 2: a year-old StatefulSet scaled up is "Scaling" on
    /// first sight, not FAIL since creation; the clock survives re-evaluation.
    #[test]
    fn tracker_starts_the_clock_at_first_unsettled_sight() {
        let sts: k8s_openapi::api::apps::v1::StatefulSet = serde_json::from_value(serde_json::json!({
            "apiVersion": "apps/v1", "kind": "StatefulSet",
            "metadata": { "name": "kafka", "namespace": "data", "uid": "u1",
                          "creationTimestamp": "2025-10-28T12:00:00Z" },
            "spec": { "replicas": 5, "serviceName": "kafka",
                      "selector": { "matchLabels": { "app": "kafka" } },
                      "template": { "spec": { "containers": [{ "name": "k", "image": "x" }] } } },
            "status": { "replicas": 3, "readyReplicas": 3, "updatedReplicas": 3 }
        }))
        .unwrap();
        let t0: Timestamp = "2026-10-28T12:00:00Z".parse().unwrap();
        let mut tr = triage::UnsettledTracker::default();
        let s = &sts;
        let rule = |now: Timestamp| move |seen| triage::statefulset_verdict(s, now, seen);

        let v = tr.eval("u1", t0, rule(t0));
        assert_eq!((v.reason.as_str(), v.tone), ("Scaling", Some(triage::Tone::Progressing)));
        // Polled every 10s (a gap > 60s would start a fresh clock).
        for s in (10..11 * 60).step_by(10) {
            let t = t0 + Duration::from_secs(s);
            tr.eval("u1", t, rule(t));
        }
        let t11 = t0 + Duration::from_secs(11 * 60);
        let v = tr.eval("u1", t11, rule(t11));
        assert_eq!((v.reason.as_str(), v.status), ("3/5 ready", triage::Status::Warn));
        assert_eq!(v.since.as_deref(), Some("2026-10-28T12:00:00Z"));
    }

    /// Re-review: a clock from before a gap in sightings (view closed, app
    /// asleep) or from before a context switch must not be reused.
    #[test]
    fn tracker_restarts_the_clock_after_a_gap_or_clear() {
        let t0: Timestamp = "2026-10-28T12:00:00Z".parse().unwrap();
        let unsettled = |seen: Option<Timestamp>| {
            triage::Verdict::progressing("TRI-WL-005", "Scaling").since(seen.map(|t| t.to_string()))
        };
        let mut tr = triage::UnsettledTracker::default();

        // Seen every 10s for a minute: the clock holds at t0.
        for s in (0..=60).step_by(10) {
            let v = tr.eval("u1", t0 + Duration::from_secs(s), unsettled);
            assert_eq!(v.since.as_deref(), Some("2026-10-28T12:00:00Z"));
        }
        // Not looked at for 2 minutes: the next sighting is fresh.
        let t3 = t0 + Duration::from_secs(60 + 120);
        let v = tr.eval("u1", t3, unsettled);
        assert_eq!(v.since.as_deref(), Some("2026-10-28T12:03:00Z"));

        // A clear (context switch) also starts over.
        tr.clear();
        let t4 = t3 + Duration::from_secs(10);
        let v = tr.eval("u1", t4, unsettled);
        assert_eq!(v.since.as_deref(), Some("2026-10-28T12:03:10Z"));
    }
}
