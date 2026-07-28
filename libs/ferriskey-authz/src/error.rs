use ferriskey_domain::common::app_errors::CoreError;

/// Failures raised by an authorization engine or its ports.
///
/// A variant of this enum always means "the decision could not be computed",
/// never "the decision is deny". Engines must keep the two apart: collapsing
/// an infrastructure failure into a denial is what makes an outage look like a
/// permission problem.
#[derive(Debug, thiserror::Error)]
pub enum AuthzError {
    #[error("policy store unavailable: {0}")]
    PolicyStore(String),

    #[error("principal slice unavailable: {0}")]
    PrincipalSlice(String),

    #[error("relation resolution failed: {0}")]
    RelationResolver(String),

    #[error("unknown action: {0}")]
    UnknownAction(String),

    #[error("unknown entity type: {0}")]
    UnknownEntityType(String),

    #[error("evaluation failed: {0}")]
    Evaluation(String),
}

impl From<AuthzError> for CoreError {
    fn from(value: AuthzError) -> Self {
        match value {
            // The caller named something that does not exist: that is a bad
            // request dressed as a decision, not an outage.
            AuthzError::UnknownAction(_) | AuthzError::UnknownEntityType(_) => {
                CoreError::Forbidden(value.to_string())
            }
            _ => CoreError::InternalServerError,
        }
    }
}
