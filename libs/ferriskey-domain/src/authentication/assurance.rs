use std::collections::BTreeMap;
use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

/// A way the user proved who they are during a login.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationFactor {
    Password,
    Otp,
    RecoveryCode,
    /// A WebAuthn credential bound to one authenticator.
    HardwareKey,
    /// A WebAuthn credential its authenticator may sync to other devices.
    SyncedKey,
    MagicLink,
    IdentityProvider,
}

impl AuthenticationFactor {
    pub fn webauthn(backup_eligible: bool) -> Self {
        if backup_eligible {
            Self::SyncedKey
        } else {
            Self::HardwareKey
        }
    }

    /// The RFC 8176 value, when one describes this factor.
    pub fn amr(self) -> Option<&'static str> {
        match self {
            Self::Password => Some("pwd"),
            Self::Otp | Self::RecoveryCode => Some("otp"),
            Self::HardwareKey => Some("hwk"),
            Self::SyncedKey => Some("swk"),
            Self::MagicLink | Self::IdentityProvider => None,
        }
    }

    /// WebAuthn always requires user verification here, so a key alone is
    /// already two factors.
    fn reaches_multi_factor(self) -> bool {
        matches!(
            self,
            Self::Otp | Self::RecoveryCode | Self::HardwareKey | Self::SyncedKey
        )
    }
}

/// A factor the user answered, and when.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactorUse {
    pub factor: AuthenticationFactor,
    pub at: DateTime<Utc>,
}

impl FactorUse {
    pub fn now(factor: AuthenticationFactor) -> Self {
        Self {
            factor,
            at: Utc::now(),
        }
    }
}

/// A factor answered during a login that has not opened its session yet,
/// bound to the account that answered it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedFactor {
    pub user_id: Uuid,
    #[serde(flatten)]
    pub factor: FactorUse,
}

impl VerifiedFactor {
    pub fn now(user_id: Uuid, factor: AuthenticationFactor) -> Self {
        Self {
            user_id,
            factor: FactorUse::now(factor),
        }
    }
}

/// The factors `user_id` answered, dropping those another account answered
/// in the same login.
pub fn factors_of(verified: &[VerifiedFactor], user_id: Uuid) -> Vec<FactorUse> {
    verified
        .iter()
        .filter(|verified| verified.user_id == user_id)
        .map(|verified| verified.factor)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub enum LevelOfAssurance {
    SingleFactor = 1,
    MultiFactor = 2,
}

impl LevelOfAssurance {
    /// `None` when nothing was recorded, as for a session opened before
    /// factors were.
    pub fn of(factors: &[FactorUse]) -> Option<Self> {
        if factors.is_empty() {
            return None;
        }

        if factors
            .iter()
            .any(|used| used.factor.reaches_multi_factor())
        {
            Some(Self::MultiFactor)
        } else {
            Some(Self::SingleFactor)
        }
    }

    /// The built-in `acr` value naming this level.
    pub fn acr(self) -> &'static str {
        match self {
            Self::SingleFactor => "1",
            Self::MultiFactor => "2",
        }
    }

    fn from_builtin_acr(value: &str) -> Option<Self> {
        match value {
            "1" => Some(Self::SingleFactor),
            "2" => Some(Self::MultiFactor),
            _ => None,
        }
    }
}

impl TryFrom<u8> for LevelOfAssurance {
    type Error = AcrLoaMapError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::SingleFactor),
            2 => Ok(Self::MultiFactor),
            other => Err(AcrLoaMapError::UnknownLevel(other)),
        }
    }
}

impl From<LevelOfAssurance> for u8 {
    fn from(value: LevelOfAssurance) -> Self {
        value as u8
    }
}

impl Display for LevelOfAssurance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.acr())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AcrLoaMapError {
    #[error("level of assurance {0} does not exist, use 1 or 2")]
    UnknownLevel(u8),
    #[error("acr value {0:?} must be non-empty and contain no whitespace")]
    InvalidAcr(String),
}

/// The realm's own `acr` names, each mapped to a level of assurance. The
/// built-in values "1" and "2" always exist.
#[derive(
    Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema,
)]
#[serde(try_from = "BTreeMap<String, LevelOfAssurance>")]
#[schema(value_type = BTreeMap<String, u8>)]
pub struct AcrLoaMap(BTreeMap<String, LevelOfAssurance>);

impl AcrLoaMap {
    pub fn level_of(&self, acr: &str) -> Option<LevelOfAssurance> {
        self.0
            .get(acr)
            .copied()
            .or_else(|| LevelOfAssurance::from_builtin_acr(acr))
    }

    /// Every `acr` value a client may request, for discovery.
    pub fn supported_values(&self) -> Vec<String> {
        let builtins = [
            LevelOfAssurance::SingleFactor,
            LevelOfAssurance::MultiFactor,
        ]
        .map(|level| level.acr().to_string());

        builtins
            .into_iter()
            .chain(
                self.0
                    .keys()
                    .filter(|acr| LevelOfAssurance::from_builtin_acr(acr).is_none())
                    .cloned(),
            )
            .collect()
    }

    pub fn as_map(&self) -> &BTreeMap<String, LevelOfAssurance> {
        &self.0
    }
}

impl TryFrom<BTreeMap<String, LevelOfAssurance>> for AcrLoaMap {
    type Error = AcrLoaMapError;

    fn try_from(value: BTreeMap<String, LevelOfAssurance>) -> Result<Self, Self::Error> {
        if let Some(acr) = value
            .keys()
            .find(|acr| acr.is_empty() || acr.chars().any(char::is_whitespace))
        {
            return Err(AcrLoaMapError::InvalidAcr(acr.clone()));
        }

        Ok(Self(value))
    }
}

/// The `acr` values a client asked for, in its order of preference.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AcrRequest(Vec<String>);

impl AcrRequest {
    /// Reads a space-separated `acr_values` parameter.
    pub fn parse(acr_values: Option<&str>) -> Self {
        Self(
            acr_values
                .unwrap_or_default()
                .split_whitespace()
                .map(str::to_string)
                .collect(),
        )
    }

    pub fn from_values(values: Vec<String>) -> Self {
        Self(values)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The space-separated form, as stored with the authorization request.
    pub fn to_param(&self) -> Option<String> {
        (!self.0.is_empty()).then(|| self.0.join(" "))
    }

    /// The level the most preferred value the realm knows asks for.
    pub fn required_level(&self, map: &AcrLoaMap) -> Option<LevelOfAssurance> {
        self.0.iter().find_map(|acr| map.level_of(acr))
    }

    /// Whether at least one value names a level the realm knows.
    pub fn names_a_known_level(&self, map: &AcrLoaMap) -> bool {
        self.required_level(map).is_some()
    }

    /// Whether a session at `achieved` answers this request.
    pub fn is_satisfied_by(&self, map: &AcrLoaMap, achieved: Option<LevelOfAssurance>) -> bool {
        match self.required_level(map) {
            None => true,
            Some(required) => achieved.is_some_and(|achieved| achieved >= required),
        }
    }

    /// The `acr` to assert: the most preferred requested value the session
    /// reaches, else the built-in name of the level it reached.
    fn acr_for(&self, map: &AcrLoaMap, achieved: LevelOfAssurance) -> String {
        self.0
            .iter()
            .find(|acr| map.level_of(acr).is_some_and(|level| level <= achieved))
            .cloned()
            .unwrap_or_else(|| achieved.acr().to_string())
    }
}

/// How a session authenticated, as the `acr` and `amr` claims tell it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticationContext {
    pub acr: String,
    pub amr: Vec<&'static str>,
}

impl AuthenticationContext {
    /// `None` when the session recorded no factor.
    pub fn resolve(factors: &[FactorUse], map: &AcrLoaMap, request: &AcrRequest) -> Option<Self> {
        let level = LevelOfAssurance::of(factors)?;

        let mut amr: Vec<&'static str> = Vec::with_capacity(factors.len() + 1);
        for value in factors.iter().filter_map(|used| used.factor.amr()) {
            if !amr.contains(&value) {
                amr.push(value);
            }
        }
        if level == LevelOfAssurance::MultiFactor {
            amr.push("mfa");
        }

        Some(Self {
            acr: request.acr_for(map, level),
            amr,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn used(factors: &[AuthenticationFactor]) -> Vec<FactorUse> {
        factors.iter().copied().map(FactorUse::now).collect()
    }

    fn map(entries: &[(&str, LevelOfAssurance)]) -> AcrLoaMap {
        AcrLoaMap::try_from(
            entries
                .iter()
                .map(|(acr, level)| (acr.to_string(), *level))
                .collect::<BTreeMap<_, _>>(),
        )
        .unwrap()
    }

    #[test]
    fn a_password_alone_is_a_single_factor_login() {
        let context = AuthenticationContext::resolve(
            &used(&[AuthenticationFactor::Password]),
            &AcrLoaMap::default(),
            &AcrRequest::default(),
        )
        .unwrap();

        assert_eq!(context.acr, "1");
        assert_eq!(context.amr, vec!["pwd"]);
    }

    #[test]
    fn a_password_and_an_otp_reach_multi_factor() {
        let context = AuthenticationContext::resolve(
            &used(&[AuthenticationFactor::Password, AuthenticationFactor::Otp]),
            &AcrLoaMap::default(),
            &AcrRequest::default(),
        )
        .unwrap();

        assert_eq!(context.acr, "2");
        assert_eq!(context.amr, vec!["pwd", "otp", "mfa"]);
    }

    #[test]
    fn a_recovery_code_counts_as_the_otp_it_replaces() {
        let context = AuthenticationContext::resolve(
            &used(&[
                AuthenticationFactor::Password,
                AuthenticationFactor::RecoveryCode,
            ]),
            &AcrLoaMap::default(),
            &AcrRequest::default(),
        )
        .unwrap();

        assert_eq!(context.amr, vec!["pwd", "otp", "mfa"]);
    }

    #[test]
    fn a_user_verified_passkey_alone_is_multi_factor() {
        for (backup_eligible, amr) in [(false, "hwk"), (true, "swk")] {
            let context = AuthenticationContext::resolve(
                &used(&[AuthenticationFactor::webauthn(backup_eligible)]),
                &AcrLoaMap::default(),
                &AcrRequest::default(),
            )
            .unwrap();

            assert_eq!(context.acr, "2");
            assert_eq!(context.amr, vec![amr, "mfa"]);
        }
    }

    #[test]
    fn a_factor_without_an_rfc_8176_value_still_sets_the_level() {
        let context = AuthenticationContext::resolve(
            &used(&[AuthenticationFactor::MagicLink]),
            &AcrLoaMap::default(),
            &AcrRequest::default(),
        )
        .unwrap();

        assert_eq!(context.acr, "1");
        assert!(context.amr.is_empty());
    }

    #[test]
    fn a_method_used_twice_is_listed_once() {
        let context = AuthenticationContext::resolve(
            &used(&[
                AuthenticationFactor::Otp,
                AuthenticationFactor::Password,
                AuthenticationFactor::Otp,
            ]),
            &AcrLoaMap::default(),
            &AcrRequest::default(),
        )
        .unwrap();

        assert_eq!(context.amr, vec!["otp", "pwd", "mfa"]);
    }

    #[test]
    fn a_session_without_recorded_factors_asserts_nothing() {
        assert_eq!(
            AuthenticationContext::resolve(&[], &AcrLoaMap::default(), &AcrRequest::default()),
            None
        );
    }

    #[test]
    fn the_most_preferred_requested_value_the_session_reaches_is_asserted() {
        let map = map(&[
            ("gold", LevelOfAssurance::MultiFactor),
            ("silver", LevelOfAssurance::SingleFactor),
        ]);
        let request = AcrRequest::parse(Some("gold silver"));

        let mfa = AuthenticationContext::resolve(
            &used(&[AuthenticationFactor::Password, AuthenticationFactor::Otp]),
            &map,
            &request,
        )
        .unwrap();
        let single = AuthenticationContext::resolve(
            &used(&[AuthenticationFactor::Password]),
            &map,
            &request,
        )
        .unwrap();

        assert_eq!(mfa.acr, "gold");
        assert_eq!(single.acr, "silver");
    }

    #[test]
    fn a_request_the_session_falls_short_of_gets_the_level_it_reached() {
        let context = AuthenticationContext::resolve(
            &used(&[AuthenticationFactor::Password]),
            &AcrLoaMap::default(),
            &AcrRequest::parse(Some("2")),
        )
        .unwrap();

        assert_eq!(context.acr, "1");
    }

    #[test]
    fn the_required_level_follows_the_first_value_the_realm_knows() {
        let map = map(&[("gold", LevelOfAssurance::MultiFactor)]);

        assert_eq!(
            AcrRequest::parse(Some("urn:unknown gold 1")).required_level(&map),
            Some(LevelOfAssurance::MultiFactor)
        );
        assert_eq!(
            AcrRequest::parse(Some("urn:unknown")).required_level(&map),
            None
        );
        assert_eq!(AcrRequest::parse(None).required_level(&map), None);
    }

    #[test]
    fn only_a_session_at_the_required_level_satisfies_a_request() {
        let map = AcrLoaMap::default();
        let request = AcrRequest::parse(Some("2"));

        assert!(request.is_satisfied_by(&map, Some(LevelOfAssurance::MultiFactor)));
        assert!(!request.is_satisfied_by(&map, Some(LevelOfAssurance::SingleFactor)));
        assert!(!request.is_satisfied_by(&map, None));
        assert!(AcrRequest::default().is_satisfied_by(&map, None));
    }

    #[test]
    fn a_realm_name_cannot_carry_whitespace() {
        let mut entries = BTreeMap::new();
        entries.insert("two words".to_string(), LevelOfAssurance::SingleFactor);

        assert_eq!(
            AcrLoaMap::try_from(entries),
            Err(AcrLoaMapError::InvalidAcr("two words".to_string()))
        );
    }

    #[test]
    fn a_map_deserializes_from_a_json_object_and_refuses_unknown_levels() {
        let map: AcrLoaMap = serde_json::from_str(r#"{"gold": 2, "silver": 1}"#).unwrap();
        assert_eq!(map.level_of("gold"), Some(LevelOfAssurance::MultiFactor));
        assert_eq!(map.level_of("1"), Some(LevelOfAssurance::SingleFactor));
        assert_eq!(map.supported_values(), vec!["1", "2", "gold", "silver"]);

        assert!(serde_json::from_str::<AcrLoaMap>(r#"{"platinum": 3}"#).is_err());
    }

    #[test]
    fn factors_answered_by_another_account_are_dropped() {
        let alice = Uuid::new_v4();
        let mallory = Uuid::new_v4();
        let verified = vec![
            VerifiedFactor::now(mallory, AuthenticationFactor::Password),
            VerifiedFactor::now(alice, AuthenticationFactor::HardwareKey),
        ];

        let factors = factors_of(&verified, alice);

        assert_eq!(factors.len(), 1);
        assert_eq!(factors[0].factor, AuthenticationFactor::HardwareKey);
    }

    #[test]
    fn a_verified_factor_round_trips_through_its_stored_form() {
        let verified = VerifiedFactor::now(Uuid::new_v4(), AuthenticationFactor::Otp);
        let json = serde_json::to_value(verified).unwrap();

        assert_eq!(json["factor"], "otp");
        assert_eq!(
            serde_json::from_value::<VerifiedFactor>(json).unwrap(),
            verified
        );
    }
}
