use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Kubernetes error: {0}")]
    Kube(#[from] kube::Error),

    #[error("Kubeconfig error: {0}")]
    KubeConfig(#[from] kube::config::KubeconfigError),

    #[error("Infer config error: {0}")]
    InferConfig(#[from] kube::config::InferConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    Custom(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
