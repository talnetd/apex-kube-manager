mod commands;
mod error;
mod kubernetes;
mod pty;

use pty::PtyManager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Fix environment on macOS when launched from Finder (GUI apps don't inherit shell env)
/// This is needed for exec-based Kubernetes auth (EKS, GKE, AKS) to find auth binaries
#[cfg(target_os = "macos")]
fn fix_path_env() {
    use std::process::Command;

    // Get the user's shell
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());

    // Run the shell in login mode to get all environment variables
    // Using `env` command to output all variables in a parseable format
    let output = Command::new(&shell)
        .args(["-l", "-c", "env"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            if let Ok(env_output) = String::from_utf8(output.stdout) {
                let mut count = 0;
                for line in env_output.lines() {
                    // Handle multi-line values by only parsing lines with = at reasonable position
                    if let Some(eq_pos) = line.find('=') {
                        if eq_pos > 0 && eq_pos < 64 {
                            let key = &line[..eq_pos];
                            let value = &line[eq_pos + 1..];
                            // Skip some variables that shouldn't be overwritten
                            if key == "PWD" || key == "_" || key == "SHLVL" || key == "OLDPWD" {
                                continue;
                            }
                            // Only set valid env var names
                            if key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                                std::env::set_var(key, value);
                                count += 1;
                            }
                        }
                    }
                }
                eprintln!("[apex-kube-manager] Inherited {} environment variables from login shell", count);
            }
        }
        Ok(output) => {
            eprintln!("[apex-kube-manager] Shell env command failed: {:?}", output.status);
        }
        Err(e) => {
            eprintln!("[apex-kube-manager] Failed to run shell: {}", e);
        }
    }
}

/// Fix environment on Linux when launched from desktop (GUI apps don't inherit shell env)
#[cfg(target_os = "linux")]
fn fix_path_env() {
    use std::process::Command;

    // Get the user's shell
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

    // Run the shell in login mode to get all environment variables
    let output = Command::new(&shell)
        .args(["-l", "-c", "env"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            if let Ok(env_output) = String::from_utf8(output.stdout) {
                let mut count = 0;
                for line in env_output.lines() {
                    if let Some(eq_pos) = line.find('=') {
                        if eq_pos > 0 && eq_pos < 64 {
                            let key = &line[..eq_pos];
                            let value = &line[eq_pos + 1..];
                            if key == "PWD" || key == "_" || key == "SHLVL" || key == "OLDPWD" {
                                continue;
                            }
                            if key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                                std::env::set_var(key, value);
                                count += 1;
                            }
                        }
                    }
                }
                eprintln!("[apex-kube-manager] Inherited {} environment variables from login shell", count);
            }
        }
        Ok(output) => {
            eprintln!("[apex-kube-manager] Shell env command failed: {:?}", output.status);
        }
        Err(e) => {
            eprintln!("[apex-kube-manager] Failed to run shell: {}", e);
        }
    }
}

#[cfg(target_os = "windows")]
fn fix_path_env() {
    // Windows GUI apps typically inherit PATH correctly
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Fix PATH before doing anything else
    fix_path_env();

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
