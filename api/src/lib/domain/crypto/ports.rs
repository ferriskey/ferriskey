pub trait HasherRepository: Clone + Send + Sync + 'static {
    fn hash_password(
        &self,
        password: &str,
    ) -> impl Future<Output = Result<(String, String), anyhow::Error>> + Send;
    fn verify_password(
        &self,
        password: &str,
        secret_data: &str,
        credential_data: &str,
    ) -> impl Future<Output = Result<bool, anyhow::Error>> + Send;
}
