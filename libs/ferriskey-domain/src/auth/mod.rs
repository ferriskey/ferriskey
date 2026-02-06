use uuid::Uuid;

use crate::client::entities::Client;
use crate::realm::RealmId;
use crate::user::entities::User;

impl std::fmt::Display for IdentityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityKind::User => write!(f, "user"),
            IdentityKind::Client => write!(f, "client"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Identity {
    User(User),
    Client(Client),
}

impl Identity {
    pub fn id(&self) -> Uuid {
        match self {
            Self::User(user) => user.id,
            Self::Client(client) => client.id,
        }
    }

    pub fn is_service_account(&self) -> bool {
        matches!(self, Self::Client(_))
    }

    pub fn is_regular_user(&self) -> bool {
        matches!(self, Self::User(user) if user.client_id.is_none())
    }

    pub fn as_user(&self) -> Option<&User> {
        match self {
            Self::User(user) => Some(user),
            _ => None,
        }
    }

    pub fn as_client(&self) -> Option<&Client> {
        match self {
            Self::Client(client) => Some(client),
            _ => None,
        }
    }

    pub fn realm_id(&self) -> RealmId {
        match self {
            Self::User(user) => user.realm_id,
            Self::Client(client) => client.realm_id,
        }
    }

    pub fn has_access_to_realm(&self, realm_id: Uuid) -> bool {
        self.realm_id() == realm_id
    }

    pub fn display_name(&self) -> String {
        match self {
            Self::User(user) => user.username.clone(),
            Self::Client(client) => format!("client:{}", client.client_id),
        }
    }

    pub fn kind(&self) -> IdentityKind {
        match self {
            Self::User(_) => IdentityKind::User,
            Self::Client(_) => IdentityKind::Client,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityKind {
    User,
    Client,
}
