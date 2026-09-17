use crate::validators::UpdateOwnLocaleValidator;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorBody, ApiErrorResponse, ValidateJson},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::common::locale::{Locale, LocaleError, SupportedLocales};
use ferriskey_core::domain::realm::ports::RealmService;
use ferriskey_core::domain::user::entities::{UpdateOwnLocaleInput, User};
use ferriskey_core::domain::user::ports::UserService;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdateOwnLocaleResponse {
    pub data: User,
}

fn locale_rejected(error: &LocaleError) -> ApiError {
    let code = match error {
        LocaleError::Malformed(_) => "malformed_locale",
        LocaleError::Unsupported(_) => "unsupported_locale",
        LocaleError::EmptySupportedSet => "empty_supported_locales",
        LocaleError::DefaultNotSupported => "default_locale_not_supported",
    };

    ApiError::validation_error("locale", code, error.to_string())
}

fn realm_locales_corrupted(error: &LocaleError) -> ApiError {
    ApiError::InternalServerError(ApiErrorBody::new(
        format!("the realm carries an unusable locale configuration: {error}"),
        "invalid_realm_locales",
    ))
}

async fn supported_locales(
    state: &AppState,
    realm_name: &str,
) -> Result<SupportedLocales, ApiError> {
    let settings = state
        .service
        .get_login_settings(realm_name.to_string())
        .await
        .map_err(ApiError::from)?;

    let default =
        Locale::parse(&settings.default_locale).map_err(|error| realm_locales_corrupted(&error))?;
    let all = settings
        .supported_locales
        .iter()
        .map(|raw| Locale::parse(raw).map_err(|error| realm_locales_corrupted(&error)))
        .collect::<Result<Vec<Locale>, ApiError>>()?;

    SupportedLocales::new(default, all).map_err(|error| realm_locales_corrupted(&error))
}

#[utoipa::path(
    put,
    path = "/@me/locale",
    tag = "user",
    summary = "Set the caller's own locale",
    description = "Stores the locale the currently authenticated user reads the product in. The locale must belong to the realm's supported set; sending null clears it so the realm default applies.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body(
        content = UpdateOwnLocaleValidator,
        description = "Locale to store, or null to fall back to the realm default",
        content_type = "application/json",
    ),
    responses(
        (status = 200, description = "Locale updated successfully", body = UpdateOwnLocaleResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 422, description = "Locale is malformed or not supported by the realm", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn update_me_locale(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateOwnLocaleValidator>,
) -> Result<Response<UpdateOwnLocaleResponse>, ApiError> {
    let locale = match payload.locale {
        Some(raw) => {
            let supported = supported_locales(&state, &realm_name).await?;
            let locale = Locale::parse(&raw).map_err(|error| locale_rejected(&error))?;

            if !supported.contains(&locale) {
                return Err(locale_rejected(&LocaleError::Unsupported(
                    locale.to_string(),
                )));
            }

            Some(locale.to_string())
        }
        None => None,
    };

    let user = state
        .service
        .update_own_locale(identity, UpdateOwnLocaleInput { realm_name, locale })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Updated(UpdateOwnLocaleResponse { data: user }))
}
