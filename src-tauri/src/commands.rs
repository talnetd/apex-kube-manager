use crate::error::Result;
use crate::kubernetes::{
    self, ClusterEventInfo, ClusterMetrics, ConfigMapInfo, ConfigMapDetail, ConfigMapEvent,
    CronJobInfo, CronJobDetail, CronJobEvent,
    DaemonSetInfo, DaemonSetDetail, DaemonSetEvent,
    DeploymentDetail, DeploymentEvent, DeploymentInfo,
    HPAInfo, HPADetail, HPAEvent,
    IngressInfo, IngressDetail, IngressEvent,
    JobInfo, JobDetail, JobEvent,
    KubeContext, NamespaceInfo, NamespaceDetail, NamespaceEvent,
    NetworkPolicyInfo, NetworkPolicyDetail, NetworkPolicyEvent,
    NodeInfo, NodeDetail, NodeEvent,
    PersistentVolumeClaimInfo, PVCDetail, PVCEvent,
    PersistentVolumeInfo, PVDetail, PVEvent,
    PodDetail, PodEvent, PodInfo, PulseMetrics,
    ReplicaSetInfo, ReplicaSetDetail, ReplicaSetEvent,
    SecretInfo, SecretDetail, SecretEvent,
    ServiceAccountInfo, ServiceAccountDetail, ServiceAccountEvent,
    ServiceInfo, ServiceDetail, ServiceEndpoint, ServiceEvent,
    StatefulSetDetail, StatefulSetEvent, StatefulSetInfo,
};
use crate::portforward::{self, PortForwardManager, PortForwardInfo, ResourceType, AvailablePort};
use crate::watch::WatchManager;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

// Startup check commands
#[tauri::command]
pub async fn check_kubeconfig() -> Result<String> {
    kubernetes::get_kubeconfig_path_string().await
}

#[tauri::command]
pub async fn validate_kubeconfig() -> Result<()> {
    kubernetes::validate_kubeconfig().await
}

#[tauri::command]
pub async fn get_context_names() -> Result<Vec<String>> {
    kubernetes::get_context_names().await
}

#[tauri::command]
pub async fn test_cluster_connection() -> Result<()> {
    kubernetes::test_connection().await
}

#[tauri::command]
pub async fn get_contexts() -> Result<Vec<KubeContext>> {
    kubernetes::list_contexts().await
}

#[tauri::command]
pub async fn get_current_context() -> Result<String> {
    kubernetes::get_current_context_name().await
}

#[tauri::command]
pub async fn switch_context(context_name: String) -> Result<()> {
    kubernetes::switch_to_context(&context_name).await
}

#[tauri::command]
pub async fn get_namespaces() -> Result<Vec<String>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_namespaces(&client).await
}

#[tauri::command]
pub async fn get_pods(namespace: Option<String>) -> Result<Vec<PodInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_pods(&client, namespace.as_deref()).await
}

#[tauri::command]
pub async fn get_pod_logs(
    namespace: String,
    pod_name: String,
    container: Option<String>,
    tail_lines: Option<i64>,
    previous: Option<bool>,
) -> Result<String> {
    let client = kubernetes::create_client().await?;
    kubernetes::get_logs(
        &client,
        &namespace,
        &pod_name,
        container.as_deref(),
        tail_lines,
        previous,
    )
    .await
}

#[tauri::command]
pub async fn delete_pod(namespace: String, pod_name: String) -> Result<()> {
    let client = kubernetes::create_client().await?;
    kubernetes::delete_pod_by_name(&client, &namespace, &pod_name).await
}

#[tauri::command]
pub async fn get_deployments(namespace: Option<String>) -> Result<Vec<DeploymentInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_deployments(&client, namespace.as_deref()).await
}

// ============ Deployment Commands ============

#[tauri::command]
pub async fn scale_deployment(namespace: String, name: String, replicas: i32) -> Result<()> {
    let client = kubernetes::create_client().await?;
    kubernetes::scale_deployment(&client, &namespace, &name, replicas).await
}

#[tauri::command]
pub async fn restart_deployment(namespace: String, name: String) -> Result<()> {
    let client = kubernetes::create_client().await?;
    kubernetes::restart_deployment(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_deployment_detail(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<DeploymentDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_deployment_detail(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_deployment_yaml(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_deployment_yaml(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_deployment_events(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<Vec<DeploymentEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_deployment_events(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_deployment_pods(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<Vec<PodInfo>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_deployment_pods(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_statefulsets(namespace: Option<String>) -> Result<Vec<StatefulSetInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_statefulsets(&client, namespace.as_deref()).await
}

// ============ StatefulSet Commands ============

#[tauri::command]
pub async fn scale_statefulset(namespace: String, name: String, replicas: i32) -> Result<()> {
    let client = kubernetes::create_client().await?;
    kubernetes::scale_statefulset(&client, &namespace, &name, replicas).await
}

#[tauri::command]
pub async fn restart_statefulset(namespace: String, name: String) -> Result<()> {
    let client = kubernetes::create_client().await?;
    kubernetes::restart_statefulset(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_statefulset_detail(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<StatefulSetDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_statefulset_detail(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_statefulset_yaml(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_statefulset_yaml(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_statefulset_events(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<Vec<StatefulSetEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_statefulset_events(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_statefulset_pods(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<Vec<PodInfo>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_statefulset_pods(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_daemonsets(namespace: Option<String>) -> Result<Vec<DaemonSetInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_daemonsets(&client, namespace.as_deref()).await
}

#[tauri::command]
pub async fn get_replicasets(namespace: Option<String>) -> Result<Vec<ReplicaSetInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_replicasets(&client, namespace.as_deref()).await
}

#[tauri::command]
pub async fn get_jobs(namespace: Option<String>) -> Result<Vec<JobInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_jobs(&client, namespace.as_deref()).await
}

#[tauri::command]
pub async fn get_cronjobs(namespace: Option<String>) -> Result<Vec<CronJobInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_cronjobs(&client, namespace.as_deref()).await
}

#[tauri::command]
pub async fn get_services(namespace: Option<String>) -> Result<Vec<ServiceInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_services(&client, namespace.as_deref()).await
}

// ============ Service Commands ============

#[tauri::command]
pub async fn get_service_detail(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<ServiceDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_service_detail(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_service_yaml(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_service_yaml(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_service_events(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<Vec<ServiceEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_service_events(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_service_endpoints(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<Vec<ServiceEndpoint>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_service_endpoints(&client, &namespace, &name).await
}

// Network resources
#[tauri::command]
pub async fn get_ingresses(namespace: Option<String>) -> Result<Vec<IngressInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_ingresses(&client, namespace.as_deref()).await
}

// ============ Ingress Commands ============

#[tauri::command]
pub async fn get_ingress_detail(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<IngressDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_ingress_detail(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_ingress_yaml(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_ingress_yaml(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_ingress_events(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<Vec<IngressEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_ingress_events(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_network_policies(namespace: Option<String>) -> Result<Vec<NetworkPolicyInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_network_policies(&client, namespace.as_deref()).await
}

// Config resources
#[tauri::command]
pub async fn get_configmaps(namespace: Option<String>) -> Result<Vec<ConfigMapInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_configmaps(&client, namespace.as_deref()).await
}

#[tauri::command]
pub async fn get_secrets(namespace: Option<String>) -> Result<Vec<SecretInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_secrets(&client, namespace.as_deref()).await
}

#[tauri::command]
pub async fn get_hpas(namespace: Option<String>) -> Result<Vec<HPAInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_hpas(&client, namespace.as_deref()).await
}

// Storage resources
#[tauri::command]
pub async fn get_pvs() -> Result<Vec<PersistentVolumeInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_pvs(&client).await
}

#[tauri::command]
pub async fn get_pvcs(namespace: Option<String>) -> Result<Vec<PersistentVolumeClaimInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_pvcs(&client, namespace.as_deref()).await
}

// Cluster resources
#[tauri::command]
pub async fn get_namespaces_info() -> Result<Vec<NamespaceInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_namespaces_info(&client).await
}

#[tauri::command]
pub async fn get_nodes() -> Result<Vec<NodeInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_nodes(&client).await
}

#[tauri::command]
pub async fn get_service_accounts(namespace: Option<String>) -> Result<Vec<ServiceAccountInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_service_accounts(&client, namespace.as_deref()).await
}

#[tauri::command]
pub async fn get_events(namespace: Option<String>) -> Result<Vec<ClusterEventInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_events(&client, namespace.as_deref()).await
}

#[tauri::command]
pub async fn get_cluster_metrics() -> Result<ClusterMetrics> {
    let client = kubernetes::create_client().await?;
    kubernetes::get_metrics(&client).await
}

#[tauri::command]
pub async fn get_pulse_metrics(namespace: Option<String>) -> Result<PulseMetrics> {
    let client = kubernetes::create_client().await?;
    kubernetes::get_pulse_metrics(&client, namespace.as_deref()).await
}

#[tauri::command]
pub async fn exec_pod(
    _namespace: String,
    _pod_name: String,
    _container: Option<String>,
) -> Result<String> {
    // Terminal exec will be handled via Tauri events for streaming
    // This is a placeholder - actual implementation uses websocket streaming
    Ok("exec_session_id".to_string())
}

// ============ Pod Detail Commands ============

#[tauri::command]
pub async fn get_pod_detail(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<PodDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_pod_detail(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_pod_yaml(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_pod_yaml(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_pod_events(
    context_name: String,
    namespace: String,
    name: String,
) -> Result<Vec<PodEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_pod_events(&client, &namespace, &name).await
}

// ============ Window Management Commands ============

#[tauri::command]
pub async fn open_resource_detail(
    app: AppHandle,
    resource_type: String,
    name: String,
    namespace: String,
    context: String,
) -> Result<()> {
    // Create a unique window label
    let window_label = format!(
        "detail-{}-{}-{}",
        resource_type,
        namespace.replace(['.', '/', '\\', ' '], "-"),
        name.replace(['.', '/', '\\', ' '], "-")
    );

    // Check if window already exists
    if let Some(window) = app.get_webview_window(&window_label) {
        // Focus existing window
        window.set_focus().ok();
        return Ok(());
    }

    // Build the URL with query parameters
    let url = format!(
        "/detail.html?type={}&name={}&namespace={}&context={}",
        urlencoding::encode(&resource_type),
        urlencoding::encode(&name),
        urlencoding::encode(&namespace),
        urlencoding::encode(&context)
    );

    // Capitalize resource type for title
    let capitalized_type = {
        let mut chars = resource_type.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    };

    // Create a new window for the resource detail
    let title = format!("{}: {} ({}/{})", capitalized_type, name, context, namespace);

    WebviewWindowBuilder::new(
        &app,
        &window_label,
        WebviewUrl::App(url.into()),
    )
    .title(&title)
    .inner_size(900.0, 700.0)
    .min_inner_size(600.0, 400.0)
    .resizable(true)
    .decorations(false)
    .build()
    .map_err(|e| crate::error::AppError::Custom(format!("Failed to create window: {}", e)))?;

    Ok(())
}

#[tauri::command]
pub async fn open_terminal_window(
    app: AppHandle,
    pod_name: String,
    namespace: String,
    context: String,
    container: Option<String>,
) -> Result<()> {
    // Create a unique window label
    let window_label = if let Some(ref c) = container {
        format!(
            "terminal-{}-{}-{}",
            namespace.replace(['.', '/', '\\', ' '], "-"),
            pod_name.replace(['.', '/', '\\', ' '], "-"),
            c.replace(['.', '/', '\\', ' '], "-")
        )
    } else {
        format!(
            "terminal-{}-{}",
            namespace.replace(['.', '/', '\\', ' '], "-"),
            pod_name.replace(['.', '/', '\\', ' '], "-")
        )
    };

    // Check if window already exists
    if let Some(window) = app.get_webview_window(&window_label) {
        // Focus existing window
        window.set_focus().ok();
        return Ok(());
    }

    // Build the URL with query parameters
    let url = if let Some(ref c) = container {
        format!(
            "/terminal.html?pod={}&namespace={}&context={}&container={}",
            urlencoding::encode(&pod_name),
            urlencoding::encode(&namespace),
            urlencoding::encode(&context),
            urlencoding::encode(c)
        )
    } else {
        format!(
            "/terminal.html?pod={}&namespace={}&context={}",
            urlencoding::encode(&pod_name),
            urlencoding::encode(&namespace),
            urlencoding::encode(&context)
        )
    };

    // Create a new window for the terminal
    let title = if let Some(ref c) = container {
        format!("Terminal: {} ({}) - {}/{}", pod_name, c, context, namespace)
    } else {
        format!("Terminal: {} - {}/{}", pod_name, context, namespace)
    };

    WebviewWindowBuilder::new(
        &app,
        &window_label,
        WebviewUrl::App(url.into()),
    )
    .title(&title)
    .inner_size(800.0, 500.0)
    .min_inner_size(500.0, 300.0)
    .resizable(true)
    .decorations(false)
    .build()
    .map_err(|e| crate::error::AppError::Custom(format!("Failed to create terminal window: {}", e)))?;

    Ok(())
}

// ============ ConfigMap Detail Commands ============

#[tauri::command]
pub async fn get_configmap_detail(context_name: String, namespace: String, name: String) -> Result<ConfigMapDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_configmap_detail(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_configmap_yaml(context_name: String, namespace: String, name: String) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_configmap_yaml(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_configmap_events(context_name: String, namespace: String, name: String) -> Result<Vec<ConfigMapEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_configmap_events(&client, &namespace, &name).await
}

// ============ Secret Detail Commands ============

#[tauri::command]
pub async fn get_secret_detail(context_name: String, namespace: String, name: String) -> Result<SecretDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_secret_detail(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_secret_yaml(context_name: String, namespace: String, name: String) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_secret_yaml(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_secret_data(context_name: String, namespace: String, name: String) -> Result<std::collections::BTreeMap<String, String>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_secret_data(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_secret_events(context_name: String, namespace: String, name: String) -> Result<Vec<SecretEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_secret_events(&client, &namespace, &name).await
}

// ============ Job Detail Commands ============

#[tauri::command]
pub async fn get_job_detail(context_name: String, namespace: String, name: String) -> Result<JobDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_job_detail(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_job_yaml(context_name: String, namespace: String, name: String) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_job_yaml(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_job_events(context_name: String, namespace: String, name: String) -> Result<Vec<JobEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_job_events(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_job_pods(context_name: String, namespace: String, job_name: String) -> Result<Vec<PodInfo>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_job_pods(&client, &namespace, &job_name).await
}

// ============ CronJob Detail Commands ============

#[tauri::command]
pub async fn get_cronjob_detail(context_name: String, namespace: String, name: String) -> Result<CronJobDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_cronjob_detail(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_cronjob_yaml(context_name: String, namespace: String, name: String) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_cronjob_yaml(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_cronjob_events(context_name: String, namespace: String, name: String) -> Result<Vec<CronJobEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_cronjob_events(&client, &namespace, &name).await
}

// ============ DaemonSet Detail Commands ============

#[tauri::command]
pub async fn get_daemonset_detail(context_name: String, namespace: String, name: String) -> Result<DaemonSetDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_daemonset_detail(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_daemonset_yaml(context_name: String, namespace: String, name: String) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_daemonset_yaml(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_daemonset_events(context_name: String, namespace: String, name: String) -> Result<Vec<DaemonSetEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_daemonset_events(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_daemonset_pods(context_name: String, namespace: String, daemonset_name: String) -> Result<Vec<PodInfo>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_daemonset_pods(&client, &namespace, &daemonset_name).await
}

// ============ ReplicaSet Detail Commands ============

#[tauri::command]
pub async fn get_replicaset_detail(context_name: String, namespace: String, name: String) -> Result<ReplicaSetDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_replicaset_detail(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_replicaset_yaml(context_name: String, namespace: String, name: String) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_replicaset_yaml(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_replicaset_events(context_name: String, namespace: String, name: String) -> Result<Vec<ReplicaSetEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_replicaset_events(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_replicaset_pods(context_name: String, namespace: String, replicaset_name: String) -> Result<Vec<PodInfo>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_replicaset_pods(&client, &namespace, &replicaset_name).await
}

// ============ NetworkPolicy Detail Commands ============

#[tauri::command]
pub async fn get_networkpolicy_detail(context_name: String, namespace: String, name: String) -> Result<NetworkPolicyDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_networkpolicy_detail(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_networkpolicy_yaml(context_name: String, namespace: String, name: String) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_networkpolicy_yaml(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_networkpolicy_events(context_name: String, namespace: String, name: String) -> Result<Vec<NetworkPolicyEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_networkpolicy_events(&client, &namespace, &name).await
}

// ============ HPA Detail Commands ============

#[tauri::command]
pub async fn get_hpa_detail(context_name: String, namespace: String, name: String) -> Result<HPADetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_hpa_detail(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_hpa_yaml(context_name: String, namespace: String, name: String) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_hpa_yaml(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_hpa_events(context_name: String, namespace: String, name: String) -> Result<Vec<HPAEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_hpa_events(&client, &namespace, &name).await
}

// ============ PV Detail Commands ============

#[tauri::command]
pub async fn get_pv_detail(context_name: String, name: String) -> Result<PVDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_pv_detail(&client, &name).await
}

#[tauri::command]
pub async fn get_pv_yaml(context_name: String, name: String) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_pv_yaml(&client, &name).await
}

#[tauri::command]
pub async fn get_pv_events(context_name: String, name: String) -> Result<Vec<PVEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_pv_events(&client, &name).await
}

// ============ PVC Detail Commands ============

#[tauri::command]
pub async fn get_pvc_detail(context_name: String, namespace: String, name: String) -> Result<PVCDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_pvc_detail(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_pvc_yaml(context_name: String, namespace: String, name: String) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_pvc_yaml(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_pvc_events(context_name: String, namespace: String, name: String) -> Result<Vec<PVCEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_pvc_events(&client, &namespace, &name).await
}

// ============ Namespace Detail Commands ============

#[tauri::command]
pub async fn get_namespace_detail(context_name: String, name: String) -> Result<NamespaceDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_namespace_detail(&client, &name).await
}

#[tauri::command]
pub async fn get_namespace_yaml(context_name: String, name: String) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_namespace_yaml(&client, &name).await
}

#[tauri::command]
pub async fn get_namespace_events(context_name: String, name: String) -> Result<Vec<NamespaceEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_namespace_events(&client, &name).await
}

// ============ Node Detail Commands ============

#[tauri::command]
pub async fn get_node_detail(context_name: String, name: String) -> Result<NodeDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_node_detail(&client, &name).await
}

#[tauri::command]
pub async fn get_node_yaml(context_name: String, name: String) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_node_yaml(&client, &name).await
}

#[tauri::command]
pub async fn get_node_events(context_name: String, name: String) -> Result<Vec<NodeEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_node_events(&client, &name).await
}

#[tauri::command]
pub async fn get_node_pods(context_name: String, node_name: String) -> Result<Vec<PodInfo>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_node_pods(&client, &node_name).await
}

// ============ ServiceAccount Detail Commands ============

#[tauri::command]
pub async fn get_serviceaccount_detail(context_name: String, namespace: String, name: String) -> Result<ServiceAccountDetail> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_serviceaccount_detail(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_serviceaccount_yaml(context_name: String, namespace: String, name: String) -> Result<String> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_serviceaccount_yaml(&client, &namespace, &name).await
}

#[tauri::command]
pub async fn get_serviceaccount_events(context_name: String, namespace: String, name: String) -> Result<Vec<ServiceAccountEvent>> {
    let client = kubernetes::create_client_for_context(&context_name).await?;
    kubernetes::get_serviceaccount_events(&client, &namespace, &name).await
}

// ============ PTY Commands ============

use crate::pty::PtyManager;

#[tauri::command]
pub fn pty_spawn(
    app: AppHandle,
    pty_manager: tauri::State<PtyManager>,
    namespace: String,
    pod_name: String,
    container: Option<String>,
    shell: Option<String>,
) -> std::result::Result<String, String> {
    pty_manager.spawn_session(app, &namespace, &pod_name, container.as_deref(), shell.as_deref())
}

#[tauri::command]
pub fn pty_write(
    pty_manager: tauri::State<PtyManager>,
    session_id: String,
    data: String,
) -> std::result::Result<(), String> {
    pty_manager.write_to_session(&session_id, &data)
}

#[tauri::command]
pub fn pty_resize(
    pty_manager: tauri::State<PtyManager>,
    session_id: String,
    rows: u16,
    cols: u16,
) -> std::result::Result<(), String> {
    pty_manager.resize_session(&session_id, rows, cols)
}

#[tauri::command]
pub fn pty_close(
    pty_manager: tauri::State<PtyManager>,
    session_id: String,
) -> std::result::Result<(), String> {
    pty_manager.close_session(&session_id)
}

// ============ YAML Apply Command ============

#[tauri::command]
pub async fn apply_yaml(
    context_name: String,
    yaml_content: String,
) -> Result<String> {
    kubernetes::apply_yaml(&context_name, &yaml_content).await
}

// ============ Port Forward Commands ============

#[tauri::command]
pub async fn start_port_forward(
    pf_manager: tauri::State<'_, PortForwardManager>,
    context: String,
    namespace: String,
    resource_type: ResourceType,
    resource_name: String,
    local_port: u16,
    remote_port: u16,
) -> Result<PortForwardInfo> {
    pf_manager.start_forward(context, namespace, resource_type, resource_name, local_port, remote_port).await
}

#[tauri::command]
pub async fn stop_port_forward(
    pf_manager: tauri::State<'_, PortForwardManager>,
    id: String,
) -> Result<()> {
    pf_manager.stop_forward(&id).await
}

#[tauri::command]
pub async fn list_port_forwards(
    pf_manager: tauri::State<'_, PortForwardManager>,
) -> Result<Vec<PortForwardInfo>> {
    Ok(pf_manager.list_forwards().await)
}

#[tauri::command]
pub async fn stop_all_port_forwards(
    pf_manager: tauri::State<'_, PortForwardManager>,
) -> Result<()> {
    pf_manager.stop_all().await;
    Ok(())
}

#[tauri::command]
pub async fn get_resource_ports(
    context: String,
    namespace: String,
    resource_type: ResourceType,
    resource_name: String,
) -> Result<Vec<AvailablePort>> {
    portforward::get_resource_ports(&context, &namespace, &resource_type, &resource_name).await
}

// ============ Watch Stream Commands ============

#[tauri::command]
pub async fn watch_pods(
    app: AppHandle,
    watch_manager: tauri::State<'_, WatchManager>,
    namespace: Option<String>,
) -> Result<String> {
    watch_manager.start_pod_watch(app, namespace).await
}

#[tauri::command]
pub async fn watch_deployments(
    app: AppHandle,
    watch_manager: tauri::State<'_, WatchManager>,
    namespace: Option<String>,
) -> Result<String> {
    watch_manager.start_deployment_watch(app, namespace).await
}

#[tauri::command]
pub async fn watch_jobs(
    app: AppHandle,
    watch_manager: tauri::State<'_, WatchManager>,
    namespace: Option<String>,
) -> Result<String> {
    watch_manager.start_job_watch(app, namespace).await
}

#[tauri::command]
pub async fn watch_nodes(
    app: AppHandle,
    watch_manager: tauri::State<'_, WatchManager>,
) -> Result<String> {
    watch_manager.start_node_watch(app).await
}

#[tauri::command]
pub async fn watch_events(
    app: AppHandle,
    watch_manager: tauri::State<'_, WatchManager>,
    namespace: Option<String>,
) -> Result<String> {
    watch_manager.start_event_watch(app, namespace).await
}

#[tauri::command]
pub async fn stop_watch(
    watch_manager: tauri::State<'_, WatchManager>,
    watch_id: String,
) -> Result<()> {
    watch_manager.stop_watch(&watch_id).await
}

#[tauri::command]
pub async fn stop_all_watches(
    watch_manager: tauri::State<'_, WatchManager>,
) -> Result<()> {
    watch_manager.stop_all().await;
    Ok(())
}
