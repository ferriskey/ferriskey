use axum::{
    Extension,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    realm_transfer::{
        bundle::{BUNDLE_EXTENSION, BUNDLE_MEDIA_TYPE, BundlePassphrase},
        ports::RealmTransferService,
        value_objects::ImportReport,
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const PASSPHRASE_HEADER: &str = "x-ferriskey-bundle-passphrase";

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ImportReportResponse {
    pub data: ImportReport,
}

fn passphrase(headers: &HeaderMap) -> Result<BundlePassphrase, ApiError> {
    let value = headers
        .get(PASSPHRASE_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or(CoreError::InvalidBundlePassphrase)?;

    Ok(BundlePassphrase::try_from(value.to_owned())?)
}

#[utoipa::path(
    post,
    path = "/{realm_name}/export",
    tag = "realm",
    summary = "Export a realm as an encrypted bundle",
    description = "Streams every row of the realm that a move to another instance needs: users and their credentials, clients, roles, scopes, organizations, identity providers, all signing keys with their kid, sessions, refresh tokens and settings. The file is encrypted with a key derived from the passphrase; secrets are opened with this instance's key ring and only exist in clear inside the encrypted file. Stop the traffic to the realm before exporting. Needs the right to create realms and to manage this realm. The master realm cannot be exported.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("X-Ferriskey-Bundle-Passphrase" = String, Header, description = "16 to 1024 printable ASCII characters"),
    ),
    responses(
        (status = 200, description = "The bundle", content_type = "application/vnd.ferriskey.realm-bundle", body = Vec<u8>),
        (status = 400, description = "Invalid passphrase, or the master realm", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn export_realm(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let passphrase = passphrase(&headers)?;
    let disposition = format!("attachment; filename=\"{realm_name}.{BUNDLE_EXTENSION}\"");
    let bundle = state
        .service
        .export_realm(identity, realm_name, passphrase)
        .await?;

    Ok((
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static(BUNDLE_MEDIA_TYPE),
            ),
            (header::CACHE_CONTROL, HeaderValue::from_static("no-store")),
            (
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&disposition)
                    .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
            ),
        ],
        Body::from_stream(bundle),
    ))
}

#[utoipa::path(
    post,
    path = "/import",
    tag = "realm",
    summary = "Import a realm bundle",
    description = "Reads a bundle written by the export endpoint and writes the realm with every id it carries, in one transaction. Secrets are sealed with this instance's key ring. Importing the same bundle again overwrites the realm's rows with the bundle values and keeps the rows the bundle does not hold. A bundle without an active signing key, cut short, tampered with or opened with the wrong passphrase is refused and leaves nothing behind. Needs the right to create realms, and to manage the realm when it already exists.",
    params(
        ("X-Ferriskey-Bundle-Passphrase" = String, Header, description = "The passphrase given at export"),
    ),
    request_body(content = Vec<u8>, content_type = "application/vnd.ferriskey.realm-bundle"),
    responses(
        (status = 201, description = "The realm was created", body = ImportReportResponse),
        (status = 200, description = "The realm existed and was overwritten", body = ImportReportResponse),
        (status = 400, description = "Unreadable bundle, missing signing key or invalid passphrase", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 409, description = "The bundle clashes with a realm or a row of this instance", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn import_realm(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    headers: HeaderMap,
    body: Body,
) -> Result<Response<ImportReportResponse>, ApiError> {
    let passphrase = passphrase(&headers)?;
    let report = state
        .service
        .import_realm(identity, passphrase, body.into_data_stream())
        .await?;

    Ok(if report.created {
        Response::Created(ImportReportResponse { data: report })
    } else {
        Response::OK(ImportReportResponse { data: report })
    })
}
