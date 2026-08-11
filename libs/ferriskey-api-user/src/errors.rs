// The `From<CredentialError> for ApiError` conversion previously defined here
// now lives in `ferriskey_api_core::error` (alongside the other `From<_> for
// ApiError` impls), because `ApiError` moved into the `ferriskey-api-core`
// crate and the orphan rule forbids implementing a foreign trait for a foreign
// type from `ferriskey-api`. The impl is in scope automatically wherever the
// crate is linked, so this module is intentionally empty and only preserves the
// old `crate::application::http::user::errors` path.
