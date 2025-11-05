use uuid::Uuid;

pub struct LdapProvider {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub url: String, // e.g. ldap://ldap.example.com
    pub bind_dn: String,
    pub bind_password: String,
    pub user_base_dn: String,  // e.g. ou=Users,dc=example,dc=com
    pub user_filter: String,   // e.g. (objectClass=person)
    pub username_attr: String, // e.g. uid or sAMAccountName
    pub email_attr: Option<String>,
    pub display_name_attr: Option<String>,
    pub enabled: bool,
}
