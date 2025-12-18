use kube::{
    api::{Api, DeleteParams, ListParams, LogParams, Patch, PatchParams},
    Client, Config,
};
use k8s_openapi::api::core::v1::{
    Namespace, Node, PersistentVolume, PersistentVolumeClaim, Pod, Secret,
    Service, ServiceAccount, ConfigMap,
};
use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet};
use k8s_openapi::api::batch::v1::{CronJob, Job};
use k8s_openapi::api::networking::v1::{Ingress, NetworkPolicy};
use k8s_openapi::api::autoscaling::v1::HorizontalPodAutoscaler;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubeContext {
    pub name: String,
    pub cluster: String,
    pub user: String,
    pub namespace: Option<String>,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub ready: String,
    pub restarts: i32,
    pub age: String,
    pub node: Option<String>,
    pub ip: Option<String>,
    pub containers: Vec<ContainerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub name: String,
    pub image: String,
    pub ready: bool,
    pub restart_count: i32,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    pub name: String,
    pub namespace: String,
    pub ready: String,
    pub up_to_date: i32,
    pub available: i32,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentDetail {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub updated_replicas: i32,
    pub available_replicas: i32,
    pub strategy: String,
    pub min_ready_seconds: i32,
    pub revision_history_limit: Option<i32>,
    pub selector: std::collections::BTreeMap<String, String>,
    pub conditions: Vec<DeploymentCondition>,
    pub container_images: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentCondition {
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_update_time: Option<String>,
    pub last_transition_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatefulSetInfo {
    pub name: String,
    pub namespace: String,
    pub ready: String,
    pub replicas: i32,
    pub age: String,
    pub service_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatefulSetDetail {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub current_replicas: i32,
    pub updated_replicas: i32,
    pub service_name: String,
    pub pod_management_policy: String,
    pub update_strategy: String,
    pub revision_history_limit: Option<i32>,
    pub selector: std::collections::BTreeMap<String, String>,
    pub conditions: Vec<StatefulSetCondition>,
    pub container_images: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatefulSetCondition {
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_transition_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatefulSetEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSetInfo {
    pub name: String,
    pub namespace: String,
    pub desired: i32,
    pub current: i32,
    pub ready: i32,
    pub up_to_date: i32,
    pub available: i32,
    pub node_selector: Option<String>,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaSetInfo {
    pub name: String,
    pub namespace: String,
    pub desired: i32,
    pub current: i32,
    pub ready: i32,
    pub age: String,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub name: String,
    pub namespace: String,
    pub completions: String,
    pub duration: Option<String>,
    pub age: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobInfo {
    pub name: String,
    pub namespace: String,
    pub schedule: String,
    pub suspend: bool,
    pub active: i32,
    pub last_schedule: Option<String>,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub namespace: String,
    pub service_type: String,
    pub cluster_ip: Option<String>,
    pub external_ip: Option<String>,
    pub ports: Vec<String>,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDetail {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub service_type: String,
    pub cluster_ip: Option<String>,
    pub cluster_ips: Vec<String>,
    pub external_ips: Vec<String>,
    pub ports: Vec<ServicePortDetail>,
    pub selector: std::collections::BTreeMap<String, String>,
    pub session_affinity: String,
    pub load_balancer_ip: Option<String>,
    pub load_balancer_ingress: Vec<String>,
    pub external_name: Option<String>,
    pub internal_traffic_policy: Option<String>,
    pub external_traffic_policy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePortDetail {
    pub name: Option<String>,
    pub protocol: String,
    pub port: i32,
    pub target_port: String,
    pub node_port: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub ip: String,
    pub port: i32,
    pub protocol: String,
    pub pod_name: Option<String>,
    pub node_name: Option<String>,
    pub ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

// Ingress Detail structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressDetail {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub ingress_class: Option<String>,
    pub rules: Vec<IngressRuleDetail>,
    pub tls: Vec<IngressTlsDetail>,
    pub default_backend: Option<IngressBackendDetail>,
    pub load_balancer_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressRuleDetail {
    pub host: Option<String>,
    pub paths: Vec<IngressPathDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressPathDetail {
    pub path: String,
    pub path_type: String,
    pub backend_service: Option<String>,
    pub backend_port: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressTlsDetail {
    pub hosts: Vec<String>,
    pub secret_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressBackendDetail {
    pub service_name: String,
    pub service_port: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

// Network resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressInfo {
    pub name: String,
    pub namespace: String,
    pub class: Option<String>,
    pub hosts: Vec<String>,
    pub address: Option<String>,
    pub ports: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicyInfo {
    pub name: String,
    pub namespace: String,
    pub pod_selector: String,
    pub policy_types: Vec<String>,
    pub age: String,
}

// Config resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMapInfo {
    pub name: String,
    pub namespace: String,
    pub data_count: i32,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretInfo {
    pub name: String,
    pub namespace: String,
    pub secret_type: String,
    pub data_count: i32,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HPAInfo {
    pub name: String,
    pub namespace: String,
    pub reference: String,
    pub targets: String,
    pub min_pods: i32,
    pub max_pods: i32,
    pub replicas: i32,
    pub age: String,
}

// Storage resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentVolumeInfo {
    pub name: String,
    pub capacity: String,
    pub access_modes: Vec<String>,
    pub reclaim_policy: String,
    pub status: String,
    pub claim: Option<String>,
    pub storage_class: Option<String>,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentVolumeClaimInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub volume: Option<String>,
    pub capacity: Option<String>,
    pub access_modes: Vec<String>,
    pub storage_class: Option<String>,
    pub age: String,
}

// Cluster resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceInfo {
    pub name: String,
    pub status: String,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub name: String,
    pub status: String,
    pub roles: Vec<String>,
    pub age: String,
    pub version: String,
    pub internal_ip: Option<String>,
    pub os_image: String,
    pub kernel: String,
    pub container_runtime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccountInfo {
    pub name: String,
    pub namespace: String,
    pub secrets: i32,
    pub age: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMetrics {
    pub total_pods: i32,
    pub running_pods: i32,
    pub pending_pods: i32,
    pub failed_pods: i32,
    pub total_deployments: i32,
    pub total_services: i32,
    pub total_namespaces: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCount {
    pub ok: i32,
    pub fail: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseMetrics {
    // Cluster info
    pub context: String,
    pub cluster: String,
    pub user: String,
    pub k8s_version: String,

    // Resource counts with OK/FAIL status
    pub pods: ResourceCount,
    pub deployments: ResourceCount,
    pub statefulsets: ResourceCount,
    pub daemonsets: ResourceCount,
    pub replicasets: ResourceCount,
    pub jobs: ResourceCount,

    // Resources without fail state (just counts)
    pub cronjobs: i32,
    pub services: i32,
    pub configmaps: i32,
    pub secrets: i32,
    pub pvs: i32,
    pub pvcs: i32,
    pub hpas: i32,
    pub ingresses: i32,
    pub network_policies: i32,
    pub service_accounts: i32,
    pub namespaces: i32,
    pub nodes: i32,

    // Resource metrics (in millicores for CPU, bytes for memory)
    pub cpu_capacity: i64,
    pub cpu_allocatable: i64,
    pub memory_capacity: i64,
    pub memory_allocatable: i64,
}

fn get_kubeconfig_path() -> PathBuf {
    std::env::var("KUBECONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_default()
                .join(".kube")
                .join("config")
        })
}

// Startup check functions
pub async fn get_kubeconfig_path_string() -> Result<String> {
    let path = get_kubeconfig_path();
    if !path.exists() {
        return Err(AppError::Custom(format!(
            "Kubeconfig not found at: {}",
            path.display()
        )));
    }
    Ok(path.display().to_string())
}

pub async fn validate_kubeconfig() -> Result<()> {
    let path = get_kubeconfig_path();
    let _kubeconfig = kube::config::Kubeconfig::read_from(&path)?;
    Ok(())
}

pub async fn get_context_names() -> Result<Vec<String>> {
    let kubeconfig = kube::config::Kubeconfig::read_from(get_kubeconfig_path())?;
    Ok(kubeconfig.contexts.iter().map(|c| c.name.clone()).collect())
}

pub async fn test_connection() -> Result<()> {
    let client = create_client().await?;
    // Try to get server version as a connection test
    let _namespaces: Api<Namespace> = Api::all(client);
    Ok(())
}

pub async fn list_contexts() -> Result<Vec<KubeContext>> {
    let kubeconfig = kube::config::Kubeconfig::read_from(get_kubeconfig_path())?;
    let current_context = kubeconfig.current_context.clone();

    let contexts: Vec<KubeContext> = kubeconfig
        .contexts
        .iter()
        .map(|ctx| {
            let context = ctx.context.as_ref();
            KubeContext {
                name: ctx.name.clone(),
                cluster: context.map(|c| c.cluster.clone()).unwrap_or_default(),
                user: context.and_then(|c| c.user.clone()).unwrap_or_default(),
                namespace: context.and_then(|c| c.namespace.clone()),
                is_current: Some(&ctx.name) == current_context.as_ref(),
            }
        })
        .collect();

    Ok(contexts)
}

pub async fn get_current_context_name() -> Result<String> {
    let kubeconfig = kube::config::Kubeconfig::read_from(get_kubeconfig_path())?;
    kubeconfig
        .current_context
        .ok_or_else(|| AppError::Custom("No current context set".into()))
}

pub async fn switch_to_context(context_name: &str) -> Result<()> {
    let config_path = get_kubeconfig_path();
    let mut kubeconfig = kube::config::Kubeconfig::read_from(&config_path)?;

    // Verify context exists
    if !kubeconfig.contexts.iter().any(|c| c.name == context_name) {
        return Err(AppError::Custom(format!(
            "Context '{}' not found",
            context_name
        )));
    }

    kubeconfig.current_context = Some(context_name.to_string());

    let yaml = serde_yaml::to_string(&kubeconfig)
        .map_err(|e| AppError::Custom(format!("Failed to serialize config: {}", e)))?;
    std::fs::write(&config_path, yaml)?;

    Ok(())
}

pub async fn create_client() -> Result<Client> {
    let config = Config::infer().await?;
    let client = Client::try_from(config)?;
    Ok(client)
}

pub async fn list_namespaces(client: &Client) -> Result<Vec<String>> {
    let namespaces: Api<Namespace> = Api::all(client.clone());
    let ns_list = namespaces.list(&ListParams::default()).await?;

    Ok(ns_list
        .items
        .iter()
        .filter_map(|ns| ns.metadata.name.clone())
        .collect())
}

pub async fn list_pods(client: &Client, namespace: Option<&str>) -> Result<Vec<PodInfo>> {
    let pods: Api<Pod> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };

    let pod_list = pods.list(&ListParams::default()).await?;

    let pod_infos: Vec<PodInfo> = pod_list
        .items
        .iter()
        .map(|pod| {
            let metadata = &pod.metadata;
            let spec = pod.spec.as_ref();
            let status = pod.status.as_ref();

            let containers: Vec<ContainerInfo> = spec
                .map(|s| {
                    s.containers
                        .iter()
                        .map(|c| {
                            let container_status = status
                                .and_then(|st| st.container_statuses.as_ref())
                                .and_then(|statuses| {
                                    statuses.iter().find(|cs| cs.name == c.name)
                                });

                            ContainerInfo {
                                name: c.name.clone(),
                                image: c.image.clone().unwrap_or_default(),
                                ready: container_status.map(|cs| cs.ready).unwrap_or(false),
                                restart_count: container_status
                                    .map(|cs| cs.restart_count)
                                    .unwrap_or(0),
                                state: get_container_state(container_status),
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            let ready_count = containers.iter().filter(|c| c.ready).count();
            let total_count = containers.len();
            let total_restarts: i32 = containers.iter().map(|c| c.restart_count).sum();

            PodInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                status: get_pod_phase(status),
                ready: format!("{}/{}", ready_count, total_count),
                restarts: total_restarts,
                age: get_age(metadata.creation_timestamp.as_ref()),
                node: spec.and_then(|s| s.node_name.clone()),
                ip: status.and_then(|s| s.pod_ip.clone()),
                containers,
            }
        })
        .collect();

    Ok(pod_infos)
}

pub async fn get_logs(client: &Client, namespace: &str, pod_name: &str, container: Option<&str>, tail_lines: Option<i64>, previous: Option<bool>) -> Result<String> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);

    let mut log_params = LogParams {
        tail_lines,
        previous: previous.unwrap_or(false),
        ..Default::default()
    };

    if let Some(c) = container {
        log_params.container = Some(c.to_string());
    }

    let logs = pods.logs(pod_name, &log_params).await?;
    Ok(logs)
}

pub async fn delete_pod_by_name(client: &Client, namespace: &str, pod_name: &str) -> Result<()> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    pods.delete(pod_name, &DeleteParams::default()).await?;
    Ok(())
}

pub async fn list_deployments(client: &Client, namespace: Option<&str>) -> Result<Vec<DeploymentInfo>> {
    let deployments: Api<Deployment> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };

    let deploy_list = deployments.list(&ListParams::default()).await?;

    let deploy_infos: Vec<DeploymentInfo> = deploy_list
        .items
        .iter()
        .map(|deploy| {
            let metadata = &deploy.metadata;
            let status = deploy.status.as_ref();

            let replicas = status.and_then(|s| s.replicas).unwrap_or(0);
            let ready = status.and_then(|s| s.ready_replicas).unwrap_or(0);
            let updated = status.and_then(|s| s.updated_replicas).unwrap_or(0);
            let available = status.and_then(|s| s.available_replicas).unwrap_or(0);

            DeploymentInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                ready: format!("{}/{}", ready, replicas),
                up_to_date: updated,
                available,
                age: get_age(metadata.creation_timestamp.as_ref()),
            }
        })
        .collect();

    Ok(deploy_infos)
}

// ============ Deployment Operations ============

pub async fn scale_deployment(client: &Client, namespace: &str, name: &str, replicas: i32) -> Result<()> {
    let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);

    let patch = serde_json::json!({
        "spec": {
            "replicas": replicas
        }
    });

    deployments.patch(name, &PatchParams::default(), &Patch::Merge(&patch)).await?;
    Ok(())
}

pub async fn restart_deployment(client: &Client, namespace: &str, name: &str) -> Result<()> {
    let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);

    // Rollout restart is done by updating the pod template annotation
    let now = chrono::Utc::now().to_rfc3339();
    let patch = serde_json::json!({
        "spec": {
            "template": {
                "metadata": {
                    "annotations": {
                        "kubectl.kubernetes.io/restartedAt": now
                    }
                }
            }
        }
    });

    deployments.patch(name, &PatchParams::default(), &Patch::Merge(&patch)).await?;
    Ok(())
}

pub async fn get_deployment_detail(client: &Client, namespace: &str, name: &str) -> Result<DeploymentDetail> {
    let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);
    let deploy = deployments.get(name).await?;

    let metadata = &deploy.metadata;
    let spec = deploy.spec.as_ref();
    let status = deploy.status.as_ref();

    // Get container images from pod template
    let container_images = spec
        .and_then(|s| s.template.spec.as_ref())
        .map(|ps| {
            ps.containers
                .iter()
                .map(|c| c.image.clone().unwrap_or_default())
                .collect()
        })
        .unwrap_or_default();

    // Get strategy
    let strategy = spec
        .and_then(|s| s.strategy.as_ref())
        .and_then(|s| s.type_.as_ref())
        .cloned()
        .unwrap_or_else(|| "RollingUpdate".to_string());

    // Get selector labels
    let selector = spec
        .and_then(|s| s.selector.match_labels.as_ref())
        .cloned()
        .unwrap_or_default();

    // Get conditions
    let conditions = status
        .and_then(|s| s.conditions.as_ref())
        .map(|conds| {
            conds
                .iter()
                .map(|c| DeploymentCondition {
                    condition_type: c.type_.clone(),
                    status: c.status.clone(),
                    reason: c.reason.clone(),
                    message: c.message.clone(),
                    last_update_time: c.last_update_time.as_ref().map(|t| t.0.to_rfc3339()),
                    last_transition_time: c.last_transition_time.as_ref().map(|t| t.0.to_rfc3339()),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(DeploymentDetail {
        name: metadata.name.clone().unwrap_or_default(),
        namespace: metadata.namespace.clone().unwrap_or_default(),
        uid: metadata.uid.clone().unwrap_or_default(),
        creation_timestamp: metadata
            .creation_timestamp
            .as_ref()
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.clone().unwrap_or_default(),
        annotations: metadata.annotations.clone().unwrap_or_default(),
        replicas: spec.and_then(|s| s.replicas).unwrap_or(0),
        ready_replicas: status.and_then(|s| s.ready_replicas).unwrap_or(0),
        updated_replicas: status.and_then(|s| s.updated_replicas).unwrap_or(0),
        available_replicas: status.and_then(|s| s.available_replicas).unwrap_or(0),
        strategy,
        min_ready_seconds: spec.and_then(|s| s.min_ready_seconds).unwrap_or(0),
        revision_history_limit: spec.and_then(|s| s.revision_history_limit),
        selector,
        conditions,
        container_images,
    })
}

pub async fn get_deployment_yaml(client: &Client, namespace: &str, name: &str) -> Result<String> {
    let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);
    let deploy = deployments.get(name).await?;

    let yaml = serde_yaml::to_string(&deploy)
        .map_err(|e| AppError::Custom(format!("Failed to serialize deployment to YAML: {}", e)))?;

    Ok(yaml)
}

pub async fn get_deployment_events(client: &Client, namespace: &str, name: &str) -> Result<Vec<DeploymentEvent>> {
    use k8s_openapi::api::core::v1::Event;

    let events: Api<Event> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().fields(&format!(
        "involvedObject.name={},involvedObject.kind=Deployment",
        name
    ));

    let event_list = events.list(&lp).await?;

    let deployment_events: Vec<DeploymentEvent> = event_list
        .items
        .iter()
        .map(|e| {
            DeploymentEvent {
                event_type: e.type_.clone().unwrap_or_else(|| "Normal".to_string()),
                reason: e.reason.clone().unwrap_or_default(),
                message: e.message.clone().unwrap_or_default(),
                count: e.count.unwrap_or(1),
                first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
                last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
                source: e
                    .source
                    .as_ref()
                    .and_then(|s| s.component.clone())
                    .unwrap_or_default(),
            }
        })
        .collect();

    Ok(deployment_events)
}

pub async fn get_deployment_pods(client: &Client, namespace: &str, name: &str) -> Result<Vec<PodInfo>> {
    // First get the deployment to find its selector
    let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);
    let deploy = deployments.get(name).await?;

    // Get selector labels
    let selector = deploy
        .spec
        .as_ref()
        .and_then(|s| s.selector.match_labels.as_ref())
        .map(|labels| {
            labels
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();

    // List pods with matching labels
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().labels(&selector);
    let pod_list = pods.list(&lp).await?;

    let pod_infos: Vec<PodInfo> = pod_list
        .items
        .iter()
        .map(|pod| {
            let metadata = &pod.metadata;
            let status = pod.status.as_ref();
            let spec = pod.spec.as_ref();

            let phase = status
                .and_then(|s| s.phase.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            let container_statuses = status.and_then(|s| s.container_statuses.clone());

            let (ready_count, total_count, restarts) = container_statuses
                .as_ref()
                .map(|statuses| {
                    let total = statuses.len();
                    let ready = statuses.iter().filter(|s| s.ready).count();
                    let restarts: i32 = statuses.iter().map(|s| s.restart_count).sum();
                    (ready, total, restarts)
                })
                .unwrap_or((0, 0, 0));

            let containers: Vec<ContainerInfo> = container_statuses
                .map(|statuses| {
                    statuses
                        .iter()
                        .map(|cs| {
                            let state = if cs.state.as_ref().and_then(|s| s.running.as_ref()).is_some() {
                                "Running".to_string()
                            } else if let Some(waiting) = cs.state.as_ref().and_then(|s| s.waiting.as_ref()) {
                                waiting.reason.clone().unwrap_or_else(|| "Waiting".to_string())
                            } else if let Some(terminated) = cs.state.as_ref().and_then(|s| s.terminated.as_ref()) {
                                terminated.reason.clone().unwrap_or_else(|| "Terminated".to_string())
                            } else {
                                "Unknown".to_string()
                            };

                            ContainerInfo {
                                name: cs.name.clone(),
                                image: cs.image.clone(),
                                ready: cs.ready,
                                restart_count: cs.restart_count,
                                state,
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            PodInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                status: phase,
                ready: format!("{}/{}", ready_count, total_count),
                restarts,
                age: get_age(metadata.creation_timestamp.as_ref()),
                node: spec.and_then(|s| s.node_name.clone()),
                ip: status.and_then(|s| s.pod_ip.clone()),
                containers,
            }
        })
        .collect();

    Ok(pod_infos)
}

pub async fn list_statefulsets(client: &Client, namespace: Option<&str>) -> Result<Vec<StatefulSetInfo>> {
    let statefulsets: Api<StatefulSet> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };

    let sts_list = statefulsets.list(&ListParams::default()).await?;

    let sts_infos: Vec<StatefulSetInfo> = sts_list
        .items
        .iter()
        .map(|sts| {
            let metadata = &sts.metadata;
            let spec = sts.spec.as_ref();
            let status = sts.status.as_ref();

            let replicas = spec.map(|s| s.replicas.unwrap_or(0)).unwrap_or(0);
            let ready = status.map(|s| s.ready_replicas.unwrap_or(0)).unwrap_or(0);
            let service_name = spec.map(|s| s.service_name.clone());

            StatefulSetInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                ready: format!("{}/{}", ready, replicas),
                replicas,
                age: get_age(metadata.creation_timestamp.as_ref()),
                service_name,
            }
        })
        .collect();

    Ok(sts_infos)
}

// ============ StatefulSet Operations ============

pub async fn scale_statefulset(client: &Client, namespace: &str, name: &str, replicas: i32) -> Result<()> {
    let statefulsets: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);

    let patch = serde_json::json!({
        "spec": {
            "replicas": replicas
        }
    });

    statefulsets.patch(name, &PatchParams::default(), &Patch::Merge(&patch)).await?;
    Ok(())
}

pub async fn restart_statefulset(client: &Client, namespace: &str, name: &str) -> Result<()> {
    let statefulsets: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);

    // Rollout restart is done by updating the pod template annotation
    let now = chrono::Utc::now().to_rfc3339();
    let patch = serde_json::json!({
        "spec": {
            "template": {
                "metadata": {
                    "annotations": {
                        "kubectl.kubernetes.io/restartedAt": now
                    }
                }
            }
        }
    });

    statefulsets.patch(name, &PatchParams::default(), &Patch::Merge(&patch)).await?;
    Ok(())
}

pub async fn get_statefulset_detail(client: &Client, namespace: &str, name: &str) -> Result<StatefulSetDetail> {
    let statefulsets: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
    let sts = statefulsets.get(name).await?;

    let metadata = &sts.metadata;
    let spec = sts.spec.as_ref();
    let status = sts.status.as_ref();

    // Get container images from pod template
    let container_images = spec
        .and_then(|s| s.template.spec.as_ref())
        .map(|ps| {
            ps.containers
                .iter()
                .map(|c| c.image.clone().unwrap_or_default())
                .collect()
        })
        .unwrap_or_default();

    // Get update strategy
    let update_strategy = spec
        .and_then(|s| s.update_strategy.as_ref())
        .and_then(|s| s.type_.as_ref())
        .cloned()
        .unwrap_or_else(|| "RollingUpdate".to_string());

    // Get pod management policy
    let pod_management_policy = spec
        .and_then(|s| s.pod_management_policy.clone())
        .unwrap_or_else(|| "OrderedReady".to_string());

    // Get selector labels
    let selector = spec
        .and_then(|s| s.selector.match_labels.as_ref())
        .cloned()
        .unwrap_or_default();

    // Get conditions
    let conditions = status
        .and_then(|s| s.conditions.as_ref())
        .map(|conds| {
            conds
                .iter()
                .map(|c| StatefulSetCondition {
                    condition_type: c.type_.clone(),
                    status: c.status.clone(),
                    reason: c.reason.clone(),
                    message: c.message.clone(),
                    last_transition_time: c.last_transition_time.as_ref().map(|t| t.0.to_rfc3339()),
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(StatefulSetDetail {
        name: metadata.name.clone().unwrap_or_default(),
        namespace: metadata.namespace.clone().unwrap_or_default(),
        uid: metadata.uid.clone().unwrap_or_default(),
        creation_timestamp: metadata
            .creation_timestamp
            .as_ref()
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.clone().unwrap_or_default(),
        annotations: metadata.annotations.clone().unwrap_or_default(),
        replicas: spec.and_then(|s| s.replicas).unwrap_or(0),
        ready_replicas: status.and_then(|s| s.ready_replicas).unwrap_or(0),
        current_replicas: status.and_then(|s| s.current_replicas).unwrap_or(0),
        updated_replicas: status.and_then(|s| s.updated_replicas).unwrap_or(0),
        service_name: spec.map(|s| s.service_name.clone()).unwrap_or_default(),
        pod_management_policy,
        update_strategy,
        revision_history_limit: spec.and_then(|s| s.revision_history_limit),
        selector,
        conditions,
        container_images,
    })
}

pub async fn get_statefulset_yaml(client: &Client, namespace: &str, name: &str) -> Result<String> {
    let statefulsets: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
    let sts = statefulsets.get(name).await?;
    let yaml = serde_yaml::to_string(&sts)
        .map_err(|e| AppError::Custom(format!("Failed to serialize statefulset to YAML: {}", e)))?;
    Ok(yaml)
}

pub async fn get_statefulset_events(client: &Client, namespace: &str, name: &str) -> Result<Vec<StatefulSetEvent>> {
    use k8s_openapi::api::core::v1::Event;

    let events: Api<Event> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default()
        .fields(&format!("involvedObject.name={},involvedObject.kind=StatefulSet", name));
    let event_list = events.list(&lp).await?;

    let sts_events: Vec<StatefulSetEvent> = event_list
        .items
        .iter()
        .map(|e| StatefulSetEvent {
            event_type: e.type_.clone().unwrap_or_default(),
            reason: e.reason.clone().unwrap_or_default(),
            message: e.message.clone().unwrap_or_default(),
            count: e.count.unwrap_or(1),
            first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
            last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
            source: e
                .source
                .as_ref()
                .map(|s| s.component.clone().unwrap_or_default())
                .unwrap_or_default(),
        })
        .collect();

    Ok(sts_events)
}

pub async fn get_statefulset_pods(client: &Client, namespace: &str, name: &str) -> Result<Vec<PodInfo>> {
    // First get the statefulset to find its selector
    let statefulsets: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
    let sts = statefulsets.get(name).await?;

    // Get selector labels
    let selector = sts
        .spec
        .as_ref()
        .and_then(|s| s.selector.match_labels.as_ref())
        .map(|labels| {
            labels
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();

    // List pods with matching labels
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().labels(&selector);
    let pod_list = pods.list(&lp).await?;

    let pod_infos: Vec<PodInfo> = pod_list
        .items
        .iter()
        .map(|pod| {
            let metadata = &pod.metadata;
            let status = pod.status.as_ref();
            let spec = pod.spec.as_ref();

            let phase = status
                .and_then(|s| s.phase.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            let container_statuses = status.and_then(|s| s.container_statuses.clone());

            let (ready_count, total_count, restarts) = container_statuses
                .as_ref()
                .map(|statuses| {
                    let total = statuses.len();
                    let ready = statuses.iter().filter(|s| s.ready).count();
                    let restarts: i32 = statuses.iter().map(|s| s.restart_count).sum();
                    (ready, total, restarts)
                })
                .unwrap_or((0, 0, 0));

            let containers: Vec<ContainerInfo> = container_statuses
                .map(|statuses| {
                    statuses
                        .iter()
                        .map(|cs| {
                            let state = if cs.state.as_ref().and_then(|s| s.running.as_ref()).is_some() {
                                "Running".to_string()
                            } else if let Some(waiting) = cs.state.as_ref().and_then(|s| s.waiting.as_ref()) {
                                waiting.reason.clone().unwrap_or_else(|| "Waiting".to_string())
                            } else if let Some(terminated) = cs.state.as_ref().and_then(|s| s.terminated.as_ref()) {
                                terminated.reason.clone().unwrap_or_else(|| "Terminated".to_string())
                            } else {
                                "Unknown".to_string()
                            };

                            ContainerInfo {
                                name: cs.name.clone(),
                                image: cs.image.clone(),
                                ready: cs.ready,
                                restart_count: cs.restart_count,
                                state,
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            PodInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                status: phase,
                ready: format!("{}/{}", ready_count, total_count),
                restarts,
                age: get_age(metadata.creation_timestamp.as_ref()),
                node: spec.and_then(|s| s.node_name.clone()),
                ip: status.and_then(|s| s.pod_ip.clone()),
                containers,
            }
        })
        .collect();

    Ok(pod_infos)
}

pub async fn list_daemonsets(client: &Client, namespace: Option<&str>) -> Result<Vec<DaemonSetInfo>> {
    let daemonsets: Api<DaemonSet> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };

    let ds_list = daemonsets.list(&ListParams::default()).await?;

    let ds_infos: Vec<DaemonSetInfo> = ds_list
        .items
        .iter()
        .map(|ds| {
            let metadata = &ds.metadata;
            let spec = ds.spec.as_ref();
            let status = ds.status.as_ref();

            let node_selector = spec
                .and_then(|s| s.template.spec.as_ref())
                .and_then(|ps| ps.node_selector.as_ref())
                .map(|ns| {
                    ns.iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join(", ")
                });

            DaemonSetInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                desired: status.map(|s| s.desired_number_scheduled).unwrap_or(0),
                current: status.map(|s| s.current_number_scheduled).unwrap_or(0),
                ready: status.map(|s| s.number_ready).unwrap_or(0),
                up_to_date: status.and_then(|s| s.updated_number_scheduled).unwrap_or(0),
                available: status.and_then(|s| s.number_available).unwrap_or(0),
                node_selector,
                age: get_age(metadata.creation_timestamp.as_ref()),
            }
        })
        .collect();

    Ok(ds_infos)
}

pub async fn list_replicasets(client: &Client, namespace: Option<&str>) -> Result<Vec<ReplicaSetInfo>> {
    let replicasets: Api<ReplicaSet> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };

    let rs_list = replicasets.list(&ListParams::default()).await?;

    let rs_infos: Vec<ReplicaSetInfo> = rs_list
        .items
        .iter()
        .map(|rs| {
            let metadata = &rs.metadata;
            let status = rs.status.as_ref();

            let owner = metadata
                .owner_references
                .as_ref()
                .and_then(|refs| refs.first())
                .map(|owner| format!("{}/{}", owner.kind, owner.name));

            ReplicaSetInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                desired: status.map(|s| s.replicas).unwrap_or(0),
                current: status.map(|s| s.replicas).unwrap_or(0),
                ready: status.and_then(|s| s.ready_replicas).unwrap_or(0),
                age: get_age(metadata.creation_timestamp.as_ref()),
                owner,
            }
        })
        .collect();

    Ok(rs_infos)
}

pub async fn list_jobs(client: &Client, namespace: Option<&str>) -> Result<Vec<JobInfo>> {
    let jobs: Api<Job> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };

    let job_list = jobs.list(&ListParams::default()).await?;

    let job_infos: Vec<JobInfo> = job_list
        .items
        .iter()
        .map(|job| {
            let metadata = &job.metadata;
            let spec = job.spec.as_ref();
            let status = job.status.as_ref();

            let completions_spec = spec.and_then(|s| s.completions).unwrap_or(1);
            let succeeded = status.and_then(|s| s.succeeded).unwrap_or(0);
            let failed = status.and_then(|s| s.failed).unwrap_or(0);
            let active = status.and_then(|s| s.active).unwrap_or(0);

            let job_status = if succeeded >= completions_spec {
                "Complete".to_string()
            } else if failed > 0 {
                "Failed".to_string()
            } else if active > 0 {
                "Running".to_string()
            } else {
                "Pending".to_string()
            };

            let duration = status
                .and_then(|s| s.start_time.as_ref())
                .map(|start| {
                    let end = status
                        .and_then(|s| s.completion_time.as_ref())
                        .map(|t| chrono::DateTime::parse_from_rfc3339(&t.0.to_rfc3339()).ok())
                        .flatten()
                        .unwrap_or_else(|| chrono::Utc::now().into());
                    let start_dt = chrono::DateTime::parse_from_rfc3339(&start.0.to_rfc3339())
                        .unwrap_or_else(|_| chrono::Utc::now().into());
                    let dur = end.signed_duration_since(start_dt);
                    if dur.num_hours() > 0 {
                        format!("{}h{}m", dur.num_hours(), dur.num_minutes() % 60)
                    } else if dur.num_minutes() > 0 {
                        format!("{}m{}s", dur.num_minutes(), dur.num_seconds() % 60)
                    } else {
                        format!("{}s", dur.num_seconds())
                    }
                });

            JobInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                completions: format!("{}/{}", succeeded, completions_spec),
                duration,
                age: get_age(metadata.creation_timestamp.as_ref()),
                status: job_status,
            }
        })
        .collect();

    Ok(job_infos)
}

pub async fn list_cronjobs(client: &Client, namespace: Option<&str>) -> Result<Vec<CronJobInfo>> {
    let cronjobs: Api<CronJob> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };

    let cj_list = cronjobs.list(&ListParams::default()).await?;

    let cj_infos: Vec<CronJobInfo> = cj_list
        .items
        .iter()
        .map(|cj| {
            let metadata = &cj.metadata;
            let spec = cj.spec.as_ref();
            let status = cj.status.as_ref();

            let last_schedule = status
                .and_then(|s| s.last_schedule_time.as_ref())
                .map(|t| get_age(Some(t)));

            CronJobInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                schedule: spec.map(|s| s.schedule.clone()).unwrap_or_default(),
                suspend: spec.and_then(|s| s.suspend).unwrap_or(false),
                active: status.and_then(|s| s.active.as_ref()).map(|a| a.len() as i32).unwrap_or(0),
                last_schedule,
                age: get_age(metadata.creation_timestamp.as_ref()),
            }
        })
        .collect();

    Ok(cj_infos)
}

pub async fn list_services(client: &Client, namespace: Option<&str>) -> Result<Vec<ServiceInfo>> {
    let services: Api<Service> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };

    let svc_list = services.list(&ListParams::default()).await?;

    let svc_infos: Vec<ServiceInfo> = svc_list
        .items
        .iter()
        .map(|svc| {
            let metadata = &svc.metadata;
            let spec = svc.spec.as_ref();
            let status = svc.status.as_ref();

            let ports: Vec<String> = spec
                .and_then(|s| s.ports.as_ref())
                .map(|ports| {
                    ports
                        .iter()
                        .map(|p| {
                            let port = p.port;
                            let target = p.target_port.as_ref()
                                .map(|tp| match tp {
                                    k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(i) => i.to_string(),
                                    k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::String(s) => s.clone(),
                                })
                                .unwrap_or_default();
                            let protocol = p.protocol.as_deref().unwrap_or("TCP");
                            format!("{}:{}/{}", port, target, protocol)
                        })
                        .collect()
                })
                .unwrap_or_default();

            let external_ip = status
                .and_then(|s| s.load_balancer.as_ref())
                .and_then(|lb| lb.ingress.as_ref())
                .and_then(|ingress| ingress.first())
                .and_then(|i| i.ip.clone().or_else(|| i.hostname.clone()));

            ServiceInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                service_type: spec.and_then(|s| s.type_.clone()).unwrap_or_default(),
                cluster_ip: spec.and_then(|s| s.cluster_ip.clone()),
                external_ip,
                ports,
                age: get_age(metadata.creation_timestamp.as_ref()),
            }
        })
        .collect();

    Ok(svc_infos)
}

// ============ Service Operations ============

pub async fn get_service_detail(client: &Client, namespace: &str, name: &str) -> Result<ServiceDetail> {
    let services: Api<Service> = Api::namespaced(client.clone(), namespace);
    let svc = services.get(name).await?;

    let metadata = &svc.metadata;
    let spec = svc.spec.as_ref();
    let status = svc.status.as_ref();

    // Get ports with detailed info
    let ports: Vec<ServicePortDetail> = spec
        .and_then(|s| s.ports.as_ref())
        .map(|ports| {
            ports
                .iter()
                .map(|p| ServicePortDetail {
                    name: p.name.clone(),
                    protocol: p.protocol.clone().unwrap_or_else(|| "TCP".to_string()),
                    port: p.port,
                    target_port: p.target_port.as_ref()
                        .map(|tp| match tp {
                            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(i) => i.to_string(),
                            k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::String(s) => s.clone(),
                        })
                        .unwrap_or_default(),
                    node_port: p.node_port,
                })
                .collect()
        })
        .unwrap_or_default();

    // Get load balancer ingress IPs/hostnames
    let load_balancer_ingress: Vec<String> = status
        .and_then(|s| s.load_balancer.as_ref())
        .and_then(|lb| lb.ingress.as_ref())
        .map(|ingress| {
            ingress
                .iter()
                .filter_map(|i| i.ip.clone().or_else(|| i.hostname.clone()))
                .collect()
        })
        .unwrap_or_default();

    Ok(ServiceDetail {
        name: metadata.name.clone().unwrap_or_default(),
        namespace: metadata.namespace.clone().unwrap_or_default(),
        uid: metadata.uid.clone().unwrap_or_default(),
        creation_timestamp: metadata
            .creation_timestamp
            .as_ref()
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.clone().unwrap_or_default(),
        annotations: metadata.annotations.clone().unwrap_or_default(),
        service_type: spec.and_then(|s| s.type_.clone()).unwrap_or_else(|| "ClusterIP".to_string()),
        cluster_ip: spec.and_then(|s| s.cluster_ip.clone()),
        cluster_ips: spec.and_then(|s| s.cluster_ips.clone()).unwrap_or_default(),
        external_ips: spec.and_then(|s| s.external_ips.clone()).unwrap_or_default(),
        ports,
        selector: spec.and_then(|s| s.selector.clone()).unwrap_or_default(),
        session_affinity: spec.and_then(|s| s.session_affinity.clone()).unwrap_or_else(|| "None".to_string()),
        load_balancer_ip: spec.and_then(|s| s.load_balancer_ip.clone()),
        load_balancer_ingress,
        external_name: spec.and_then(|s| s.external_name.clone()),
        internal_traffic_policy: spec.and_then(|s| s.internal_traffic_policy.clone()),
        external_traffic_policy: spec.and_then(|s| s.external_traffic_policy.clone()),
    })
}

pub async fn get_service_yaml(client: &Client, namespace: &str, name: &str) -> Result<String> {
    let services: Api<Service> = Api::namespaced(client.clone(), namespace);
    let svc = services.get(name).await?;
    let yaml = serde_yaml::to_string(&svc)
        .map_err(|e| AppError::Custom(format!("Failed to serialize service to YAML: {}", e)))?;
    Ok(yaml)
}

pub async fn get_service_events(client: &Client, namespace: &str, name: &str) -> Result<Vec<ServiceEvent>> {
    use k8s_openapi::api::core::v1::Event;

    let events: Api<Event> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default()
        .fields(&format!("involvedObject.name={},involvedObject.kind=Service", name));
    let event_list = events.list(&lp).await?;

    let svc_events: Vec<ServiceEvent> = event_list
        .items
        .iter()
        .map(|e| ServiceEvent {
            event_type: e.type_.clone().unwrap_or_default(),
            reason: e.reason.clone().unwrap_or_default(),
            message: e.message.clone().unwrap_or_default(),
            count: e.count.unwrap_or(1),
            first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
            last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
            source: e
                .source
                .as_ref()
                .map(|s| s.component.clone().unwrap_or_default())
                .unwrap_or_default(),
        })
        .collect();

    Ok(svc_events)
}

pub async fn get_service_endpoints(client: &Client, namespace: &str, name: &str) -> Result<Vec<ServiceEndpoint>> {
    use k8s_openapi::api::core::v1::Endpoints;

    let endpoints: Api<Endpoints> = Api::namespaced(client.clone(), namespace);

    // Endpoints have the same name as the service
    let ep = match endpoints.get(name).await {
        Ok(ep) => ep,
        Err(_) => return Ok(vec![]), // No endpoints found
    };

    let mut result: Vec<ServiceEndpoint> = vec![];

    if let Some(subsets) = ep.subsets {
        for subset in subsets {
            let ports = subset.ports.unwrap_or_default();

            // Ready addresses
            if let Some(addresses) = subset.addresses {
                for addr in addresses {
                    for port in &ports {
                        result.push(ServiceEndpoint {
                            ip: addr.ip.clone(),
                            port: port.port,
                            protocol: port.protocol.clone().unwrap_or_else(|| "TCP".to_string()),
                            pod_name: addr.target_ref.as_ref().and_then(|tr| tr.name.clone()),
                            node_name: addr.node_name.clone(),
                            ready: true,
                        });
                    }
                }
            }

            // Not ready addresses
            if let Some(not_ready_addresses) = subset.not_ready_addresses {
                for addr in not_ready_addresses {
                    for port in &ports {
                        result.push(ServiceEndpoint {
                            ip: addr.ip.clone(),
                            port: port.port,
                            protocol: port.protocol.clone().unwrap_or_else(|| "TCP".to_string()),
                            pod_name: addr.target_ref.as_ref().and_then(|tr| tr.name.clone()),
                            node_name: addr.node_name.clone(),
                            ready: false,
                        });
                    }
                }
            }
        }
    }

    Ok(result)
}

// Network resources
pub async fn list_ingresses(client: &Client, namespace: Option<&str>) -> Result<Vec<IngressInfo>> {
    let ingresses: Api<Ingress> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };

    let ing_list = ingresses.list(&ListParams::default()).await?;

    let ing_infos: Vec<IngressInfo> = ing_list
        .items
        .iter()
        .map(|ing| {
            let metadata = &ing.metadata;
            let spec = ing.spec.as_ref();
            let status = ing.status.as_ref();

            let class = spec.and_then(|s| s.ingress_class_name.clone());

            let hosts: Vec<String> = spec
                .and_then(|s| s.rules.as_ref())
                .map(|rules| {
                    rules.iter()
                        .filter_map(|r| r.host.clone())
                        .collect()
                })
                .unwrap_or_default();

            let address = status
                .and_then(|s| s.load_balancer.as_ref())
                .and_then(|lb| lb.ingress.as_ref())
                .and_then(|ingress| ingress.first())
                .and_then(|i| i.ip.clone().or_else(|| i.hostname.clone()));

            let ports = spec
                .and_then(|s| s.tls.as_ref())
                .map(|_| "80, 443".to_string())
                .unwrap_or_else(|| "80".to_string());

            IngressInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                class,
                hosts,
                address,
                ports,
                age: get_age(metadata.creation_timestamp.as_ref()),
            }
        })
        .collect();

    Ok(ing_infos)
}

// ============ Ingress Detail Functions ============

pub async fn get_ingress_detail(client: &Client, namespace: &str, name: &str) -> Result<IngressDetail> {
    let ingresses: Api<Ingress> = Api::namespaced(client.clone(), namespace);
    let ingress = ingresses.get(name).await?;

    let metadata = &ingress.metadata;
    let spec = ingress.spec.as_ref();
    let status = ingress.status.as_ref();

    let labels = metadata.labels.clone().unwrap_or_default();
    let annotations = metadata.annotations.clone().unwrap_or_default();

    let ingress_class = spec.and_then(|s| s.ingress_class_name.clone());

    let rules: Vec<IngressRuleDetail> = spec
        .and_then(|s| s.rules.as_ref())
        .map(|rules| {
            rules.iter().map(|rule| {
                let paths: Vec<IngressPathDetail> = rule.http.as_ref()
                    .map(|http| {
                        http.paths.iter().map(|path| {
                            let backend_service = path.backend.service.as_ref().map(|s| s.name.clone());
                            let backend_port = path.backend.service.as_ref()
                                .and_then(|s| s.port.as_ref())
                                .map(|p| {
                                    p.name.clone().unwrap_or_else(|| {
                                        p.number.map(|n| n.to_string()).unwrap_or_default()
                                    })
                                });

                            IngressPathDetail {
                                path: path.path.clone().unwrap_or_else(|| "/".to_string()),
                                path_type: path.path_type.clone(),
                                backend_service,
                                backend_port,
                            }
                        }).collect()
                    })
                    .unwrap_or_default();

                IngressRuleDetail {
                    host: rule.host.clone(),
                    paths,
                }
            }).collect()
        })
        .unwrap_or_default();

    let tls: Vec<IngressTlsDetail> = spec
        .and_then(|s| s.tls.as_ref())
        .map(|tls_list| {
            tls_list.iter().map(|tls| {
                IngressTlsDetail {
                    hosts: tls.hosts.clone().unwrap_or_default(),
                    secret_name: tls.secret_name.clone(),
                }
            }).collect()
        })
        .unwrap_or_default();

    let default_backend = spec
        .and_then(|s| s.default_backend.as_ref())
        .and_then(|db| db.service.as_ref())
        .map(|svc| {
            let port = svc.port.as_ref()
                .map(|p| p.name.clone().unwrap_or_else(|| p.number.map(|n| n.to_string()).unwrap_or_default()))
                .unwrap_or_default();
            IngressBackendDetail {
                service_name: svc.name.clone(),
                service_port: port,
            }
        });

    let load_balancer_addresses: Vec<String> = status
        .and_then(|s| s.load_balancer.as_ref())
        .and_then(|lb| lb.ingress.as_ref())
        .map(|ingress_list| {
            ingress_list.iter()
                .filter_map(|i| i.ip.clone().or_else(|| i.hostname.clone()))
                .collect()
        })
        .unwrap_or_default();

    Ok(IngressDetail {
        name: metadata.name.clone().unwrap_or_default(),
        namespace: metadata.namespace.clone().unwrap_or_default(),
        uid: metadata.uid.clone().unwrap_or_default(),
        creation_timestamp: metadata.creation_timestamp
            .as_ref()
            .map(|t| t.0.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_default(),
        labels,
        annotations,
        ingress_class,
        rules,
        tls,
        default_backend,
        load_balancer_addresses,
    })
}

pub async fn get_ingress_yaml(client: &Client, namespace: &str, name: &str) -> Result<String> {
    let ingresses: Api<Ingress> = Api::namespaced(client.clone(), namespace);
    let ingress = ingresses.get(name).await?;
    serde_yaml::to_string(&ingress)
        .map_err(|e| AppError::Custom(format!("Failed to serialize ingress to YAML: {}", e)))
}

pub async fn get_ingress_events(client: &Client, namespace: &str, name: &str) -> Result<Vec<IngressEvent>> {
    use k8s_openapi::api::core::v1::Event;

    let events: Api<Event> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().fields(&format!("involvedObject.name={},involvedObject.kind=Ingress", name));
    let event_list = events.list(&lp).await?;

    let result: Vec<IngressEvent> = event_list.items.iter().map(|e| {
        IngressEvent {
            event_type: e.type_.clone().unwrap_or_else(|| "Unknown".to_string()),
            reason: e.reason.clone().unwrap_or_default(),
            message: e.message.clone().unwrap_or_default(),
            count: e.count.unwrap_or(1),
            first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.format("%Y-%m-%d %H:%M:%S").to_string()),
            last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.format("%Y-%m-%d %H:%M:%S").to_string()),
            source: e.source.as_ref()
                .map(|s| format!("{}/{}", s.component.as_deref().unwrap_or(""), s.host.as_deref().unwrap_or("")))
                .unwrap_or_default(),
        }
    }).collect();

    Ok(result)
}

pub async fn list_network_policies(client: &Client, namespace: Option<&str>) -> Result<Vec<NetworkPolicyInfo>> {
    let netpols: Api<NetworkPolicy> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };

    let np_list = netpols.list(&ListParams::default()).await?;

    let np_infos: Vec<NetworkPolicyInfo> = np_list
        .items
        .iter()
        .map(|np| {
            let metadata = &np.metadata;
            let spec = np.spec.as_ref();

            let pod_selector = spec
                .map(|s| {
                    s.pod_selector.match_labels.as_ref()
                        .map(|labels| {
                            labels.iter()
                                .map(|(k, v)| format!("{}={}", k, v))
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_else(|| "<all>".to_string())
                })
                .unwrap_or_default();

            let policy_types: Vec<String> = spec
                .and_then(|s| s.policy_types.clone())
                .unwrap_or_default();

            NetworkPolicyInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                pod_selector,
                policy_types,
                age: get_age(metadata.creation_timestamp.as_ref()),
            }
        })
        .collect();

    Ok(np_infos)
}

// Config resources
pub async fn list_configmaps(client: &Client, namespace: Option<&str>) -> Result<Vec<ConfigMapInfo>> {
    let configmaps: Api<ConfigMap> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };

    let cm_list = configmaps.list(&ListParams::default()).await?;

    let cm_infos: Vec<ConfigMapInfo> = cm_list
        .items
        .iter()
        .map(|cm| {
            let metadata = &cm.metadata;
            let data_count = cm.data.as_ref().map(|d| d.len() as i32).unwrap_or(0)
                + cm.binary_data.as_ref().map(|d| d.len() as i32).unwrap_or(0);

            ConfigMapInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                data_count,
                age: get_age(metadata.creation_timestamp.as_ref()),
            }
        })
        .collect();

    Ok(cm_infos)
}

pub async fn list_secrets(client: &Client, namespace: Option<&str>) -> Result<Vec<SecretInfo>> {
    let secrets: Api<Secret> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };

    let sec_list = secrets.list(&ListParams::default()).await?;

    let sec_infos: Vec<SecretInfo> = sec_list
        .items
        .iter()
        .map(|sec| {
            let metadata = &sec.metadata;
            let data_count = sec.data.as_ref().map(|d| d.len() as i32).unwrap_or(0);

            SecretInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                secret_type: sec.type_.clone().unwrap_or_default(),
                data_count,
                age: get_age(metadata.creation_timestamp.as_ref()),
            }
        })
        .collect();

    Ok(sec_infos)
}

pub async fn list_hpas(client: &Client, namespace: Option<&str>) -> Result<Vec<HPAInfo>> {
    let hpas: Api<HorizontalPodAutoscaler> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };

    let hpa_list = hpas.list(&ListParams::default()).await?;

    let hpa_infos: Vec<HPAInfo> = hpa_list
        .items
        .iter()
        .map(|hpa| {
            let metadata = &hpa.metadata;
            let spec = hpa.spec.as_ref();
            let status = hpa.status.as_ref();

            let reference = spec
                .map(|s| format!("{}/{}", s.scale_target_ref.kind, s.scale_target_ref.name))
                .unwrap_or_default();

            let current_cpu = status
                .and_then(|s| s.current_cpu_utilization_percentage)
                .map(|p| format!("{}%", p))
                .unwrap_or_else(|| "<unknown>".to_string());

            let target_cpu = spec
                .and_then(|s| s.target_cpu_utilization_percentage)
                .map(|p| format!("{}%", p))
                .unwrap_or_else(|| "-".to_string());

            let targets = format!("{}/{}", current_cpu, target_cpu);

            HPAInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                reference,
                targets,
                min_pods: spec.and_then(|s| s.min_replicas).unwrap_or(1),
                max_pods: spec.map(|s| s.max_replicas).unwrap_or(0),
                replicas: status.map(|s| s.current_replicas).unwrap_or(0),
                age: get_age(metadata.creation_timestamp.as_ref()),
            }
        })
        .collect();

    Ok(hpa_infos)
}

// Storage resources
pub async fn list_pvs(client: &Client) -> Result<Vec<PersistentVolumeInfo>> {
    let pvs: Api<PersistentVolume> = Api::all(client.clone());
    let pv_list = pvs.list(&ListParams::default()).await?;

    let pv_infos: Vec<PersistentVolumeInfo> = pv_list
        .items
        .iter()
        .map(|pv| {
            let metadata = &pv.metadata;
            let spec = pv.spec.as_ref();
            let status = pv.status.as_ref();

            let capacity = spec
                .and_then(|s| s.capacity.as_ref())
                .and_then(|c| c.get("storage"))
                .map(|q| q.0.clone())
                .unwrap_or_default();

            let access_modes: Vec<String> = spec
                .and_then(|s| s.access_modes.clone())
                .unwrap_or_default();

            let claim = spec
                .and_then(|s| s.claim_ref.as_ref())
                .map(|c| format!("{}/{}", c.namespace.as_deref().unwrap_or(""), c.name.as_deref().unwrap_or("")));

            PersistentVolumeInfo {
                name: metadata.name.clone().unwrap_or_default(),
                capacity,
                access_modes,
                reclaim_policy: spec.and_then(|s| s.persistent_volume_reclaim_policy.clone()).unwrap_or_default(),
                status: status.and_then(|s| s.phase.clone()).unwrap_or_default(),
                claim,
                storage_class: spec.and_then(|s| s.storage_class_name.clone()),
                age: get_age(metadata.creation_timestamp.as_ref()),
            }
        })
        .collect();

    Ok(pv_infos)
}

pub async fn list_pvcs(client: &Client, namespace: Option<&str>) -> Result<Vec<PersistentVolumeClaimInfo>> {
    let pvcs: Api<PersistentVolumeClaim> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };

    let pvc_list = pvcs.list(&ListParams::default()).await?;

    let pvc_infos: Vec<PersistentVolumeClaimInfo> = pvc_list
        .items
        .iter()
        .map(|pvc| {
            let metadata = &pvc.metadata;
            let spec = pvc.spec.as_ref();
            let status = pvc.status.as_ref();

            let access_modes: Vec<String> = spec
                .and_then(|s| s.access_modes.clone())
                .unwrap_or_default();

            let capacity = status
                .and_then(|s| s.capacity.as_ref())
                .and_then(|c| c.get("storage"))
                .map(|q| q.0.clone());

            PersistentVolumeClaimInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                status: status.and_then(|s| s.phase.clone()).unwrap_or_default(),
                volume: spec.and_then(|s| s.volume_name.clone()),
                capacity,
                access_modes,
                storage_class: spec.and_then(|s| s.storage_class_name.clone()),
                age: get_age(metadata.creation_timestamp.as_ref()),
            }
        })
        .collect();

    Ok(pvc_infos)
}

// Cluster resources
pub async fn list_namespaces_info(client: &Client) -> Result<Vec<NamespaceInfo>> {
    let namespaces: Api<Namespace> = Api::all(client.clone());
    let ns_list = namespaces.list(&ListParams::default()).await?;

    let ns_infos: Vec<NamespaceInfo> = ns_list
        .items
        .iter()
        .map(|ns| {
            let metadata = &ns.metadata;
            let status = ns.status.as_ref();

            NamespaceInfo {
                name: metadata.name.clone().unwrap_or_default(),
                status: status.and_then(|s| s.phase.clone()).unwrap_or_default(),
                age: get_age(metadata.creation_timestamp.as_ref()),
            }
        })
        .collect();

    Ok(ns_infos)
}

pub async fn list_nodes(client: &Client) -> Result<Vec<NodeInfo>> {
    let nodes: Api<Node> = Api::all(client.clone());
    let node_list = nodes.list(&ListParams::default()).await?;

    let node_infos: Vec<NodeInfo> = node_list
        .items
        .iter()
        .map(|node| {
            let metadata = &node.metadata;
            let spec = node.spec.as_ref();
            let status = node.status.as_ref();

            // Get node roles from labels
            let roles: Vec<String> = metadata
                .labels
                .as_ref()
                .map(|labels| {
                    labels.iter()
                        .filter_map(|(k, _)| {
                            if k.starts_with("node-role.kubernetes.io/") {
                                Some(k.trim_start_matches("node-role.kubernetes.io/").to_string())
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Get node status
            let node_status = status
                .and_then(|s| s.conditions.as_ref())
                .and_then(|conditions| {
                    conditions.iter().find(|c| c.type_ == "Ready")
                })
                .map(|c| {
                    if c.status == "True" { "Ready" } else { "NotReady" }
                })
                .unwrap_or("Unknown")
                .to_string();

            // Check if node is unschedulable
            let is_unschedulable = spec.and_then(|s| s.unschedulable).unwrap_or(false);
            let final_status = if is_unschedulable {
                format!("{},SchedulingDisabled", node_status)
            } else {
                node_status
            };

            let node_info = status.and_then(|s| s.node_info.as_ref());

            let internal_ip = status
                .and_then(|s| s.addresses.as_ref())
                .and_then(|addrs| {
                    addrs.iter()
                        .find(|a| a.type_ == "InternalIP")
                        .map(|a| a.address.clone())
                });

            NodeInfo {
                name: metadata.name.clone().unwrap_or_default(),
                status: final_status,
                roles: if roles.is_empty() { vec!["<none>".to_string()] } else { roles },
                age: get_age(metadata.creation_timestamp.as_ref()),
                version: node_info.map(|i| i.kubelet_version.clone()).unwrap_or_default(),
                internal_ip,
                os_image: node_info.map(|i| i.os_image.clone()).unwrap_or_default(),
                kernel: node_info.map(|i| i.kernel_version.clone()).unwrap_or_default(),
                container_runtime: node_info.map(|i| i.container_runtime_version.clone()).unwrap_or_default(),
            }
        })
        .collect();

    Ok(node_infos)
}

pub async fn list_service_accounts(client: &Client, namespace: Option<&str>) -> Result<Vec<ServiceAccountInfo>> {
    let sas: Api<ServiceAccount> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };

    let sa_list = sas.list(&ListParams::default()).await?;

    let sa_infos: Vec<ServiceAccountInfo> = sa_list
        .items
        .iter()
        .map(|sa| {
            let metadata = &sa.metadata;
            let secrets = sa.secrets.as_ref().map(|s| s.len() as i32).unwrap_or(0);

            ServiceAccountInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_default(),
                secrets,
                age: get_age(metadata.creation_timestamp.as_ref()),
            }
        })
        .collect();

    Ok(sa_infos)
}

pub async fn get_metrics(client: &Client) -> Result<ClusterMetrics> {
    let pods: Api<Pod> = Api::all(client.clone());
    let deployments: Api<Deployment> = Api::all(client.clone());
    let services: Api<Service> = Api::all(client.clone());
    let namespaces: Api<Namespace> = Api::all(client.clone());

    let pod_list = pods.list(&ListParams::default()).await?;
    let deploy_list = deployments.list(&ListParams::default()).await?;
    let svc_list = services.list(&ListParams::default()).await?;
    let ns_list = namespaces.list(&ListParams::default()).await?;

    let total_pods = pod_list.items.len() as i32;
    let running_pods = pod_list
        .items
        .iter()
        .filter(|p| {
            p.status
                .as_ref()
                .and_then(|s| s.phase.as_ref())
                .map(|phase| phase == "Running")
                .unwrap_or(false)
        })
        .count() as i32;
    let pending_pods = pod_list
        .items
        .iter()
        .filter(|p| {
            p.status
                .as_ref()
                .and_then(|s| s.phase.as_ref())
                .map(|phase| phase == "Pending")
                .unwrap_or(false)
        })
        .count() as i32;
    let failed_pods = pod_list
        .items
        .iter()
        .filter(|p| {
            p.status
                .as_ref()
                .and_then(|s| s.phase.as_ref())
                .map(|phase| phase == "Failed")
                .unwrap_or(false)
        })
        .count() as i32;

    Ok(ClusterMetrics {
        total_pods,
        running_pods,
        pending_pods,
        failed_pods,
        total_deployments: deploy_list.items.len() as i32,
        total_services: svc_list.items.len() as i32,
        total_namespaces: ns_list.items.len() as i32,
    })
}

pub async fn get_pulse_metrics(client: &Client, namespace: Option<&str>) -> Result<PulseMetrics> {
    // Get cluster info from kubeconfig
    let kubeconfig = kube::config::Kubeconfig::read_from(get_kubeconfig_path())?;
    let current_context_name = kubeconfig.current_context.clone().unwrap_or_default();
    let current_ctx = kubeconfig.contexts.iter().find(|c| c.name == current_context_name);
    let context_info = current_ctx.and_then(|c| c.context.as_ref());

    let cluster_name = context_info.map(|c| c.cluster.clone()).unwrap_or_default();
    let user_name = context_info.and_then(|c| c.user.clone()).unwrap_or_default();

    // Get K8s version from nodes
    let nodes_api: Api<Node> = Api::all(client.clone());
    let nodes = nodes_api.list(&ListParams::default()).await?;
    let k8s_version = nodes.items.first()
        .and_then(|n| n.status.as_ref())
        .and_then(|s| s.node_info.as_ref())
        .map(|info| info.kubelet_version.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    // Create APIs based on namespace filter
    let pods_api: Api<Pod> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let deployments_api: Api<Deployment> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let statefulsets_api: Api<StatefulSet> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let daemonsets_api: Api<DaemonSet> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let replicasets_api: Api<ReplicaSet> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let jobs_api: Api<Job> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let cronjobs_api: Api<CronJob> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let services_api: Api<Service> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let configmaps_api: Api<ConfigMap> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let secrets_api: Api<Secret> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let pvcs_api: Api<PersistentVolumeClaim> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let hpas_api: Api<HorizontalPodAutoscaler> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let ingresses_api: Api<Ingress> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let netpol_api: Api<NetworkPolicy> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };
    let sa_api: Api<ServiceAccount> = match namespace {
        Some(ns) => Api::namespaced(client.clone(), ns),
        None => Api::all(client.clone()),
    };

    // Cluster-scoped resources (always all)
    let pvs_api: Api<PersistentVolume> = Api::all(client.clone());
    let ns_api: Api<Namespace> = Api::all(client.clone());

    let params = ListParams::default();

    // Execute all queries
    let (pods, deployments, statefulsets, daemonsets, replicasets, jobs, cronjobs,
         services, configmaps, secrets, pvs, pvcs, hpas, ingresses, netpols, sas, nss) = tokio::try_join!(
        pods_api.list(&params),
        deployments_api.list(&params),
        statefulsets_api.list(&params),
        daemonsets_api.list(&params),
        replicasets_api.list(&params),
        jobs_api.list(&params),
        cronjobs_api.list(&params),
        services_api.list(&params),
        configmaps_api.list(&params),
        secrets_api.list(&params),
        pvs_api.list(&params),
        pvcs_api.list(&params),
        hpas_api.list(&params),
        ingresses_api.list(&params),
        netpol_api.list(&params),
        sa_api.list(&params),
        ns_api.list(&params),
    )?;

    // Calculate CPU and memory from nodes
    let mut cpu_capacity: i64 = 0;
    let mut cpu_allocatable: i64 = 0;
    let mut memory_capacity: i64 = 0;
    let mut memory_allocatable: i64 = 0;

    for node in &nodes.items {
        if let Some(status) = &node.status {
            if let Some(capacity) = &status.capacity {
                if let Some(cpu) = capacity.get("cpu") {
                    cpu_capacity += parse_cpu_quantity(&cpu.0);
                }
                if let Some(mem) = capacity.get("memory") {
                    memory_capacity += parse_memory_quantity(&mem.0);
                }
            }
            if let Some(allocatable) = &status.allocatable {
                if let Some(cpu) = allocatable.get("cpu") {
                    cpu_allocatable += parse_cpu_quantity(&cpu.0);
                }
                if let Some(mem) = allocatable.get("memory") {
                    memory_allocatable += parse_memory_quantity(&mem.0);
                }
            }
        }
    }

    // Calculate OK/FAIL for pods
    let pods_ok = pods.items.iter().filter(|p| {
        p.status.as_ref()
            .and_then(|s| s.phase.as_ref())
            .map(|phase| phase == "Running" || phase == "Succeeded")
            .unwrap_or(false)
    }).count() as i32;
    let pods_fail = pods.items.len() as i32 - pods_ok;

    // Calculate OK/FAIL for deployments
    let deployments_ok = deployments.items.iter().filter(|d| {
        let status = d.status.as_ref();
        let replicas = status.and_then(|s| s.replicas).unwrap_or(0);
        let available = status.and_then(|s| s.available_replicas).unwrap_or(0);
        replicas > 0 && available == replicas
    }).count() as i32;
    let deployments_fail = deployments.items.len() as i32 - deployments_ok;

    // Calculate OK/FAIL for statefulsets
    let statefulsets_ok = statefulsets.items.iter().filter(|s| {
        let status = s.status.as_ref();
        let replicas = status.map(|st| st.replicas).unwrap_or(0);
        let ready = status.and_then(|st| st.ready_replicas).unwrap_or(0);
        replicas > 0 && ready == replicas
    }).count() as i32;
    let statefulsets_fail = statefulsets.items.len() as i32 - statefulsets_ok;

    // Calculate OK/FAIL for daemonsets
    let daemonsets_ok = daemonsets.items.iter().filter(|d| {
        let status = d.status.as_ref();
        let desired = status.map(|s| s.desired_number_scheduled).unwrap_or(0);
        let ready = status.map(|s| s.number_ready).unwrap_or(0);
        desired > 0 && ready == desired
    }).count() as i32;
    let daemonsets_fail = daemonsets.items.len() as i32 - daemonsets_ok;

    // Calculate OK/FAIL for replicasets
    let replicasets_ok = replicasets.items.iter().filter(|r| {
        let status = r.status.as_ref();
        let replicas = status.map(|s| s.replicas).unwrap_or(0);
        let ready = status.and_then(|s| s.ready_replicas).unwrap_or(0);
        replicas == 0 || ready == replicas
    }).count() as i32;
    let replicasets_fail = replicasets.items.len() as i32 - replicasets_ok;

    // Calculate OK/FAIL for jobs
    let jobs_ok = jobs.items.iter().filter(|j| {
        let status = j.status.as_ref();
        let succeeded = status.and_then(|s| s.succeeded).unwrap_or(0);
        let failed = status.and_then(|s| s.failed).unwrap_or(0);
        let active = status.and_then(|s| s.active).unwrap_or(0);
        (succeeded > 0 && failed == 0) || active > 0
    }).count() as i32;
    let jobs_fail = jobs.items.iter().filter(|j| {
        let status = j.status.as_ref();
        status.and_then(|s| s.failed).unwrap_or(0) > 0
    }).count() as i32;

    Ok(PulseMetrics {
        context: current_context_name,
        cluster: cluster_name,
        user: user_name,
        k8s_version,
        pods: ResourceCount { ok: pods_ok, fail: pods_fail },
        deployments: ResourceCount { ok: deployments_ok, fail: deployments_fail },
        statefulsets: ResourceCount { ok: statefulsets_ok, fail: statefulsets_fail },
        daemonsets: ResourceCount { ok: daemonsets_ok, fail: daemonsets_fail },
        replicasets: ResourceCount { ok: replicasets_ok, fail: replicasets_fail },
        jobs: ResourceCount { ok: jobs_ok, fail: jobs_fail },
        cronjobs: cronjobs.items.len() as i32,
        services: services.items.len() as i32,
        configmaps: configmaps.items.len() as i32,
        secrets: secrets.items.len() as i32,
        pvs: pvs.items.len() as i32,
        pvcs: pvcs.items.len() as i32,
        hpas: hpas.items.len() as i32,
        ingresses: ingresses.items.len() as i32,
        network_policies: netpols.items.len() as i32,
        service_accounts: sas.items.len() as i32,
        namespaces: nss.items.len() as i32,
        nodes: nodes.items.len() as i32,
        cpu_capacity,
        cpu_allocatable,
        memory_capacity,
        memory_allocatable,
    })
}

fn get_pod_phase(status: Option<&k8s_openapi::api::core::v1::PodStatus>) -> String {
    status
        .and_then(|s| s.phase.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn get_container_state(status: Option<&k8s_openapi::api::core::v1::ContainerStatus>) -> String {
    status
        .and_then(|s| s.state.as_ref())
        .map(|state| {
            if state.running.is_some() {
                "Running".to_string()
            } else if let Some(waiting) = &state.waiting {
                waiting.reason.clone().unwrap_or_else(|| "Waiting".to_string())
            } else if let Some(terminated) = &state.terminated {
                terminated.reason.clone().unwrap_or_else(|| "Terminated".to_string())
            } else {
                "Unknown".to_string()
            }
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

fn pod_to_pod_info(pod: &Pod) -> PodInfo {
    let metadata = &pod.metadata;
    let spec = pod.spec.as_ref();
    let status = pod.status.as_ref();

    let containers: Vec<ContainerInfo> = spec
        .map(|s| {
            s.containers
                .iter()
                .map(|c| {
                    let container_status = status
                        .and_then(|st| st.container_statuses.as_ref())
                        .and_then(|statuses| {
                            statuses.iter().find(|cs| cs.name == c.name)
                        });

                    ContainerInfo {
                        name: c.name.clone(),
                        image: c.image.clone().unwrap_or_default(),
                        ready: container_status.map(|cs| cs.ready).unwrap_or(false),
                        restart_count: container_status
                            .map(|cs| cs.restart_count)
                            .unwrap_or(0),
                        state: get_container_state(container_status),
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let ready_count = containers.iter().filter(|c| c.ready).count();
    let total_count = containers.len();
    let total_restarts: i32 = containers.iter().map(|c| c.restart_count).sum();

    PodInfo {
        name: metadata.name.clone().unwrap_or_default(),
        namespace: metadata.namespace.clone().unwrap_or_default(),
        status: get_pod_phase(status),
        ready: format!("{}/{}", ready_count, total_count),
        restarts: total_restarts,
        age: get_age(metadata.creation_timestamp.as_ref()),
        node: spec.and_then(|s| s.node_name.clone()),
        ip: status.and_then(|s| s.pod_ip.clone()),
        containers,
    }
}

fn get_age(timestamp: Option<&k8s_openapi::apimachinery::pkg::apis::meta::v1::Time>) -> String {
    timestamp
        .map(|ts| {
            let now = chrono::Utc::now();
            let created = chrono::DateTime::parse_from_rfc3339(&ts.0.to_rfc3339())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or(now);
            let duration = now.signed_duration_since(created);

            if duration.num_days() > 0 {
                format!("{}d", duration.num_days())
            } else if duration.num_hours() > 0 {
                format!("{}h", duration.num_hours())
            } else if duration.num_minutes() > 0 {
                format!("{}m", duration.num_minutes())
            } else {
                format!("{}s", duration.num_seconds())
            }
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Parse CPU quantity string to millicores (e.g., "4" -> 4000, "500m" -> 500)
fn parse_cpu_quantity(quantity: &str) -> i64 {
    let quantity = quantity.trim();
    if quantity.ends_with('m') {
        // Already in millicores
        quantity[..quantity.len() - 1].parse::<i64>().unwrap_or(0)
    } else if quantity.ends_with('n') {
        // Nanocores to millicores
        quantity[..quantity.len() - 1].parse::<i64>().unwrap_or(0) / 1_000_000
    } else {
        // Whole cores to millicores
        quantity.parse::<f64>().unwrap_or(0.0) as i64 * 1000
    }
}

/// Parse memory quantity string to bytes (e.g., "16Gi" -> 17179869184, "1024Mi" -> 1073741824)
fn parse_memory_quantity(quantity: &str) -> i64 {
    let quantity = quantity.trim();

    // Binary suffixes (Ki, Mi, Gi, Ti, Pi, Ei)
    if quantity.ends_with("Ki") {
        return quantity[..quantity.len() - 2].parse::<i64>().unwrap_or(0) * 1024;
    }
    if quantity.ends_with("Mi") {
        return quantity[..quantity.len() - 2].parse::<i64>().unwrap_or(0) * 1024 * 1024;
    }
    if quantity.ends_with("Gi") {
        return quantity[..quantity.len() - 2].parse::<i64>().unwrap_or(0) * 1024 * 1024 * 1024;
    }
    if quantity.ends_with("Ti") {
        return quantity[..quantity.len() - 2].parse::<i64>().unwrap_or(0) * 1024 * 1024 * 1024 * 1024;
    }

    // Decimal suffixes (k, M, G, T)
    if quantity.ends_with('k') {
        return quantity[..quantity.len() - 1].parse::<i64>().unwrap_or(0) * 1000;
    }
    if quantity.ends_with('M') {
        return quantity[..quantity.len() - 1].parse::<i64>().unwrap_or(0) * 1000 * 1000;
    }
    if quantity.ends_with('G') {
        return quantity[..quantity.len() - 1].parse::<i64>().unwrap_or(0) * 1000 * 1000 * 1000;
    }
    if quantity.ends_with('T') {
        return quantity[..quantity.len() - 1].parse::<i64>().unwrap_or(0) * 1000 * 1000 * 1000 * 1000;
    }

    // Plain bytes
    quantity.parse::<i64>().unwrap_or(0)
}

// ============ Pod Detail Types ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodDetail {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub status: String,
    pub phase: String,
    pub pod_ip: Option<String>,
    pub node_name: Option<String>,
    pub service_account: Option<String>,
    pub conditions: Vec<PodCondition>,
    pub containers: Vec<ContainerDetail>,
    pub init_containers: Vec<ContainerDetail>,
    pub volumes: Vec<VolumeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodCondition {
    #[serde(rename = "type")]
    pub type_: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_transition_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerDetail {
    pub name: String,
    pub image: String,
    pub ready: bool,
    pub restart_count: i32,
    pub state: String,
    pub state_reason: Option<String>,
    pub started_at: Option<String>,
    pub ports: Vec<ContainerPort>,
    pub resources: ContainerResources,
    pub env_vars: Vec<EnvVarInfo>,
    pub volume_mounts: Vec<VolumeMountInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerPort {
    pub name: Option<String>,
    pub container_port: i32,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerResources {
    pub cpu_request: Option<String>,
    pub cpu_limit: Option<String>,
    pub memory_request: Option<String>,
    pub memory_limit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarInfo {
    pub name: String,
    pub value: Option<String>,
    pub source: String, // "literal", "configMapKeyRef", "secretKeyRef", "fieldRef", "resourceFieldRef"
    pub source_name: Option<String>, // name of configmap/secret/field
    pub source_key: Option<String>,  // key in configmap/secret
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMountInfo {
    pub name: String,
    pub mount_path: String,
    pub read_only: bool,
    pub sub_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub name: String,
    pub volume_type: String,
    pub source_name: Option<String>, // PVC name, ConfigMap name, Secret name, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodEvent {
    #[serde(rename = "type")]
    pub type_: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

// ============ Pod Detail Functions ============

pub async fn get_pod_detail(client: &Client, namespace: &str, name: &str) -> Result<PodDetail> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let pod = pods.get(name).await?;

    let metadata = &pod.metadata;
    let spec = pod.spec.as_ref();
    let status = pod.status.as_ref();

    // Build container details
    let containers: Vec<ContainerDetail> = spec
        .map(|s| {
            s.containers
                .iter()
                .map(|c| {
                    let container_status = status
                        .and_then(|st| st.container_statuses.as_ref())
                        .and_then(|statuses| statuses.iter().find(|cs| cs.name == c.name));

                    let (state, state_reason, started_at) = get_container_state_detail(container_status);

                    let ports: Vec<ContainerPort> = c.ports.as_ref()
                        .map(|ps| ps.iter().map(|p| ContainerPort {
                            name: p.name.clone(),
                            container_port: p.container_port,
                            protocol: p.protocol.clone().unwrap_or_else(|| "TCP".to_string()),
                        }).collect())
                        .unwrap_or_default();

                    let resources = c.resources.as_ref();
                    let container_resources = ContainerResources {
                        cpu_request: resources.and_then(|r| r.requests.as_ref())
                            .and_then(|req| req.get("cpu")).map(|q| q.0.clone()),
                        cpu_limit: resources.and_then(|r| r.limits.as_ref())
                            .and_then(|lim| lim.get("cpu")).map(|q| q.0.clone()),
                        memory_request: resources.and_then(|r| r.requests.as_ref())
                            .and_then(|req| req.get("memory")).map(|q| q.0.clone()),
                        memory_limit: resources.and_then(|r| r.limits.as_ref())
                            .and_then(|lim| lim.get("memory")).map(|q| q.0.clone()),
                    };

                    let volume_mounts: Vec<VolumeMountInfo> = c.volume_mounts.as_ref()
                        .map(|vms| vms.iter().map(|vm| VolumeMountInfo {
                            name: vm.name.clone(),
                            mount_path: vm.mount_path.clone(),
                            read_only: vm.read_only.unwrap_or(false),
                            sub_path: vm.sub_path.clone(),
                        }).collect())
                        .unwrap_or_default();

                    let env_vars: Vec<EnvVarInfo> = c.env.as_ref()
                        .map(|envs| envs.iter().map(|e| {
                            if let Some(value_from) = &e.value_from {
                                if let Some(config_map_ref) = &value_from.config_map_key_ref {
                                    EnvVarInfo {
                                        name: e.name.clone(),
                                        value: None,
                                        source: "configMapKeyRef".to_string(),
                                        source_name: Some(config_map_ref.name.clone()),
                                        source_key: Some(config_map_ref.key.clone()),
                                    }
                                } else if let Some(secret_ref) = &value_from.secret_key_ref {
                                    EnvVarInfo {
                                        name: e.name.clone(),
                                        value: None,
                                        source: "secretKeyRef".to_string(),
                                        source_name: Some(secret_ref.name.clone()),
                                        source_key: Some(secret_ref.key.clone()),
                                    }
                                } else if let Some(field_ref) = &value_from.field_ref {
                                    EnvVarInfo {
                                        name: e.name.clone(),
                                        value: None,
                                        source: "fieldRef".to_string(),
                                        source_name: Some(field_ref.field_path.clone()),
                                        source_key: None,
                                    }
                                } else if let Some(resource_ref) = &value_from.resource_field_ref {
                                    EnvVarInfo {
                                        name: e.name.clone(),
                                        value: None,
                                        source: "resourceFieldRef".to_string(),
                                        source_name: Some(resource_ref.resource.clone()),
                                        source_key: resource_ref.container_name.clone(),
                                    }
                                } else {
                                    EnvVarInfo {
                                        name: e.name.clone(),
                                        value: e.value.clone(),
                                        source: "literal".to_string(),
                                        source_name: None,
                                        source_key: None,
                                    }
                                }
                            } else {
                                EnvVarInfo {
                                    name: e.name.clone(),
                                    value: e.value.clone(),
                                    source: "literal".to_string(),
                                    source_name: None,
                                    source_key: None,
                                }
                            }
                        }).collect())
                        .unwrap_or_default();

                    ContainerDetail {
                        name: c.name.clone(),
                        image: c.image.clone().unwrap_or_default(),
                        ready: container_status.map(|cs| cs.ready).unwrap_or(false),
                        restart_count: container_status.map(|cs| cs.restart_count).unwrap_or(0),
                        state,
                        state_reason,
                        started_at,
                        ports,
                        resources: container_resources,
                        env_vars,
                        volume_mounts,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    // Build init container details
    let init_containers: Vec<ContainerDetail> = spec
        .and_then(|s| s.init_containers.as_ref())
        .map(|ics| {
            ics.iter()
                .map(|c| {
                    let container_status = status
                        .and_then(|st| st.init_container_statuses.as_ref())
                        .and_then(|statuses| statuses.iter().find(|cs| cs.name == c.name));

                    let (state, state_reason, started_at) = get_container_state_detail(container_status);

                    let env_vars: Vec<EnvVarInfo> = c.env.as_ref()
                        .map(|envs| envs.iter().map(|e| {
                            if let Some(value_from) = &e.value_from {
                                if let Some(config_map_ref) = &value_from.config_map_key_ref {
                                    EnvVarInfo {
                                        name: e.name.clone(),
                                        value: None,
                                        source: "configMapKeyRef".to_string(),
                                        source_name: Some(config_map_ref.name.clone()),
                                        source_key: Some(config_map_ref.key.clone()),
                                    }
                                } else if let Some(secret_ref) = &value_from.secret_key_ref {
                                    EnvVarInfo {
                                        name: e.name.clone(),
                                        value: None,
                                        source: "secretKeyRef".to_string(),
                                        source_name: Some(secret_ref.name.clone()),
                                        source_key: Some(secret_ref.key.clone()),
                                    }
                                } else if let Some(field_ref) = &value_from.field_ref {
                                    EnvVarInfo {
                                        name: e.name.clone(),
                                        value: None,
                                        source: "fieldRef".to_string(),
                                        source_name: Some(field_ref.field_path.clone()),
                                        source_key: None,
                                    }
                                } else {
                                    EnvVarInfo {
                                        name: e.name.clone(),
                                        value: e.value.clone(),
                                        source: "literal".to_string(),
                                        source_name: None,
                                        source_key: None,
                                    }
                                }
                            } else {
                                EnvVarInfo {
                                    name: e.name.clone(),
                                    value: e.value.clone(),
                                    source: "literal".to_string(),
                                    source_name: None,
                                    source_key: None,
                                }
                            }
                        }).collect())
                        .unwrap_or_default();

                    let volume_mounts: Vec<VolumeMountInfo> = c.volume_mounts.as_ref()
                        .map(|vms| vms.iter().map(|vm| VolumeMountInfo {
                            name: vm.name.clone(),
                            mount_path: vm.mount_path.clone(),
                            read_only: vm.read_only.unwrap_or(false),
                            sub_path: vm.sub_path.clone(),
                        }).collect())
                        .unwrap_or_default();

                    ContainerDetail {
                        name: c.name.clone(),
                        image: c.image.clone().unwrap_or_default(),
                        ready: container_status.map(|cs| cs.ready).unwrap_or(false),
                        restart_count: container_status.map(|cs| cs.restart_count).unwrap_or(0),
                        state,
                        state_reason,
                        started_at,
                        ports: vec![],
                        resources: ContainerResources {
                            cpu_request: None,
                            cpu_limit: None,
                            memory_request: None,
                            memory_limit: None,
                        },
                        env_vars,
                        volume_mounts,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    // Build conditions
    let conditions: Vec<PodCondition> = status
        .and_then(|s| s.conditions.as_ref())
        .map(|conds| {
            conds.iter().map(|c| PodCondition {
                type_: c.type_.clone(),
                status: c.status.clone(),
                reason: c.reason.clone(),
                message: c.message.clone(),
                last_transition_time: c.last_transition_time.as_ref()
                    .map(|t| t.0.to_rfc3339()),
            }).collect()
        })
        .unwrap_or_default();

    // Build volumes
    let volumes: Vec<VolumeInfo> = spec
        .and_then(|s| s.volumes.as_ref())
        .map(|vols| {
            vols.iter().map(|v| {
                let volume_type = get_volume_type(v);
                let source_name: Option<String> = if let Some(cm) = &v.config_map {
                    Some(cm.name.clone())
                } else if let Some(secret) = &v.secret {
                    secret.secret_name.clone()
                } else if let Some(pvc) = &v.persistent_volume_claim {
                    Some(pvc.claim_name.clone())
                } else if let Some(hp) = &v.host_path {
                    Some(hp.path.clone())
                } else if let Some(nfs) = &v.nfs {
                    Some(format!("{}:{}", nfs.server, nfs.path))
                } else {
                    None
                };
                VolumeInfo {
                    name: v.name.clone(),
                    volume_type,
                    source_name,
                }
            }).collect()
        })
        .unwrap_or_default();

    Ok(PodDetail {
        name: metadata.name.clone().unwrap_or_default(),
        namespace: metadata.namespace.clone().unwrap_or_default(),
        uid: metadata.uid.clone().unwrap_or_default(),
        creation_timestamp: metadata.creation_timestamp.as_ref()
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.clone().unwrap_or_default(),
        annotations: metadata.annotations.clone().unwrap_or_default(),
        status: get_pod_phase(status),
        phase: status.and_then(|s| s.phase.clone()).unwrap_or_default(),
        pod_ip: status.and_then(|s| s.pod_ip.clone()),
        node_name: spec.and_then(|s| s.node_name.clone()),
        service_account: spec.and_then(|s| s.service_account_name.clone()),
        conditions,
        containers,
        init_containers,
        volumes,
    })
}

fn get_container_state_detail(status: Option<&k8s_openapi::api::core::v1::ContainerStatus>) -> (String, Option<String>, Option<String>) {
    status
        .and_then(|s| s.state.as_ref())
        .map(|state| {
            if let Some(running) = &state.running {
                let started = running.started_at.as_ref().map(|t| t.0.to_rfc3339());
                ("Running".to_string(), None, started)
            } else if let Some(waiting) = &state.waiting {
                let reason = waiting.reason.clone();
                ("Waiting".to_string(), reason, None)
            } else if let Some(terminated) = &state.terminated {
                let reason = terminated.reason.clone();
                let finished = terminated.finished_at.as_ref().map(|t| t.0.to_rfc3339());
                ("Terminated".to_string(), reason, finished)
            } else {
                ("Unknown".to_string(), None, None)
            }
        })
        .unwrap_or_else(|| ("Unknown".to_string(), None, None))
}

fn get_volume_type(volume: &k8s_openapi::api::core::v1::Volume) -> String {
    if volume.config_map.is_some() {
        "ConfigMap".to_string()
    } else if volume.secret.is_some() {
        "Secret".to_string()
    } else if volume.persistent_volume_claim.is_some() {
        "PersistentVolumeClaim".to_string()
    } else if volume.empty_dir.is_some() {
        "EmptyDir".to_string()
    } else if volume.host_path.is_some() {
        "HostPath".to_string()
    } else if volume.downward_api.is_some() {
        "DownwardAPI".to_string()
    } else if volume.projected.is_some() {
        "Projected".to_string()
    } else if volume.nfs.is_some() {
        "NFS".to_string()
    } else if volume.csi.is_some() {
        "CSI".to_string()
    } else {
        "Unknown".to_string()
    }
}

pub async fn get_pod_yaml(client: &Client, namespace: &str, name: &str) -> Result<String> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let pod = pods.get(name).await?;

    let yaml = serde_yaml::to_string(&pod)
        .map_err(|e| AppError::Custom(format!("Failed to serialize pod to YAML: {}", e)))?;
    Ok(yaml)
}

pub async fn get_pod_events(client: &Client, namespace: &str, pod_name: &str) -> Result<Vec<PodEvent>> {
    use k8s_openapi::api::core::v1::Event;

    let events: Api<Event> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default()
        .fields(&format!("involvedObject.name={}", pod_name));

    let event_list = events.list(&lp).await?;

    let pod_events: Vec<PodEvent> = event_list
        .items
        .iter()
        .map(|e| {
            let source = e.source.as_ref()
                .map(|s| format!("{}/{}", s.component.as_deref().unwrap_or(""), s.host.as_deref().unwrap_or("")))
                .unwrap_or_default();

            PodEvent {
                type_: e.type_.clone().unwrap_or_default(),
                reason: e.reason.clone().unwrap_or_default(),
                message: e.message.clone().unwrap_or_default(),
                count: e.count.unwrap_or(1),
                first_timestamp: e.first_timestamp.as_ref().map(|t| get_age(Some(t))),
                last_timestamp: e.last_timestamp.as_ref().map(|t| get_age(Some(t))),
                source,
            }
        })
        .collect();

    Ok(pod_events)
}

// ============ ConfigMap Detail ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMapDetail {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub data: std::collections::BTreeMap<String, String>,
    pub binary_data_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMapEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

pub async fn get_configmap_detail(client: &Client, namespace: &str, name: &str) -> Result<ConfigMapDetail> {
    let cms: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
    let cm = cms.get(name).await?;

    let metadata = cm.metadata;

    Ok(ConfigMapDetail {
        name: metadata.name.unwrap_or_default(),
        namespace: metadata.namespace.unwrap_or_default(),
        uid: metadata.uid.unwrap_or_default(),
        creation_timestamp: metadata.creation_timestamp
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.unwrap_or_default(),
        annotations: metadata.annotations.unwrap_or_default(),
        data: cm.data.unwrap_or_default(),
        binary_data_keys: cm.binary_data.map(|b| b.keys().cloned().collect()).unwrap_or_default(),
    })
}

pub async fn get_configmap_yaml(client: &Client, namespace: &str, name: &str) -> Result<String> {
    let cms: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
    let cm = cms.get(name).await?;
    Ok(serde_yaml::to_string(&cm)?)
}

pub async fn get_configmap_events(client: &Client, namespace: &str, name: &str) -> Result<Vec<ConfigMapEvent>> {
    let events: Api<k8s_openapi::api::core::v1::Event> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().fields(&format!("involvedObject.name={},involvedObject.kind=ConfigMap", name));
    let event_list = events.list(&lp).await?;

    Ok(event_list.items.iter().map(|e| ConfigMapEvent {
        event_type: e.type_.clone().unwrap_or_else(|| "Normal".to_string()),
        reason: e.reason.clone().unwrap_or_default(),
        message: e.message.clone().unwrap_or_default(),
        count: e.count.unwrap_or(1),
        first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        source: e.source.as_ref().map(|s| s.component.clone().unwrap_or_default()).unwrap_or_default(),
    }).collect())
}

// ============ Secret Detail ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretDetail {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub secret_type: String,
    pub data_keys: Vec<String>,
    pub immutable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

pub async fn get_secret_detail(client: &Client, namespace: &str, name: &str) -> Result<SecretDetail> {
    let secrets: Api<k8s_openapi::api::core::v1::Secret> = Api::namespaced(client.clone(), namespace);
    let secret = secrets.get(name).await?;

    let metadata = secret.metadata;

    Ok(SecretDetail {
        name: metadata.name.unwrap_or_default(),
        namespace: metadata.namespace.unwrap_or_default(),
        uid: metadata.uid.unwrap_or_default(),
        creation_timestamp: metadata.creation_timestamp
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.unwrap_or_default(),
        annotations: metadata.annotations.unwrap_or_default(),
        secret_type: secret.type_.unwrap_or_else(|| "Opaque".to_string()),
        data_keys: secret.data.map(|d| d.keys().cloned().collect()).unwrap_or_default(),
        immutable: secret.immutable.unwrap_or(false),
    })
}

pub async fn get_secret_yaml(client: &Client, namespace: &str, name: &str) -> Result<String> {
    let secrets: Api<k8s_openapi::api::core::v1::Secret> = Api::namespaced(client.clone(), namespace);
    let secret = secrets.get(name).await?;
    Ok(serde_yaml::to_string(&secret)?)
}

pub async fn get_secret_data(client: &Client, namespace: &str, name: &str) -> Result<std::collections::BTreeMap<String, String>> {
    let secrets: Api<k8s_openapi::api::core::v1::Secret> = Api::namespaced(client.clone(), namespace);
    let secret = secrets.get(name).await?;

    let data = secret.data.unwrap_or_default();
    let mut decoded = std::collections::BTreeMap::new();
    for (key, value) in data {
        let decoded_value = String::from_utf8(value.0).unwrap_or_else(|_| "<binary data>".to_string());
        decoded.insert(key, decoded_value);
    }
    Ok(decoded)
}

pub async fn get_secret_events(client: &Client, namespace: &str, name: &str) -> Result<Vec<SecretEvent>> {
    let events: Api<k8s_openapi::api::core::v1::Event> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().fields(&format!("involvedObject.name={},involvedObject.kind=Secret", name));
    let event_list = events.list(&lp).await?;

    Ok(event_list.items.iter().map(|e| SecretEvent {
        event_type: e.type_.clone().unwrap_or_else(|| "Normal".to_string()),
        reason: e.reason.clone().unwrap_or_default(),
        message: e.message.clone().unwrap_or_default(),
        count: e.count.unwrap_or(1),
        first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        source: e.source.as_ref().map(|s| s.component.clone().unwrap_or_default()).unwrap_or_default(),
    }).collect())
}

// ============ Job Detail ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDetail {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub completions: Option<i32>,
    pub parallelism: Option<i32>,
    pub backoff_limit: Option<i32>,
    pub active_deadline_seconds: Option<i64>,
    pub ttl_seconds_after_finished: Option<i32>,
    pub completion_mode: String,
    pub suspend: bool,
    pub active: i32,
    pub succeeded: i32,
    pub failed: i32,
    pub start_time: Option<String>,
    pub completion_time: Option<String>,
    pub conditions: Vec<JobCondition>,
    pub selector: std::collections::BTreeMap<String, String>,
    pub container_images: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCondition {
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_transition_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

pub async fn get_job_detail(client: &Client, namespace: &str, name: &str) -> Result<JobDetail> {
    use k8s_openapi::api::batch::v1::Job;
    let jobs: Api<Job> = Api::namespaced(client.clone(), namespace);
    let job = jobs.get(name).await?;

    let metadata = job.metadata;
    let spec = job.spec.unwrap_or_default();
    let status = job.status.unwrap_or_default();

    let container_images: Vec<String> = spec.template.spec
        .map(|s| s.containers.iter().map(|c| c.image.clone().unwrap_or_default()).collect())
        .unwrap_or_default();

    let conditions = status.conditions.unwrap_or_default().iter().map(|c| JobCondition {
        condition_type: c.type_.clone(),
        status: c.status.clone(),
        reason: c.reason.clone(),
        message: c.message.clone(),
        last_transition_time: c.last_transition_time.as_ref().map(|t| t.0.to_rfc3339()),
    }).collect();

    Ok(JobDetail {
        name: metadata.name.unwrap_or_default(),
        namespace: metadata.namespace.unwrap_or_default(),
        uid: metadata.uid.unwrap_or_default(),
        creation_timestamp: metadata.creation_timestamp
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.unwrap_or_default(),
        annotations: metadata.annotations.unwrap_or_default(),
        completions: spec.completions,
        parallelism: spec.parallelism,
        backoff_limit: spec.backoff_limit,
        active_deadline_seconds: spec.active_deadline_seconds,
        ttl_seconds_after_finished: spec.ttl_seconds_after_finished,
        completion_mode: spec.completion_mode.unwrap_or_else(|| "NonIndexed".to_string()),
        suspend: spec.suspend.unwrap_or(false),
        active: status.active.unwrap_or(0),
        succeeded: status.succeeded.unwrap_or(0),
        failed: status.failed.unwrap_or(0),
        start_time: status.start_time.as_ref().map(|t| t.0.to_rfc3339()),
        completion_time: status.completion_time.as_ref().map(|t| t.0.to_rfc3339()),
        conditions,
        selector: spec.selector.map(|s| s.match_labels.unwrap_or_default()).unwrap_or_default(),
        container_images,
    })
}

pub async fn get_job_yaml(client: &Client, namespace: &str, name: &str) -> Result<String> {
    use k8s_openapi::api::batch::v1::Job;
    let jobs: Api<Job> = Api::namespaced(client.clone(), namespace);
    let job = jobs.get(name).await?;
    Ok(serde_yaml::to_string(&job)?)
}

pub async fn get_job_events(client: &Client, namespace: &str, name: &str) -> Result<Vec<JobEvent>> {
    let events: Api<k8s_openapi::api::core::v1::Event> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().fields(&format!("involvedObject.name={},involvedObject.kind=Job", name));
    let event_list = events.list(&lp).await?;

    Ok(event_list.items.iter().map(|e| JobEvent {
        event_type: e.type_.clone().unwrap_or_else(|| "Normal".to_string()),
        reason: e.reason.clone().unwrap_or_default(),
        message: e.message.clone().unwrap_or_default(),
        count: e.count.unwrap_or(1),
        first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        source: e.source.as_ref().map(|s| s.component.clone().unwrap_or_default()).unwrap_or_default(),
    }).collect())
}

pub async fn get_job_pods(client: &Client, namespace: &str, job_name: &str) -> Result<Vec<PodInfo>> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().labels(&format!("job-name={}", job_name));
    let pod_list = pods.list(&lp).await?;
    Ok(pod_list.items.iter().map(pod_to_pod_info).collect())
}

// ============ CronJob Detail ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobDetail {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub schedule: String,
    pub timezone: Option<String>,
    pub suspend: bool,
    pub concurrency_policy: String,
    pub starting_deadline_seconds: Option<i64>,
    pub successful_jobs_history_limit: i32,
    pub failed_jobs_history_limit: i32,
    pub last_schedule_time: Option<String>,
    pub last_successful_time: Option<String>,
    pub active_jobs: Vec<String>,
    pub container_images: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

pub async fn get_cronjob_detail(client: &Client, namespace: &str, name: &str) -> Result<CronJobDetail> {
    use k8s_openapi::api::batch::v1::CronJob;
    let cronjobs: Api<CronJob> = Api::namespaced(client.clone(), namespace);
    let cj = cronjobs.get(name).await?;

    let metadata = cj.metadata;
    let spec = cj.spec.unwrap_or_default();
    let status = cj.status.unwrap_or_default();

    let container_images: Vec<String> = spec.job_template.spec
        .and_then(|js| js.template.spec)
        .map(|s| s.containers.iter().map(|c| c.image.clone().unwrap_or_default()).collect())
        .unwrap_or_default();

    let active_jobs: Vec<String> = status.active.unwrap_or_default()
        .iter()
        .map(|r| r.name.clone().unwrap_or_default())
        .collect();

    Ok(CronJobDetail {
        name: metadata.name.unwrap_or_default(),
        namespace: metadata.namespace.unwrap_or_default(),
        uid: metadata.uid.unwrap_or_default(),
        creation_timestamp: metadata.creation_timestamp
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.unwrap_or_default(),
        annotations: metadata.annotations.unwrap_or_default(),
        schedule: spec.schedule,
        timezone: spec.time_zone,
        suspend: spec.suspend.unwrap_or(false),
        concurrency_policy: spec.concurrency_policy.unwrap_or_else(|| "Allow".to_string()),
        starting_deadline_seconds: spec.starting_deadline_seconds,
        successful_jobs_history_limit: spec.successful_jobs_history_limit.unwrap_or(3),
        failed_jobs_history_limit: spec.failed_jobs_history_limit.unwrap_or(1),
        last_schedule_time: status.last_schedule_time.as_ref().map(|t| t.0.to_rfc3339()),
        last_successful_time: status.last_successful_time.as_ref().map(|t| t.0.to_rfc3339()),
        active_jobs,
        container_images,
    })
}

pub async fn get_cronjob_yaml(client: &Client, namespace: &str, name: &str) -> Result<String> {
    use k8s_openapi::api::batch::v1::CronJob;
    let cronjobs: Api<CronJob> = Api::namespaced(client.clone(), namespace);
    let cj = cronjobs.get(name).await?;
    Ok(serde_yaml::to_string(&cj)?)
}

pub async fn get_cronjob_events(client: &Client, namespace: &str, name: &str) -> Result<Vec<CronJobEvent>> {
    let events: Api<k8s_openapi::api::core::v1::Event> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().fields(&format!("involvedObject.name={},involvedObject.kind=CronJob", name));
    let event_list = events.list(&lp).await?;

    Ok(event_list.items.iter().map(|e| CronJobEvent {
        event_type: e.type_.clone().unwrap_or_else(|| "Normal".to_string()),
        reason: e.reason.clone().unwrap_or_default(),
        message: e.message.clone().unwrap_or_default(),
        count: e.count.unwrap_or(1),
        first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        source: e.source.as_ref().map(|s| s.component.clone().unwrap_or_default()).unwrap_or_default(),
    }).collect())
}

// ============ DaemonSet Detail ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSetDetail {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub desired_number_scheduled: i32,
    pub current_number_scheduled: i32,
    pub number_ready: i32,
    pub number_available: i32,
    pub number_misscheduled: i32,
    pub updated_number_scheduled: i32,
    pub update_strategy: String,
    pub min_ready_seconds: i32,
    pub selector: std::collections::BTreeMap<String, String>,
    pub conditions: Vec<DaemonSetCondition>,
    pub container_images: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSetCondition {
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_transition_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSetEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

pub async fn get_daemonset_detail(client: &Client, namespace: &str, name: &str) -> Result<DaemonSetDetail> {
    let daemonsets: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
    let ds = daemonsets.get(name).await?;

    let metadata = ds.metadata;
    let spec = ds.spec.unwrap_or_default();
    let status = ds.status.unwrap_or_default();

    let container_images: Vec<String> = spec.template.spec
        .map(|s| s.containers.iter().map(|c| c.image.clone().unwrap_or_default()).collect())
        .unwrap_or_default();

    let conditions = status.conditions.unwrap_or_default().iter().map(|c| DaemonSetCondition {
        condition_type: c.type_.clone(),
        status: c.status.clone(),
        reason: c.reason.clone(),
        message: c.message.clone(),
        last_transition_time: c.last_transition_time.as_ref().map(|t| t.0.to_rfc3339()),
    }).collect();

    let update_strategy = spec.update_strategy
        .and_then(|s| s.type_)
        .unwrap_or_else(|| "RollingUpdate".to_string());

    Ok(DaemonSetDetail {
        name: metadata.name.unwrap_or_default(),
        namespace: metadata.namespace.unwrap_or_default(),
        uid: metadata.uid.unwrap_or_default(),
        creation_timestamp: metadata.creation_timestamp
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.unwrap_or_default(),
        annotations: metadata.annotations.unwrap_or_default(),
        desired_number_scheduled: status.desired_number_scheduled,
        current_number_scheduled: status.current_number_scheduled,
        number_ready: status.number_ready,
        number_available: status.number_available.unwrap_or(0),
        number_misscheduled: status.number_misscheduled,
        updated_number_scheduled: status.updated_number_scheduled.unwrap_or(0),
        update_strategy,
        min_ready_seconds: spec.min_ready_seconds.unwrap_or(0),
        selector: spec.selector.match_labels.unwrap_or_default(),
        conditions,
        container_images,
    })
}

pub async fn get_daemonset_yaml(client: &Client, namespace: &str, name: &str) -> Result<String> {
    let daemonsets: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
    let ds = daemonsets.get(name).await?;
    Ok(serde_yaml::to_string(&ds)?)
}

pub async fn get_daemonset_events(client: &Client, namespace: &str, name: &str) -> Result<Vec<DaemonSetEvent>> {
    let events: Api<k8s_openapi::api::core::v1::Event> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().fields(&format!("involvedObject.name={},involvedObject.kind=DaemonSet", name));
    let event_list = events.list(&lp).await?;

    Ok(event_list.items.iter().map(|e| DaemonSetEvent {
        event_type: e.type_.clone().unwrap_or_else(|| "Normal".to_string()),
        reason: e.reason.clone().unwrap_or_default(),
        message: e.message.clone().unwrap_or_default(),
        count: e.count.unwrap_or(1),
        first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        source: e.source.as_ref().map(|s| s.component.clone().unwrap_or_default()).unwrap_or_default(),
    }).collect())
}

pub async fn get_daemonset_pods(client: &Client, namespace: &str, daemonset_name: &str) -> Result<Vec<PodInfo>> {
    let daemonsets: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);
    let ds = daemonsets.get(daemonset_name).await?;

    let selector = ds.spec
        .and_then(|s| s.selector.match_labels)
        .unwrap_or_default();

    if selector.is_empty() {
        return Ok(vec![]);
    }

    let label_selector = selector.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(",");

    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().labels(&label_selector);
    let pod_list = pods.list(&lp).await?;
    Ok(pod_list.items.iter().map(pod_to_pod_info).collect())
}

// ============ ReplicaSet Detail ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaSetDetail {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub replicas: i32,
    pub ready_replicas: i32,
    pub available_replicas: i32,
    pub fully_labeled_replicas: i32,
    pub owner_references: Vec<OwnerRef>,
    pub selector: std::collections::BTreeMap<String, String>,
    pub conditions: Vec<ReplicaSetCondition>,
    pub container_images: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerRef {
    pub kind: String,
    pub name: String,
    pub uid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaSetCondition {
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_transition_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaSetEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

pub async fn get_replicaset_detail(client: &Client, namespace: &str, name: &str) -> Result<ReplicaSetDetail> {
    let replicasets: Api<ReplicaSet> = Api::namespaced(client.clone(), namespace);
    let rs = replicasets.get(name).await?;

    let metadata = rs.metadata;
    let spec = rs.spec.unwrap_or_default();
    let status = rs.status.unwrap_or_default();

    let container_images: Vec<String> = spec.template
        .and_then(|t| t.spec)
        .map(|s| s.containers.iter().map(|c| c.image.clone().unwrap_or_default()).collect())
        .unwrap_or_default();

    let owner_references = metadata.owner_references.clone().unwrap_or_default()
        .iter()
        .map(|o| OwnerRef {
            kind: o.kind.clone(),
            name: o.name.clone(),
            uid: o.uid.clone(),
        })
        .collect();

    let conditions = status.conditions.unwrap_or_default().iter().map(|c| ReplicaSetCondition {
        condition_type: c.type_.clone(),
        status: c.status.clone(),
        reason: c.reason.clone(),
        message: c.message.clone(),
        last_transition_time: c.last_transition_time.as_ref().map(|t| t.0.to_rfc3339()),
    }).collect();

    Ok(ReplicaSetDetail {
        name: metadata.name.unwrap_or_default(),
        namespace: metadata.namespace.unwrap_or_default(),
        uid: metadata.uid.unwrap_or_default(),
        creation_timestamp: metadata.creation_timestamp
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.unwrap_or_default(),
        annotations: metadata.annotations.unwrap_or_default(),
        replicas: status.replicas,
        ready_replicas: status.ready_replicas.unwrap_or(0),
        available_replicas: status.available_replicas.unwrap_or(0),
        fully_labeled_replicas: status.fully_labeled_replicas.unwrap_or(0),
        owner_references,
        selector: spec.selector.match_labels.unwrap_or_default(),
        conditions,
        container_images,
    })
}

pub async fn get_replicaset_yaml(client: &Client, namespace: &str, name: &str) -> Result<String> {
    let replicasets: Api<ReplicaSet> = Api::namespaced(client.clone(), namespace);
    let rs = replicasets.get(name).await?;
    Ok(serde_yaml::to_string(&rs)?)
}

pub async fn get_replicaset_events(client: &Client, namespace: &str, name: &str) -> Result<Vec<ReplicaSetEvent>> {
    let events: Api<k8s_openapi::api::core::v1::Event> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().fields(&format!("involvedObject.name={},involvedObject.kind=ReplicaSet", name));
    let event_list = events.list(&lp).await?;

    Ok(event_list.items.iter().map(|e| ReplicaSetEvent {
        event_type: e.type_.clone().unwrap_or_else(|| "Normal".to_string()),
        reason: e.reason.clone().unwrap_or_default(),
        message: e.message.clone().unwrap_or_default(),
        count: e.count.unwrap_or(1),
        first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        source: e.source.as_ref().map(|s| s.component.clone().unwrap_or_default()).unwrap_or_default(),
    }).collect())
}

pub async fn get_replicaset_pods(client: &Client, namespace: &str, replicaset_name: &str) -> Result<Vec<PodInfo>> {
    let replicasets: Api<ReplicaSet> = Api::namespaced(client.clone(), namespace);
    let rs = replicasets.get(replicaset_name).await?;

    let selector = rs.spec
        .and_then(|s| s.selector.match_labels)
        .unwrap_or_default();

    if selector.is_empty() {
        return Ok(vec![]);
    }

    let label_selector = selector.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(",");

    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().labels(&label_selector);
    let pod_list = pods.list(&lp).await?;
    Ok(pod_list.items.iter().map(pod_to_pod_info).collect())
}

// ============ NetworkPolicy Detail ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicyDetail {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub pod_selector: std::collections::BTreeMap<String, String>,
    pub policy_types: Vec<String>,
    pub ingress_rules: Vec<NetworkPolicyIngressRule>,
    pub egress_rules: Vec<NetworkPolicyEgressRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicyIngressRule {
    pub from: Vec<NetworkPolicyPeer>,
    pub ports: Vec<NetworkPolicyPort>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicyEgressRule {
    pub to: Vec<NetworkPolicyPeer>,
    pub ports: Vec<NetworkPolicyPort>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicyPeer {
    pub peer_type: String,
    pub namespace_selector: std::collections::BTreeMap<String, String>,
    pub pod_selector: std::collections::BTreeMap<String, String>,
    pub ip_block: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicyPort {
    pub protocol: String,
    pub port: String,
    pub end_port: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicyEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

pub async fn get_networkpolicy_detail(client: &Client, namespace: &str, name: &str) -> Result<NetworkPolicyDetail> {
    use k8s_openapi::api::networking::v1::NetworkPolicy;
    let netpols: Api<NetworkPolicy> = Api::namespaced(client.clone(), namespace);
    let np = netpols.get(name).await?;

    let metadata = np.metadata;
    let spec = np.spec.unwrap_or_default();

    let pod_selector = spec.pod_selector.match_labels.unwrap_or_default();
    let policy_types = spec.policy_types.unwrap_or_default();

    let ingress_rules = spec.ingress.unwrap_or_default().iter().map(|r| {
        let from = r.from.clone().unwrap_or_default().iter().map(|p| {
            let peer_type = if p.ip_block.is_some() {
                "IPBlock".to_string()
            } else if p.namespace_selector.is_some() && p.pod_selector.is_some() {
                "NamespaceAndPod".to_string()
            } else if p.namespace_selector.is_some() {
                "Namespace".to_string()
            } else if p.pod_selector.is_some() {
                "Pod".to_string()
            } else {
                "Unknown".to_string()
            };

            NetworkPolicyPeer {
                peer_type,
                namespace_selector: p.namespace_selector.as_ref()
                    .and_then(|s| s.match_labels.clone())
                    .unwrap_or_default(),
                pod_selector: p.pod_selector.as_ref()
                    .and_then(|s| s.match_labels.clone())
                    .unwrap_or_default(),
                ip_block: p.ip_block.as_ref().map(|b| {
                    let excepts = b.except.as_ref()
                        .map(|e| format!(" (except: {})", e.join(", ")))
                        .unwrap_or_default();
                    format!("{}{}", b.cidr, excepts)
                }),
            }
        }).collect();

        let ports = r.ports.clone().unwrap_or_default().iter().map(|p| {
            NetworkPolicyPort {
                protocol: p.protocol.clone().unwrap_or_else(|| "TCP".to_string()),
                port: p.port.as_ref().map(|p| format!("{:?}", p)).unwrap_or_else(|| "All".to_string()),
                end_port: p.end_port,
            }
        }).collect();

        NetworkPolicyIngressRule { from, ports }
    }).collect();

    let egress_rules = spec.egress.unwrap_or_default().iter().map(|r| {
        let to = r.to.clone().unwrap_or_default().iter().map(|p| {
            let peer_type = if p.ip_block.is_some() {
                "IPBlock".to_string()
            } else if p.namespace_selector.is_some() && p.pod_selector.is_some() {
                "NamespaceAndPod".to_string()
            } else if p.namespace_selector.is_some() {
                "Namespace".to_string()
            } else if p.pod_selector.is_some() {
                "Pod".to_string()
            } else {
                "Unknown".to_string()
            };

            NetworkPolicyPeer {
                peer_type,
                namespace_selector: p.namespace_selector.as_ref()
                    .and_then(|s| s.match_labels.clone())
                    .unwrap_or_default(),
                pod_selector: p.pod_selector.as_ref()
                    .and_then(|s| s.match_labels.clone())
                    .unwrap_or_default(),
                ip_block: p.ip_block.as_ref().map(|b| {
                    let excepts = b.except.as_ref()
                        .map(|e| format!(" (except: {})", e.join(", ")))
                        .unwrap_or_default();
                    format!("{}{}", b.cidr, excepts)
                }),
            }
        }).collect();

        let ports = r.ports.clone().unwrap_or_default().iter().map(|p| {
            NetworkPolicyPort {
                protocol: p.protocol.clone().unwrap_or_else(|| "TCP".to_string()),
                port: p.port.as_ref().map(|p| format!("{:?}", p)).unwrap_or_else(|| "All".to_string()),
                end_port: p.end_port,
            }
        }).collect();

        NetworkPolicyEgressRule { to, ports }
    }).collect();

    Ok(NetworkPolicyDetail {
        name: metadata.name.unwrap_or_default(),
        namespace: metadata.namespace.unwrap_or_default(),
        uid: metadata.uid.unwrap_or_default(),
        creation_timestamp: metadata.creation_timestamp
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.unwrap_or_default(),
        annotations: metadata.annotations.unwrap_or_default(),
        pod_selector,
        policy_types,
        ingress_rules,
        egress_rules,
    })
}

pub async fn get_networkpolicy_yaml(client: &Client, namespace: &str, name: &str) -> Result<String> {
    use k8s_openapi::api::networking::v1::NetworkPolicy;
    let netpols: Api<NetworkPolicy> = Api::namespaced(client.clone(), namespace);
    let np = netpols.get(name).await?;
    Ok(serde_yaml::to_string(&np)?)
}

pub async fn get_networkpolicy_events(client: &Client, namespace: &str, name: &str) -> Result<Vec<NetworkPolicyEvent>> {
    let events: Api<k8s_openapi::api::core::v1::Event> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().fields(&format!("involvedObject.name={},involvedObject.kind=NetworkPolicy", name));
    let event_list = events.list(&lp).await?;

    Ok(event_list.items.iter().map(|e| NetworkPolicyEvent {
        event_type: e.type_.clone().unwrap_or_else(|| "Normal".to_string()),
        reason: e.reason.clone().unwrap_or_default(),
        message: e.message.clone().unwrap_or_default(),
        count: e.count.unwrap_or(1),
        first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        source: e.source.as_ref().map(|s| s.component.clone().unwrap_or_default()).unwrap_or_default(),
    }).collect())
}

// ============ HPA Detail ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HPADetail {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub scale_target_ref: String,
    pub min_replicas: i32,
    pub max_replicas: i32,
    pub current_replicas: i32,
    pub desired_replicas: i32,
    pub metrics: Vec<HPAMetric>,
    pub conditions: Vec<HPACondition>,
    pub last_scale_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HPAMetric {
    pub metric_type: String,
    pub name: String,
    pub current_value: String,
    pub target_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HPACondition {
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_transition_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HPAEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

pub async fn get_hpa_detail(client: &Client, namespace: &str, name: &str) -> Result<HPADetail> {
    use k8s_openapi::api::autoscaling::v2::HorizontalPodAutoscaler;
    let hpas: Api<HorizontalPodAutoscaler> = Api::namespaced(client.clone(), namespace);
    let hpa = hpas.get(name).await?;

    let metadata = hpa.metadata;
    let spec = hpa.spec.unwrap_or_default();
    let status = hpa.status.unwrap_or_default();

    let scale_target_ref = format!("{}/{}",
        spec.scale_target_ref.kind,
        spec.scale_target_ref.name
    );

    let metrics: Vec<HPAMetric> = spec.metrics.unwrap_or_default().iter().map(|m| {
        match m.type_.as_str() {
            "Resource" => {
                let resource = m.resource.as_ref();
                HPAMetric {
                    metric_type: "Resource".to_string(),
                    name: resource.map(|r| r.name.clone()).unwrap_or_default(),
                    current_value: "-".to_string(),
                    target_value: resource
                        .and_then(|r| r.target.average_utilization.map(|v| format!("{}%", v)))
                        .or_else(|| resource.and_then(|r| r.target.average_value.as_ref().map(|v| v.0.clone())))
                        .unwrap_or_else(|| "-".to_string()),
                }
            },
            "Pods" => {
                let pods = m.pods.as_ref();
                HPAMetric {
                    metric_type: "Pods".to_string(),
                    name: pods.map(|p| p.metric.name.clone()).unwrap_or_default(),
                    current_value: "-".to_string(),
                    target_value: pods
                        .and_then(|p| p.target.average_value.as_ref().map(|v| v.0.clone()))
                        .unwrap_or_else(|| "-".to_string()),
                }
            },
            "Object" => {
                let obj = m.object.as_ref();
                HPAMetric {
                    metric_type: "Object".to_string(),
                    name: obj.map(|o| o.metric.name.clone()).unwrap_or_default(),
                    current_value: "-".to_string(),
                    target_value: obj
                        .and_then(|o| o.target.value.as_ref().map(|v| v.0.clone()))
                        .unwrap_or_else(|| "-".to_string()),
                }
            },
            "External" => {
                let ext = m.external.as_ref();
                HPAMetric {
                    metric_type: "External".to_string(),
                    name: ext.map(|e| e.metric.name.clone()).unwrap_or_default(),
                    current_value: "-".to_string(),
                    target_value: ext
                        .and_then(|e| e.target.value.as_ref().map(|v| v.0.clone()))
                        .unwrap_or_else(|| "-".to_string()),
                }
            },
            _ => HPAMetric {
                metric_type: m.type_.clone(),
                name: "-".to_string(),
                current_value: "-".to_string(),
                target_value: "-".to_string(),
            }
        }
    }).collect();

    let conditions = status.conditions.unwrap_or_default().iter().map(|c| HPACondition {
        condition_type: c.type_.clone(),
        status: c.status.clone(),
        reason: c.reason.clone(),
        message: c.message.clone(),
        last_transition_time: c.last_transition_time.as_ref().map(|t| t.0.to_rfc3339()),
    }).collect();

    Ok(HPADetail {
        name: metadata.name.unwrap_or_default(),
        namespace: metadata.namespace.unwrap_or_default(),
        uid: metadata.uid.unwrap_or_default(),
        creation_timestamp: metadata.creation_timestamp
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.unwrap_or_default(),
        annotations: metadata.annotations.unwrap_or_default(),
        scale_target_ref,
        min_replicas: spec.min_replicas.unwrap_or(1),
        max_replicas: spec.max_replicas,
        current_replicas: status.current_replicas.unwrap_or(0),
        desired_replicas: status.desired_replicas,
        metrics,
        conditions,
        last_scale_time: status.last_scale_time.as_ref().map(|t| t.0.to_rfc3339()),
    })
}

pub async fn get_hpa_yaml(client: &Client, namespace: &str, name: &str) -> Result<String> {
    use k8s_openapi::api::autoscaling::v2::HorizontalPodAutoscaler;
    let hpas: Api<HorizontalPodAutoscaler> = Api::namespaced(client.clone(), namespace);
    let hpa = hpas.get(name).await?;
    Ok(serde_yaml::to_string(&hpa)?)
}

pub async fn get_hpa_events(client: &Client, namespace: &str, name: &str) -> Result<Vec<HPAEvent>> {
    let events: Api<k8s_openapi::api::core::v1::Event> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().fields(&format!("involvedObject.name={},involvedObject.kind=HorizontalPodAutoscaler", name));
    let event_list = events.list(&lp).await?;

    Ok(event_list.items.iter().map(|e| HPAEvent {
        event_type: e.type_.clone().unwrap_or_else(|| "Normal".to_string()),
        reason: e.reason.clone().unwrap_or_default(),
        message: e.message.clone().unwrap_or_default(),
        count: e.count.unwrap_or(1),
        first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        source: e.source.as_ref().map(|s| s.component.clone().unwrap_or_default()).unwrap_or_default(),
    }).collect())
}

// ============ PersistentVolume Detail ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PVDetail {
    pub name: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub capacity: String,
    pub access_modes: Vec<String>,
    pub reclaim_policy: String,
    pub status: String,
    pub claim: Option<String>,
    pub storage_class: String,
    pub volume_mode: String,
    pub mount_options: Vec<String>,
    pub source_type: String,
    pub source_details: std::collections::BTreeMap<String, String>,
    pub node_affinity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PVEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

pub async fn get_pv_detail(client: &Client, name: &str) -> Result<PVDetail> {
    let pvs: Api<PersistentVolume> = Api::all(client.clone());
    let pv = pvs.get(name).await?;

    let metadata = pv.metadata;
    let spec = pv.spec.unwrap_or_default();
    let status = pv.status.unwrap_or_default();

    let capacity = spec.capacity.as_ref()
        .and_then(|c| c.get("storage").map(|q| q.0.clone()))
        .unwrap_or_default();

    let claim = spec.claim_ref.as_ref().map(|c| {
        format!("{}/{}", c.namespace.clone().unwrap_or_default(), c.name.clone().unwrap_or_default())
    });

    let (source_type, source_details) = get_pv_source_info(&spec);

    let node_affinity = spec.node_affinity.as_ref().map(|_| "Configured".to_string());

    Ok(PVDetail {
        name: metadata.name.unwrap_or_default(),
        uid: metadata.uid.unwrap_or_default(),
        creation_timestamp: metadata.creation_timestamp
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.unwrap_or_default(),
        annotations: metadata.annotations.unwrap_or_default(),
        capacity,
        access_modes: spec.access_modes.unwrap_or_default(),
        reclaim_policy: spec.persistent_volume_reclaim_policy.unwrap_or_default(),
        status: status.phase.unwrap_or_default(),
        claim,
        storage_class: spec.storage_class_name.unwrap_or_default(),
        volume_mode: spec.volume_mode.unwrap_or_else(|| "Filesystem".to_string()),
        mount_options: spec.mount_options.unwrap_or_default(),
        source_type,
        source_details,
        node_affinity,
    })
}

fn get_pv_source_info(spec: &k8s_openapi::api::core::v1::PersistentVolumeSpec) -> (String, std::collections::BTreeMap<String, String>) {
    let mut details = std::collections::BTreeMap::new();

    if let Some(h) = &spec.host_path {
        details.insert("path".to_string(), h.path.clone());
        if let Some(t) = &h.type_ {
            details.insert("type".to_string(), t.clone());
        }
        return ("HostPath".to_string(), details);
    }

    if let Some(n) = &spec.nfs {
        details.insert("server".to_string(), n.server.clone());
        details.insert("path".to_string(), n.path.clone());
        return ("NFS".to_string(), details);
    }

    if let Some(c) = &spec.csi {
        details.insert("driver".to_string(), c.driver.clone());
        details.insert("volumeHandle".to_string(), c.volume_handle.clone());
        return ("CSI".to_string(), details);
    }

    if let Some(l) = &spec.local {
        details.insert("path".to_string(), l.path.clone());
        return ("Local".to_string(), details);
    }

    if spec.aws_elastic_block_store.is_some() {
        return ("AWSElasticBlockStore".to_string(), details);
    }

    if spec.gce_persistent_disk.is_some() {
        return ("GCEPersistentDisk".to_string(), details);
    }

    if spec.azure_disk.is_some() {
        return ("AzureDisk".to_string(), details);
    }

    ("Unknown".to_string(), details)
}

pub async fn get_pv_yaml(client: &Client, name: &str) -> Result<String> {
    let pvs: Api<PersistentVolume> = Api::all(client.clone());
    let pv = pvs.get(name).await?;
    Ok(serde_yaml::to_string(&pv)?)
}

pub async fn get_pv_events(client: &Client, name: &str) -> Result<Vec<PVEvent>> {
    let events: Api<k8s_openapi::api::core::v1::Event> = Api::all(client.clone());
    let lp = ListParams::default().fields(&format!("involvedObject.name={},involvedObject.kind=PersistentVolume", name));
    let event_list = events.list(&lp).await?;

    Ok(event_list.items.iter().map(|e| PVEvent {
        event_type: e.type_.clone().unwrap_or_else(|| "Normal".to_string()),
        reason: e.reason.clone().unwrap_or_default(),
        message: e.message.clone().unwrap_or_default(),
        count: e.count.unwrap_or(1),
        first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        source: e.source.as_ref().map(|s| s.component.clone().unwrap_or_default()).unwrap_or_default(),
    }).collect())
}

// ============ PersistentVolumeClaim Detail ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PVCDetail {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub status: String,
    pub volume_name: Option<String>,
    pub storage_class: Option<String>,
    pub access_modes: Vec<String>,
    pub volume_mode: String,
    pub requested_storage: String,
    pub capacity: String,
    pub conditions: Vec<PVCCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PVCCondition {
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_transition_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PVCEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

pub async fn get_pvc_detail(client: &Client, namespace: &str, name: &str) -> Result<PVCDetail> {
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);
    let pvc = pvcs.get(name).await?;

    let metadata = pvc.metadata;
    let spec = pvc.spec.unwrap_or_default();
    let status = pvc.status.unwrap_or_default();

    let requested_storage = spec.resources
        .and_then(|r| r.requests)
        .and_then(|req| req.get("storage").map(|q| q.0.clone()))
        .unwrap_or_default();

    let capacity = status.capacity
        .and_then(|c| c.get("storage").map(|q| q.0.clone()))
        .unwrap_or_default();

    let conditions = status.conditions.unwrap_or_default().iter().map(|c| PVCCondition {
        condition_type: c.type_.clone(),
        status: c.status.clone(),
        reason: c.reason.clone(),
        message: c.message.clone(),
        last_transition_time: c.last_probe_time.as_ref().map(|t| t.0.to_rfc3339()),
    }).collect();

    Ok(PVCDetail {
        name: metadata.name.unwrap_or_default(),
        namespace: metadata.namespace.unwrap_or_default(),
        uid: metadata.uid.unwrap_or_default(),
        creation_timestamp: metadata.creation_timestamp
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.unwrap_or_default(),
        annotations: metadata.annotations.unwrap_or_default(),
        status: status.phase.unwrap_or_default(),
        volume_name: spec.volume_name,
        storage_class: spec.storage_class_name,
        access_modes: spec.access_modes.unwrap_or_default(),
        volume_mode: spec.volume_mode.unwrap_or_else(|| "Filesystem".to_string()),
        requested_storage,
        capacity,
        conditions,
    })
}

pub async fn get_pvc_yaml(client: &Client, namespace: &str, name: &str) -> Result<String> {
    let pvcs: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), namespace);
    let pvc = pvcs.get(name).await?;
    Ok(serde_yaml::to_string(&pvc)?)
}

pub async fn get_pvc_events(client: &Client, namespace: &str, name: &str) -> Result<Vec<PVCEvent>> {
    let events: Api<k8s_openapi::api::core::v1::Event> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().fields(&format!("involvedObject.name={},involvedObject.kind=PersistentVolumeClaim", name));
    let event_list = events.list(&lp).await?;

    Ok(event_list.items.iter().map(|e| PVCEvent {
        event_type: e.type_.clone().unwrap_or_else(|| "Normal".to_string()),
        reason: e.reason.clone().unwrap_or_default(),
        message: e.message.clone().unwrap_or_default(),
        count: e.count.unwrap_or(1),
        first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        source: e.source.as_ref().map(|s| s.component.clone().unwrap_or_default()).unwrap_or_default(),
    }).collect())
}

// ============ Namespace Detail ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceDetail {
    pub name: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub status: String,
    pub conditions: Vec<NamespaceCondition>,
    pub finalizers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceCondition {
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_transition_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

pub async fn get_namespace_detail(client: &Client, name: &str) -> Result<NamespaceDetail> {
    let namespaces: Api<Namespace> = Api::all(client.clone());
    let ns = namespaces.get(name).await?;

    let metadata = ns.metadata;
    let status = ns.status.unwrap_or_default();

    let conditions = status.conditions.unwrap_or_default().iter().map(|c| NamespaceCondition {
        condition_type: c.type_.clone(),
        status: c.status.clone(),
        reason: c.reason.clone(),
        message: c.message.clone(),
        last_transition_time: c.last_transition_time.as_ref().map(|t| t.0.to_rfc3339()),
    }).collect();

    Ok(NamespaceDetail {
        name: metadata.name.unwrap_or_default(),
        uid: metadata.uid.unwrap_or_default(),
        creation_timestamp: metadata.creation_timestamp
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.unwrap_or_default(),
        annotations: metadata.annotations.unwrap_or_default(),
        status: status.phase.unwrap_or_else(|| "Active".to_string()),
        conditions,
        finalizers: metadata.finalizers.unwrap_or_default(),
    })
}

pub async fn get_namespace_yaml(client: &Client, name: &str) -> Result<String> {
    let namespaces: Api<Namespace> = Api::all(client.clone());
    let ns = namespaces.get(name).await?;
    Ok(serde_yaml::to_string(&ns)?)
}

pub async fn get_namespace_events(client: &Client, name: &str) -> Result<Vec<NamespaceEvent>> {
    let events: Api<k8s_openapi::api::core::v1::Event> = Api::all(client.clone());
    let lp = ListParams::default().fields(&format!("involvedObject.name={},involvedObject.kind=Namespace", name));
    let event_list = events.list(&lp).await?;

    Ok(event_list.items.iter().map(|e| NamespaceEvent {
        event_type: e.type_.clone().unwrap_or_else(|| "Normal".to_string()),
        reason: e.reason.clone().unwrap_or_default(),
        message: e.message.clone().unwrap_or_default(),
        count: e.count.unwrap_or(1),
        first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        source: e.source.as_ref().map(|s| s.component.clone().unwrap_or_default()).unwrap_or_default(),
    }).collect())
}

// ============ Node Detail ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDetail {
    pub name: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub taints: Vec<NodeTaint>,
    pub conditions: Vec<NodeCondition>,
    pub addresses: Vec<NodeAddress>,
    pub capacity: NodeResources,
    pub allocatable: NodeResources,
    pub node_info: NodeSystemInfo,
    pub pod_cidr: Option<String>,
    pub unschedulable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTaint {
    pub key: String,
    pub value: Option<String>,
    pub effect: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCondition {
    pub condition_type: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_transition_time: Option<String>,
    pub last_heartbeat_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAddress {
    pub address_type: String,
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResources {
    pub cpu: String,
    pub memory: String,
    pub pods: String,
    pub ephemeral_storage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSystemInfo {
    pub machine_id: String,
    pub system_uuid: String,
    pub boot_id: String,
    pub kernel_version: String,
    pub os_image: String,
    pub container_runtime_version: String,
    pub kubelet_version: String,
    pub kube_proxy_version: String,
    pub operating_system: String,
    pub architecture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

pub async fn get_node_detail(client: &Client, name: &str) -> Result<NodeDetail> {
    let nodes: Api<Node> = Api::all(client.clone());
    let node = nodes.get(name).await?;

    let metadata = node.metadata;
    let spec = node.spec.unwrap_or_default();
    let status = node.status.unwrap_or_default();

    let taints = spec.taints.unwrap_or_default().iter().map(|t| NodeTaint {
        key: t.key.clone(),
        value: t.value.clone(),
        effect: t.effect.clone(),
    }).collect();

    let conditions = status.conditions.unwrap_or_default().iter().map(|c| NodeCondition {
        condition_type: c.type_.clone(),
        status: c.status.clone(),
        reason: c.reason.clone(),
        message: c.message.clone(),
        last_transition_time: c.last_transition_time.as_ref().map(|t| t.0.to_rfc3339()),
        last_heartbeat_time: c.last_heartbeat_time.as_ref().map(|t| t.0.to_rfc3339()),
    }).collect();

    let addresses = status.addresses.unwrap_or_default().iter().map(|a| NodeAddress {
        address_type: a.type_.clone(),
        address: a.address.clone(),
    }).collect();

    let capacity_map = status.capacity.clone().unwrap_or_default();
    let capacity = NodeResources {
        cpu: capacity_map.get("cpu").map(|q| q.0.clone()).unwrap_or_default(),
        memory: capacity_map.get("memory").map(|q| q.0.clone()).unwrap_or_default(),
        pods: capacity_map.get("pods").map(|q| q.0.clone()).unwrap_or_default(),
        ephemeral_storage: capacity_map.get("ephemeral-storage").map(|q| q.0.clone()).unwrap_or_default(),
    };

    let allocatable_map = status.allocatable.clone().unwrap_or_default();
    let allocatable = NodeResources {
        cpu: allocatable_map.get("cpu").map(|q| q.0.clone()).unwrap_or_default(),
        memory: allocatable_map.get("memory").map(|q| q.0.clone()).unwrap_or_default(),
        pods: allocatable_map.get("pods").map(|q| q.0.clone()).unwrap_or_default(),
        ephemeral_storage: allocatable_map.get("ephemeral-storage").map(|q| q.0.clone()).unwrap_or_default(),
    };

    let info = status.node_info.unwrap_or_default();
    let node_info = NodeSystemInfo {
        machine_id: info.machine_id,
        system_uuid: info.system_uuid,
        boot_id: info.boot_id,
        kernel_version: info.kernel_version,
        os_image: info.os_image,
        container_runtime_version: info.container_runtime_version,
        kubelet_version: info.kubelet_version,
        kube_proxy_version: info.kube_proxy_version,
        operating_system: info.operating_system,
        architecture: info.architecture,
    };

    Ok(NodeDetail {
        name: metadata.name.unwrap_or_default(),
        uid: metadata.uid.unwrap_or_default(),
        creation_timestamp: metadata.creation_timestamp
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.unwrap_or_default(),
        annotations: metadata.annotations.unwrap_or_default(),
        taints,
        conditions,
        addresses,
        capacity,
        allocatable,
        node_info,
        pod_cidr: spec.pod_cidr,
        unschedulable: spec.unschedulable.unwrap_or(false),
    })
}

pub async fn get_node_yaml(client: &Client, name: &str) -> Result<String> {
    let nodes: Api<Node> = Api::all(client.clone());
    let node = nodes.get(name).await?;
    Ok(serde_yaml::to_string(&node)?)
}

pub async fn get_node_events(client: &Client, name: &str) -> Result<Vec<NodeEvent>> {
    let events: Api<k8s_openapi::api::core::v1::Event> = Api::all(client.clone());
    let lp = ListParams::default().fields(&format!("involvedObject.name={},involvedObject.kind=Node", name));
    let event_list = events.list(&lp).await?;

    Ok(event_list.items.iter().map(|e| NodeEvent {
        event_type: e.type_.clone().unwrap_or_else(|| "Normal".to_string()),
        reason: e.reason.clone().unwrap_or_default(),
        message: e.message.clone().unwrap_or_default(),
        count: e.count.unwrap_or(1),
        first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        source: e.source.as_ref().map(|s| s.component.clone().unwrap_or_default()).unwrap_or_default(),
    }).collect())
}

pub async fn get_node_pods(client: &Client, node_name: &str) -> Result<Vec<PodInfo>> {
    let pods: Api<Pod> = Api::all(client.clone());
    let lp = ListParams::default().fields(&format!("spec.nodeName={}", node_name));
    let pod_list = pods.list(&lp).await?;
    Ok(pod_list.items.iter().map(pod_to_pod_info).collect())
}

// ============ ServiceAccount Detail ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccountDetail {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub creation_timestamp: String,
    pub labels: std::collections::BTreeMap<String, String>,
    pub annotations: std::collections::BTreeMap<String, String>,
    pub secrets: Vec<String>,
    pub image_pull_secrets: Vec<String>,
    pub automount_service_account_token: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccountEvent {
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub source: String,
}

pub async fn get_serviceaccount_detail(client: &Client, namespace: &str, name: &str) -> Result<ServiceAccountDetail> {
    let sas: Api<ServiceAccount> = Api::namespaced(client.clone(), namespace);
    let sa = sas.get(name).await?;

    let metadata = sa.metadata;

    let secrets: Vec<String> = sa.secrets.unwrap_or_default()
        .iter()
        .map(|s| s.name.clone().unwrap_or_default())
        .collect();

    let image_pull_secrets: Vec<String> = sa.image_pull_secrets.unwrap_or_default()
        .iter()
        .map(|s| s.name.clone())
        .collect();

    Ok(ServiceAccountDetail {
        name: metadata.name.unwrap_or_default(),
        namespace: metadata.namespace.unwrap_or_default(),
        uid: metadata.uid.unwrap_or_default(),
        creation_timestamp: metadata.creation_timestamp
            .map(|t| t.0.to_rfc3339())
            .unwrap_or_default(),
        labels: metadata.labels.unwrap_or_default(),
        annotations: metadata.annotations.unwrap_or_default(),
        secrets,
        image_pull_secrets,
        automount_service_account_token: sa.automount_service_account_token,
    })
}

pub async fn get_serviceaccount_yaml(client: &Client, namespace: &str, name: &str) -> Result<String> {
    let sas: Api<ServiceAccount> = Api::namespaced(client.clone(), namespace);
    let sa = sas.get(name).await?;
    Ok(serde_yaml::to_string(&sa)?)
}

pub async fn get_serviceaccount_events(client: &Client, namespace: &str, name: &str) -> Result<Vec<ServiceAccountEvent>> {
    let events: Api<k8s_openapi::api::core::v1::Event> = Api::namespaced(client.clone(), namespace);
    let lp = ListParams::default().fields(&format!("involvedObject.name={},involvedObject.kind=ServiceAccount", name));
    let event_list = events.list(&lp).await?;

    Ok(event_list.items.iter().map(|e| ServiceAccountEvent {
        event_type: e.type_.clone().unwrap_or_else(|| "Normal".to_string()),
        reason: e.reason.clone().unwrap_or_default(),
        message: e.message.clone().unwrap_or_default(),
        count: e.count.unwrap_or(1),
        first_timestamp: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        last_timestamp: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
        source: e.source.as_ref().map(|s| s.component.clone().unwrap_or_default()).unwrap_or_default(),
    }).collect())
}

/// Create a client for a specific context
pub async fn create_client_for_context(context_name: &str) -> Result<Client> {
    let kubeconfig = kube::config::Kubeconfig::read_from(get_kubeconfig_path())?;

    // Build config options for the specific context
    let options = kube::config::KubeConfigOptions {
        context: Some(context_name.to_string()),
        ..Default::default()
    };

    let config = Config::from_custom_kubeconfig(kubeconfig, &options).await?;
    let client = Client::try_from(config)?;
    Ok(client)
}

// ============ YAML Apply Function ============

/// Apply a YAML manifest to the cluster using kubectl
/// This is the most reliable way to handle all resource types and edge cases
pub async fn apply_yaml(context_name: &str, yaml_content: &str) -> Result<String> {
    use std::process::Command;
    use std::io::Write;

    // Create a temporary file for the YAML
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("apex-kube-apply-{}.yaml", std::process::id()));

    // Write YAML to temp file
    let mut file = std::fs::File::create(&temp_file)
        .map_err(|e| AppError::Custom(format!("Failed to create temp file: {}", e)))?;
    file.write_all(yaml_content.as_bytes())
        .map_err(|e| AppError::Custom(format!("Failed to write YAML: {}", e)))?;
    drop(file);

    // Run kubectl apply
    let output = Command::new("kubectl")
        .args([
            "--context", context_name,
            "apply",
            "-f", temp_file.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| AppError::Custom(format!("Failed to execute kubectl: {}", e)))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(AppError::Custom(format!("kubectl apply failed: {}", stderr)))
    }
}
