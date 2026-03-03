use crate::{entities::EmailMessage, error::EmailError};

pub trait EmailSender: Send + Sync {
    fn send(&self, message: EmailMessage) -> impl Future<Output = Result<(), EmailError>> + Send;
}
