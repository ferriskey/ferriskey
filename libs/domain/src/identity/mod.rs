use enum_display::EnumDisplay;
use uuid::Uuid;

use crate::{client::Client, realm::RealmId, user::User};

#[derive(Debug, Clone)]
pub enum Identity {
    User(User),
    Client(Client),
}

impl Identity {
    /// Get the unique identifier of the identity
    pub fn id(&self) -> Uuid {
        match self {
            Self::User(user) => user.id,
            Self::Client(client) => client.id,
        }
    }

    /// Check if this identity is a service account
    pub fn is_service_account(&self) -> bool {
        matches!(self, Self::Client(_))
    }

    /// Check if this identity is a regular user (not associated with a client)
    pub fn is_regular_user(&self) -> bool {
        matches!(self, Self::User(user) if user.client_id.is_none())
    }

    /// Get the user if this identity represents a user
    pub fn as_user(&self) -> Option<&User> {
        match self {
            Self::User(user) => Some(user),
            _ => None,
        }
    }

    /// Get the client if this identity represents a client
    pub fn as_client(&self) -> Option<&Client> {
        match self {
            Self::Client(client) => Some(client),
            _ => None,
        }
    }

    /// Get the realm ID this identity belongs to
    pub fn realm_id(&self) -> RealmId {
        match self {
            Self::User(user) => user.realm_id,
            Self::Client(client) => client.realm_id,
        }
    }

    /// Check if this identity has access to the specified realm
    ///
    /// Business rule: An identity can only access resources in its own realm
    pub fn has_access_to_realm(&self, realm_id: Uuid) -> bool {
        self.realm_id() == realm_id
    }

    /// Get a display name for this identity
    pub fn display_name(&self) -> String {
        match self {
            Self::User(user) => user.username.clone(),
            Self::Client(client) => format!("client:{}", client.client_id),
        }
    }

    /// Get the kind of this identity
    pub fn kind(&self) -> IdentityKind {
        match self {
            Self::User(_) => IdentityKind::User,
            Self::Client(_) => IdentityKind::Client,
        }
    }
}

#[derive(Clone, Copy, Debug, EnumDisplay, Eq, PartialEq)]
#[display(case = "Kebab")]
pub enum IdentityKind {
    User,
    Client,
}
