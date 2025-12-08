use crate::error::Result;
use crate::kubernetes::{
    self, ClusterMetrics, ConfigMapInfo, CronJobInfo, DaemonSetInfo, DeploymentInfo, HPAInfo,
    IngressInfo, JobInfo, KubeContext, NamespaceInfo, NetworkPolicyInfo, NodeInfo,
    PersistentVolumeClaimInfo, PersistentVolumeInfo, PodDetail, PodEvent, PodInfo, PulseMetrics,
    ReplicaSetInfo, SecretInfo, ServiceAccountInfo, ServiceInfo, StatefulSetInfo,
};
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
) -> Result<String> {
    let client = kubernetes::create_client().await?;
    kubernetes::get_logs(
        &client,
        &namespace,
        &pod_name,
        container.as_deref(),
        tail_lines,
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

#[tauri::command]
pub async fn get_statefulsets(namespace: Option<String>) -> Result<Vec<StatefulSetInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_statefulsets(&client, namespace.as_deref()).await
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

// Network resources
#[tauri::command]
pub async fn get_ingresses(namespace: Option<String>) -> Result<Vec<IngressInfo>> {
    let client = kubernetes::create_client().await?;
    kubernetes::list_ingresses(&client, namespace.as_deref()).await
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

    // Create a new window for the resource detail
    let title = format!("{}: {} ({}/{})", resource_type, name, context, namespace);

    WebviewWindowBuilder::new(
        &app,
        &window_label,
        WebviewUrl::App(url.into()),
    )
    .title(&title)
    .inner_size(900.0, 700.0)
    .min_inner_size(600.0, 400.0)
    .resizable(true)
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
    .build()
    .map_err(|e| crate::error::AppError::Custom(format!("Failed to create terminal window: {}", e)))?;

    Ok(())
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
