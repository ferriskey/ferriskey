#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permissions {
    // Permissions de création et de gestion
    CreateClient = 0x0000000000000001,            // 1 << 0
    ManageAuthorization = 0x0000000000000002,     // 1 << 1
    ManageClients = 0x0000000000000004,           // 1 << 2
    ManageEvents = 0x0000000000000008,            // 1 << 3
    ManageIdentityProviders = 0x0000000000000010, // 1 << 4
    ManageRealm = 0x0000000000000020,             // 1 << 5
    ManageUsers = 0x0000000000000040,             // 1 << 6

    // Permissions de requête/lecture
    QueryClients = 0x0000000000000100, // 1 << 8
    QueryGroups = 0x0000000000000200,  // 1 << 9
    QueryRealms = 0x0000000000000400,  // 1 << 10
    QueryUsers = 0x0000000000000800,   // 1 << 11

    // Permissions de visualisation
    ViewAuthorization = 0x0000000000001000,     // 1 << 12
    ViewClients = 0x0000000000002000,           // 1 << 13
    ViewEvents = 0x0000000000004000,            // 1 << 14
    ViewIdentityProviders = 0x0000000000008000, // 1 << 15
    ViewRealm = 0x0000000000010000,             // 1 << 16
    ViewUsers = 0x0000000000020000,             // 1 << 17
}

impl Permissions {
    pub fn from_bitfield(bitfield: u64) -> Vec<Self> {
        let all_permissions = vec![
            Self::CreateClient,
            Self::ManageAuthorization,
            Self::ManageClients,
            Self::ManageEvents,
            Self::ManageIdentityProviders,
            Self::ManageRealm,
            Self::ManageUsers,
            Self::QueryClients,
            Self::QueryGroups,
            Self::QueryRealms,
            Self::QueryUsers,
            Self::ViewAuthorization,
            Self::ViewClients,
            Self::ViewEvents,
            Self::ViewIdentityProviders,
            Self::ViewRealm,
            Self::ViewUsers,
        ];

        all_permissions
            .into_iter()
            .filter(|&permission| (bitfield & (1 << permission as u64)) != 0)
            .collect()
    }

    pub fn name(&self) -> String {
        match self {
            Self::CreateClient => "create_client".to_string(),
            Self::ManageAuthorization => "manage_authorization".to_string(),
            Self::ManageClients => "manage_clients".to_string(),
            Self::ManageEvents => "manage_events".to_string(),
            Self::ManageIdentityProviders => "manage_identity_providers".to_string(),
            Self::ManageRealm => "manage_realm".to_string(),
            Self::ManageUsers => "manage_users".to_string(),
            Self::QueryClients => "query_clients".to_string(),
            Self::QueryGroups => "query_groups".to_string(),
            Self::QueryRealms => "query_realms".to_string(),
            Self::QueryUsers => "query_users".to_string(),
            Self::ViewAuthorization => "view_authorization".to_string(),
            Self::ViewClients => "view_clients".to_string(),
            Self::ViewEvents => "view_events".to_string(),
            Self::ViewIdentityProviders => "view_identity_providers".to_string(),
            Self::ViewRealm => "view_realm".to_string(),
            Self::ViewUsers => "view_users".to_string(),
        }
    }

    pub fn has_permissions(
        permissions: &[Permissions],
        required_permissions: &[Permissions],
    ) -> bool {
        required_permissions
            .iter()
            .all(|required_permission| permissions.contains(required_permission))
    }

    pub fn has_one_of_permissions(
        permissions: &[Permissions],
        required_permissions: &[Permissions],
    ) -> bool {
        required_permissions
            .iter()
            .any(|required_permission| permissions.contains(required_permission))
    }

    pub fn to_bitfield(permissions: &[Permissions]) -> u64 {
        permissions
            .iter()
            .fold(0, |acc, &permission| acc | (1 << permission as u64))
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "create_client" => Some(Self::CreateClient),
            "manage_authorization" => Some(Self::ManageAuthorization),
            "manage_clients" => Some(Self::ManageClients),
            "manage_events" => Some(Self::ManageEvents),
            "manage_identity_providers" => Some(Self::ManageIdentityProviders),
            "manage_realm" => Some(Self::ManageRealm),
            "manage_users" => Some(Self::ManageUsers),
            "query_clients" => Some(Self::QueryClients),
            "query_groups" => Some(Self::QueryGroups),
            "query_realms" => Some(Self::QueryRealms),
            "query_users" => Some(Self::QueryUsers),
            "view_authorization" => Some(Self::ViewAuthorization),
            "view_clients" => Some(Self::ViewClients),
            "view_events" => Some(Self::ViewEvents),
            "view_identity_providers" => Some(Self::ViewIdentityProviders),
            "view_realm" => Some(Self::ViewRealm),
            "view_users" => Some(Self::ViewUsers),
            _ => None,
        }
    }
}
