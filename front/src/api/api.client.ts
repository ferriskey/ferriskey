export namespace Schemas {
  // <Schemas>
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
    | { OAuthError: { error: string; error_description: string } }
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
  export type ClientType = 'confidential' | 'public' | 'system'
  export type RealmId = string
  export type Client = {
    client_id: string
    client_type: ClientType
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
  export type ScopeType = 'NONE' | 'OPTIONAL' | 'DEFAULT'
  export type ClientScope = {
    attributes?: (Array<ClientScopeAttribute> | null) | undefined
    created_at: string
    default_scope_type: ScopeType
    description?: (string | null) | undefined
    id: string
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
    default_scope_type: ScopeType
    scope_id: string
  }
  export type ClientScopesResponse = { data: Array<ClientScope> }
  export type ClientsResponse = { data: Array<Client> }
  export type FlowId = string
  export type FlowStatus = 'pending' | 'success' | 'failure' | 'expired'
  export type FlowStepId = string
  export type StepStatus = 'success' | 'failure' | 'skipped'
  export type FlowStepName =
    | 'authorize'
    | 'credential_validation'
    | 'mfa_challenge'
    | 'token_exchange'
    | 'idp_redirect'
    | 'idp_callback'
    | 'finalize'
  export type CompassFlowStep = {
    duration_ms?: (number | null) | undefined
    error_code?: (string | null) | undefined
    error_message?: (string | null) | undefined
    flow_id: FlowId
    id: FlowStepId
    started_at: string
    status: StepStatus
    step_name: FlowStepName
  }
  export type CompassFlow = {
    client_id?: (string | null) | undefined
    completed_at?: (string | null) | undefined
    duration_ms?: (number | null) | undefined
    grant_type: string
    id: FlowId
    ip_address?: (string | null) | undefined
    realm_id: RealmId
    started_at: string
    status: FlowStatus
    steps: Array<CompassFlowStep>
    user_agent?: (string | null) | undefined
    user_id?: (string | null) | undefined
  }
  export type CreateClientScopeValidator = Partial<{
    description: string | null
    is_default: boolean
    name: string
    protocol: string
  }>
  export type CreateClientValidator = {
    client_id?: string | undefined
    client_type: ClientType
    direct_access_grants_enabled?: boolean | undefined
    enabled?: boolean | undefined
    name?: string | undefined
    protocol?: string | undefined
    public_client?: boolean | undefined
    service_account_enabled?: boolean | undefined
  }
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
  export type CreateRoleResponse = { data: Role }
  export type CreateRoleValidator = {
    description?: (string | null) | undefined
    name: string
    permissions: Array<string>
  }
  export type RealmSetting = {
    compass_enabled: boolean
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
  export type FlowStats = {
    avg_duration_ms?: (number | null) | undefined
    failure_count: number
    pending_count: number
    success_count: number
    total: number
  }
  export type ForgotPasswordRequest = { email: string }
  export type ForgotPasswordResponse = unknown
  export type GenerateRecoveryCodesRequest = { amount: number; code_format: string }
  export type GenerateRecoveryCodesResponse = { codes: Array<string> }
  export type JwkKey = { alg: string; e: string; kid: string; kty: string; n: string; use: string }
  export type GetCertsResponse = { keys: Array<JwkKey> }
  export type GetClientResponse = { data: Client }
  export type GetClientRolesResponse = { data: Array<Role> }
  export type GetFlowResponse = { data: CompassFlow }
  export type GetFlowsResponse = { data: Array<CompassFlow> }
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
    | 'password_reset_requested'
    | 'password_reset_completed'
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
  export type GetStatsResponse = { data: FlowStats }
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
  export type ResetPasswordRequest = { new_password: string; token: string; token_id: string }
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
    compass_enabled: boolean | null
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

  // </Schemas>
}

export namespace Endpoints {
  // <Endpoints>

  export type post_Create_realm = {
    method: 'POST'
    path: '/realms'
    requestFormat: 'json'
    parameters: {
      body: Schemas.CreateRealmValidator
    }
    responses: {
      201: Schemas.Realm
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_realm = {
    method: 'GET'
    path: '/realms/{name}'
    requestFormat: 'json'
    parameters: {
      path: { name: string }
    }
    responses: {
      200: Schemas.Realm
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type put_Update_realm = {
    method: 'PUT'
    path: '/realms/{name}'
    requestFormat: 'json'
    parameters: {
      path: { name: string }

      body: Schemas.UpdateRealmValidator
    }
    responses: {
      200: Schemas.UpdateRealmResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type delete_Delete_realm = {
    method: 'DELETE'
    path: '/realms/{name}'
    requestFormat: 'json'
    parameters: {
      path: { name: string }
    }
    responses: {
      200: Schemas.DeleteRealmResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_login_realm_settings_handler = {
    method: 'GET'
    path: '/realms/{name}/login-settings'
    requestFormat: 'json'
    parameters: {
      path: { name: string }
    }
    responses: {
      200: Schemas.RealmLoginSetting
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type put_Update_realm_setting = {
    method: 'PUT'
    path: '/realms/{name}/settings'
    requestFormat: 'json'
    parameters: {
      path: { name: string }

      body: Schemas.UpdateRealmSettingValidator
    }
    responses: {
      200: Schemas.UpdateRealmSettingResponse
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_openid_configuration = {
    method: 'GET'
    path: '/realms/{realm_name}/.well-known/openid-configuration'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    responses: { 200: Schemas.GetOpenIdConfigurationResponse }
  }
  export type post_Broker_callback = {
    method: 'POST'
    path: '/realms/{realm_name}/broker/{alias}/endpoint'
    requestFormat: 'json'
    parameters: {
      query: {
        code?: string | undefined
        state: string
        error?: string | undefined
        error_description?: string | undefined
      }
      path: { realm_name: string; alias: string }
    }
    responses: {
      302: unknown
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      502: Schemas.ApiErrorResponse
    }
  }
  export type get_Broker_login = {
    method: 'GET'
    path: '/realms/{realm_name}/broker/{alias}/login'
    requestFormat: 'json'
    parameters: {
      query: Partial<{
        client_id: string
        redirect_uri: string
        response_type: string
        scope: string
        state: string
        nonce: string
        session_id: string
      }>
      path: { realm_name: string; alias: string }
    }
    responses: {
      302: unknown
      400: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_client_scopes = {
    method: 'GET'
    path: '/realms/{realm_name}/client-scopes'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    responses: { 200: Schemas.ClientScopesResponse }
  }
  export type post_Create_client_scope = {
    method: 'POST'
    path: '/realms/{realm_name}/client-scopes'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.CreateClientScopeValidator
    }
    responses: { 201: Schemas.ClientScope; 400: unknown; 401: unknown; 403: unknown }
  }
  export type get_Get_client_scope = {
    method: 'GET'
    path: '/realms/{realm_name}/client-scopes/{scope_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; scope_id: string }
    }
    responses: { 200: Schemas.ClientScope }
  }
  export type delete_Delete_client_scope = {
    method: 'DELETE'
    path: '/realms/{realm_name}/client-scopes/{scope_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; scope_id: string }
    }
    responses: { 200: Schemas.DeleteClientScopeResponse }
  }
  export type patch_Update_client_scope = {
    method: 'PATCH'
    path: '/realms/{realm_name}/client-scopes/{scope_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; scope_id: string }

      body: Schemas.UpdateClientScopeValidator
    }
    responses: { 200: Schemas.ClientScope }
  }
  export type post_Create_protocol_mapper = {
    method: 'POST'
    path: '/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; scope_id: string }

      body: Schemas.CreateProtocolMapperValidator
    }
    responses: { 201: Schemas.ProtocolMapper; 400: unknown; 401: unknown; 403: unknown }
  }
  export type delete_Delete_protocol_mapper = {
    method: 'DELETE'
    path: '/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; scope_id: string; mapper_id: string }
    }
    responses: { 200: Schemas.DeleteProtocolMapperResponse }
  }
  export type patch_Update_protocol_mapper = {
    method: 'PATCH'
    path: '/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; scope_id: string; mapper_id: string }

      body: Schemas.UpdateProtocolMapperValidator
    }
    responses: { 200: Schemas.ProtocolMapper }
  }
  export type get_Get_clients = {
    method: 'GET'
    path: '/realms/{realm_name}/clients'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    responses: {
      200: Schemas.ClientsResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Create_client = {
    method: 'POST'
    path: '/realms/{realm_name}/clients'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.CreateClientValidator
    }
    responses: {
      201: Schemas.Client
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_client = {
    method: 'GET'
    path: '/realms/{realm_name}/clients/{client_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }
    }
    responses: {
      200: Schemas.GetClientResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type delete_Delete_client = {
    method: 'DELETE'
    path: '/realms/{realm_name}/clients/{client_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }
    }
    responses: {
      200: Schemas.DeleteClientResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type patch_Update_client = {
    method: 'PATCH'
    path: '/realms/{realm_name}/clients/{client_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }

      body: Schemas.UpdateClientValidator
    }
    responses: {
      200: Schemas.UpdateClientResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_client_client_scopes = {
    method: 'GET'
    path: '/realms/{realm_name}/clients/{client_id}/client-scopes'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }
    }
    responses: {
      200: Array<Schemas.ClientScope>
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
    }
  }
  export type put_Assign_default_scope = {
    method: 'PUT'
    path: '/realms/{realm_name}/clients/{client_id}/default-client-scopes/{scope_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string; scope_id: string }
    }
    responses: { 200: Schemas.ClientScopeMapping }
  }
  export type delete_Unassign_default_scope = {
    method: 'DELETE'
    path: '/realms/{realm_name}/clients/{client_id}/default-client-scopes/{scope_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string; scope_id: string }
    }
    responses: { 200: unknown }
  }
  export type put_Assign_optional_scope = {
    method: 'PUT'
    path: '/realms/{realm_name}/clients/{client_id}/optional-client-scopes/{scope_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string; scope_id: string }
    }
    responses: { 200: Schemas.ClientScopeMapping }
  }
  export type delete_Unassign_optional_scope = {
    method: 'DELETE'
    path: '/realms/{realm_name}/clients/{client_id}/optional-client-scopes/{scope_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string; scope_id: string }
    }
    responses: { 200: unknown }
  }
  export type get_Get_post_logout_redirect_uris = {
    method: 'GET'
    path: '/realms/{realm_name}/clients/{client_id}/post-logout-redirects'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }
    }
    responses: { 200: Array<Schemas.RedirectUri> }
  }
  export type post_Create_post_logout_redirect_uri = {
    method: 'POST'
    path: '/realms/{realm_name}/clients/{client_id}/post-logout-redirects'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }

      body: Schemas.CreateRedirectUriValidator
    }
    responses: { 201: Schemas.RedirectUri }
  }
  export type put_Update_post_logout_redirect_uri = {
    method: 'PUT'
    path: '/realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string; uri_id: string }

      body: Schemas.UpdateRedirectUriValidator
    }
    responses: { 200: Schemas.UpdatePostLogoutRedirectUriResponse }
  }
  export type delete_Delete_post_logout_redirect_uri = {
    method: 'DELETE'
    path: '/realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string; uri_id: string }
    }
    responses: { 200: unknown }
  }
  export type get_Get_redirect_uris = {
    method: 'GET'
    path: '/realms/{realm_name}/clients/{client_id}/redirects'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }
    }
    responses: {
      200: Array<Schemas.RedirectUri>
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Create_redirect_uri = {
    method: 'POST'
    path: '/realms/{realm_name}/clients/{client_id}/redirects'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }

      body: Schemas.CreateRedirectUriValidator
    }
    responses: { 201: Schemas.RedirectUri }
  }
  export type put_Update_redirect_uri = {
    method: 'PUT'
    path: '/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string; uri_id: string }

      body: Schemas.UpdateRedirectUriValidator
    }
    responses: {
      200: Schemas.UpdateRedirectUriResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type delete_Delete_redirect_uri = {
    method: 'DELETE'
    path: '/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string; uri_id: string }
    }
    responses: {
      200: unknown
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_client_roles = {
    method: 'GET'
    path: '/realms/{realm_name}/clients/{client_id}/roles'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }
    }
    responses: {
      200: Schemas.GetClientRolesResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Create_client_role = {
    method: 'POST'
    path: '/realms/{realm_name}/clients/{client_id}/roles'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }

      body: Schemas.CreateRoleValidator
    }
    responses: {
      201: Schemas.Role
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_flows = {
    method: 'GET'
    path: '/realms/{realm_name}/compass/v1/flows'
    requestFormat: 'json'
    parameters: {
      query: Partial<{
        client_id: string
        user_id: string
        grant_type: string
        status: string
        limit: number
        offset: number
      }>
      path: { realm_name: string }
    }
    responses: {
      200: Schemas.GetFlowsResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_flow = {
    method: 'GET'
    path: '/realms/{realm_name}/compass/v1/flows/{flow_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; flow_id: string }
    }
    responses: {
      200: Schemas.GetFlowResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_stats = {
    method: 'GET'
    path: '/realms/{realm_name}/compass/v1/stats'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    responses: {
      200: Schemas.GetStatsResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_List_providers = {
    method: 'GET'
    path: '/realms/{realm_name}/federation/providers'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    responses: {
      200: Schemas.ListProvidersResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Create_provider = {
    method: 'POST'
    path: '/realms/{realm_name}/federation/providers'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.CreateProviderRequest
    }
    responses: {
      201: Schemas.ProviderResponse
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_provider = {
    method: 'GET'
    path: '/realms/{realm_name}/federation/providers/{id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; id: string }
    }
    responses: {
      200: Schemas.ProviderResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type put_Update_provider = {
    method: 'PUT'
    path: '/realms/{realm_name}/federation/providers/{id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; id: string }

      body: Schemas.UpdateProviderRequest
    }
    responses: {
      200: Schemas.UpdateProviderResponse
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type delete_Delete_provider = {
    method: 'DELETE'
    path: '/realms/{realm_name}/federation/providers/{id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; id: string }
    }
    responses: {
      204: Schemas.DeleteProviderResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Sync_users = {
    method: 'POST'
    path: '/realms/{realm_name}/federation/providers/{id}/sync-users'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; id: string }
    }
    responses: {
      200: Schemas.SyncUsersResponse
      401: unknown
      403: unknown
      404: unknown
      500: unknown
    }
  }
  export type post_Test_connection = {
    method: 'POST'
    path: '/realms/{realm_name}/federation/providers/{id}/test-connection'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; id: string }
    }
    responses: { 200: Schemas.TestConnectionResponse; 401: unknown; 403: unknown; 404: unknown }
  }
  export type get_List_identity_providers = {
    method: 'GET'
    path: '/realms/{realm_name}/identity-providers'
    requestFormat: 'json'
    parameters: {
      query: Partial<{ brief_representation: boolean }>
      path: { realm_name: string }
    }
    responses: { 200: Schemas.IdentityProvidersResponse; 401: unknown; 403: unknown; 404: unknown }
  }
  export type post_Create_identity_provider = {
    method: 'POST'
    path: '/realms/{realm_name}/identity-providers'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.CreateIdentityProviderValidator
    }
    responses: {
      201: Schemas.IdentityProviderResponse
      400: unknown
      401: unknown
      403: unknown
      404: unknown
      409: unknown
    }
  }
  export type get_Get_identity_provider = {
    method: 'GET'
    path: '/realms/{realm_name}/identity-providers/{alias}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; alias: string }
    }
    responses: { 200: Schemas.IdentityProviderResponse; 401: unknown; 403: unknown; 404: unknown }
  }
  export type put_Update_identity_provider = {
    method: 'PUT'
    path: '/realms/{realm_name}/identity-providers/{alias}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; alias: string }

      body: Schemas.UpdateIdentityProviderValidator
    }
    responses: {
      200: Schemas.UpdateIdentityProviderResponse
      400: unknown
      401: unknown
      403: unknown
      404: unknown
    }
  }
  export type delete_Delete_identity_provider = {
    method: 'DELETE'
    path: '/realms/{realm_name}/identity-providers/{alias}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; alias: string }
    }
    responses: {
      200: Schemas.DeleteIdentityProviderResponse
      401: unknown
      403: unknown
      404: unknown
      409: unknown
    }
  }
  export type post_Authenticate = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/authenticate'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.AuthenticateRequest
    }
    responses: {
      200: Schemas.AuthenticateResponse
      400: Schemas.ApiError
      401: Schemas.ApiError
      500: Schemas.ApiError
    }
  }
  export type post_Burn_recovery_code = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/burn-recovery-code'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.BurnRecoveryCodeRequest
    }
    responses: {
      200: Schemas.BurnRecoveryCodeResponse
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Challenge_otp = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/challenge-otp'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.ChallengeOtpRequest
    }
    responses: {
      200: Schemas.ChallengeOtpResponse
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Forgot_password = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/forgot-password'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.ForgotPasswordRequest
    }
    responses: {
      200: Schemas.ForgotPasswordResponse
      400: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Generate_recovery_codes = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/generate-recovery-codes'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.GenerateRecoveryCodesRequest
    }
    responses: {
      200: Schemas.GenerateRecoveryCodesResponse
      400: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Reset_password_with_token = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/reset-password'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.ResetPasswordRequest
    }
    responses: {
      200: Schemas.ResetPasswordResponse
      400: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Send_magic_link = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/send-magic-link'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.SendMagicLinkRequest
    }
    responses: { 200: Schemas.SendMagicLinkResponse; 400: unknown; 500: unknown }
  }
  export type get_Setup_otp = {
    method: 'GET'
    path: '/realms/{realm_name}/login-actions/setup-otp'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    responses: { 200: Schemas.SetupOtpResponse; 500: Schemas.ApiErrorResponse }
  }
  export type post_Update_password = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/update-password'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.UpdatePasswordRequest
    }
    responses: {
      200: Schemas.UpdatePasswordResponse
      400: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Verify_magic_link = {
    method: 'GET'
    path: '/realms/{realm_name}/login-actions/verify-magic-link'
    requestFormat: 'json'
    parameters: {
      query: { token_id: string; magic_token: string }
      path: { realm_name: string }
    }
    responses: {
      200: Schemas.AuthenticateResponse
      400: unknown
      401: unknown
      404: unknown
      500: unknown
    }
  }
  export type post_Verify_otp = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/verify-otp'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.OtpVerifyRequest
    }
    responses: {
      200: Schemas.VerifyOtpResponse
      400: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Webauthn_public_key_authenticate = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/webauthn-public-key-authenticate'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.PublicKeyCredential
    }
    responses: {
      200: Schemas.AuthenticationAttemptResponse
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Webauthn_public_key_create = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/webauthn-public-key-create'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.PublicKeyCredential
    }
    responses: {
      200: Schemas.ValidatePublicKeyResponse
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Webauthn_public_key_create_options = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/webauthn-public-key-create-options'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    responses: {
      200: Schemas.PublicKeyCredentialCreationOptionsJSON
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Webauthn_public_key_request_options = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/webauthn-public-key-request-options'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    responses: {
      200: Schemas.PublicKeyCredentialRequestOptionsJSON
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Auth_handler = {
    method: 'GET'
    path: '/realms/{realm_name}/protocol/openid-connect/auth'
    requestFormat: 'json'
    parameters: {
      query: Partial<{
        response_type: string
        client_id: string
        redirect_uri: string
        scope: string
        state: string
      }>
      path: { realm_name: string }
    }
    responses: { 302: Schemas.AuthResponse; 400: unknown; 401: unknown; 500: unknown }
  }
  export type get_Get_certs = {
    method: 'GET'
    path: '/realms/{realm_name}/protocol/openid-connect/certs'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    responses: {
      200: Schemas.GetCertsResponse
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_jwks_json = {
    method: 'GET'
    path: '/realms/{realm_name}/protocol/openid-connect/jwks.json'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    responses: {
      200: Schemas.GetCertsResponse
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Logout_get = {
    method: 'GET'
    path: '/realms/{realm_name}/protocol/openid-connect/logout'
    requestFormat: 'json'
    parameters: {
      query: Partial<{
        id_token_hint: string
        post_logout_redirect_uri: string
        state: string
        client_id: string
      }>
      path: { realm_name: string }
    }
    responses: { 204: unknown; 307: unknown }
  }
  export type post_Logout_post = {
    method: 'POST'
    path: '/realms/{realm_name}/protocol/openid-connect/logout'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.LogoutRequestValidator
    }
    responses: { 204: unknown; 307: unknown }
  }
  export type post_Registration_handler = {
    method: 'POST'
    path: '/realms/{realm_name}/protocol/openid-connect/registrations'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.RegistrationRequest
    }
    responses: {
      201: Schemas.JwtToken
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Revoke_token = {
    method: 'POST'
    path: '/realms/{realm_name}/protocol/openid-connect/revoke'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.RevokeTokenRequestValidator
    }
    responses: { 200: unknown }
  }
  export type post_Exchange_token = {
    method: 'POST'
    path: '/realms/{realm_name}/protocol/openid-connect/token'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.TokenRequestValidator
    }
    responses: {
      200: Schemas.JwtToken
      401: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Introspect_token = {
    method: 'POST'
    path: '/realms/{realm_name}/protocol/openid-connect/token/introspect'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.IntrospectRequestValidator
    }
    responses: {
      200: Schemas.TokenIntrospectionResponse
      401: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_userinfo = {
    method: 'GET'
    path: '/realms/{realm_name}/protocol/openid-connect/userinfo'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    responses: { 200: Schemas.UserInfoResponse; 401: unknown; 403: unknown; 500: unknown }
  }
  export type get_Get_roles = {
    method: 'GET'
    path: '/realms/{realm_name}/roles'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    responses: {
      200: Schemas.GetRolesResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Create_realm_role = {
    method: 'POST'
    path: '/realms/{realm_name}/roles'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.CreateRoleValidator
    }
    responses: {
      201: Schemas.CreateRoleResponse
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_role = {
    method: 'GET'
    path: '/realms/{realm_name}/roles/{role_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; role_id: string }
    }
    responses: {
      200: Schemas.GetRoleResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type put_Update_role = {
    method: 'PUT'
    path: '/realms/{realm_name}/roles/{role_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; role_id: string }

      body: Schemas.UpdateRoleValidator
    }
    responses: {
      200: Schemas.UpdateRoleResponse
      400: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type delete_Delete_role = {
    method: 'DELETE'
    path: '/realms/{realm_name}/roles/{role_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; role_id: string }
    }
    responses: {
      200: Schemas.DeleteRoleResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type patch_Update_role_permissions = {
    method: 'PATCH'
    path: '/realms/{realm_name}/roles/{role_id}/permissions'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; role_id: string }

      body: Schemas.UpdateRolePermissionsValidator
    }
    responses: {
      200: Schemas.UpdateRolePermissionsResponse
      400: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_security_events = {
    method: 'GET'
    path: '/realms/{realm_name}/seawatch/v1/security-events'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    responses: {
      200: Schemas.GetSecurityEventsResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_users = {
    method: 'GET'
    path: '/realms/{realm_name}/users'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    responses: {
      200: Schemas.UsersResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Create_user = {
    method: 'POST'
    path: '/realms/{realm_name}/users'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.CreateUserValidator
    }
    responses: {
      201: Schemas.CreateUserResponse
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_user_realms = {
    method: 'GET'
    path: '/realms/{realm_name}/users/@me/realms'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    responses: {
      200: Schemas.UserRealmsResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type delete_Bulk_delete_user = {
    method: 'DELETE'
    path: '/realms/{realm_name}/users/bulk'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.BulkDeleteUserValidator
    }
    responses: {
      200: Schemas.BulkDeleteUserResponse
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_user = {
    method: 'GET'
    path: '/realms/{realm_name}/users/{user_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string }
    }
    responses: {
      200: Schemas.UserResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type put_Update_user = {
    method: 'PUT'
    path: '/realms/{realm_name}/users/{user_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string }

      body: Schemas.UpdateUserValidator
    }
    responses: {
      200: Schemas.UpdateUserResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type delete_Delete_user = {
    method: 'DELETE'
    path: '/realms/{realm_name}/users/{user_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string }
    }
    responses: {
      200: Schemas.DeleteUserResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_user_credentials = {
    method: 'GET'
    path: '/realms/{realm_name}/users/{user_id}/credentials'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string }
    }
    responses: {
      200: Schemas.GetUserCredentialsResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type delete_Delete_user_credential = {
    method: 'DELETE'
    path: '/realms/{realm_name}/users/{user_id}/credentials/{credential_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string; credential_id: string }
    }
    responses: {
      200: Schemas.DeleteUserCredentialResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_user_permissions = {
    method: 'GET'
    path: '/realms/{realm_name}/users/{user_id}/permissions'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string }
    }
    responses: {
      200: Schemas.UserPermissionsResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      404: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type put_Reset_password = {
    method: 'PUT'
    path: '/realms/{realm_name}/users/{user_id}/reset-password'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string }

      body: Schemas.ResetPasswordValidator
    }
    responses: {
      200: Schemas.ResetPasswordResponse
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_user_roles = {
    method: 'GET'
    path: '/realms/{realm_name}/users/{user_id}/roles'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string }
    }
    responses: {
      200: Schemas.GetUserRolesResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Assign_role = {
    method: 'POST'
    path: '/realms/{realm_name}/users/{user_id}/roles/{role_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string; role_id: string }
    }
    responses: {
      200: Schemas.AssignRoleResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type delete_Unassign_role = {
    method: 'DELETE'
    path: '/realms/{realm_name}/users/{user_id}/roles/{role_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string; role_id: string }
    }
    responses: {
      200: Schemas.UnassignRoleResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Fetch_webhooks = {
    method: 'GET'
    path: '/realms/{realm_name}/webhooks'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    responses: {
      200: Schemas.GetWebhooksResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type post_Create_webhook = {
    method: 'POST'
    path: '/realms/{realm_name}/webhooks'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.CreateWebhookValidator
    }
    responses: {
      200: Schemas.CreateWebhookResponse
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type get_Get_webhook = {
    method: 'GET'
    path: '/realms/{realm_name}/webhooks/{webhook_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; webhook_id: string }
    }
    responses: {
      200: Schemas.Webhook
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type put_Update_webhook = {
    method: 'PUT'
    path: '/realms/{realm_name}/webhooks/{webhook_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; webhook_id: string }
    }
    responses: {
      200: Schemas.UpdateWebhookResponse
      400: Schemas.ApiErrorResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }
  export type delete_Delete_webhook = {
    method: 'DELETE'
    path: '/realms/{realm_name}/webhooks/{webhook_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; webhook_id: string }
    }
    responses: {
      200: Schemas.DeleteWebhookResponse
      401: Schemas.ApiErrorResponse
      403: Schemas.ApiErrorResponse
      500: Schemas.ApiErrorResponse
    }
  }

  // </Endpoints>
}

// <EndpointByMethod>
export type EndpointByMethod = {
  post: {
    '/realms': Endpoints.post_Create_realm
    '/realms/{realm_name}/broker/{alias}/endpoint': Endpoints.post_Broker_callback
    '/realms/{realm_name}/client-scopes': Endpoints.post_Create_client_scope
    '/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers': Endpoints.post_Create_protocol_mapper
    '/realms/{realm_name}/clients': Endpoints.post_Create_client
    '/realms/{realm_name}/clients/{client_id}/post-logout-redirects': Endpoints.post_Create_post_logout_redirect_uri
    '/realms/{realm_name}/clients/{client_id}/redirects': Endpoints.post_Create_redirect_uri
    '/realms/{realm_name}/clients/{client_id}/roles': Endpoints.post_Create_client_role
    '/realms/{realm_name}/federation/providers': Endpoints.post_Create_provider
    '/realms/{realm_name}/federation/providers/{id}/sync-users': Endpoints.post_Sync_users
    '/realms/{realm_name}/federation/providers/{id}/test-connection': Endpoints.post_Test_connection
    '/realms/{realm_name}/identity-providers': Endpoints.post_Create_identity_provider
    '/realms/{realm_name}/login-actions/authenticate': Endpoints.post_Authenticate
    '/realms/{realm_name}/login-actions/burn-recovery-code': Endpoints.post_Burn_recovery_code
    '/realms/{realm_name}/login-actions/challenge-otp': Endpoints.post_Challenge_otp
    '/realms/{realm_name}/login-actions/forgot-password': Endpoints.post_Forgot_password
    '/realms/{realm_name}/login-actions/generate-recovery-codes': Endpoints.post_Generate_recovery_codes
    '/realms/{realm_name}/login-actions/reset-password': Endpoints.post_Reset_password_with_token
    '/realms/{realm_name}/login-actions/send-magic-link': Endpoints.post_Send_magic_link
    '/realms/{realm_name}/login-actions/update-password': Endpoints.post_Update_password
    '/realms/{realm_name}/login-actions/verify-otp': Endpoints.post_Verify_otp
    '/realms/{realm_name}/login-actions/webauthn-public-key-authenticate': Endpoints.post_Webauthn_public_key_authenticate
    '/realms/{realm_name}/login-actions/webauthn-public-key-create': Endpoints.post_Webauthn_public_key_create
    '/realms/{realm_name}/login-actions/webauthn-public-key-create-options': Endpoints.post_Webauthn_public_key_create_options
    '/realms/{realm_name}/login-actions/webauthn-public-key-request-options': Endpoints.post_Webauthn_public_key_request_options
    '/realms/{realm_name}/protocol/openid-connect/logout': Endpoints.post_Logout_post
    '/realms/{realm_name}/protocol/openid-connect/registrations': Endpoints.post_Registration_handler
    '/realms/{realm_name}/protocol/openid-connect/revoke': Endpoints.post_Revoke_token
    '/realms/{realm_name}/protocol/openid-connect/token': Endpoints.post_Exchange_token
    '/realms/{realm_name}/protocol/openid-connect/token/introspect': Endpoints.post_Introspect_token
    '/realms/{realm_name}/roles': Endpoints.post_Create_realm_role
    '/realms/{realm_name}/users': Endpoints.post_Create_user
    '/realms/{realm_name}/users/{user_id}/roles/{role_id}': Endpoints.post_Assign_role
    '/realms/{realm_name}/webhooks': Endpoints.post_Create_webhook
  }
  get: {
    '/realms/{name}': Endpoints.get_Get_realm
    '/realms/{name}/login-settings': Endpoints.get_Get_login_realm_settings_handler
    '/realms/{realm_name}/.well-known/openid-configuration': Endpoints.get_Get_openid_configuration
    '/realms/{realm_name}/broker/{alias}/login': Endpoints.get_Broker_login
    '/realms/{realm_name}/client-scopes': Endpoints.get_Get_client_scopes
    '/realms/{realm_name}/client-scopes/{scope_id}': Endpoints.get_Get_client_scope
    '/realms/{realm_name}/clients': Endpoints.get_Get_clients
    '/realms/{realm_name}/clients/{client_id}': Endpoints.get_Get_client
    '/realms/{realm_name}/clients/{client_id}/client-scopes': Endpoints.get_Get_client_client_scopes
    '/realms/{realm_name}/clients/{client_id}/post-logout-redirects': Endpoints.get_Get_post_logout_redirect_uris
    '/realms/{realm_name}/clients/{client_id}/redirects': Endpoints.get_Get_redirect_uris
    '/realms/{realm_name}/clients/{client_id}/roles': Endpoints.get_Get_client_roles
    '/realms/{realm_name}/compass/v1/flows': Endpoints.get_Get_flows
    '/realms/{realm_name}/compass/v1/flows/{flow_id}': Endpoints.get_Get_flow
    '/realms/{realm_name}/compass/v1/stats': Endpoints.get_Get_stats
    '/realms/{realm_name}/federation/providers': Endpoints.get_List_providers
    '/realms/{realm_name}/federation/providers/{id}': Endpoints.get_Get_provider
    '/realms/{realm_name}/identity-providers': Endpoints.get_List_identity_providers
    '/realms/{realm_name}/identity-providers/{alias}': Endpoints.get_Get_identity_provider
    '/realms/{realm_name}/login-actions/setup-otp': Endpoints.get_Setup_otp
    '/realms/{realm_name}/login-actions/verify-magic-link': Endpoints.get_Verify_magic_link
    '/realms/{realm_name}/protocol/openid-connect/auth': Endpoints.get_Auth_handler
    '/realms/{realm_name}/protocol/openid-connect/certs': Endpoints.get_Get_certs
    '/realms/{realm_name}/protocol/openid-connect/jwks.json': Endpoints.get_Get_jwks_json
    '/realms/{realm_name}/protocol/openid-connect/logout': Endpoints.get_Logout_get
    '/realms/{realm_name}/protocol/openid-connect/userinfo': Endpoints.get_Get_userinfo
    '/realms/{realm_name}/roles': Endpoints.get_Get_roles
    '/realms/{realm_name}/roles/{role_id}': Endpoints.get_Get_role
    '/realms/{realm_name}/seawatch/v1/security-events': Endpoints.get_Get_security_events
    '/realms/{realm_name}/users': Endpoints.get_Get_users
    '/realms/{realm_name}/users/@me/realms': Endpoints.get_Get_user_realms
    '/realms/{realm_name}/users/{user_id}': Endpoints.get_Get_user
    '/realms/{realm_name}/users/{user_id}/credentials': Endpoints.get_Get_user_credentials
    '/realms/{realm_name}/users/{user_id}/permissions': Endpoints.get_Get_user_permissions
    '/realms/{realm_name}/users/{user_id}/roles': Endpoints.get_Get_user_roles
    '/realms/{realm_name}/webhooks': Endpoints.get_Fetch_webhooks
    '/realms/{realm_name}/webhooks/{webhook_id}': Endpoints.get_Get_webhook
  }
  put: {
    '/realms/{name}': Endpoints.put_Update_realm
    '/realms/{name}/settings': Endpoints.put_Update_realm_setting
    '/realms/{realm_name}/clients/{client_id}/default-client-scopes/{scope_id}': Endpoints.put_Assign_default_scope
    '/realms/{realm_name}/clients/{client_id}/optional-client-scopes/{scope_id}': Endpoints.put_Assign_optional_scope
    '/realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id}': Endpoints.put_Update_post_logout_redirect_uri
    '/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}': Endpoints.put_Update_redirect_uri
    '/realms/{realm_name}/federation/providers/{id}': Endpoints.put_Update_provider
    '/realms/{realm_name}/identity-providers/{alias}': Endpoints.put_Update_identity_provider
    '/realms/{realm_name}/roles/{role_id}': Endpoints.put_Update_role
    '/realms/{realm_name}/users/{user_id}': Endpoints.put_Update_user
    '/realms/{realm_name}/users/{user_id}/reset-password': Endpoints.put_Reset_password
    '/realms/{realm_name}/webhooks/{webhook_id}': Endpoints.put_Update_webhook
  }
  delete: {
    '/realms/{name}': Endpoints.delete_Delete_realm
    '/realms/{realm_name}/client-scopes/{scope_id}': Endpoints.delete_Delete_client_scope
    '/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id}': Endpoints.delete_Delete_protocol_mapper
    '/realms/{realm_name}/clients/{client_id}': Endpoints.delete_Delete_client
    '/realms/{realm_name}/clients/{client_id}/default-client-scopes/{scope_id}': Endpoints.delete_Unassign_default_scope
    '/realms/{realm_name}/clients/{client_id}/optional-client-scopes/{scope_id}': Endpoints.delete_Unassign_optional_scope
    '/realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id}': Endpoints.delete_Delete_post_logout_redirect_uri
    '/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}': Endpoints.delete_Delete_redirect_uri
    '/realms/{realm_name}/federation/providers/{id}': Endpoints.delete_Delete_provider
    '/realms/{realm_name}/identity-providers/{alias}': Endpoints.delete_Delete_identity_provider
    '/realms/{realm_name}/roles/{role_id}': Endpoints.delete_Delete_role
    '/realms/{realm_name}/users/bulk': Endpoints.delete_Bulk_delete_user
    '/realms/{realm_name}/users/{user_id}': Endpoints.delete_Delete_user
    '/realms/{realm_name}/users/{user_id}/credentials/{credential_id}': Endpoints.delete_Delete_user_credential
    '/realms/{realm_name}/users/{user_id}/roles/{role_id}': Endpoints.delete_Unassign_role
    '/realms/{realm_name}/webhooks/{webhook_id}': Endpoints.delete_Delete_webhook
  }
  patch: {
    '/realms/{realm_name}/client-scopes/{scope_id}': Endpoints.patch_Update_client_scope
    '/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id}': Endpoints.patch_Update_protocol_mapper
    '/realms/{realm_name}/clients/{client_id}': Endpoints.patch_Update_client
    '/realms/{realm_name}/roles/{role_id}/permissions': Endpoints.patch_Update_role_permissions
  }
}

// </EndpointByMethod>

// <EndpointByMethod.Shorthands>
export type PostEndpoints = EndpointByMethod['post']
export type GetEndpoints = EndpointByMethod['get']
export type PutEndpoints = EndpointByMethod['put']
export type DeleteEndpoints = EndpointByMethod['delete']
export type PatchEndpoints = EndpointByMethod['patch']
// </EndpointByMethod.Shorthands>

// <ApiClientTypes>
export type EndpointParameters = {
  body?: unknown
  query?: Record<string, unknown>
  header?: Record<string, unknown>
  path?: Record<string, unknown>
}

export type MutationMethod = 'post' | 'put' | 'patch' | 'delete'
export type Method = 'get' | 'head' | 'options' | MutationMethod

type RequestFormat = 'json' | 'form-data' | 'form-url' | 'binary' | 'text'

export type DefaultEndpoint = {
  parameters?: EndpointParameters | undefined
  responses?: Record<string, unknown>
  responseHeaders?: Record<string, unknown>
}

export type Endpoint<TConfig extends DefaultEndpoint = DefaultEndpoint> = {
  operationId: string
  method: Method
  path: string
  requestFormat: RequestFormat
  parameters?: TConfig['parameters']
  meta: {
    alias: string
    hasParameters: boolean
    areParametersRequired: boolean
  }
  responses?: TConfig['responses']
  responseHeaders?: TConfig['responseHeaders']
}

export interface Fetcher {
  decodePathParams?: (path: string, pathParams: Record<string, string>) => string
  encodeSearchParams?: (searchParams: Record<string, unknown> | undefined) => URLSearchParams
  //
  fetch: (input: {
    method: Method
    url: URL
    urlSearchParams?: URLSearchParams | undefined
    parameters?: EndpointParameters | undefined
    path: string
    overrides?: RequestInit
    throwOnStatusError?: boolean
  }) => Promise<Response>
  parseResponseData?: (response: Response) => Promise<unknown>
}

export const successStatusCodes = [
  200, 201, 202, 203, 204, 205, 206, 207, 208, 226, 300, 301, 302, 303, 304, 305, 306, 307, 308,
] as const
export type SuccessStatusCode = (typeof successStatusCodes)[number]

export const errorStatusCodes = [
  400, 401, 402, 403, 404, 405, 406, 407, 408, 409, 410, 411, 412, 413, 414, 415, 416, 417, 418,
  421, 422, 423, 424, 425, 426, 428, 429, 431, 451, 500, 501, 502, 503, 504, 505, 506, 507, 508,
  510, 511,
] as const
export type ErrorStatusCode = (typeof errorStatusCodes)[number]

// Taken from https://github.com/unjs/fetchdts/blob/ec4eaeab5d287116171fc1efd61f4a1ad34e4609/src/fetch.ts#L3
export interface TypedHeaders<
  TypedHeaderValues extends Record<string, string> | unknown,
> extends Omit<Headers, 'append' | 'delete' | 'get' | 'getSetCookie' | 'has' | 'set' | 'forEach'> {
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/append) */
  append: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(
    name: Name,
    value: Lowercase<Name> extends keyof TypedHeaderValues
      ? TypedHeaderValues[Lowercase<Name>]
      : string
  ) => void
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/delete) */
  delete: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(
    name: Name
  ) => void
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/get) */
  get: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(
    name: Name
  ) =>
    | (Lowercase<Name> extends keyof TypedHeaderValues
        ? TypedHeaderValues[Lowercase<Name>]
        : string)
    | null
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/getSetCookie) */
  getSetCookie: () => string[]
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/has) */
  has: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(
    name: Name
  ) => boolean
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/set) */
  set: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(
    name: Name,
    value: Lowercase<Name> extends keyof TypedHeaderValues
      ? TypedHeaderValues[Lowercase<Name>]
      : string
  ) => void
  forEach: (
    callbackfn: (
      value: TypedHeaderValues[keyof TypedHeaderValues] | (string & {}),
      key: Extract<keyof TypedHeaderValues, string> | (string & {}),
      parent: TypedHeaders<TypedHeaderValues>
    ) => void,
    thisArg?: any
  ) => void
}

/** @see https://developer.mozilla.org/en-US/docs/Web/API/Response */
export interface TypedSuccessResponse<TSuccess, TStatusCode, THeaders> extends Omit<
  Response,
  'ok' | 'status' | 'json' | 'headers'
> {
  ok: true
  status: TStatusCode
  headers: never extends THeaders ? Headers : TypedHeaders<THeaders>
  data: TSuccess
  /** [MDN Reference](https://developer.mozilla.org/en-US/docs/Web/API/Response/json) */
  json: () => Promise<TSuccess>
}

/** @see https://developer.mozilla.org/en-US/docs/Web/API/Response */
export interface TypedErrorResponse<TData, TStatusCode, THeaders> extends Omit<
  Response,
  'ok' | 'status' | 'json' | 'headers'
> {
  ok: false
  status: TStatusCode
  headers: never extends THeaders ? Headers : TypedHeaders<THeaders>
  data: TData
  /** [MDN Reference](https://developer.mozilla.org/en-US/docs/Web/API/Response/json) */
  json: () => Promise<TData>
}

export type TypedApiResponse<
  TAllResponses extends Record<string | number, unknown> = {},
  THeaders = {},
> = {
  [K in keyof TAllResponses]: K extends string
    ? K extends `${infer TStatusCode extends number}`
      ? TStatusCode extends SuccessStatusCode
        ? TypedSuccessResponse<
            TAllResponses[K],
            TStatusCode,
            K extends keyof THeaders ? THeaders[K] : never
          >
        : TypedErrorResponse<
            TAllResponses[K],
            TStatusCode,
            K extends keyof THeaders ? THeaders[K] : never
          >
      : never
    : K extends number
      ? K extends SuccessStatusCode
        ? TypedSuccessResponse<TAllResponses[K], K, K extends keyof THeaders ? THeaders[K] : never>
        : TypedErrorResponse<TAllResponses[K], K, K extends keyof THeaders ? THeaders[K] : never>
      : never
}[keyof TAllResponses]

export type SafeApiResponse<TEndpoint> = TEndpoint extends { responses: infer TResponses }
  ? TResponses extends Record<string, unknown>
    ? TypedApiResponse<
        TResponses,
        TEndpoint extends { responseHeaders: infer THeaders } ? THeaders : never
      >
    : never
  : never

export type InferResponseByStatus<TEndpoint, TStatusCode> = Extract<
  SafeApiResponse<TEndpoint>,
  { status: TStatusCode }
>

type RequiredKeys<T> = {
  [P in keyof T]-?: undefined extends T[P] ? never : P
}[keyof T]

type MaybeOptionalArg<T> = RequiredKeys<T> extends never ? [config?: T] : [config: T]
type NotNever<T> = [T] extends [never] ? false : true

// </ApiClientTypes>

// <TypedStatusError>
export class TypedStatusError<TData = unknown> extends Error {
  response: TypedErrorResponse<TData, ErrorStatusCode, unknown>
  status: number
  constructor(response: TypedErrorResponse<TData, ErrorStatusCode, unknown>) {
    super(`HTTP ${response.status}: ${response.statusText}`)
    this.name = 'TypedStatusError'
    this.response = response
    this.status = response.status
  }
}
// </TypedStatusError>

// <ApiClient>
export class ApiClient {
  baseUrl: string = ''
  successStatusCodes = successStatusCodes
  errorStatusCodes = errorStatusCodes

  constructor(public fetcher: Fetcher) {}

  setBaseUrl(baseUrl: string) {
    this.baseUrl = baseUrl
    return this
  }

  /**
   * Replace path parameters in URL
   * Supports both OpenAPI format {param} and Express format :param
   */
  defaultDecodePathParams = (url: string, params: Record<string, string>): string => {
    return url
      .replace(/{(\w+)}/g, (_, key: string) => params[key] || `{${key}}`)
      .replace(/:([a-zA-Z0-9_]+)/g, (_, key: string) => params[key] || `:${key}`)
  }

  /** Uses URLSearchParams, skips null/undefined values */
  defaultEncodeSearchParams = (
    queryParams: Record<string, unknown> | undefined
  ): URLSearchParams | undefined => {
    if (!queryParams) return

    const searchParams = new URLSearchParams()
    Object.entries(queryParams).forEach(([key, value]) => {
      if (value != null) {
        // Skip null/undefined values
        if (Array.isArray(value)) {
          value.forEach((val) => val != null && searchParams.append(key, String(val)))
        } else {
          searchParams.append(key, String(value))
        }
      }
    })

    return searchParams
  }

  defaultParseResponseData = async (response: Response): Promise<unknown> => {
    const contentType = response.headers.get('content-type') ?? ''
    if (contentType.startsWith('text/')) {
      return await response.text()
    }

    if (contentType === 'application/octet-stream') {
      return await response.arrayBuffer()
    }

    if (
      contentType.includes('application/json') ||
      (contentType.includes('application/') && contentType.includes('json')) ||
      contentType === '*/*'
    ) {
      try {
        return await response.json()
      } catch {
        return undefined
      }
    }

    return
  }

  // <ApiClient.post>
  post<Path extends keyof PostEndpoints, TEndpoint extends PostEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & {
              overrides?: RequestInit
              withResponse?: false
              throwOnStatusError?: boolean
            }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>['data']>

  post<Path extends keyof PostEndpoints, TEndpoint extends PostEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>

  post<Path extends keyof PostEndpoints, _TEndpoint extends PostEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<any>
  ): Promise<any> {
    return this.request('post', path, ...params)
  }
  // </ApiClient.post>

  // <ApiClient.get>
  get<Path extends keyof GetEndpoints, TEndpoint extends GetEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & {
              overrides?: RequestInit
              withResponse?: false
              throwOnStatusError?: boolean
            }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>['data']>

  get<Path extends keyof GetEndpoints, TEndpoint extends GetEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>

  get<Path extends keyof GetEndpoints, _TEndpoint extends GetEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<any>
  ): Promise<any> {
    return this.request('get', path, ...params)
  }
  // </ApiClient.get>

  // <ApiClient.put>
  put<Path extends keyof PutEndpoints, TEndpoint extends PutEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & {
              overrides?: RequestInit
              withResponse?: false
              throwOnStatusError?: boolean
            }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>['data']>

  put<Path extends keyof PutEndpoints, TEndpoint extends PutEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>

  put<Path extends keyof PutEndpoints, _TEndpoint extends PutEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<any>
  ): Promise<any> {
    return this.request('put', path, ...params)
  }
  // </ApiClient.put>

  // <ApiClient.delete>
  delete<Path extends keyof DeleteEndpoints, TEndpoint extends DeleteEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & {
              overrides?: RequestInit
              withResponse?: false
              throwOnStatusError?: boolean
            }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>['data']>

  delete<Path extends keyof DeleteEndpoints, TEndpoint extends DeleteEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>

  delete<Path extends keyof DeleteEndpoints, _TEndpoint extends DeleteEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<any>
  ): Promise<any> {
    return this.request('delete', path, ...params)
  }
  // </ApiClient.delete>

  // <ApiClient.patch>
  patch<Path extends keyof PatchEndpoints, TEndpoint extends PatchEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & {
              overrides?: RequestInit
              withResponse?: false
              throwOnStatusError?: boolean
            }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>['data']>

  patch<Path extends keyof PatchEndpoints, TEndpoint extends PatchEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>

  patch<Path extends keyof PatchEndpoints, _TEndpoint extends PatchEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<any>
  ): Promise<any> {
    return this.request('patch', path, ...params)
  }
  // </ApiClient.patch>

  // <ApiClient.request>
  /**
   * Generic request method with full type-safety for any endpoint
   */
  request<
    TMethod extends keyof EndpointByMethod,
    TPath extends keyof EndpointByMethod[TMethod],
    TEndpoint extends EndpointByMethod[TMethod][TPath],
  >(
    method: TMethod,
    path: TPath,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & {
              overrides?: RequestInit
              withResponse?: false
              throwOnStatusError?: boolean
            }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>['data']>

  request<
    TMethod extends keyof EndpointByMethod,
    TPath extends keyof EndpointByMethod[TMethod],
    TEndpoint extends EndpointByMethod[TMethod][TPath],
  >(
    method: TMethod,
    path: TPath,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>

  request<
    TMethod extends keyof EndpointByMethod,
    TPath extends keyof EndpointByMethod[TMethod],
    TEndpoint extends EndpointByMethod[TMethod][TPath],
  >(method: TMethod, path: TPath, ...params: MaybeOptionalArg<any>): Promise<any> {
    const requestParams = params[0]
    const withResponse = requestParams?.withResponse
    const {
      withResponse: _,
      throwOnStatusError = withResponse ? false : true,
      overrides,
      ...fetchParams
    } = requestParams || {}

    const parametersToSend: EndpointParameters = {}
    if (requestParams?.body !== undefined) (parametersToSend as any).body = requestParams.body
    if (requestParams?.query !== undefined) (parametersToSend as any).query = requestParams.query
    if (requestParams?.header !== undefined) (parametersToSend as any).header = requestParams.header
    if (requestParams?.path !== undefined) (parametersToSend as any).path = requestParams.path

    const resolvedPath = (this.fetcher.decodePathParams ?? this.defaultDecodePathParams)(
      this.baseUrl + (path as string),
      (parametersToSend.path ?? {}) as Record<string, string>
    )
    const url = new URL(resolvedPath)
    const urlSearchParams = (this.fetcher.encodeSearchParams ?? this.defaultEncodeSearchParams)(
      parametersToSend.query
    )

    const promise = this.fetcher
      .fetch({
        method: method,
        path: path as string,
        url,
        urlSearchParams,
        parameters: Object.keys(fetchParams).length ? fetchParams : undefined,
        overrides,
        throwOnStatusError,
      })
      .then(async (response) => {
        const data = await (this.fetcher.parseResponseData ?? this.defaultParseResponseData)(
          response
        )
        const typedResponse = Object.assign(response, {
          data: data,
          json: () => Promise.resolve(data),
        }) as SafeApiResponse<TEndpoint>

        if (throwOnStatusError && errorStatusCodes.includes(response.status as never)) {
          throw new TypedStatusError(typedResponse as never)
        }

        return withResponse ? typedResponse : data
      })

    return promise as Extract<
      InferResponseByStatus<TEndpoint, SuccessStatusCode>,
      { data: {} }
    >['data']
  }
  // </ApiClient.request>
}

export function createApiClient(fetcher: Fetcher, baseUrl?: string) {
  return new ApiClient(fetcher).setBaseUrl(baseUrl ?? '')
}

/**
 Example usage:
 const api = createApiClient((method, url, params) =>
   fetch(url, { method, body: JSON.stringify(params) }).then((res) => res.json()),
 );
 api.get("/users").then((users) => console.log(users));
 api.post("/users", { body: { name: "John" } }).then((user) => console.log(user));
 api.put("/users/:id", { path: { id: 1 }, body: { name: "John" } }).then((user) => console.log(user));

 // With error handling
 const result = await api.get("/users/{id}", { path: { id: "123" }, withResponse: true });
 if (result.ok) {
   // Access data directly
   const user = result.data;
   console.log(user);

   // Or use the json() method for compatibility
   const userFromJson = await result.json();
   console.log(userFromJson);
 } else {
   const error = result.data;
   console.error(`Error ${result.status}:`, error);
 }
*/

// </ApiClient>
