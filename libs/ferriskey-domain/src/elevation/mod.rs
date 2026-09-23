pub mod entities;
pub mod ports;

use chrono::{DateTime, Utc};
use tracing::warn;
use uuid::Uuid;

use crate::common::app_errors::CoreError;
use crate::elevation::entities::{Elevated, ElevationId};
use crate::elevation::ports::ElevationRepository;
use crate::realm::scope::RealmScope;

pub struct Claim {
    pub caller: Uuid,
    pub session_id: Uuid,
    pub elevation_id: ElevationId,
    pub now: DateTime<Utc>,
}

pub async fn claim<R>(
    elevations: &R,
    scope: &RealmScope,
    request: Claim,
) -> Result<Elevated, CoreError>
where
    R: ElevationRepository,
{
    let scoped = elevations
        .find_live(request.elevation_id, request.caller, request.now)
        .await?
        .ok_or_else(|| {
            warn!(
                caller = %request.caller,
                "Refused a sensitive operation: no live elevation proof for this caller"
            );
            CoreError::ElevationRequired
        })?
        .in_realm(scope)?;

    let elevation = scoped.get();

    if elevation.user_id != request.caller {
        warn!(
            caller = %request.caller,
            owner = %elevation.user_id,
            "Refused a sensitive operation: the elevation proof belongs to another user"
        );
        return Err(CoreError::ElevationRequired);
    }

    if elevation.session_id != request.session_id {
        warn!(
            caller = %request.caller,
            "Refused a sensitive operation: the elevation proof was minted for another session"
        );
        return Err(CoreError::ElevationRequired);
    }

    if elevation.expires_at <= request.now {
        warn!(
            caller = %request.caller,
            "Refused a sensitive operation: the elevation proof has expired"
        );
        return Err(CoreError::ElevationRequired);
    }

    Ok(Elevated::over(
        elevation.user_id,
        elevation.realm_id,
        elevation.session_id,
        elevation.proof,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elevation::entities::{Elevation, ElevationProofKind};
    use crate::elevation::ports::MockElevationRepository;
    use crate::realm::scope::Unscoped;
    use crate::realm::{Realm, RealmId};
    use chrono::Duration;

    fn scope(name: &str) -> RealmScope {
        let mut realm = Realm::new(name.to_owned());
        realm.id = RealmId::new(Uuid::now_v7());
        RealmScope::from_realm(realm)
    }

    fn elevation(user_id: Uuid, realm_id: RealmId, session_id: Uuid) -> Elevation {
        Elevation {
            id: ElevationId::new(Uuid::now_v7()),
            user_id,
            realm_id,
            session_id,
            proof: ElevationProofKind::Password,
            expires_at: Utc::now() + Duration::minutes(5),
        }
    }

    fn request(caller: Uuid, session_id: Uuid, elevation_id: ElevationId) -> Claim {
        Claim {
            caller,
            session_id,
            elevation_id,
            now: Utc::now(),
        }
    }

    fn returning(stored: Elevation) -> MockElevationRepository {
        let mut elevations = MockElevationRepository::new();
        elevations
            .expect_find_live()
            .times(1)
            .returning(move |_, _, _| {
                let stored = stored.clone();
                Box::pin(async move { Ok(Some(Unscoped::new(stored))) })
            });

        elevations
    }

    #[tokio::test]
    async fn a_live_proof_of_the_caller_yields_an_elevation() {
        let scope = scope("acme");
        let caller = Uuid::now_v7();
        let session = Uuid::now_v7();
        let stored = elevation(caller, scope.id(), session);
        let id = stored.id;

        let elevated = claim(&returning(stored), &scope, request(caller, session, id))
            .await
            .expect("a live proof of the caller must be claimable");

        assert_eq!(elevated.user_id(), caller);
        assert_eq!(elevated.realm_id(), scope.id());
        assert_eq!(elevated.session_id(), session);
        assert_eq!(elevated.proof(), ElevationProofKind::Password);
    }

    #[tokio::test]
    async fn an_absent_proof_is_refused() {
        let scope = scope("acme");

        let mut elevations = MockElevationRepository::new();
        elevations
            .expect_find_live()
            .times(1)
            .returning(|_, _, _| Box::pin(async move { Ok(None) }));

        let claimed = claim(
            &elevations,
            &scope,
            request(
                Uuid::now_v7(),
                Uuid::now_v7(),
                ElevationId::new(Uuid::now_v7()),
            ),
        )
        .await;

        assert!(matches!(claimed, Err(CoreError::ElevationRequired)));
    }

    #[tokio::test]
    async fn a_proof_minted_in_another_realm_is_refused_as_not_found() {
        let scope = scope("acme");
        let caller = Uuid::now_v7();
        let session = Uuid::now_v7();
        let stored = elevation(caller, RealmId::new(Uuid::now_v7()), session);
        let id = stored.id;

        let claimed = claim(&returning(stored), &scope, request(caller, session, id)).await;

        assert!(matches!(claimed, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn a_proof_minted_for_another_user_is_refused() {
        let scope = scope("acme");
        let session = Uuid::now_v7();
        let stored = elevation(Uuid::now_v7(), scope.id(), session);
        let id = stored.id;

        let claimed = claim(
            &returning(stored),
            &scope,
            request(Uuid::now_v7(), session, id),
        )
        .await;

        assert!(matches!(claimed, Err(CoreError::ElevationRequired)));
    }

    #[tokio::test]
    async fn a_proof_minted_for_another_session_is_refused() {
        let scope = scope("acme");
        let caller = Uuid::now_v7();
        let stored = elevation(caller, scope.id(), Uuid::now_v7());
        let id = stored.id;

        let claimed = claim(
            &returning(stored),
            &scope,
            request(caller, Uuid::now_v7(), id),
        )
        .await;

        assert!(matches!(claimed, Err(CoreError::ElevationRequired)));
    }

    #[tokio::test]
    async fn an_expired_proof_the_adapter_handed_back_anyway_is_refused() {
        let scope = scope("acme");
        let caller = Uuid::now_v7();
        let session = Uuid::now_v7();
        let mut stored = elevation(caller, scope.id(), session);
        stored.expires_at = Utc::now() - Duration::seconds(1);
        let id = stored.id;

        let claimed = claim(&returning(stored), &scope, request(caller, session, id)).await;

        assert!(matches!(claimed, Err(CoreError::ElevationRequired)));
    }

    #[tokio::test]
    async fn an_otp_proof_does_not_pass_for_a_primary_operation() {
        let scope = scope("acme");
        let caller = Uuid::now_v7();
        let session = Uuid::now_v7();
        let mut stored = elevation(caller, scope.id(), session);
        stored.proof = ElevationProofKind::Otp;
        let id = stored.id;

        let elevated = claim(&returning(stored), &scope, request(caller, session, id))
            .await
            .expect("an OTP proof is still a valid elevation");

        assert!(matches!(
            elevated.primary(),
            Err(CoreError::PrimaryProofRequired)
        ));
    }
}
