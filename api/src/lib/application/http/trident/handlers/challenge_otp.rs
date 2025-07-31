use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use axum::{Extension, extract::State};
use axum_cookie::CookieManager;
use axum_macros::TypedPath;
use ferriskey_core::domain::authentication::ports::AuthSessionService;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::common::generate_random_string;
use ferriskey_core::domain::credential::ports::CredentialService;
use ferriskey_core::domain::trident::entities::TotpSecret;
use ferriskey_core::domain::trident::ports::TotpService;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[typeshare]
pub struct ChallengeOtpRequest {
    #[validate(length(min = 6, max = 6, message = "OTP code must be exactly 6 digits"))]
    #[serde(default)]
    pub code: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/login-actions/challenge-otp")]
pub struct ChallengeOtpRoute {
    pub realm_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
#[typeshare]
pub struct ChallengeOtpResponse {
    pub url: String,
}

#[utoipa::path(
  post,
  path = "/login-actions/challenge-otp",
  tag = "auth",
  responses(
    (status = 200, body = ChallengeOtpResponse)
  )
)]
pub async fn challenge_otp(
    ChallengeOtpRoute { realm_name: _ }: ChallengeOtpRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    cookie: CookieManager,
    ValidateJson(payload): ValidateJson<ChallengeOtpRequest>,
) -> Result<Response<ChallengeOtpResponse>, ApiError> {
    let session_code = cookie.get("FERRISKEY_SESSION").unwrap();
    let session_code = session_code.value().to_string();
    let session_code = Uuid::parse_str(&session_code).unwrap();

    let user = match identity {
        Identity::User(user) => user,
        _ => {
            return Err(ApiError::Forbidden(
                "Only users can challenge OTP".to_string(),
            ));
        }
    };

    let auth_session = state
        .service_bundle
        .auth_session_service
        .get_by_session_code(session_code)
        .await
        .map_err(|_| ApiError::Unauthorized("invalid session code".to_string()))?;

    let user_credentials = state
        .service_bundle
        .credential_service
        .get_credentials_by_user_id(user.id)
        .await
        .map_err(|_| {
            ApiError::InternalServerError("Failed to fetch user credentials".to_string())
        })?;

    let otp_credential = user_credentials
        .iter()
        .find(|cred| cred.credential_type == "otp")
        .ok_or_else(|| ApiError::BadRequest("User has no OTP configured".to_string()))?;

    let secret = TotpSecret::from_base32(&otp_credential.secret_data);

    let is_valid = state
        .service_bundle
        .totp_service
        .verify(&secret, &payload.code)
        .map_err(|_| ApiError::InternalServerError("Failed to verify OTP".to_string()))?;

    if !is_valid {
        error!("Invalid OTP code for user: {}", user.email);
        return Err(ApiError::Unauthorized("failed to verify OTP".to_string()));
    }

    let authorization_code = generate_random_string();

    state
        .service_bundle
        .auth_session_service
        .update_code(session_code, authorization_code.clone(), user.id)
        .await
        .map_err(|_| ApiError::InternalServerError("Failed to update session".to_string()))?;

    let current_state = auth_session
        .state
        .ok_or(ApiError::Unauthorized("Invalid session state".to_string()))?;

    let login_url = format!(
        "{}?code={}&state={}",
        auth_session.redirect_uri, authorization_code, current_state
    );

    info!(
        "OTP challenge completed successfully for user: {}",
        user.email
    );

    let response = ChallengeOtpResponse { url: login_url };

    Ok(Response::OK(response))
}
