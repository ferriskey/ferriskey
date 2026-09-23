pub mod entities;
pub mod ports;

use chrono::{DateTime, Utc};
use tracing::warn;
use uuid::Uuid;

use crate::common::app_errors::CoreError;
use crate::elevation::entities::{Elevated, ElevationId};
use crate::elevation::ports::ElevationRepository;
use crate::realm::scope::RealmScope;

pub async fn claim<R>(
    elevations: &R,
    scope: &RealmScope,
    caller: Uuid,
    id: ElevationId,
    now: DateTime<Utc>,
) -> Result<Elevated, CoreError>
where
    R: ElevationRepository,
{
    let elevation = elevations
        .consume(id, now)
        .await?
        .ok_or_else(|| {
            warn!(
                caller = %caller,
                "Refused a sensitive operation: no live elevation proof to claim"
            );
            CoreError::ElevationRequired
        })?
        .in_realm(scope)?;

    let elevation = elevation.get();

    if elevation.user_id != caller {
        warn!(
            caller = %caller,
            owner = %elevation.user_id,
            "Refused a sensitive operation: the elevation proof belongs to another user"
        );
        return Err(CoreError::ElevationRequired);
    }

    Ok(Elevated::over(elevation.user_id, elevation.realm_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elevation::entities::Elevation;
    use crate::elevation::ports::MockElevationRepository;
    use crate::realm::scope::Unscoped;
    use crate::realm::{Realm, RealmId};
    use chrono::Duration;

    fn scope(name: &str) -> RealmScope {
        let mut realm = Realm::new(name.to_owned());
        realm.id = RealmId::new(Uuid::now_v7());
        RealmScope::from_realm(realm)
    }

    fn elevation(user_id: Uuid, realm_id: RealmId) -> Elevation {
        Elevation {
            id: ElevationId::new(Uuid::now_v7()),
            user_id,
            realm_id,
            expires_at: Utc::now() + Duration::minutes(5),
        }
    }

    #[tokio::test]
    async fn a_live_proof_of_the_caller_yields_an_elevation() {
        let scope = scope("acme");
        let caller = Uuid::now_v7();
        let stored = elevation(caller, scope.id());
        let id = stored.id;

        let mut elevations = MockElevationRepository::new();
        elevations.expect_consume().times(1).returning(move |_, _| {
            let stored = stored.clone();
            Box::pin(async move { Ok(Some(Unscoped::new(stored))) })
        });

        let elevated = claim(&elevations, &scope, caller, id, Utc::now())
            .await
            .expect("a live proof of the caller must be claimable");

        assert_eq!(elevated.user_id(), caller);
        assert_eq!(elevated.realm_id(), scope.id());
    }

    #[tokio::test]
    async fn an_absent_or_spent_proof_is_refused() {
        let scope = scope("acme");

        let mut elevations = MockElevationRepository::new();
        elevations
            .expect_consume()
            .times(1)
            .returning(|_, _| Box::pin(async move { Ok(None) }));

        let claimed = claim(
            &elevations,
            &scope,
            Uuid::now_v7(),
            ElevationId::new(Uuid::now_v7()),
            Utc::now(),
        )
        .await;

        assert!(matches!(claimed, Err(CoreError::ElevationRequired)));
    }

    #[tokio::test]
    async fn a_proof_minted_in_another_realm_is_refused_as_not_found() {
        let scope = scope("acme");
        let caller = Uuid::now_v7();
        let stored = elevation(caller, RealmId::new(Uuid::now_v7()));
        let id = stored.id;

        let mut elevations = MockElevationRepository::new();
        elevations.expect_consume().times(1).returning(move |_, _| {
            let stored = stored.clone();
            Box::pin(async move { Ok(Some(Unscoped::new(stored))) })
        });

        let claimed = claim(&elevations, &scope, caller, id, Utc::now()).await;

        assert!(matches!(claimed, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn a_proof_minted_for_another_user_is_refused() {
        let scope = scope("acme");
        let stored = elevation(Uuid::now_v7(), scope.id());
        let id = stored.id;

        let mut elevations = MockElevationRepository::new();
        elevations.expect_consume().times(1).returning(move |_, _| {
            let stored = stored.clone();
            Box::pin(async move { Ok(Some(Unscoped::new(stored))) })
        });

        let claimed = claim(&elevations, &scope, Uuid::now_v7(), id, Utc::now()).await;

        assert!(matches!(claimed, Err(CoreError::ElevationRequired)));
    }
}
