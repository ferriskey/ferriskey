use ferriskey_domain::realm::{Realm, RealmId, RealmSetting};
use uuid::Uuid;

fn main() {
    divan::main();
}

#[divan::bench]
fn realm_new() -> Realm {
    Realm::new("bench-realm".to_string())
}

#[divan::bench]
fn realm_setting_new(bencher: divan::Bencher) {
    let realm_id = RealmId::new(Uuid::new_v4());

    bencher.bench(|| RealmSetting::new(realm_id, Some("RS256".to_string())));
}

#[divan::bench]
fn realm_can_delete_regular() -> bool {
    let realm = divan::black_box(Realm::new("bench-realm".to_string()));
    realm.can_delete()
}

#[divan::bench]
fn realm_can_delete_master() -> bool {
    let realm = divan::black_box(Realm::new("master".to_string()));
    realm.can_delete()
}

#[divan::bench]
fn realm_serialize(bencher: divan::Bencher) {
    let realm = Realm::new("bench-realm".to_string());

    bencher.bench(|| serde_json::to_string(&realm).expect("serialize failed"));
}

#[divan::bench]
fn realm_deserialize(bencher: divan::Bencher) {
    let json =
        serde_json::to_string(&Realm::new("bench-realm".to_string())).expect("serialize failed");

    bencher
        .with_inputs(|| json.clone())
        .bench_values(|j| serde_json::from_str::<Realm>(&j).expect("deserialize failed"));
}
