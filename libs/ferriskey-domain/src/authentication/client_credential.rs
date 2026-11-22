use maskass::Masked;

pub const JWT_BEARER_ASSERTION_TYPE: &str =
    "urn:ietf:params:oauth:client-assertion-type:jwt-bearer";

/// What a client presented to authenticate one request. Nothing in it is
/// verified yet.
#[derive(Debug, Clone, Default)]
pub enum ClientCredential {
    #[default]
    None,
    Secret(Masked<String>),
    Assertion(ClientAssertion),
}

impl ClientCredential {
    pub fn secret(secret: impl Into<String>) -> Self {
        Self::Secret(Masked::new(secret.into()))
    }

    pub fn from_secret(secret: Option<String>) -> Self {
        secret.map_or(Self::None, Self::secret)
    }

    pub fn is_present(&self) -> bool {
        !matches!(self, Self::None)
    }
}

/// A JWT the client signed to authenticate (RFC 7523 §2.2), with the `aud`
/// values the endpoint it was sent to accepts.
#[derive(Debug, Clone)]
pub struct ClientAssertion {
    pub jwt: Masked<String>,
    pub audiences: Vec<String>,
}

impl ClientAssertion {
    pub fn new(jwt: String, audiences: Vec<String>) -> Self {
        Self {
            jwt: Masked::new(jwt),
            audiences,
        }
    }
}

/// How a client proved who it is on this request. Produced only once its
/// credential has been checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientProof {
    None,
    Secret,
    SecretJwt,
    PrivateKeyJwt,
}
