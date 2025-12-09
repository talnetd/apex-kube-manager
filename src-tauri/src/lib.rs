mod commands;
mod error;
mod kubernetes;
mod pty;

use pty::PtyManager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(PtyManager::new())
        .invoke_handler(tauri::generate_handler![
            // Startup checks
            commands::check_kubeconfig,
            commands::validate_kubeconfig,
            commands::get_context_names,
            commands::test_cluster_connection,
            // Context management
            commands::get_contexts,
            commands::get_current_context,
            commands::switch_context,
            // Resources
            commands::get_namespaces,
            commands::get_pods,
            commands::get_pod_logs,
            commands::delete_pod,
            commands::get_deployments,
            commands::scale_deployment,
            commands::restart_deployment,
            commands::get_deployment_detail,
            commands::get_deployment_yaml,
            commands::get_deployment_events,
            commands::get_deployment_pods,
            commands::get_statefulsets,
            commands::get_daemonsets,
            commands::get_replicasets,
            commands::get_jobs,
            commands::get_cronjobs,
            commands::get_services,
            // Network
            commands::get_ingresses,
            commands::get_network_policies,
            // Config
            commands::get_configmaps,
            commands::get_secrets,
            commands::get_hpas,
            // Storage
            commands::get_pvs,
            commands::get_pvcs,
            // Cluster
            commands::get_namespaces_info,
            commands::get_nodes,
            commands::get_service_accounts,
            commands::get_cluster_metrics,
            commands::get_pulse_metrics,
            commands::exec_pod,
            // Resource detail
            commands::get_pod_detail,
            commands::get_pod_yaml,
            commands::get_pod_events,
            commands::open_resource_detail,
            commands::open_terminal_window,
            // PTY commands
            commands::pty_spawn,
            commands::pty_write,
            commands::pty_resize,
            commands::pty_close,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
