use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use kube::Api;
use kube::runtime::watcher::{self, Event};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::batch::v1::Job;
use k8s_openapi::api::core::v1::{Node, Pod};
use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::error::Result;
use crate::kubernetes::{self, DeploymentInfo, JobInfo, NodeInfo, PodInfo};

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

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        tracing::info!("Pod watch {} received shutdown signal", watch_id_clone);
                        break;
                    }
                    event = watcher_stream.try_next() => {
                        match event {
                            Ok(Some(Event::Apply(pod))) => {
                                if let Some(pod_info) = kubernetes::pod_to_info(&pod) {
                                    let watch_event = PodWatchEvent {
                                        event_type: "applied".to_string(),
                                        pod: pod_info,
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                    };
                                    let event_name = format!("pod-watch-{}", watch_id_clone);
                                    if let Err(e) = app.emit(&event_name, &watch_event) {
                                        tracing::error!("Failed to emit pod watch event: {}", e);
                                    }
                                }
                            }
                            Ok(Some(Event::Delete(pod))) => {
                                if let Some(pod_info) = kubernetes::pod_to_info(&pod) {
                                    let watch_event = PodWatchEvent {
                                        event_type: "deleted".to_string(),
                                        pod: pod_info,
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                    };
                                    let event_name = format!("pod-watch-{}", watch_id_clone);
                                    if let Err(e) = app.emit(&event_name, &watch_event) {
                                        tracing::error!("Failed to emit pod watch event: {}", e);
                                    }
                                }
                            }
                            Ok(Some(Event::Init)) => {
                                tracing::debug!("Pod watch {} initialized", watch_id_clone);
                            }
                            Ok(Some(Event::InitApply(pod))) => {
                                // Initial sync - emit as applied
                                if let Some(pod_info) = kubernetes::pod_to_info(&pod) {
                                    let watch_event = PodWatchEvent {
                                        event_type: "applied".to_string(),
                                        pod: pod_info,
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                    };
                                    let event_name = format!("pod-watch-{}", watch_id_clone);
                                    if let Err(e) = app.emit(&event_name, &watch_event) {
                                        tracing::error!("Failed to emit pod watch event: {}", e);
                                    }
                                }
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

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        tracing::info!("Deployment watch {} received shutdown signal", watch_id_clone);
                        break;
                    }
                    event = watcher_stream.try_next() => {
                        match event {
                            Ok(Some(Event::Apply(deploy))) | Ok(Some(Event::InitApply(deploy))) => {
                                if let Some(info) = kubernetes::deployment_to_info(&deploy) {
                                    let watch_event = DeploymentWatchEvent {
                                        event_type: "applied".to_string(),
                                        deployment: info,
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                    };
                                    let event_name = format!("deployment-watch-{}", watch_id_clone);
                                    let _ = app.emit(&event_name, &watch_event);
                                }
                            }
                            Ok(Some(Event::Delete(deploy))) => {
                                if let Some(info) = kubernetes::deployment_to_info(&deploy) {
                                    let watch_event = DeploymentWatchEvent {
                                        event_type: "deleted".to_string(),
                                        deployment: info,
                                        timestamp: chrono::Utc::now().to_rfc3339(),
                                    };
                                    let event_name = format!("deployment-watch-{}", watch_id_clone);
                                    let _ = app.emit(&event_name, &watch_event);
                                }
                            }
                            Ok(Some(Event::Init)) | Ok(Some(Event::InitDone)) => {}
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
