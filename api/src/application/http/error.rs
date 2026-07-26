// The `From<_> for ApiError` conversions that used to live here now reside in
// `ferriskey_api_core::error`, since `ApiError` moved into the
// `ferriskey-api-core` crate. Those impls are in scope automatically wherever
// the crate is linked, so this module is intentionally empty and only preserves
// the old `crate::application::http::error` path.
