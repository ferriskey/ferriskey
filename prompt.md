api/src/application/http/authentication/handlers/introspect.rs
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(payload): Form<IntrospectRequestValidator>,
) -> Result<impl IntoResponse, ApiError> {

by convention if you don't use a redirection, please use Response<T>
-----

core/src/domain/authentication/services.rs
            let claims: JwtClaim = serde_json::from_value(stored.claims)
                .map_err(|_| CoreError::InternalServerError)?;

            return Ok(TokenIntrospectionResponse {

You do not verify whether the token has been signed by the IAM here ?

-----
