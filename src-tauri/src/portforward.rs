use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tokio::sync::RwLock;
use tokio::net::TcpListener;
use kube::{Api, Client, Config};
use kube::api::ListParams;
use k8s_openapi::api::core::v1::{Pod, Service};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Pod,
    Service,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForwardInfo {
    pub id: String,
    pub context: String,
    pub namespace: String,
    pub resource_type: ResourceType,
    pub resource_name: String,
    pub pod_name: String,  // Resolved pod name (same as resource_name for pods)
    pub local_port: u16,
    pub remote_port: u16,
    pub status: PortForwardStatus,
    // Connection stats
    pub active_connections: u64,
    pub total_connections: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// Internal stats tracking with atomic counters
pub struct ForwardStats {
    active_connections: AtomicUsize,
    total_connections: AtomicU64,
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
}

impl ForwardStats {
    fn new() -> Self {
        Self {
            active_connections: AtomicUsize::new(0),
            total_connections: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
        }
    }

    fn connection_opened(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
        self.total_connections.fetch_add(1, Ordering::Relaxed);
    }

    fn connection_closed(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    fn add_bytes_sent(&self, bytes: u64) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
    }

    fn add_bytes_received(&self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
    }

    fn snapshot(&self) -> (u64, u64, u64, u64) {
        (
            self.active_connections.load(Ordering::Relaxed) as u64,
            self.total_connections.load(Ordering::Relaxed),
            self.bytes_sent.load(Ordering::Relaxed),
            self.bytes_received.load(Ordering::Relaxed),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PortForwardStatus {
    Starting,
    Active,
    Error,
    Stopped,
}

/// Available port info for a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailablePort {
    pub port: u16,
    pub name: Option<String>,
    pub protocol: String,
}

struct ActiveForward {
    info: PortForwardInfo,
    stats: Arc<ForwardStats>,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

pub struct PortForwardManager {
    forwards: Arc<RwLock<HashMap<String, ActiveForward>>>,
}

impl PortForwardManager {
    pub fn new() -> Self {
        Self {
            forwards: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_forward(
        &self,
        context: String,
        namespace: String,
        resource_type: ResourceType,
        resource_name: String,
        local_port: u16,
        remote_port: u16,
    ) -> Result<PortForwardInfo> {
        let id = Uuid::new_v4().to_string();

        // Check if local port is already in use by us
        {
            let forwards = self.forwards.read().await;
            for (_, fwd) in forwards.iter() {
                if fwd.info.local_port == local_port && fwd.info.status == PortForwardStatus::Active {
                    return Err(AppError::Custom(format!(
                        "Port {} is already being forwarded by this app",
                        local_port
                    )));
                }
            }
        }

        // Check if local port is available on the system
        if !is_port_available(local_port).await {
            return Err(AppError::Custom(format!(
                "Port {} is already in use by another application",
                local_port
            )));
        }

        // Resolve pod name based on resource type
        let pod_name = match &resource_type {
            ResourceType::Pod => resource_name.clone(),
            ResourceType::Service => {
                // Need to resolve service to a pod
                resolve_service_to_pod(&context, &namespace, &resource_name).await?
            }
        };

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // Create stats tracker
        let stats = Arc::new(ForwardStats::new());

        let info = PortForwardInfo {
            id: id.clone(),
            context: context.clone(),
            namespace: namespace.clone(),
            resource_type,
            resource_name: resource_name.clone(),
            pod_name: pod_name.clone(),
            local_port,
            remote_port,
            status: PortForwardStatus::Starting,
            active_connections: 0,
            total_connections: 0,
            bytes_sent: 0,
            bytes_received: 0,
        };

        // Store the forward
        {
            let mut forwards = self.forwards.write().await;
            forwards.insert(id.clone(), ActiveForward {
                info: info.clone(),
                stats: stats.clone(),
                shutdown_tx,
            });
        }

        // Clone what we need for the spawned task
        let forwards_ref = self.forwards.clone();
        let id_clone = id.clone();
        let context_clone = context.clone();
        let stats_clone = stats.clone();

        // Spawn the port forward task
        tokio::spawn(async move {
            let result = run_port_forward(
                context_clone,
                namespace,
                pod_name,
                local_port,
                remote_port,
                shutdown_rx,
                forwards_ref.clone(),
                id_clone.clone(),
                stats_clone,
            ).await;

            // Update status based on result
            let mut forwards = forwards_ref.write().await;
            if let Some(fwd) = forwards.get_mut(&id_clone) {
                match result {
                    Ok(_) => fwd.info.status = PortForwardStatus::Stopped,
                    Err(e) => {
                        tracing::error!("Port forward error: {}", e);
                        fwd.info.status = PortForwardStatus::Error;
                    }
                }
            }
        });

        Ok(info)
    }

    pub async fn stop_forward(&self, id: &str) -> Result<()> {
        let mut forwards = self.forwards.write().await;
        if let Some(fwd) = forwards.remove(id) {
            // Send shutdown signal (ignore if receiver dropped)
            let _ = fwd.shutdown_tx.send(());
            Ok(())
        } else {
            Err(AppError::Custom(format!("Port forward {} not found", id)))
        }
    }

    pub async fn list_forwards(&self) -> Vec<PortForwardInfo> {
        let forwards = self.forwards.read().await;
        forwards.values().map(|f| {
            let (active, total, sent, received) = f.stats.snapshot();
            PortForwardInfo {
                active_connections: active,
                total_connections: total,
                bytes_sent: sent,
                bytes_received: received,
                ..f.info.clone()
            }
        }).collect()
    }

    pub async fn stop_all(&self) {
        let mut forwards = self.forwards.write().await;
        for (_, fwd) in forwards.drain() {
            let _ = fwd.shutdown_tx.send(());
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_port_forward(
    context: String,
    namespace: String,
    pod_name: String,
    local_port: u16,
    remote_port: u16,
    mut shutdown_rx: tokio::sync::oneshot::Receiver<()>,
    forwards_ref: Arc<RwLock<HashMap<String, ActiveForward>>>,
    id: String,
    stats: Arc<ForwardStats>,
) -> Result<()> {
    // Bind local port
    let listener = TcpListener::bind(format!("127.0.0.1:{}", local_port))
        .await
        .map_err(|e| AppError::Custom(format!("Failed to bind port {}: {}", local_port, e)))?;

    tracing::info!("Port forward listening on 127.0.0.1:{}", local_port);

    // Update status to Active
    {
        let mut forwards = forwards_ref.write().await;
        if let Some(fwd) = forwards.get_mut(&id) {
            fwd.info.status = PortForwardStatus::Active;
        }
    }

    // Create K8s client for the specific context
    let config = create_client_config(&context).await?;
    let client = Client::try_from(config)?;

    loop {
        tokio::select! {
            // Check for shutdown signal
            _ = &mut shutdown_rx => {
                tracing::info!("Port forward {} received shutdown signal", id);
                break;
            }
            // Accept new connections
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((mut socket, addr)) => {
                        tracing::debug!("New connection from {} for port forward {}", addr, id);

                        // Track connection opened
                        stats.connection_opened();

                        // Clone what we need for this connection
                        let client = client.clone();
                        let namespace = namespace.clone();
                        let pod_name = pod_name.clone();
                        let conn_stats = stats.clone();

                        // Handle each connection in a separate task
                        tokio::spawn(async move {
                            let result = handle_connection(
                                &mut socket,
                                &client,
                                &namespace,
                                &pod_name,
                                remote_port,
                                &conn_stats,
                            ).await;

                            // Track connection closed
                            conn_stats.connection_closed();

                            if let Err(e) = result {
                                tracing::error!("Connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Accept error: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_connection(
    socket: &mut tokio::net::TcpStream,
    client: &Client,
    namespace: &str,
    pod_name: &str,
    remote_port: u16,
    stats: &Arc<ForwardStats>,
) -> Result<()> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);

    // Create port forwarder
    let mut pf = pods.portforward(pod_name, &[remote_port]).await
        .map_err(|e| AppError::Custom(format!("Failed to create port forward: {}", e)))?;

    // Get the stream for this port
    let upstream = pf.take_stream(remote_port)
        .ok_or_else(|| AppError::Custom("Failed to get port forward stream".to_string()))?;

    // Split the streams for bidirectional copying
    let (mut socket_read, mut socket_write) = socket.split();
    let (mut upstream_read, mut upstream_write) = tokio::io::split(upstream);

    // Copy data bidirectionally
    let client_to_server = tokio::io::copy(&mut socket_read, &mut upstream_write);
    let server_to_client = tokio::io::copy(&mut upstream_read, &mut socket_write);

    // Run both directions concurrently, track bytes when done
    tokio::select! {
        result = client_to_server => {
            if let Ok(bytes) = result {
                stats.add_bytes_sent(bytes);
                tracing::debug!("Client to server ended, sent {} bytes", bytes);
            }
        }
        result = server_to_client => {
            if let Ok(bytes) = result {
                stats.add_bytes_received(bytes);
                tracing::debug!("Server to client ended, received {} bytes", bytes);
            }
        }
    }

    // Clean up
    drop(pf);

    Ok(())
}

async fn create_client_config(context_name: &str) -> Result<Config> {
    let kubeconfig = kube::config::Kubeconfig::read_from(crate::kubernetes::get_kubeconfig_path_for_portforward())?;

    let options = kube::config::KubeConfigOptions {
        context: Some(context_name.to_string()),
        ..Default::default()
    };

    let config = Config::from_custom_kubeconfig(kubeconfig, &options).await?;
    Ok(config)
}

/// Check if a port is available on localhost
async fn is_port_available(port: u16) -> bool {
    TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .is_ok()
}

/// Get available ports for a pod or service
pub async fn get_resource_ports(
    context: &str,
    namespace: &str,
    resource_type: &ResourceType,
    resource_name: &str,
) -> Result<Vec<AvailablePort>> {
    let config = create_client_config(context).await?;
    let client = Client::try_from(config)?;

    match resource_type {
        ResourceType::Pod => {
            let pods: Api<Pod> = Api::namespaced(client, namespace);
            let pod = pods.get(resource_name).await
                .map_err(|e| AppError::Custom(format!("Pod '{}' not found: {}", resource_name, e)))?;

            let mut ports = Vec::new();
            if let Some(spec) = pod.spec {
                for container in spec.containers {
                    if let Some(container_ports) = container.ports {
                        for cp in container_ports {
                            ports.push(AvailablePort {
                                port: cp.container_port as u16,
                                name: cp.name.clone(),
                                protocol: cp.protocol.clone().unwrap_or_else(|| "TCP".to_string()),
                            });
                        }
                    }
                }
            }
            // Remove duplicates
            ports.sort_by_key(|p| p.port);
            ports.dedup_by_key(|p| p.port);
            Ok(ports)
        }
        ResourceType::Service => {
            let services: Api<Service> = Api::namespaced(client.clone(), namespace);
            let service = services.get(resource_name).await
                .map_err(|e| AppError::Custom(format!("Service '{}' not found: {}", resource_name, e)))?;

            // Get selector to find pods for named port resolution
            let selector = service.spec.as_ref()
                .and_then(|s| s.selector.as_ref());

            // Get a pod to resolve named ports
            let pod_ports: Vec<(String, i32)> = if let Some(sel) = selector {
                let label_selector: String = sel.iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(",");
                let pods_api: Api<Pod> = Api::namespaced(client, namespace);
                if let Ok(pod_list) = pods_api.list(&ListParams::default().labels(&label_selector).limit(1)).await {
                    pod_list.items.first()
                        .and_then(|p| p.spec.as_ref())
                        .map(|spec| {
                            spec.containers.iter()
                                .flat_map(|c| c.ports.iter().flatten())
                                .filter_map(|p| p.name.as_ref().map(|n| (n.clone(), p.container_port)))
                                .collect()
                        })
                        .unwrap_or_default()
                } else {
                    vec![]
                }
            } else {
                vec![]
            };

            let mut ports = Vec::new();
            if let Some(spec) = service.spec {
                if let Some(service_ports) = spec.ports {
                    for sp in service_ports {
                        // Use targetPort (actual pod port) for forwarding, not service port
                        let target_port = match &sp.target_port {
                            Some(k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(p)) => *p as u16,
                            Some(k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::String(name)) => {
                                // Named port - look up from pod container ports
                                pod_ports.iter()
                                    .find(|(n, _)| n == name)
                                    .map(|(_, p)| *p as u16)
                                    .unwrap_or(sp.port as u16)
                            }
                            None => sp.port as u16,
                        };
                        ports.push(AvailablePort {
                            port: target_port,
                            name: sp.name.clone(),
                            protocol: sp.protocol.clone().unwrap_or_else(|| "TCP".to_string()),
                        });
                    }
                }
            }
            Ok(ports)
        }
    }
}

/// Resolve a Service to a running Pod by matching the service's selector
async fn resolve_service_to_pod(
    context: &str,
    namespace: &str,
    service_name: &str,
) -> Result<String> {
    let config = create_client_config(context).await?;
    let client = Client::try_from(config)?;

    // Get the service
    let services: Api<Service> = Api::namespaced(client.clone(), namespace);
    let service = services.get(service_name).await
        .map_err(|e| AppError::Custom(format!("Service '{}' not found: {}", service_name, e)))?;

    // Get the selector from the service spec
    let selector = service
        .spec
        .as_ref()
        .and_then(|s| s.selector.as_ref())
        .ok_or_else(|| AppError::Custom(format!("Service '{}' has no selector", service_name)))?;

    // Build label selector string
    let label_selector: String = selector
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(",");

    // Find pods matching the selector
    let pods: Api<Pod> = Api::namespaced(client, namespace);
    let pod_list = pods.list(&ListParams::default().labels(&label_selector)).await
        .map_err(|e| AppError::Custom(format!("Failed to list pods: {}", e)))?;

    // Find a running pod
    for pod in pod_list.items {
        if let Some(status) = &pod.status {
            if let Some(phase) = &status.phase {
                if phase == "Running" {
                    if let Some(name) = pod.metadata.name {
                        tracing::info!(
                            "Resolved service '{}' to pod '{}' via selector '{}'",
                            service_name, name, label_selector
                        );
                        return Ok(name);
                    }
                }
            }
        }
    }

    Err(AppError::Custom(format!(
        "No running pods found for service '{}' (selector: {})",
        service_name, label_selector
    )))
}
