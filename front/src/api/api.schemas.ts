export type ActorType = 'user' | 'service_account' | 'admin' | 'system'
  export type ValidationError = { field: string; message: string }
  export type ApiError =
    | { InternalServerError: string }
    | { UnProcessableEntity: Array<ValidationError> }
    | { NotFound: string }
    | { Unauthorized: string }
    | { Forbidden: string }
    | { BadRequest: string }
    | { ServiceUnavailable: string }
  export type ApiErrorResponse = { code: string; message: string; status: number }
  export type AssignRoleResponse = { message: string; realm_name: string; user_id: string }
  export type AuthResponse = { url: string }
  export type AuthenticateRequest = Partial<{ password: string | null; username: string | null }>
  export type AuthenticationStatus =
    | 'Success'
    | 'RequiresActions'
    | 'RequiresOtpChallenge'
    | 'Failed'
  export type AuthenticateResponse = {
    message?: (string | null) | undefined
    required_actions?: (Array<RequiredAction> | null) | undefined
    status: AuthenticationStatus
    token?: (string | null) | undefined
    url?: (string | null) | undefined
  }
  export type AuthenticationAttemptResponse = { login_url: string }
  export type BulkDeleteUserResponse = { count: number; realm_name: string }
  export type BulkDeleteUserValidator = Partial<{ ids: Array<string> }>
  export type BurnRecoveryCodeRequest = { recovery_code: string; recovery_code_format: string }
  export type BurnRecoveryCodeResponse = { login_url: string }
  export type ChallengeOtpRequest = Partial<{ code: string }>
  export type ChallengeOtpResponse = { url: string }
  export type RealmId = string
  export type Client = {
    client_id: string
    client_type: string
    created_at: string
    direct_access_grants_enabled: boolean
    enabled: boolean
    id: string
    name: string
    protocol: string
    public_client: boolean
    realm_id: RealmId
    redirect_uris?: (Array<RedirectUri> | null) | undefined
    secret?: (string | null) | undefined
    service_account_enabled: boolean
    updated_at: string
  }
  export type ClientScope = {
    attributes?: (Array<ClientScopeAttribute> | null) | undefined
    created_at: string
    description?: (string | null) | undefined
    id: string
    is_default: boolean
    name: string
    protocol: string
    protocol_mappers?: (Array<ProtocolMapper> | null) | undefined
    realm_id: RealmId
    updated_at: string
  }
  export type ClientScopeAttribute = {
    id: string
    name: string
    scope_id: string
    value?: (string | null) | undefined
  }
  export type ClientScopeMapping = {
    client_id: string
    is_default: boolean
    is_optional: boolean
    scope_id: string
  }
  export type ClientScopesResponse = { data: Array<ClientScope> }
  export type ClientsResponse = { data: Array<Client> }
  export type CreateClientScopeValidator = Partial<{
    description: string | null
    is_default: boolean
    name: string
    protocol: string
  }>
  export type CreateClientValidator = Partial<{
    client_id: string
    client_type: string
    direct_access_grants_enabled: boolean
    enabled: boolean
    name: string
    protocol: string
    public_client: boolean
    service_account_enabled: boolean
  }>
  export type CreateIdentityProviderValidator = Partial<{
    add_read_token_role_on_create: boolean
    alias: string
    config: unknown
    display_name: string | null
    enabled: boolean
    first_broker_login_flow_alias: string | null
    link_only: boolean
    post_broker_login_flow_alias: string | null
    provider_id: string
    store_token: boolean
    trust_email: boolean
  }>
  export type CreateProtocolMapperValidator = Partial<{
    config: unknown
    mapper_type: string
    name: string
  }>
  export type CreateProviderRequest = {
    config: unknown
    enabled: boolean
    name: string
    priority: number
    provider_type: string
    sync_enabled: boolean
    sync_interval_minutes?: (number | null) | undefined
    sync_mode: string
  }
  export type CreateRealmValidator = Partial<{ name: string }>
  export type CreateRedirectUriValidator = Partial<{ enabled: boolean; value: string }>
  export type CreateRoleValidator = {
    description?: (string | null) | undefined
    name: string
    permissions: Array<string>
  }
  export type RealmSetting = {
    default_signing_algorithm?: (string | null) | undefined
    forgot_password_enabled: boolean
    id: string
    magic_link_enabled: boolean
    magic_link_ttl: number
    realm_id: RealmId
    remember_me_enabled: boolean
    updated_at: string
    user_registration_enabled: boolean
  }
  export type Realm = {
    created_at: string
    id: RealmId
    name: string
    settings?: (null | RealmSetting) | undefined
    updated_at: string
  }
  export type RequiredAction = 'configure_otp' | 'verify_email' | 'update_password'
  export type User = {
    client_id?: (string | null) | undefined
    created_at: string
    email: string
    email_verified: boolean
    enabled: boolean
    firstname: string
    id: string
    lastname: string
    realm?: (null | Realm) | undefined
    realm_id: RealmId
    required_actions: Array<RequiredAction>
    roles?: (Array<Role> | null) | undefined
    updated_at: string
    username: string
  }
  export type CreateUserResponse = { data: User }
  export type CreateUserValidator = Partial<{
    email: string
    email_verified: boolean | null
    firstname: string
    lastname: string
    username: string
  }>
  export type WebhookTrigger =
    | 'user.created'
    | 'user.updated'
    | 'user.deleted'
    | 'user.role.assigned'
    | 'user.role.unassigned'
    | 'user.bulk_deleted'
    | 'user.credentials.deleted'
    | 'auth.reset_password'
    | 'client.created'
    | 'client.updated'
    | 'client.deleted'
    | 'client.role.created'
    | 'client.role.updated'
    | 'redirect_uri.created'
    | 'redirect_uri.updated'
    | 'redirect_uri.deleted'
    | 'role.created'
    | 'role.updated'
    | 'role.deleted'
    | 'role.permission.updated'
    | 'realm.created'
    | 'realm.updated'
    | 'realm.deleted'
    | 'realm.settings.updated'
    | 'webhook.created'
    | 'webhook.updated'
    | 'webhook.deleted'
  export type WebhookSubscriber = { id: string; name: WebhookTrigger; webhook_id: string }
  export type Webhook = {
    created_at: string
    description?: (string | null) | undefined
    endpoint: string
    headers: Record<string, string>
    id: string
    name?: (string | null) | undefined
    subscribers: Array<WebhookSubscriber>
    triggered_at?: (string | null) | undefined
    updated_at: string
  }
  export type CreateWebhookResponse = { data: Webhook }
  export type CreateWebhookValidator = Partial<{
    description: string | null
    endpoint: string
    headers: Record<string, string>
    name: string | null
    subscribers: Array<WebhookTrigger>
  }>
  export type CredentialDataOverview =
    | { Hash: { algorithm: string; hash_iterations: number } }
    | 'WebAuthn'
    | { Federated: { provider_id: string; provider_type: string } }
  export type CredentialOverview = {
    created_at: string
    credential_data: CredentialDataOverview
    credential_type: string
    id: string
    updated_at: string
    user_id: string
    user_label?: (string | null) | undefined
  }
  export type DeleteClientResponse = { message: string; realm_name: string }
  export type DeleteClientScopeResponse = { message: string }
  export type DeleteIdentityProviderResponse = { count: number }
  export type DeleteProtocolMapperResponse = { message: string }
  export type DeleteProviderResponse = { message: string }
  export type DeleteRealmResponse = string
  export type DeleteRoleResponse = { message: string; realm_name: string; role_id: string }
  export type DeleteUserCredentialResponse = {
    message: string
    realm_name: string
    user_id: string
  }
  export type DeleteUserResponse = { count: number }
  export type DeleteWebhookResponse = { message: string; realm_name: string }
  export type EventStatus = 'success' | 'failure'
  export type GenerateRecoveryCodesRequest = { amount: number; code_format: string }
  export type GenerateRecoveryCodesResponse = { codes: Array<string> }
  export type JwkKey = {
    alg: string
    e: string
    kid: string
    kty: string
    n: string
    use: string
    x5c: Array<string>
    x5t: string
  }
  export type GetCertsResponse = { keys: Array<JwkKey> }
  export type GetClientResponse = { data: Client }
  export type Role = {
    client?: (null | Client) | undefined
    client_id?: (string | null) | undefined
    created_at: string
    description?: (string | null) | undefined
    id: string
    name: string
    permissions: Array<string>
    realm_id: RealmId
    updated_at: string
  }
  export type GetClientRolesResponse = { data: Array<Role> }
  export type GetOpenIdConfigurationResponse = {
    authorization_endpoint: string
    end_session_endpoint: string
    grant_types_supported: Array<string>
    introspection_endpoint: string
    issuer: string
    jwks_uri: string
    revocation_endpoint: string
    token_endpoint: string
    userinfo_endpoint: string
  }
  export type GetRoleResponse = { data: Role }
  export type GetRolesResponse = { data: Array<Role> }
  export type SecurityEventType =
    | 'login_success'
    | 'login_failure'
    | 'password_reset'
    | 'user_created'
    | 'user_deleted'
    | 'role_assigned'
    | 'role_unassigned'
    | 'role_created'
    | 'role_removed'
    | 'client_created'
    | 'client_deleted'
    | 'client_secret_rotated'
    | 'realm_config_changed'
  export type SecurityEventId = string
  export type SecurityEvent = {
    actor_id?: (string | null) | undefined
    actor_type?: (null | ActorType) | undefined
    details?: unknown | undefined
    event_type: SecurityEventType
    id: SecurityEventId
    ip_address?: (string | null) | undefined
    realm_id: RealmId
    resource?: (string | null) | undefined
    status: EventStatus
    target_id?: (string | null) | undefined
    target_type?: (string | null) | undefined
    timestamp: string
    trace_id?: (string | null) | undefined
    user_agent?: (string | null) | undefined
  }
  export type GetSecurityEventsResponse = { data: Array<SecurityEvent> }
  export type GetUserCredentialsResponse = { data: Array<CredentialOverview> }
  export type GetUserRolesResponse = { data: Array<Role> }
  export type GetWebhooksResponse = { data: Array<Webhook> }
  export type GrantType = 'authorization_code' | 'password' | 'client_credentials' | 'refresh_token'
  export type IdentityProviderPresentation = {
    display_name: string
    icon: string
    id: string
    kind: string
    login_url: string
  }
  export type IdentityProviderResponse = {
    add_read_token_role_on_create: boolean
    alias: string
    config: unknown
    display_name?: (string | null) | undefined
    enabled: boolean
    first_broker_login_flow_alias?: (string | null) | undefined
    internal_id: string
    link_only: boolean
    post_broker_login_flow_alias?: (string | null) | undefined
    provider_id: string
    store_token: boolean
    trust_email: boolean
  }
  export type IdentityProvidersResponse = { data: Array<IdentityProviderResponse> }
  export type IntrospectRequestValidator = Partial<{
    client_id: string | null
    client_secret: string | null
    token: string
    token_type_hint: string | null
  }>
  export type JwtToken = {
    access_token: string
    expires_in: number
    id_token?: (string | null) | undefined
    refresh_token: string
    token_type: string
  }
  export type ProviderResponse = {
    config: unknown
    created_at: string
    enabled: boolean
    id: string
    last_sync_at?: (string | null) | undefined
    last_sync_status?: (string | null) | undefined
    name: string
    priority: number
    provider_type: string
    realm_id: string
    sync_enabled: boolean
    sync_interval_minutes?: (number | null) | undefined
    sync_mode: string
    updated_at: string
  }
  export type ListProvidersResponse = { data: Array<ProviderResponse> }
  export type LogoutRequestValidator = Partial<{
    client_id: string | null
    id_token_hint: string | null
    post_logout_redirect_uri: string | null
    state: string | null
  }>
  export type OtpVerifyRequest = { code: string; label: string; secret: string }
  export type Permissions =
    | 'create_client'
    | 'manage_authorization'
    | 'manage_clients'
    | 'manage_events'
    | 'manage_identity_providers'
    | 'manage_realm'
    | 'manage_users'
    | 'manage_roles'
    | 'query_clients'
    | 'query_groups'
    | 'query_realms'
    | 'query_users'
    | 'view_authorization'
    | 'view_clients'
    | 'view_events'
    | 'view_identity_providers'
    | 'view_realm'
    | 'view_users'
    | 'view_roles'
    | 'manage_webhooks'
    | 'query_webhooks'
    | 'view_webhooks'
    | 'manage_client_scopes'
    | 'query_client_scopes'
    | 'view_client_scopes'
  export type ProtocolMapper = {
    client_scope_id: string
    config: unknown
    created_at: string
    id: string
    mapper_type: string
    name: string
  }
  export type PublicKeyCredential = Record<string, unknown>
  export type PublicKeyCredentialCreationOptionsJSON = Record<string, unknown>
  export type PublicKeyCredentialRequestOptionsJSON = Record<string, unknown>
  export type RealmLoginSetting = {
    forgot_password_enabled: boolean
    identity_providers: Array<IdentityProviderPresentation>
    magic_link_enabled: boolean
    magic_link_ttl: number
    remember_me_enabled: boolean
    user_registration_enabled: boolean
  }
  export type RedirectUri = {
    client_id: string
    created_at: string
    enabled: boolean
    id: string
    updated_at: string
    value: string
  }
  export type RegistrationRequest = Partial<{
    email: string
    first_name: string | null
    last_name: string | null
    password: string
    username: string
  }>
  export type ResetPasswordResponse = { message: string; realm_name: string; user_id: string }
  export type ResetPasswordValidator = Partial<{
    credential_type: string
    temporary: boolean
    value: string
  }>
  export type RevokeTokenRequestValidator = Partial<{
    client_id: string
    token: string
    token_type_hint: string | null
  }>
  export type SendMagicLinkRequest = { email: string }
  export type SendMagicLinkResponse = { message: string }
  export type SetupOtpResponse = { issuer: string; otpauth_url: string; secret: string }
  export type SyncUsersResponse = {
    completed_at?: (string | null) | undefined
    created: number
    disabled: number
    failed: number
    started_at?: (string | null) | undefined
    total_processed: number
    updated: number
  }
  export type TestConnectionResponse = {
    details?: unknown | undefined
    message: string
    success: boolean
  }
  export type TokenIntrospectionResponse = {
    active: boolean
    aud?: (string | null) | undefined
    client_id?: (string | null) | undefined
    exp?: (number | null) | undefined
    iat?: (number | null) | undefined
    iss?: (string | null) | undefined
    jti?: (string | null) | undefined
    nbf?: (number | null) | undefined
    realm?: (string | null) | undefined
    scope?: (string | null) | undefined
    sub?: (string | null) | undefined
    token_type?: (string | null) | undefined
    username?: (string | null) | undefined
  }
  export type TokenRequestValidator = Partial<{
    client_id: string
    client_secret: string | null
    code: string | null
    grant_type: GrantType
    password: string | null
    refresh_token: string | null
    scope: string | null
    username: string | null
  }>
  export type UnassignRoleResponse = { message: string; realm_name: string; user_id: string }
  export type UpdateClientResponse = { data: Client }
  export type UpdateClientScopeValidator = Partial<{
    description: string | null
    is_default: boolean | null
    name: string | null
    protocol: string | null
  }>
  export type UpdateClientValidator = Partial<{
    client_id: string | null
    direct_access_grants_enabled: boolean | null
    enabled: boolean | null
    name: string | null
  }>
  export type UpdateIdentityProviderResponse = { data: IdentityProviderResponse }
  export type UpdateIdentityProviderValidator = Partial<{
    add_read_token_role_on_create: boolean | null
    config: unknown
    display_name: string | null
    enabled: boolean | null
    first_broker_login_flow_alias: string | null
    link_only: boolean | null
    post_broker_login_flow_alias: string | null
    store_token: boolean | null
    trust_email: boolean | null
  }>
  export type UpdatePasswordRequest = Partial<{ value: string }>
  export type UpdatePasswordResponse = { message: string }
  export type UpdatePostLogoutRedirectUriResponse = { data: RedirectUri }
  export type UpdateProtocolMapperValidator = Partial<{
    config: unknown
    mapper_type: string | null
    name: string | null
  }>
  export type UpdateProviderRequest = Partial<{
    config: unknown
    enabled: boolean | null
    name: string | null
    priority: number | null
    provider_type: string | null
    sync_enabled: boolean | null
    sync_interval_minutes: number | null
    sync_mode: string | null
  }>
  export type UpdateProviderResponse = { data: ProviderResponse }
  export type UpdateRealmResponse = { data: Realm }
  export type UpdateRealmSettingResponse = { data: Realm }
  export type UpdateRealmSettingValidator = Partial<{
    default_signing_algorithm: string | null
    forgot_password_enabled: boolean | null
    magic_link_enabled: boolean | null
    magic_link_ttl: number | null
    remember_me_enabled: boolean | null
    user_registration_enabled: boolean | null
  }>
  export type UpdateRealmValidator = { name: string }
  export type UpdateRedirectUriResponse = { data: RedirectUri }
  export type UpdateRedirectUriValidator = Partial<{ enabled: boolean }>
  export type UpdateRolePermissionsResponse = { data: Role }
  export type UpdateRolePermissionsValidator = { permissions: Array<string> }
  export type UpdateRoleResponse = { data: Role }
  export type UpdateRoleValidator = Partial<{ description: string | null; name: string | null }>
  export type UpdateUserResponse = { data: User }
  export type UpdateUserValidator = Partial<{
    email: string
    email_verified: boolean | null
    enabled: boolean | null
    firstname: string
    lastname: string
    required_actions: Array<string> | null
  }>
  export type UpdateWebhookResponse = { data: Webhook }
  export type UserInfoResponse = {
    email?: (string | null) | undefined
    email_verified?: (boolean | null) | undefined
    family_name?: (string | null) | undefined
    given_name?: (string | null) | undefined
    name?: (string | null) | undefined
    preferred_username?: (string | null) | undefined
    sub: string
  }
  export type UserPermissionsResponse = { data: Array<Permissions> }
  export type UserRealmsResponse = { data: Array<Realm> }
  export type UserResponse = { data: User }
  export type UsersResponse = { data: Array<User> }
  export type ValidatePublicKeyResponse = Record<string, unknown>
  export type VerifyOtpResponse = { message: string }
