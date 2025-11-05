use uuid::Uuid;

pub struct LdapUser {
    pub dn: String,
    pub username: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub realm_id: Uuid,
}
