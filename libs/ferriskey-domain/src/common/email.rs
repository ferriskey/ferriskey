use crate::common::app_errors::CoreError;
use crate::realm::SmtpConfig;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait EmailPort: Send + Sync {
    fn send_email(
        &self,
        config: &SmtpConfig,
        to_email: &str,
        subject: &str,
        body: &str,
        html_body: Option<String>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
