use crate::domain::{common::entities::app_errors::CoreError, realm::entities::SmtpConfig};

#[cfg_attr(test, mockall::automock)]
pub trait EmailPort: Send + Sync {
    fn send_email(
        &self,
        config: &SmtpConfig,
        to_email: &str,
        subject: &str,
        body: &str,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
