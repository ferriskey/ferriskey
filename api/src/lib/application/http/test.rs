use chrono::Utc;

use crate::domain::{
    realm::entities::realm::Realm, role::entities::models::Role, user::entities::model::User,
    utils::generate_uuid_v7,
};

use super::server::app_state::AppState;

pub struct UserFactory {
    pub state: AppState,
}

#[derive(Default)]
pub struct UserBuilder {
    username: Option<String>,
    email: Option<String>,
    firstname: Option<String>,
    lastname: Option<String>,
    enabled: Option<bool>,
    email_verified: Option<bool>,
    roles: Vec<Role>,
}

impl UserFactory {
    pub fn neww(state: AppState) -> Self {
        Self { state }
    }

    pub async fn create_user_with_realm(
        &self,
        _user_data: User,
        _realm_data: Option<Realm>,
    ) -> Result<(User, Realm), anyhow::Error> {
        todo!("Implement create_user_with_realm method");
    }
}

impl UserBuilder {
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn firstname(mut self, firstname: impl Into<String>) -> Self {
        self.firstname = Some(firstname.into());
        self
    }

    pub fn lastname(mut self, lastname: impl Into<String>) -> Self {
        self.lastname = Some(lastname.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    pub fn email_verified(mut self, verified: bool) -> Self {
        self.email_verified = Some(verified);
        self
    }

    pub fn role(mut self, role: Role) -> Self {
        self.roles.push(role);
        self
    }

    pub fn roles(mut self, roles: Vec<Role>) -> Self {
        self.roles = roles;
        self
    }

    pub fn build(self) -> User {
        let id = generate_uuid_v7();
        User {
            id,
            realm_id: generate_uuid_v7(), // Sera remplacé lors de la création
            client_id: None,
            username: self.username.unwrap_or_else(|| format!("user-{}", id)),
            firstname: self.firstname.unwrap_or_else(|| "Test".to_string()),
            lastname: self.lastname.unwrap_or_else(|| "User".to_string()),
            email: self
                .email
                .unwrap_or_else(|| format!("test-{}@example.com", id)),
            email_verified: self.email_verified.unwrap_or(true),
            enabled: self.enabled.unwrap_or(true),
            roles: self.roles,
            realm: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
