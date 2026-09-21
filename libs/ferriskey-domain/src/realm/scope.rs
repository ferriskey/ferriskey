use tracing::warn;

use crate::common::app_errors::CoreError;
use crate::realm::ports::RealmRepository;
use crate::realm::{Realm, RealmId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealmScope(Realm);

impl RealmScope {
    pub async fn resolve<R>(realms: &R, name: &str) -> Result<Self, CoreError>
    where
        R: RealmRepository,
    {
        let realm = realms
            .get_by_name(name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or_else(|| {
                warn!(realm_name = %name, "Refused a request for an unknown realm");
                CoreError::InvalidRealm
            })?;

        Ok(Self(realm))
    }

    pub fn from_realm(realm: Realm) -> Self {
        Self(realm)
    }

    pub fn id(&self) -> RealmId {
        self.0.id
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn realm(&self) -> &Realm {
        &self.0
    }
}

pub trait RealmOwned {
    fn realm_id(&self) -> RealmId;
}

#[must_use = "an unscoped value proves nothing until in_realm has accepted it"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unscoped<T>(T);

impl<T> Unscoped<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn across_realms(self) -> T {
        self.0
    }
}

impl<T> Unscoped<T>
where
    T: RealmOwned,
{
    pub fn in_realm(self, scope: &RealmScope) -> Result<Scoped<T>, CoreError> {
        let owner = self.0.realm_id();

        if owner == scope.id() {
            return Ok(Scoped(self.0));
        }

        warn!(
            owner_realm_id = %uuid::Uuid::from(owner),
            request_realm_id = %uuid::Uuid::from(scope.id()),
            request_realm_name = %scope.name(),
            "Refused cross-realm access to a resource"
        );

        Err(CoreError::NotFound)
    }
}

pub trait UnscopedOption<T> {
    fn in_realm(self, scope: &RealmScope) -> Result<Option<Scoped<T>>, CoreError>;
}

impl<T> UnscopedOption<T> for Option<Unscoped<T>>
where
    T: RealmOwned,
{
    fn in_realm(self, scope: &RealmScope) -> Result<Option<Scoped<T>>, CoreError> {
        self.map(|value| value.in_realm(scope)).transpose()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scoped<T>(T);

impl<T> Scoped<T> {
    pub fn get(&self) -> &T {
        &self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::realm::ports::MockRealmRepository;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Resource {
        realm_id: RealmId,
    }

    impl RealmOwned for Resource {
        fn realm_id(&self) -> RealmId {
            self.realm_id
        }
    }

    fn realm(name: &str) -> Realm {
        let mut realm = Realm::new(name.to_owned());
        realm.id = RealmId::new(Uuid::now_v7());
        realm
    }

    fn scope(name: &str) -> RealmScope {
        RealmScope::from_realm(realm(name))
    }

    #[test]
    fn a_resource_of_the_scoped_realm_is_accepted() {
        let scope = scope("acme");
        let resource = Resource {
            realm_id: scope.id(),
        };

        let accepted = Unscoped::new(resource.clone())
            .in_realm(&scope)
            .expect("a resource of the scoped realm must be accepted");

        assert_eq!(accepted.get(), &resource);
        assert_eq!(accepted.into_inner(), resource);
    }

    #[test]
    fn a_resource_of_another_realm_is_refused_as_not_found() {
        let scope = scope("acme");
        let resource = Resource {
            realm_id: RealmId::new(Uuid::now_v7()),
        };

        let refused = Unscoped::new(resource).in_realm(&scope);

        assert!(matches!(refused, Err(CoreError::NotFound)));
    }

    #[test]
    fn an_absent_resource_stays_absent_without_a_realm_verdict() {
        let scope = scope("acme");
        let absent: Option<Unscoped<Resource>> = None;

        let resolved = absent
            .in_realm(&scope)
            .expect("an absent resource must not be a realm refusal");

        assert!(resolved.is_none());
    }

    #[test]
    fn an_optional_resource_of_another_realm_is_refused() {
        let scope = scope("acme");
        let foreign = Some(Unscoped::new(Resource {
            realm_id: RealmId::new(Uuid::now_v7()),
        }));

        assert!(matches!(foreign.in_realm(&scope), Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn resolving_an_existing_realm_by_name_yields_its_scope() {
        let expected = realm("acme");
        let returned = expected.clone();

        let mut realms = MockRealmRepository::new();
        realms
            .expect_get_by_name()
            .withf(|name| name == "acme")
            .times(1)
            .returning(move |_| {
                let realm = returned.clone();
                Box::pin(async move { Ok(Some(realm)) })
            });

        let resolved = RealmScope::resolve(&realms, "acme")
            .await
            .expect("an existing realm must resolve");

        assert_eq!(resolved.id(), expected.id);
        assert_eq!(resolved.name(), "acme");
    }

    #[tokio::test]
    async fn resolving_an_unknown_realm_is_refused() {
        let mut realms = MockRealmRepository::new();
        realms
            .expect_get_by_name()
            .times(1)
            .returning(|_| Box::pin(async move { Ok(None) }));

        let resolved = RealmScope::resolve(&realms, "ghost").await;

        assert!(matches!(resolved, Err(CoreError::InvalidRealm)));
    }
}
