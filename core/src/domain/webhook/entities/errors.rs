use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum WebhookError {
    #[error("Webhook not found")]
    NotFound,

    #[error("Internal server error")]
    InternalServerError,

    #[error("Forbidden")]
    Forbidden,

    #[error("Realm not found")]
    RealmNotFound,
}
