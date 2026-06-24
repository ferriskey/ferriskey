use std::collections::HashSet;

use divan::{self, black_box};
use ferriskey_core::domain::role::entities::permission::Permissions;

fn main() {
    divan::main();
}

// -- Bitfield encoding / decoding -------------------------------------------

#[divan::bench]
fn to_bitfield_few() -> u64 {
    Permissions::to_bitfield(black_box(&[
        Permissions::ManageUsers,
        Permissions::ViewClients,
    ]))
}

#[divan::bench]
fn to_bitfield_many() -> u64 {
    Permissions::to_bitfield(black_box(&[
        Permissions::CreateClient,
        Permissions::ManageAuthorization,
        Permissions::ManageClients,
        Permissions::ManageEvents,
        Permissions::ManageIdentityProviders,
        Permissions::ManageRealm,
        Permissions::ManageUsers,
        Permissions::ManageRoles,
        Permissions::QueryClients,
        Permissions::QueryGroups,
        Permissions::QueryRealms,
        Permissions::QueryUsers,
        Permissions::ViewAuthorization,
        Permissions::ViewClients,
        Permissions::ViewEvents,
        Permissions::ViewIdentityProviders,
        Permissions::ViewRealm,
        Permissions::ViewUsers,
        Permissions::ViewRoles,
    ]))
}

#[divan::bench]
fn from_bitfield_few() -> Vec<Permissions> {
    let bitfield = Permissions::ManageUsers as u64 | Permissions::ViewClients as u64;
    Permissions::from_bitfield(black_box(bitfield))
}

#[divan::bench]
fn from_bitfield_all() -> Vec<Permissions> {
    let bitfield: u64 = (1u64 << 27) - 1; // all 27 permission bits set
    Permissions::from_bitfield(black_box(bitfield))
}

// -- Permission checks ------------------------------------------------------

#[divan::bench]
fn has_permissions_match() -> bool {
    let user = vec![
        Permissions::ManageUsers,
        Permissions::ViewClients,
        Permissions::QueryUsers,
    ];
    Permissions::has_permissions(
        black_box(&user),
        black_box(&[Permissions::ManageUsers, Permissions::ViewClients]),
    )
}

#[divan::bench]
fn has_permissions_no_match() -> bool {
    let user = vec![Permissions::ManageUsers];
    Permissions::has_permissions(
        black_box(&user),
        black_box(&[Permissions::ManageUsers, Permissions::ViewRealm]),
    )
}

#[divan::bench]
fn has_one_of_permissions_match() -> bool {
    let user: HashSet<Permissions> =
        HashSet::from([Permissions::ManageUsers, Permissions::ViewClients]);
    Permissions::has_one_of_permissions(
        black_box(&user),
        black_box(&[Permissions::ManageClients, Permissions::ViewClients]),
    )
}

// -- Name round-trip --------------------------------------------------------

#[divan::bench]
fn from_name() -> Option<Permissions> {
    Permissions::from_name(black_box("manage_users"))
}

#[divan::bench]
fn from_names_batch() -> Vec<Permissions> {
    Permissions::from_names(black_box(&[
        "create_client".to_string(),
        "manage_users".to_string(),
        "query_realms".to_string(),
        "view_webhooks".to_string(),
    ]))
}
