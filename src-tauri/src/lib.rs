mod commands;
mod error;
mod kubernetes;
mod portforward;
mod pty;
mod resources;
mod shell;
mod tray;
mod triage;
mod updates;
mod watch;

use portforward::PortForwardManager;
use pty::PtyManager;
use tauri::Manager;
use watch::WatchManager;
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
        .plugin(tauri_plugin_notification::init())
        .manage(PtyManager::new())
        .manage(PortForwardManager::new())
        .manage(WatchManager::new())
        .manage(shell::ContextSync::default())
        .manage(tray::TrayState::new())
        .setup(|app| {
            let handle = app.handle().clone();
            let tray_on = cfg!(target_os = "macos") && tray::load_enabled(&handle);
            // macOS only: elsewhere a set menu draws a bar inside our
            // frameless windows; the frontend handles Ctrl+N there.
            #[cfg(target_os = "macos")]
            {
                let menu = shell::build_menu(&handle, tray_on)?;
                app.set_menu(menu)?;
            }
            shell::start_context_watcher(handle.clone());
            if tray_on {
                if let Err(e) = tray::set_enabled(&handle, true) {
                    eprintln!("[apex-kube-manager] menu bar icon: {e}");
                }
            }
            Ok(())
        })
        .on_menu_event(|app, event| {
            if event.id() == shell::MENU_NEW_WINDOW {
                let _ = shell::open_main_window(app);
            } else if event.id() == shell::MENU_TRAY {
                let on = !app.state::<tray::TrayState>().is_enabled();
                let _ = tray::set_enabled(app, on);
            }
        })
        .on_window_event(|window, event| {
            let label = window.label().to_string();
            match event {
                tauri::WindowEvent::Focused(false) if label == "tray" => tray::on_popover_blur(window.app_handle()),
                tauri::WindowEvent::Destroyed => {
                    eprintln!("[apex-kube-manager] window {label} closed");
                    shell::on_window_destroyed(window.app_handle(), &label);
                }
                _ => {}
            }
        })
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
            commands::scale_statefulset,
            commands::restart_statefulset,
            commands::get_statefulset_detail,
            commands::get_statefulset_yaml,
            commands::get_statefulset_events,
            commands::get_statefulset_pods,
            commands::get_daemonsets,
            commands::get_daemonset_detail,
            commands::get_daemonset_yaml,
            commands::get_daemonset_events,
            commands::get_daemonset_pods,
            commands::get_replicasets,
            commands::get_replicaset_detail,
            commands::get_replicaset_yaml,
            commands::get_replicaset_events,
            commands::get_replicaset_pods,
            commands::get_jobs,
            commands::get_job_detail,
            commands::get_job_yaml,
            commands::get_job_events,
            commands::get_job_pods,
            commands::get_cronjobs,
            commands::get_cronjob_detail,
            commands::get_cronjob_yaml,
            commands::get_cronjob_events,
            commands::get_services,
            commands::get_service_detail,
            commands::get_service_yaml,
            commands::get_service_events,
            commands::get_service_endpoints,
            // Network
            commands::get_ingresses,
            commands::get_ingress_detail,
            commands::get_ingress_yaml,
            commands::get_ingress_events,
            commands::get_network_policies,
            commands::get_networkpolicy_detail,
            commands::get_networkpolicy_yaml,
            commands::get_networkpolicy_events,
            commands::create_network_policy,
            commands::update_network_policy,
            commands::delete_network_policy,
            // Config
            commands::get_configmaps,
            commands::get_configmap_detail,
            commands::get_configmap_yaml,
            commands::get_configmap_events,
            commands::get_secrets,
            commands::get_secret_detail,
            commands::get_secret_yaml,
            commands::get_secret_data,
            commands::get_secret_events,
            commands::create_secret,
            commands::update_secret,
            commands::delete_secret,
            commands::get_hpas,
            commands::get_hpa_detail,
            commands::get_hpa_yaml,
            commands::get_hpa_events,
            // Storage
            commands::get_pvs,
            commands::get_pv_detail,
            commands::get_pv_yaml,
            commands::get_pv_events,
            commands::get_pvcs,
            commands::get_pvc_detail,
            commands::get_pvc_yaml,
            commands::get_pvc_events,
            // Cluster
            commands::get_namespaces_info,
            commands::get_namespace_detail,
            commands::get_namespace_yaml,
            commands::get_namespace_events,
            commands::get_nodes,
            commands::get_node_metrics,
            commands::get_node_detail,
            commands::get_node_yaml,
            commands::get_node_events,
            commands::get_node_pods,
            commands::add_node_taint,
            commands::remove_node_taint,
            commands::cordon_node,
            commands::uncordon_node,
            commands::get_service_accounts,
            commands::get_serviceaccount_detail,
            commands::get_serviceaccount_yaml,
            commands::get_serviceaccount_events,
            commands::get_roles,
            commands::get_cluster_roles,
            commands::get_role_bindings,
            commands::get_cluster_role_bindings,
            commands::get_storage_classes,
            commands::get_endpoint_slices,
            commands::get_ingress_classes,
            commands::get_pdbs,
            commands::get_resource_quotas,
            commands::get_limit_ranges,
            commands::get_priority_classes,
            commands::get_crds,
            commands::get_custom_resources,
            commands::get_resource_detail,
            commands::get_resource_yaml,
            commands::get_resource_events,
            commands::get_custom_resource_detail,
            commands::get_custom_resource_yaml,
            commands::get_custom_resource_events,
            commands::get_events,
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
            // YAML editing
            commands::apply_yaml,
            // Port forwarding
            commands::start_port_forward,
            commands::stop_port_forward,
            commands::list_port_forwards,
            commands::stop_all_port_forwards,
            commands::get_resource_ports,
            // Watch streams
            commands::watch_pods,
            commands::watch_deployments,
            commands::watch_jobs,
            commands::watch_nodes,
            commands::watch_events,
            commands::stop_watch,
            commands::stop_all_watches,
            commands::check_for_updates,
            // Multi-window + menu bar (shell.rs, tray.rs)
            commands::open_main_window,
            commands::get_shared_context,
            commands::focus_main_window,
            commands::get_main_window_count,
            commands::get_tray_status,
            commands::refresh_tray_status,
            commands::set_tray_enabled,
            commands::hide_tray_popover,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // Closing the last window keeps Apex in the menu bar (spec 13.4).
            match event {
                tauri::RunEvent::ExitRequested { code, api, .. } => {
                    let tray_on = app.state::<tray::TrayState>().is_enabled();
                    // Logged so an unexpected exit shows its cause in the dev log.
                    eprintln!("[apex-kube-manager] exit requested (code {code:?}, menu bar item {})", if tray_on { "on" } else { "off" });
                    if code.is_none() && tray_on {
                        api.prevent_exit();
                    }
                }
                tauri::RunEvent::Exit => eprintln!("[apex-kube-manager] exiting"),
                // Dock icon clicked with no visible window: bring one back.
                #[cfg(target_os = "macos")]
                tauri::RunEvent::Reopen { has_visible_windows, .. } => {
                    if !has_visible_windows {
                        let _ = shell::focus_or_open_main(app);
                    }
                }
                _ => {}
            }
        });
}
