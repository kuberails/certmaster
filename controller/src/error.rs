use thiserror::Error;
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    KubeError(#[from] kube::error::Error),
    #[error("missing object key in {0})")]
    MissingObjectKey(&'static str),
}
