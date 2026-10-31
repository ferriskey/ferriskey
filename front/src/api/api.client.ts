

  export namespace Schemas {
    // <Schemas>
  export type AccountEntryState = ("active" | "soft_expired" | "expired")
export type AccountEntryDto = { display_name: string, is_active: boolean, last_seen_at?: (string | null), masked_email?: (string | null), requires_reauthentication: boolean, session_id: string, state: AccountEntryState }
export type AccountListResponse = { client_name?: (string | null), data: Array<AccountEntryDto> }
/**
 * Incomplete schema. see https://w3c.github.io/webauthn/#dictdef-publickeycredentialcreationoptionsjson
 */
export type AccountPublicKeyCredentialCreationOptionsJSON = Record<string, unknown>
export type AcsUrl = string
export type ActivateThemeResponse = { message: string }
export type ThemeShadow = ("none" | "small" | "large")
export type ThemeBorders = Partial<{
  buttonBorderWeight: number;
  buttonRadius: number;
  inputBorderWeight: number;
  inputRadius: number;
  /**
   * Border thickness for the Magic link button. Same rationale as
   * `social_button_border_weight` — alternative auth buttons often want
   * a different visual weight than primary/secondary CTAs.
   */
  magicLinkButtonBorderWeight: number;
  /**
   * Border thickness for the Passkey button.
   */
  passkeyButtonBorderWeight: number;
  /**
   * Border thickness applied specifically to social auth buttons. Pulled
   * out from `button_border_weight` so an admin can give provider pills a
   * thicker outline (the usual visual treatment for OAuth buttons on a
   * white-on-white surface) without affecting primary/secondary CTAs.
   */
  socialButtonBorderWeight: number;
  widgetBorderWeight: number;
  widgetRadius: number;
  widgetShadow: ThemeShadow;
}>
export type ThemeColors = Partial<{
  bodyText: string;
  error: string;
  links: string;
  /**
   * Magic-link button (alternative auth path). Distinct from the generic
   * secondary button so admins can keep "Sign in with a magic link"
   * neutral while branding the primary CTAs separately.
   */
  magicLinkButtonBackground: string;
  magicLinkButtonBorder: string;
  magicLinkButtonLabel: string;
  pageBackground: string;
  /**
   * Passkey button (alternative auth path). Same rationale as magic-link.
   */
  passkeyButtonBackground: string;
  passkeyButtonBorder: string;
  passkeyButtonLabel: string;
  primaryButton: string;
  primaryButtonLabel: string;
  secondaryButton: string;
  secondaryButtonLabel: string;
  /**
   * Background of social auth buttons (Google, GitHub, …). Defaults to
   * white because most provider icons assume a light surface. Distinct
   * from `secondary_button` so admins can keep a neutral "Sign in with"
   * look while branding their primary/secondary CTAs.
   */
  socialButtonBackground: string;
  /**
   * Outline color of social buttons. Typically a faint grey — provider
   * pills tend to read better with a visible border than a flat
   * background, even when the background matches the page surface.
   */
  socialButtonBorder: string;
  socialButtonLabel: string;
  widgetBackground: string;
}>
export type ThemeFontStyle = { sizePct: number, weight: number }
export type ThemeLinkStyle = ("normal" | "underline")
export type ThemeFontLinkStyle = { sizePct: number, style: ThemeLinkStyle, weight: number }
export type ThemeFonts = Partial<{ baseSize: number, body: ThemeFontStyle, buttons: ThemeFontStyle, inputLabels: ThemeFontStyle, links: ThemeFontLinkStyle, subtitle: ThemeFontStyle, title: ThemeFontStyle, url: (string | null) }>
export type ThemeSpacing = Partial<{ fieldGap: number, sectionGap: number, widgetPadding: number }>
export type PortalThemeConfig = Partial<{ borders: ThemeBorders, colors: ThemeColors, fonts: ThemeFonts, spacing: ThemeSpacing }>
export type ActiveThemeResponse = { design_tokens: PortalThemeConfig, layout_id?: (string | null), page_tree: unknown, theme_id?: (string | null) }
export type ActorType = ("user" | "service_account" | "admin" | "system")
export type MaintenanceWhitelistEntry = { client_id: string, created_at: string, id: string, role_id?: (string | null), user_id?: (string | null) }
export type AddClientWhitelistEntryResponse = { data: MaintenanceWhitelistEntry }
export type AddGroupMemberValidator = { user_id: string }
export type AddMemberValidator = { user_id: string }
export type RealmId = string
export type RealmMaintenanceWhitelistEntry = { created_at: string, id: string, realm_id: RealmId, role_id?: (string | null), user_id?: (string | null) }
export type AddRealmWhitelistEntryResponse = { data: RealmMaintenanceWhitelistEntry }
export type AddWhitelistEntryValidator = Partial<{ role_id: (string | null), user_id: (string | null) }>
export type ValidationError = { code: string, field: string, message: string }
export type ApiError = ({ InternalServerError: string } | { UnProcessableEntity: Array<ValidationError> } | { NotFound: string } | { Unauthorized: string } | { Forbidden: string } | { BadRequest: string } | { Conflict: string } | { ServiceUnavailable: string } | {
  /**
   * RFC 6749 §5.2 OAuth2 error response
   */
  OAuthError: { error: string, error_description: string };
} | {
  /**
   * RFC 6749 §5.2 OAuth2 error response for a failed client
   * authentication: HTTP 401 with a `WWW-Authenticate` challenge.
   */
  OAuthUnauthorized: { error: string, error_description: string };
} | { OAuthTooManyRequests: { error: string, error_description: string, retry_after_seconds: number } } | {
  /**
   * RFC 6749 §5.2 `temporarily_unavailable`: HTTP 503 with a `Retry-After`.
   */
  OAuthUnavailable: { error: string, error_description: string, retry_after_seconds: number };
})
export type ApiErrorResponse = { code: string, message: string, reason: string, status: number }
export type AssignGroupRoleValidator = { role_id: string }
export type AssignMemberRoleValidator = { role_id: string }
export type AssignRoleResponse = { message: string, realm_name: string, user_id: string }
export type AuthProtocol = ("openid-connect" | "saml")
export type AuthResponse = { url: string }
export type AuthenticateRequest = Partial<{
  password: (string | null);
  /**
   * Keep the SSO session past the browser closing. Honoured only when the
   * realm enables "remember me".
   */
  remember_me: boolean;
  username: (string | null);
}>
export type RequiredAction = ("configure_otp" | "verify_email" | "update_password" | "configure_passkey")
export type AuthenticationStatus = ("Success" | "RequiresActions" | "RequiresOtpChallenge" | "Failed")
export type AuthenticateResponse = { email?: (string | null), message?: (string | null), required_actions?: (Array<RequiredAction> | null), status: AuthenticationStatus, url?: (string | null) }
export type AuthenticationAttemptResponse = { login_url: string }
export type AuthorizationServerMetadata = { authorization_endpoint: string, authorization_response_iss_parameter_supported: boolean, client_id_metadata_document_supported?: (boolean | null), code_challenge_methods_supported: Array<string>, grant_types_supported: Array<string>, introspection_endpoint: string, issuer: string, jwks_uri: string, registration_endpoint?: (string | null), response_types_supported: Array<string>, revocation_endpoint: string, scopes_supported: Array<string>, token_endpoint: string, token_endpoint_auth_methods_supported: Array<string> }
export type AuthzStateBody = { enabled: boolean, version: number }
export type AuthzenAction = { name: string, properties?: (Record<string, unknown> | null) }
export type AuthzenConfiguration = { access_evaluation_endpoint: string, access_evaluations_endpoint: string, policy_decision_point: string }
export type AuthzenEntity = { id: string, properties?: (Record<string, unknown> | null), type: string }
export type EvaluationError = { message: string, status: number }
export type ReasonAdmin = { policies: Array<string> }
export type BatchDecisionContext = Partial<{ error: (EvaluationError | null), reason_admin: (ReasonAdmin | null) }>
export type BatchDecision = { context?: (BatchDecisionContext | null), decision: boolean }
export type BuiltinSchemaBody = { namespace: string, source: string }
export type BulkDeleteUserResponse = { count: number, realm_name: string }
export type BulkDeleteUserValidator = Partial<{ ids: Array<string> }>
export type BurnRecoveryCodeRequest = { recovery_code: string, recovery_code_format: string }
export type BurnRecoveryCodeResponse = Partial<{ login_url: (string | null), required_actions: Array<RequiredAction> }>
export type EntityRefBody = { id: string, type: string }
export type PolicyBodyBody = ({ source: string, type: "static" } | { links: Record<string, EntityRefBody>, template_id: string, type: "linked" })
export type CandidateValidator = { body: PolicyBodyBody, name: string }
export type ChallengeOtpRequest = Partial<{ code: string }>
export type ChallengeOtpResponse = Partial<{ required_actions: Array<RequiredAction>, url: (string | null) }>
export type ChangeOwnPasswordResponse = { message: string }
export type ChangeOwnPasswordValidator = { current_password?: string, elevation_id: string, new_password?: string }
export type ClientType = ("confidential" | "public" | "system")
export type MaintenanceSessionStrategy = ("terminate" | "expire")
export type RedirectUri = { client_id: string, created_at: string, enabled: boolean, id: string, updated_at: string, value: string }
export type ClientRegistrationSource = ("admin" | "dynamic" | "metadata_document")
export type Masked_String = string
export type Client = {
  access_token_lifetime?: (number | null);
  /**
   * Whether the client needs the `sid` claim in its logout tokens.
   * FerrisKey always sends it, so this only records what the client asked.
   */
  backchannel_logout_session_required: boolean;
  /**
   * Where FerrisKey POSTs a logout token when a session this client took
   * part in ends (OIDC Back-Channel Logout 1.0). `None` opts out.
   */
  backchannel_logout_uri?: (string | null);
  client_id: string;
  client_type: ClientType;
  consent_required: boolean;
  created_at: string;
  direct_access_grants_enabled: boolean;
  enabled: boolean;
  id: string;
  id_token_lifetime?: (number | null);
  maintenance_enabled: boolean;
  maintenance_reason?: (string | null);
  maintenance_session_strategy: MaintenanceSessionStrategy;
  name: string;
  /**
   * Whether this client is allowed to use the OAuth 2.0 Device
   * Authorization Grant (RFC 8628). Opt-in: most clients should keep
   * this disabled; only browserless devices (CLI, IoT, TVs) need it.
   */
  oauth_device_code_grant_enabled: boolean;
  protocol: AuthProtocol;
  public_client: boolean;
  realm_id: RealmId;
  redirect_uris?: (Array<RedirectUri> | null);
  refresh_token_lifetime?: (number | null);
  registration_source: ClientRegistrationSource;
  require_pkce: boolean;
  secret?: (Masked_String | null);
  service_account_enabled: boolean;
  temporary_token_lifetime?: (number | null);
  token_exchange_enabled: boolean;
  updated_at: string;
}
export type ClientRegistrationRequest = Partial<{ client_name: (string | null), grant_types: (Array<string> | null), redirect_uris: Array<string>, response_types: (Array<string> | null), scope: (string | null), token_endpoint_auth_method: (string | null) }>
export type ClientRegistrationResponse = { client_id: string, client_id_issued_at: number, client_name: string, client_secret?: (string | null), client_secret_expires_at?: (number | null), grant_types: Array<string>, redirect_uris: Array<string>, response_types: Array<string>, scope: string, token_endpoint_auth_method: string }
export type NameIdFormat = ("urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress" | "urn:oasis:names:tc:SAML:2.0:nameid-format:persistent" | "urn:oasis:names:tc:SAML:2.0:nameid-format:transient" | "urn:oasis:names:tc:SAML:1.1:nameid-format:unspecified")
export type SpEntityId = string
export type ClientSamlConfig = { acs_url: AcsUrl, client_id: string, created_at: string, name_id_format: NameIdFormat, realm_id: RealmId, sign_assertions: boolean, sign_documents: boolean, sp_entity_id: SpEntityId, updated_at: string, want_authn_requests_signed: boolean }
export type ClientScopeAttribute = { id: string, name: string, scope_id: string, value?: (string | null) }
export type ScopeType = ("NONE" | "OPTIONAL" | "DEFAULT")
export type ProtocolMapper = { client_scope_id: string, config: unknown, created_at: string, id: string, mapper_type: string, name: string }
export type ClientScope = { attributes?: (Array<ClientScopeAttribute> | null), created_at: string, default_scope_type: ScopeType, description?: (string | null), dynamic_registration_allowed: boolean, id: string, name: string, protocol: string, protocol_mappers?: (Array<ProtocolMapper> | null), realm_id: RealmId, updated_at: string }
export type ClientScopeMapping = { client_id: string, default_scope_type: ScopeType, scope_id: string }
export type ClientSecretResponse = Partial<{ client_secret: (string | null) }>
export type CodeChallengeMethod = ("S256" | "PLAIN")
export type FlowId = string
export type FlowStatus = ("pending" | "success" | "failure" | "expired")
export type FlowStepId = string
export type StepStatus = ("success" | "failure" | "skipped")
export type FlowStepName = ("authorize" | "credential_validation" | "mfa_challenge" | "token_exchange" | "idp_redirect" | "idp_callback" | "finalize" | "saml_authn_request" | "saml_assertion" | "subject_token_exchange" | "account_selection")
export type CompassFlowStep = { details?: unknown, duration_ms?: (number | null), error_code?: (string | null), error_message?: (string | null), flow_id: FlowId, id: FlowStepId, started_at: string, status: StepStatus, step_name: FlowStepName }
export type CompassFlow = { client_id?: (string | null), completed_at?: (string | null), duration_ms?: (number | null), grant_type: string, id: FlowId, ip_address?: (string | null), realm_id: RealmId, started_at: string, status: FlowStatus, steps: Array<CompassFlowStep>, user_agent?: (string | null), user_id?: (string | null) }
export type JwtToken = { access_token: string, expires_in: number, id_token?: (string | null), refresh_expires_in?: (number | null), refresh_token?: (string | null), session_state?: (string | null), token_type: string }
export type CompletePasswordResetResponse = (JwtToken & Partial<{
  /**
   * Present when the password reset happened inside an OAuth flow.
   * The frontend should perform a full-page redirect to this URL to
   * complete the original authorization flow and reach the client's
   * `redirect_uri`.
   */
  login_url: (string | null);
}>)
export type ConfirmOtpEnrollmentResponse = { message: string }
export type ConfirmOwnOtpEnrollmentValidator = { code?: string, elevation_id: string, label?: (string | null) }
export type ConfirmOwnPasskeyRegistrationValidator = { credential: Record<string, unknown>, elevation_id: string }
export type ConfirmPasskeyResponse = { message: string }
export type CreateClientScopeValidator = Partial<{ description: (string | null), dynamic_registration_allowed: boolean, is_default: boolean, name: string, protocol: string }>
export type CreateClientValidator = { client_id?: string, client_type: ClientType, consent_required?: boolean, direct_access_grants_enabled?: boolean, enabled?: boolean, name?: string, oauth_device_code_grant_enabled?: boolean, protocol?: AuthProtocol, public_client?: boolean, require_pkce?: boolean, service_account_enabled?: boolean, token_exchange_enabled?: boolean }
export type EmailType = ("reset_password" | "magic_link" | "email_verification")
export type EmailTemplate = { created_at: string, email_type: EmailType, id: string, mjml: string, name: string, realm_id: string, structure: unknown, updated_at: string }
export type CreateEmailTemplateResponse = { data: EmailTemplate }
export type CreateEmailTemplateValidator = { email_type: string, name: string, structure: unknown }
export type CreateGroupValidator = {
  description?: (string | null);
  name: string;
  /**
   * Optional parent group; omit for a top-level group.
   */
  parent_group_id?: (string | null);
}
export type CreateIdentityProviderValidator = Partial<{ add_read_token_role_on_create: boolean, alias: string, config: unknown, display_name: (string | null), enabled: boolean, first_broker_login_flow_alias: (string | null), link_only: boolean, post_broker_login_flow_alias: (string | null), provider_id: string, store_token: boolean, trust_email: boolean }>
export type CreateOrganizationValidator = { alias: string, description?: (string | null), domain?: (string | null), enabled?: boolean, name: string, redirect_url?: (string | null) }
export type PortalLayout = { created_at: string, id: string, is_default: boolean, name: string, realm_id: RealmId, tree: Record<string, unknown>, updated_at: string }
export type CreatePortalLayoutResponse = { data: PortalLayout }
export type CreatePortalLayoutValidator = { name: string, tree: unknown }
export type CreateProtocolMapperValidator = Partial<{ config: unknown, mapper_type: string, name: string }>
export type CreateProviderRequest = { config: unknown, enabled: boolean, name: string, priority: number, provider_type: string, sync_enabled: boolean, sync_interval_minutes?: (number | null), sync_mode: string }
export type CreateRealmValidator = Partial<{ display_name: (string | null), name: string }>
export type CreateRedirectUriValidator = Partial<{ enabled: boolean, value: string }>
export type Role = { client?: (Client | null), client_id?: (string | null), created_at: string, description?: (string | null), id: string, name: string, permissions: Array<string>, realm_id: RealmId, require_mfa: boolean, updated_at: string }
export type CreateRoleResponse = { data: Role }
export type CreateRoleValidator = { description?: (string | null), name: string, permissions: Array<string> }
export type CreateSamlAttributeMapperValidator = Partial<{ name: string, name_format: string, source: string }>
export type PortalThemePages = Partial<{ accountChooser: unknown, consent: unknown, deviceVerified: unknown, deviceVerify: unknown, emailVerified: unknown, forgotPassword: unknown, login: unknown, magicLinkRequest: unknown, magicLinkVerify: unknown, register: unknown, resetPassword: unknown, totp: unknown, totpSetup: unknown, verifyEmail: unknown }>
export type PortalTheme = { config: PortalThemeConfig, created_at: string, id: string, layout_id?: (string | null), name: string, pages: PortalThemePages, realm_id: RealmId, updated_at: string }
export type CreateThemeResponse = { data: PortalTheme }
export type CreateThemeValidator = { config?: PortalThemeConfig, layout_id?: (string | null), name: string }
export type CreateTokenExchangePolicyValidator = Partial<{ allow_delegation: boolean, allow_impersonation: boolean, allowed_scopes: (Array<string> | null), target_audience: string }>
export type LoginAlias = ("username" | "email")
/**
 * Ordered, non-empty, duplicate-free list of login identifiers a realm accepts.
 * Order encodes precedence: the first alias that resolves a user wins.
 */
export type LoginAliases = Array<LoginAlias>
export type RealmSetting = {
  access_token_lifetime: number;
  account_chooser_enabled: boolean;
  account_chooser_show_email: boolean;
  allowed_resources: Array<string>;
  cimd_allowed_hosts: Array<string>;
  cimd_enabled: boolean;
  compass_enabled: boolean;
  dcr_enabled: boolean;
  default_locale: string;
  default_signing_algorithm?: (string | null);
  /**
   * Whether users may change their own username from the self-service
   * account page. Off by default: usernames are used as-is by login
   * aliases and federation mappings, so letting every user rename
   * themselves is an operator choice, not a default.
   */
  edit_username_enabled: boolean;
  email_verification_enabled: boolean;
  email_verification_template_id?: (string | null);
  email_verification_ttl_hours: number;
  forgot_password_enabled: boolean;
  id: string;
  id_token_lifetime: number;
  lockout_duration_seconds: number;
  lockout_threshold: number;
  login_aliases: LoginAliases;
  magic_link_enabled: boolean;
  magic_link_template_id?: (string | null);
  magic_link_ttl: number;
  passkey_enabled: boolean;
  realm_id: RealmId;
  refresh_token_lifetime: number;
  remember_me_enabled: boolean;
  require_mfa: boolean;
  reset_password_template_id?: (string | null);
  seawatch_pii_mode: string;
  supported_locales: Array<string>;
  temporary_token_lifetime: number;
  updated_at: string;
  user_registration_enabled: boolean;
  webhook_retry_base_delay_ms?: (number | null);
  webhook_retry_max_attempts?: (number | null);
  webhook_retry_max_delay_ms?: (number | null);
  webhook_retry_max_total_delay_ms?: (number | null);
}
export type Realm = {
  created_at: string;
  /**
   * Human-readable label shown in the login and admin UI.
   *
   * Unlike `name` (which is the URL slug and must stay URL-safe), this can
   * contain whitespace and be changed freely without breaking realm URLs.
   * Falls back to `name` when unset.
   */
  display_name?: (string | null);
  id: RealmId;
  name: string;
  settings?: (RealmSetting | null);
  updated_at: string;
}
export type User = { client_id?: (string | null), created_at: string, email?: (string | null), email_verified: boolean, enabled: boolean, failed_login_attempts: number, federation_provider_id?: (string | null), firstname?: (string | null), id: string, lastname?: (string | null), locale?: (string | null), locked_until?: (string | null), realm?: (Realm | null), realm_id: RealmId, required_actions: Array<RequiredAction>, roles?: (Array<Role> | null), updated_at: string, username: string }
export type CreateUserResponse = { data: User }
export type CreateUserValidator = Partial<{ email: (string | null), email_verified: (boolean | null), firstname: (string | null), id: (string | null), lastname: (string | null), username: string }>
export type CreateWebOriginValidator = Partial<{ value: string }>
export type RetryPolicyOverride = Partial<{ base_delay_ms: (number | null), max_attempts: (number | null), max_delay_ms: (number | null), max_total_delay_ms: (number | null) }>
export type WebhookTrigger = ("user.created" | "user.email_verified" | "user.updated" | "user.deleted" | "user.role.assigned" | "user.role.unassigned" | "user.bulk_deleted" | "user.credentials.deleted" | "auth.reset_password" | "auth.device_flow.initiated" | "auth.device_flow.denied" | "auth.device_flow.expired" | "client.created" | "client.updated" | "client.deleted" | "client.role.created" | "client.role.updated" | "redirect_uri.created" | "redirect_uri.updated" | "redirect_uri.deleted" | "web_origin.created" | "web_origin.deleted" | "client.saml_config.updated" | "client.saml_attribute_mapper.created" | "client.saml_attribute_mapper.deleted" | "role.created" | "role.updated" | "role.deleted" | "role.permission.updated" | "realm.created" | "realm.updated" | "realm.deleted" | "realm.settings.updated" | "webhook.created" | "webhook.updated" | "webhook.deleted" | "client.maintenance.enabled" | "client.maintenance.disabled")
export type WebhookSubscriber = { id: string, name: WebhookTrigger, webhook_id: string }
/**
 * A configured webhook subscription.
 *
 * `headers` and `secret` are `skip_serializing`: no serialized representation of a `Webhook` —
 * an HTTP read response, or a `webhook.*` notification naming this row as its payload `data` —
 * can disclose this webhook's stored credentials or another webhook's. Both fields stay plain,
 * unrestricted struct fields otherwise: the code that composes an outbound delivery for this
 * webhook reads them directly, which is the only path meant to see them again after creation.
 */
export type Webhook = { created_at: string, description?: (string | null), effective_retry_policy?: (RetryPolicyOverride | null), endpoint: string, id: string, name?: (string | null), retry_policy?: RetryPolicyOverride, subscribers: Array<WebhookSubscriber>, triggered_at?: (string | null), updated_at: string }
export type CreateWebhookResponse = { data: Webhook }
export type CreateWebhookValidator = Partial<{ description: (string | null), endpoint: string, headers: Record<string, string>, name: (string | null), retry_policy: (RetryPolicyOverride | null), subscribers: Array<WebhookTrigger> }>
export type CredentialDataOverview = ({ Hash: { algorithm: string, hash_iterations: number } } | "WebAuthn" | { Federated: { provider_id: string, provider_type: string } })
export type CredentialOverview = { created_at: string, credential_data: CredentialDataOverview, credential_type: string, id: string, updated_at: string, user_id: string, user_label?: (string | null) }
export type DailyActivityStats = { avg_login_duration_ms?: (number | null), date: string, expired_logins: number, login_failures: number, logins: number, pending_logins: number, signups: number, total_flows: number, unique_login_users: number }
export type DecisionContext = { reason_admin: ReasonAdmin }
export type DeleteClientResponse = { message: string, realm_name: string }
export type DeleteClientScopeResponse = { message: string }
export type DeleteEmailTemplateResponse = { message: string }
export type DeleteIdentityProviderLinkResponse = { count: number }
export type DeleteIdentityProviderResponse = { count: number }
export type DeletePasskeyResponse = { message: string }
export type DeletePortalLayoutResponse = { message: string }
export type DeleteProtocolMapperResponse = { message: string }
export type DeleteProviderResponse = { message: string }
export type DeleteRealmResponse = string
export type DeleteRoleResponse = { message: string, realm_name: string, role_id: string }
export type DeleteThemeResponse = { message: string }
export type DeleteUserAttributeResponse = Record<string, unknown>
export type DeleteUserCredentialResponse = { message: string, realm_name: string, user_id: string }
export type DeleteUserResponse = { count: number }
export type DeleteWebhookResponse = { message: string, realm_name: string }
export type DeliverySummary = { attempt_count: number, created_at: string, event: WebhookTrigger, id: string, last_attempt_at?: (string | null), last_error_code?: (string | null), last_error_detail?: (string | null), last_status_code?: (number | null), next_attempt_at?: (string | null), resource_id: string, status: string, updated_at: string, webhook_id: string }
export type DeliveryDetail = (DeliverySummary & { payload: unknown })
/**
 * Form body for the device authorization request (RFC 8628 §3.1).
 */
export type DeviceAuthorizationRequest = Partial<{
  /**
   * OAuth 2.0 client identifier. Optional in the body when the client
   * authenticates with HTTP Basic (confidential clients).
   */
  client_id: (string | null);
  client_secret: (string | null);
  /**
   * Space-delimited list of requested scopes.
   */
  scope: (string | null);
}>
export type DeviceVerificationPreview = { client_id: string, client_name: string, scopes: Array<string> }
/**
 * Approve or deny choice submitted from the verification page.
 */
export type DeviceVerifyAction = ("approve" | "deny")
export type DeviceVerifyRequest = {
  action: DeviceVerifyAction;
  /**
   * The end-user code shown on the device (e.g. `WDJB-MJHT`).
   */
  user_code: string;
}
export type DeviceVerifyResponse = {
  /**
   * Resulting session status: `approved` or `denied`.
   */
  status: string;
}
export type DisableOtpResponse = { message: string }
export type ElevationOnlyValidator = { elevation_id: string }
export type ElevationResponse = { elevation_id: string, expires_at: string }
export type EmailTemplateExportFormat = ("json" | "mjml")
export type EvaluatedMapper = { config: unknown, mapper_type: string, name: string }
export type EvaluatedRoles = { client_roles: Record<string, Array<string>>, realm_roles: Array<string> }
export type EvaluatedScope = { default_scope_type: string, name: string, protocol: string }
/**
 * Result of a client-scope evaluation: the effective scopes/mappers/roles plus the decoded
 * (unsigned, non-persisted) token claims a real token would carry.
 */
export type EvaluateClientScopesResult = { access_token: unknown, effective_mappers: Array<EvaluatedMapper>, effective_roles: EvaluatedRoles, effective_scopes: Array<EvaluatedScope>, id_token?: unknown, userinfo: unknown }
export type EvaluateScopesValidator = {
  /**
   * Requested scope string (space-separated). Default scopes always apply; optional scopes
   * apply only when named here.
   */
  scope?: (string | null);
  /**
   * User whose token would be previewed.
   */
  user_id: string;
}
export type EvaluationItem = Partial<{ action: (AuthzenAction | null), context: (Record<string, unknown> | null), resource: (AuthzenEntity | null), subject: (AuthzenEntity | null) }>
export type EvaluationResponse = { context?: (DecisionContext | null), decision: boolean }
export type EvaluationValidator = { action: AuthzenAction, context?: (Record<string, unknown> | null), resource: AuthzenEntity, subject: AuthzenEntity }
export type EvaluationsSemanticBody = ("execute_all" | "deny_on_first_deny" | "permit_on_first_permit")
export type EvaluationsOptions = Partial<{ evaluations_semantic: (EvaluationsSemanticBody | null) }>
export type EvaluationsResponse = { evaluations: Array<BatchDecision> }
export type EvaluationsValidator = (EvaluationItem & Partial<{ evaluations: (Array<EvaluationItem> | null), options: (EvaluationsOptions | null) }>)
export type EventStatus = ("success" | "failure")
export type ExportEnvelope = {
  /**
   * Email templates only.
   */
  email_type?: (string | null);
  /**
   * Which builder the file belongs to: `email-template` or `portal-layout`.
   */
  ferriskey: string;
  name: string;
  /**
   * The builder tree (`BuilderNode[]`).
   */
  tree: Array<Record<string, unknown>>;
  /**
   * Format version, bumped when the envelope itself changes shape.
   */
  version: number;
}
export type FlowStats = { avg_duration_ms?: (number | null), failure_count: number, pending_count: number, success_count: number, total: number }
export type ForgotPasswordRequest = { email: string }
export type ForgotPasswordResponse = unknown
export type GenerateRecoveryCodesRequest = { amount: number, code_format: string, elevation_id: string }
export type GenerateRecoveryCodesResponse = { codes: Array<string> }
export type JwkKey = { alg: string, e: string, kid: string, kty: string, n: string, use: string }
export type GetCertsResponse = { keys: Array<JwkKey> }
export type GetClientResponse = { data: Client }
export type GetClientRolesResponse = { data: Array<Role> }
export type GetClientWhitelistResponse = { data: Array<MaintenanceWhitelistEntry> }
export type ScopeView = { description?: (string | null), name: string }
export type GetConsentResponse = { awaiting_decision: Array<ScopeView>, client_name: string, client_uri_host?: (string | null), granted_by_default: Array<ScopeView> }
export type GetDailyActivityStatsResponse = { data: Array<DailyActivityStats> }
export type GetDeliveryResponse = { data: DeliveryDetail }
export type GetEmailTemplateResponse = { data: EmailTemplate }
export type GetFlowResponse = { data: CompassFlow }
export type GetOpenIdConfigurationResponse = {
  authorization_endpoint: string;
  authorization_response_iss_parameter_supported: boolean;
  /**
   * Logout tokens carry the `sid` of the session that ended.
   */
  backchannel_logout_session_supported: boolean;
  /**
   * OIDC Back-Channel Logout 1.0: clients may register a
   * `backchannel_logout_uri` to be told when a session ends.
   */
  backchannel_logout_supported: boolean;
  client_id_metadata_document_supported?: (boolean | null);
  code_challenge_methods_supported: Array<string>;
  device_authorization_endpoint: string;
  end_session_endpoint: string;
  grant_types_supported: Array<string>;
  id_token_signing_alg_values_supported: Array<string>;
  introspection_endpoint: string;
  issuer: string;
  jwks_uri: string;
  prompt_values_supported: Array<string>;
  registration_endpoint?: (string | null);
  response_types_supported: Array<string>;
  revocation_endpoint: string;
  scopes_supported: Array<string>;
  subject_types_supported: Array<string>;
  token_endpoint: string;
  token_endpoint_auth_methods_supported: Array<string>;
  userinfo_endpoint: string;
}
export type GetPortalLayoutResponse = { data: PortalLayout }
export type GetPublicDefaultPortalLayoutResponse = Partial<{ data: (PortalLayout | null) }>
export type GetPublicPortalLayoutResponse = Partial<{ data: (PortalLayout | null) }>
export type GetRealmWhitelistResponse = { data: Array<RealmMaintenanceWhitelistEntry> }
export type GetRoleResponse = { data: Role }
export type GetStatsResponse = { data: FlowStats }
export type TemplateVariable = { description: string, name: string }
export type GetTemplateVariablesResponse = { data: Array<TemplateVariable> }
export type GetThemeByIdResponse = { data: PortalTheme }
export type GetThemeResponse = { data: PortalThemeConfig }
export type GetUserCredentialsResponse = { data: Array<CredentialOverview> }
export type GetUserRolesResponse = { data: Array<Role> }
export type GrantType = ("authorization_code" | "password" | "client_credentials" | "refresh_token" | "urn:ietf:params:oauth:grant-type:device_code" | "urn:ietf:params:oauth:grant-type:token-exchange")
/**
 * Unique identifier for a group.
 */
export type GroupId = string
/**
 * Unique identifier for an Organization
 */
export type OrganizationId = string
/**
 * A group within an organization. Groups form a tree via `parent_group_id`
 * (`None` = top-level group). Membership is recursive: a member of a group is
 * effectively a member of all its ancestors.
 */
export type Group = { created_at: string, description?: (string | null), id: GroupId, name: string, organization_id: OrganizationId, parent_group_id?: (GroupId | null), updated_at: string }
/**
 * A key/value attribute on a group.
 */
export type GroupAttribute = { created_at: string, group_id: GroupId, id: string, key: string, value: string }
/**
 * A user's membership in a group.
 */
export type GroupMember = { created_at: string, group_id: GroupId, id: string, user_id: string }
export type IdentityProviderLinkResponse = { created_at: string, id: string, identity_provider_alias: string, identity_provider_id: string, identity_provider_user_id: string, updated_at: string }
export type IdentityProviderLinksResponse = { data: Array<IdentityProviderLinkResponse> }
export type IdentityProviderPresentation = { display_name: string, icon: string, id: string, kind: string, login_url: string }
export type IdentityProviderResponse = { add_read_token_role_on_create: boolean, alias: string, config: unknown, created_at: string, display_name?: (string | null), enabled: boolean, first_broker_login_flow_alias?: (string | null), internal_id: string, link_only: boolean, post_broker_login_flow_alias?: (string | null), provider_id: string, store_token: boolean, trust_email: boolean, updated_at: string }
export type ImportEmailTemplateResponse = { data: EmailTemplate }
/**
 * Import payload. Exactly one of `structure`, `tree` or `mjml` carries the
 * template body: `structure` is the persisted `{ children: [...] }` wrapper,
 * `tree` the bare `BuilderNode[]` array exported by the builder, and `mjml`
 * raw markup that is parsed back into a builder structure.
 */
export type ImportEmailTemplateValidator = {
  /**
   * Optional at deserialisation so a file from the other builder is answered
   * by the envelope check — which says what the file actually is — rather
   * than by a "missing field email_type" the person importing cannot act on.
   * The handler still requires it.
   */
  email_type?: (string | null);
  /**
   * Which builder the file came from, when it came from an export. Checked
   * so a portal layout cannot be imported as an email template.
   */
  ferriskey?: (string | null);
  mjml?: (string | null);
  name: string;
  structure?: unknown;
  tree?: unknown;
  /**
   * Export format version, when the payload came from an export.
   */
  version?: (number | null);
}
export type ImportPasswordCredentialResponse = { message: string, realm_name: string, user_id: string }
export type ImportPasswordCredentialValidator = { algorithm?: string, hash_iterations: number, salt?: (string | null), secret_data?: string, temporary?: boolean }
export type ImportPortalLayoutResponse = { data: PortalLayout }
export type ImportPortalLayoutValidator = {
  /**
   * Which builder the file came from, when it came from an export. Checked
   * so an email template cannot be imported as a layout.
   */
  ferriskey?: (string | null);
  name: string;
  /**
   * Optional at deserialisation so a file from another builder is answered
   * by the envelope check — which says what the file actually is — rather
   * than by a "missing field tree" the person importing cannot act on. The
   * handler still requires it.
   */
  tree?: unknown;
  /**
   * Export format version, when the payload came from an export.
   */
  version?: (number | null);
}
export type ImportPortalThemeLayoutValidator = { name: string, tree: Record<string, unknown> }
export type ImportPortalThemeResponse = { data: PortalTheme }
/**
 * Import payload — an exported theme envelope, sent back verbatim.
 */
export type ImportPortalThemeValidator = {
  /**
   * The theme's design tokens. Missing keys fall back to their defaults.
   */
  config?: Record<string, unknown>;
  /**
   * Which builder the file came from, when it came from an export. Checked
   * so a layout or an email template cannot be imported as a theme.
   */
  ferriskey?: (string | null);
  layout?: (ImportPortalThemeLayoutValidator | null);
  name: string;
  /**
   * One builder tree per page, keyed by page type.
   */
  pages?: Record<string, unknown>;
  /**
   * Export format version, when the payload came from an export.
   */
  version?: (number | null);
}
/**
 * Output of the device authorization endpoint (RFC 8628 §3.2).
 */
export type InitiateDeviceFlowOutput = {
  device_code: string;
  /**
   * Session lifetime, in seconds.
   */
  expires_in: number;
  /**
   * Minimum polling interval, in seconds.
   */
  interval: number;
  user_code: string;
  verification_uri: string;
  verification_uri_complete: string;
}
export type IntrospectRequestValidator = Partial<{ client_id: (string | null), client_secret: (string | null), token: string, token_type_hint: (string | null) }>
export type IssueBody = { column?: (number | null), line?: (number | null), message: string, policy?: (string | null) }
export type OrganizationAttribute = { created_at: string, id: string, key: string, organization_id: OrganizationId, value: string }
export type ListOrganizationAttributesResponse = { data: Array<OrganizationAttribute> }
export type LogoutRequestValidator = Partial<{ client_id: (string | null), id_token_hint: (string | null), post_logout_redirect_uri: (string | null), state: (string | null) }>
/**
 * RFC 6749 §5.2 OAuth2 error response body
 */
export type OAuth2ErrorResponse = { error: string, error_description: string }
/**
 * Organization domain entity
 *
 * `name` is the primary human-readable label shown in admin or end-user surfaces.
 * `alias` is the stable URL-safe identifier used for lookups and routing within a realm.
 * The first organizations release does not persist a separate `display_name`, so callers
 * should rely on `name` as the display label.
 */
export type Organization = { alias: string, created_at: string, description?: (string | null), domain?: (string | null), enabled: boolean, id: OrganizationId, name: string, realm_id: RealmId, redirect_url?: (string | null), updated_at: string }
/**
 * A user's membership in an organization
 */
export type OrganizationMember = { created_at: string, id: string, organization_id: OrganizationId, user_id: string }
/**
 * The TOTP secret is deliberately absent: it is read from the enrollment the server
 * recorded during `setup-otp`. Accepting it here let a caller enroll a secret of their
 * own choosing and verify a code against it (FK-003).
 */
export type OtpVerifyRequest = { code: string, label: string }
export type OwnCredentialDto = { created_at: string, credential_type: string, id: string, label?: (string | null) }
export type OwnCredentialsResponse = { data: Array<OwnCredentialDto> }
export type OwnProfileResponse = { data: User }
export type PageMetadata = { first_page: number, last_page: number, limit: number, next_page?: (number | null), page: number, prev_page?: (number | null), total: number, total_pages: number }
export type SigningKeyStatus = ("active" | "passive" | "retired" | "revoked")
export type SigningKey = { algorithm: string, created_at: string, kid: string, realm_id: string, retire_at?: (string | null), revoked_at?: (string | null), rotated_at?: (string | null), status: SigningKeyStatus }
export type SigningKeyResponse = { data: SigningKey }
export type SigningKeysResponse = { data: Array<SigningKey> }
export type Paginated_OwnSessionDto = { data: Array<{ created_at: string, current: boolean, expires_at: string, id: string, ip_address?: (string | null), last_seen_at?: (string | null), persistent: boolean, user_agent?: (string | null) }>, metadata: PageMetadata }
export type RevokeOtherSessionsResponse = { revoked: number }
export type PortalPageType = ("login" | "register" | "totp" | "forgot_password" | "reset_password" | "magic_link_verify" | "magic_link_request" | "verify_email" | "email_verified" | "totp_setup" | "device_verify" | "device_verified" | "consent" | "account_chooser")
export type PageRequirement = { page_type: PortalPageType, required_blocks: Array<string> }
export type PageRequirementsResponse = { data: Array<PageRequirement> }
export type Paginated_Client = {
  data: Array<{
    access_token_lifetime?: (number | null);
    /**
     * Whether the client needs the `sid` claim in its logout tokens.
     * FerrisKey always sends it, so this only records what the client asked.
     */
    backchannel_logout_session_required: boolean;
    /**
     * Where FerrisKey POSTs a logout token when a session this client took
     * part in ends (OIDC Back-Channel Logout 1.0). `None` opts out.
     */
    backchannel_logout_uri?: (string | null);
    client_id: string;
    client_type: ClientType;
    consent_required: boolean;
    created_at: string;
    direct_access_grants_enabled: boolean;
    enabled: boolean;
    id: string;
    id_token_lifetime?: (number | null);
    maintenance_enabled: boolean;
    maintenance_reason?: (string | null);
    maintenance_session_strategy: MaintenanceSessionStrategy;
    name: string;
    /**
     * Whether this client is allowed to use the OAuth 2.0 Device
     * Authorization Grant (RFC 8628). Opt-in: most clients should keep
     * this disabled; only browserless devices (CLI, IoT, TVs) need it.
     */
    oauth_device_code_grant_enabled: boolean;
    protocol: AuthProtocol;
    public_client: boolean;
    realm_id: RealmId;
    redirect_uris?: (Array<RedirectUri> | null);
    refresh_token_lifetime?: (number | null);
    registration_source: ClientRegistrationSource;
    require_pkce: boolean;
    secret?: (Masked_String | null);
    service_account_enabled: boolean;
    temporary_token_lifetime?: (number | null);
    token_exchange_enabled: boolean;
    updated_at: string;
  }>;
  metadata: PageMetadata;
}
export type Paginated_ClientScope = { data: Array<{ attributes?: (Array<ClientScopeAttribute> | null), created_at: string, default_scope_type: ScopeType, description?: (string | null), dynamic_registration_allowed: boolean, id: string, name: string, protocol: string, protocol_mappers?: (Array<ProtocolMapper> | null), realm_id: RealmId, updated_at: string }>, metadata: PageMetadata }
export type Paginated_CompassFlow = { data: Array<{ client_id?: (string | null), completed_at?: (string | null), duration_ms?: (number | null), grant_type: string, id: FlowId, ip_address?: (string | null), realm_id: RealmId, started_at: string, status: FlowStatus, steps: Array<CompassFlowStep>, user_agent?: (string | null), user_id?: (string | null) }>, metadata: PageMetadata }
export type Paginated_DeliverySummary = { data: Array<{ attempt_count: number, created_at: string, event: WebhookTrigger, id: string, last_attempt_at?: (string | null), last_error_code?: (string | null), last_error_detail?: (string | null), last_status_code?: (number | null), next_attempt_at?: (string | null), resource_id: string, status: string, updated_at: string, webhook_id: string }>, metadata: PageMetadata }
export type Paginated_EmailTemplate = { data: Array<{ created_at: string, email_type: EmailType, id: string, mjml: string, name: string, realm_id: string, structure: unknown, updated_at: string }>, metadata: PageMetadata }
export type Paginated_GroupListItem = { data: Array<(Group & { child_count: number })>, metadata: PageMetadata }
export type Paginated_GroupMemberDetail = { data: Array<{ created_at: string, email?: (string | null), enabled: boolean, firstname?: (string | null), group_id: GroupId, id: string, lastname?: (string | null), user_id: string, username: string }>, metadata: PageMetadata }
export type Paginated_IdentityProviderResponse = { data: Array<{ add_read_token_role_on_create: boolean, alias: string, config: unknown, created_at: string, display_name?: (string | null), enabled: boolean, first_broker_login_flow_alias?: (string | null), internal_id: string, link_only: boolean, post_broker_login_flow_alias?: (string | null), provider_id: string, store_token: boolean, trust_email: boolean, updated_at: string }>, metadata: PageMetadata }
export type Paginated_Organization = { data: Array<{ alias: string, created_at: string, description?: (string | null), domain?: (string | null), enabled: boolean, id: OrganizationId, name: string, realm_id: RealmId, redirect_url?: (string | null), updated_at: string }>, metadata: PageMetadata }
export type Paginated_OrganizationMember = { data: Array<{ created_at: string, id: string, organization_id: OrganizationId, user_id: string }>, metadata: PageMetadata }
export type Paginated_PortalLayoutListItem = { data: Array<(PortalLayout & { theme_count: number })>, metadata: PageMetadata }
export type Paginated_PortalTheme = { data: Array<{ config: PortalThemeConfig, created_at: string, id: string, layout_id?: (string | null), name: string, pages: PortalThemePages, realm_id: RealmId, updated_at: string }>, metadata: PageMetadata }
export type Paginated_ProviderResponse = { data: Array<{ config: unknown, created_at: string, enabled: boolean, id: string, last_sync_at?: (string | null), last_sync_status?: (string | null), name: string, priority: number, provider_type: string, realm_id: string, sync_enabled: boolean, sync_interval_minutes?: (number | null), sync_mode: string, updated_at: string }>, metadata: PageMetadata }
export type Paginated_Realm = {
  data: Array<{
    created_at: string;
    /**
     * Human-readable label shown in the login and admin UI.
     *
     * Unlike `name` (which is the URL slug and must stay URL-safe), this can
     * contain whitespace and be changed freely without breaking realm URLs.
     * Falls back to `name` when unset.
     */
    display_name?: (string | null);
    id: RealmId;
    name: string;
    settings?: (RealmSetting | null);
    updated_at: string;
  }>;
  metadata: PageMetadata;
}
export type Paginated_Role = { data: Array<{ client?: (Client | null), client_id?: (string | null), created_at: string, description?: (string | null), id: string, name: string, permissions: Array<string>, realm_id: RealmId, require_mfa: boolean, updated_at: string }>, metadata: PageMetadata }
export type SecurityEventType = ("login_success" | "login_failure" | "password_reset" | "password_reset_requested" | "password_reset_completed" | "password_imported" | "user_created" | "user_email_verified" | "user_deleted" | "role_assigned" | "role_unassigned" | "role_created" | "role_removed" | "role_updated" | "role_permission_updated" | "client_created" | "client_deleted" | "client_secret_rotated" | "client_secret_viewed" | "realm_config_changed" | "email_not_sent" | "email_sent" | "client_maintenance_enabled" | "client_maintenance_disabled" | "session_created" | "session_revoked" | "identity_provider_link_removed" | "webhook_delivery_exhausted" | "token_exchanged" | "account_selected" | "account_selection_denied" | "device_session_created" | "signing_key_rotated" | "signing_key_revoked" | "unknown")
export type SecurityEventId = string
export type Paginated_SecurityEvent = {
  data: Array<{
    actor_id?: (string | null);
    actor_type?: (ActorType | null);
    details?: unknown;
    /**
     * SHA-256 of the canonical preimage (hex). `None` for legacy rows without hashes.
     */
    event_hash?: (Array<number> | null);
    event_type: SecurityEventType;
    id: SecurityEventId;
    ip_address?: (string | null);
    /**
     * Previous event's `event_hash` in the realm chain (hex-decoded). Genesis = `[0u8; 32]`.
     */
    prev_hash?: (Array<number> | null);
    realm_id: RealmId;
    resource?: (string | null);
    status: EventStatus;
    target_id?: (string | null);
    target_type?: (string | null);
    timestamp: string;
    trace_id?: (string | null);
    user_agent?: (string | null);
  }>;
  metadata: PageMetadata;
}
export type Paginated_User = { data: Array<{ client_id?: (string | null), created_at: string, email?: (string | null), email_verified: boolean, enabled: boolean, failed_login_attempts: number, federation_provider_id?: (string | null), firstname?: (string | null), id: string, lastname?: (string | null), locale?: (string | null), locked_until?: (string | null), realm?: (Realm | null), realm_id: RealmId, required_actions: Array<RequiredAction>, roles?: (Array<Role> | null), updated_at: string, username: string }>, metadata: PageMetadata }
export type Paginated_UserSessionDto = { data: Array<{ created_at: string, expires_at: string, id: string, ip_address?: (string | null), last_seen_at?: (string | null), persistent: boolean, realm_id: string, user_agent?: (string | null), user_id: string }>, metadata: PageMetadata }
export type Paginated_Webhook = { data: Array<{ created_at: string, description?: (string | null), effective_retry_policy?: (RetryPolicyOverride | null), endpoint: string, id: string, name?: (string | null), retry_policy?: RetryPolicyOverride, subscribers: Array<WebhookSubscriber>, triggered_at?: (string | null), updated_at: string }>, metadata: PageMetadata }
export type PasskeyAuthenticateResponse = { login_url: string, status: string }
/**
 * PublicKeyCredential assertion response for passkey authentication. See https://w3c.github.io/webauthn/#dictdef-publickeycredentialjson
 */
export type PasskeyPublicKeyCredential = Record<string, unknown>
/**
 * PublicKeyCredentialRequestOptionsJSON for passkey authentication. See https://w3c.github.io/webauthn/#dictdef-publickeycredentialrequestoptionsjson
 */
export type PasskeyPublicKeyCredentialRequestOptionsJSON = Record<string, unknown>
export type PasskeyRequestOptionsRequest = Partial<{ username: (string | null) }>
export type PasswordPolicy = {
  /**
   * Interface flag — reject passwords reported as breached (requires external check; no
   * network call is made when the provider is not wired up).
   */
  check_breached: boolean;
  created_at: string;
  /**
   * Reject passwords found in the embedded common-password list or matching username/email.
   */
  forbid_common: boolean;
  id: string;
  max_age_days?: (number | null);
  /**
   * Minimum Shannon entropy in bits (charset-pool model). CNIL 2022-100 default: 80.
   */
  min_entropy_bits: number;
  min_length: number;
  realm_id: string;
  require_lowercase: boolean;
  require_number: boolean;
  require_special: boolean;
  require_uppercase: boolean;
  updated_at: string;
}
export type PendingActionResponse = { message: string, user_id: string }
export type Permissions = ("create_client" | "manage_authorization" | "manage_clients" | "manage_events" | "manage_identity_providers" | "manage_realm" | "manage_users" | "manage_roles" | "query_clients" | "query_groups" | "query_realms" | "query_users" | "view_authorization" | "view_clients" | "view_events" | "view_identity_providers" | "view_realm" | "view_users" | "view_roles" | "manage_webhooks" | "query_webhooks" | "view_webhooks" | "manage_client_scopes" | "query_client_scopes" | "view_client_scopes" | "manage_email_templates" | "view_email_templates" | "manage_organizations" | "view_organizations" | "evaluate_authorization")
export type PolicyResponse = { body: PolicyBodyBody, description?: (string | null), enabled: boolean, id: string, managed_by?: (string | null), name: string, origin: string }
export type PolicyValidator = { body: PolicyBodyBody, description?: (string | null), enabled?: boolean, name: string }
export type PostConsentRequest = { approved_scopes?: Array<string>, consent_token: string }
export type PostConsentResponse = { redirect_url: string }
export type ProviderResponse = { config: unknown, created_at: string, enabled: boolean, id: string, last_sync_at?: (string | null), last_sync_status?: (string | null), name: string, priority: number, provider_type: string, realm_id: string, sync_enabled: boolean, sync_interval_minutes?: (number | null), sync_mode: string, updated_at: string }
/**
 * Incomplete schema. See https://w3c.github.io/webauthn/#dictdef-publickeycredentialjson
 */
export type PublicKeyCredential = Record<string, unknown>
/**
 * Incomplete schema. see https://w3c.github.io/webauthn/#dictdef-publickeycredentialcreationoptionsjson
 */
export type PublicKeyCredentialCreationOptionsJSON = Record<string, unknown>
/**
 * Incomplete schema. see https://w3c.github.io/webauthn/#dictdef-publickeycredentialrequestoptionsjson
 */
export type PublicKeyCredentialRequestOptionsJSON = Record<string, unknown>
/**
 * A trimmed, anonymous-safe view of the realm password policy.
 */
export type PublicPasswordPolicy = { min_length: number, require_lowercase: boolean, require_number: boolean, require_special: boolean, require_uppercase: boolean }
export type RealmLoginSetting = { account_chooser_enabled: boolean, default_locale: string, display_name?: (string | null), email_verification_enabled: boolean, forgot_password_enabled: boolean, identity_providers: Array<IdentityProviderPresentation>, login_aliases: LoginAliases, magic_link_enabled: boolean, magic_link_ttl: number, name: string, passkey_enabled: boolean, remember_me_enabled: boolean, supported_locales: Array<string>, theme: PortalThemeConfig, user_registration_enabled: boolean }
export type RedirectRegistrationResponse = { url: string }
export type RegistrationErrorResponse = { error: string, error_description: string }
export type RegistrationRequest = { email: string, first_name?: (string | null), last_name?: (string | null), password: string, username?: (string | null) }
export type RegistrationResponse = ({ data: JwtToken, status: "authenticated" } | { data: RedirectRegistrationResponse, status: "redirect" } | { data: PendingActionResponse, status: "pending_action" })
export type RemoveClientWhitelistEntryResponse = { message: string }
export type RemoveRealmWhitelistEntryResponse = { message: string }
export type ResourceOwner = { client_id: string, uri: string }
export type ReplaceResourceOwnersValidator = { owners: Array<ResourceOwner> }
export type ReplaceSchemaValidator = { namespace?: (string | null), source: string }
export type RequestElevationValidator = Partial<{ otp_code: (string | null), password: (string | null) }>
export type ResendVerificationEmailResponse = { message: string }
export type ResetPasswordRequest = { new_password: string, token: string, token_id: string }
export type ResetPasswordResponse = { message: string, realm_name: string, user_id: string }
export type ResetPasswordValidator = Partial<{ credential_type: string, temporary: boolean, value: string }>
export type ResourceOwnersResponse = { data: Array<ResourceOwner> }
export type RetryDeliveryResponse = { delivery_id: string, requeued: boolean }
export type RevokeTokenRequestValidator = Partial<{ client_id: (string | null), client_secret: (string | null), token: string, token_type_hint: (string | null) }>
export type RotateSecretResponse = { secret: string }
export type SamlAttributeName = string
export type SamlAttributeNameFormat = ("urn:oasis:names:tc:SAML:2.0:attrname-format:basic" | "urn:oasis:names:tc:SAML:2.0:attrname-format:uri" | "urn:oasis:names:tc:SAML:2.0:attrname-format:unspecified")
export type SamlAttributeSource = string
export type SamlAttributeMapper = { client_id: string, created_at: string, id: string, name: SamlAttributeName, name_format: SamlAttributeNameFormat, source: SamlAttributeSource, updated_at: string }
export type SamlAuthnRequestParams = { RelayState?: (string | null), SAMLRequest: string }
export type SchemaBody = { namespace: string, source: string, updated_at: string }
export type SelectAccountRequest = { session_id: string }
export type SelectAccountStatus = "requires_credentials"
export type SelectAccountRequiresCredentials = { login_hint: string, status: SelectAccountStatus }
export type SelectAccountResponse = (AuthenticateResponse | SelectAccountRequiresCredentials)
export type SendMagicLinkRequest = {
  email: string;
  /**
   * Keep the SSO session past the browser closing once the link is used.
   * Honoured only when the realm enables "remember me".
   */
  remember_me?: boolean;
}
export type SendMagicLinkResponse = { message: string }
export type SetClientSamlConfigValidator = Partial<{ acs_url: string, name_id_format: string, sign_assertions: boolean, sign_documents: boolean, sp_entity_id: string, want_authn_requests_signed: boolean }>
export type SetDefaultPortalLayoutResponse = { data: PortalLayout }
export type SetStateValidator = { enabled: boolean }
export type UserAttribute = { created_at: string, id: string, key: string, realm_id: RealmId, updated_at: string, user_id: string, value: string }
export type SetUserAttributesResponse = { data: Array<UserAttribute> }
export type SetUserAttributesValidator = {
  /**
   * Key-value pairs to upsert. Existing attributes with the same key are updated.
   */
  attributes: Record<string, string>;
}
export type SetupOtpResponse = { issuer: string, otpauth_url: string, secret: string }
export type SimulatedEntity = { id: string, parents?: (Array<EntityRefBody> | null), properties?: (Record<string, unknown> | null), type: string }
export type SimulatedRequest = { action: AuthzenAction, context?: (Record<string, unknown> | null), resource: SimulatedEntity, subject: SimulatedEntity }
export type SimulateValidator = { entities?: Array<SimulatedEntity>, policies?: (Array<CandidateValidator> | null), request: SimulatedRequest }
export type SmtpEncryption = ("tls" | "starttls" | "none")
export type SmtpConfig = { created_at: string, encryption: SmtpEncryption, from_email: string, from_name: string, host: string, id: string, port: number, realm_id: string, updated_at: string, username: string }
export type StartOtpEnrollmentResponse = { otpauth_uri: string, secret: string }
export type StartOwnOtpEnrollmentValidator = { elevation_id: string }
export type SyncUsersResponse = { completed_at?: (string | null), created: number, disabled: number, failed: number, started_at?: (string | null), total_processed: number, updated: number }
export type TemplateResponse = { description?: (string | null), id: string, name: string, source: string }
export type TemplateValidator = { description?: (string | null), name: string, source: string }
export type TestConnectionResponse = { details?: unknown, message: string, success: boolean }
export type ThemeExportLayout = { name: string, tree: Array<Record<string, unknown>> }
/**
 * The envelope of an exported portal theme.
 *
 * A theme is more than a tree: it carries the token config plus one tree per
 * page of the authentication flow, and it points at the layout that frames
 * them. The layout travels *inside* the file rather than as an id — ids are
 * realm-local, so an id would dangle the moment the file crosses into another
 * deployment, which is the whole point of exporting one.
 */
export type ThemeExportEnvelope = {
  /**
   * The theme's design tokens, verbatim.
   */
  config: Record<string, unknown>;
  /**
   * Always `portal-theme`; the kind check reads this.
   */
  ferriskey: string;
  layout?: (ThemeExportLayout | null);
  name: string;
  /**
   * One builder tree per page, keyed by page type.
   */
  pages: Record<string, unknown>;
  version: number;
}
export type ToggleMaintenanceResponse = { message: string }
export type ToggleMaintenanceValidator = { enabled: boolean, reason?: (string | null), session_strategy?: (MaintenanceSessionStrategy | null) }
/**
 * Token type identifiers defined by RFC 8693 §3, serialized as their URNs.
 *
 * All three URNs parse, but only `access_token` is handled for now (see
 * #1050): [`TokenType::ensure_supported`] rejects the others with
 * `unsupported_token_type`.
 */
export type TokenType = ("urn:ietf:params:oauth:token-type:access_token" | "urn:ietf:params:oauth:token-type:id_token" | "urn:ietf:params:oauth:token-type:jwt")
/**
 * Output of a successful token exchange (RFC 8693 §2.2.1).
 */
export type TokenExchangeOutput = {
  access_token: string;
  /**
   * Issued token lifetime, in seconds.
   */
  expires_in: number;
  issued_token_type: TokenType;
  /**
   * Scope of the issued token; omitted when identical to the subject
   * token's scope.
   */
  scope?: (string | null);
  /**
   * `Bearer`, per RFC 8693 §2.2.1 for access tokens.
   */
  token_type: string;
}
export type TokenExchangePolicy = { allow_delegation: boolean, allow_impersonation: boolean, allowed_scopes?: (Array<string> | null), client_id: string, created_at: string, id: string, realm_id: RealmId, target_audience: string, updated_at: string }
export type TokenIntrospectionResponse = { active: boolean, aud?: (string | null), client_id?: (string | null), exp?: (number | null), iat?: (number | null), iss?: (string | null), jti?: (string | null), nbf?: (number | null), realm?: (string | null), scope?: (string | null), sub?: (string | null), token_type?: (string | null), username?: (string | null) }
export type TokenRequestValidator = Partial<{ actor_token: (string | null), actor_token_type: (string | null), audience: (string | null), client_id: (string | null), client_secret: (string | null), code: (string | null), code_verifier: (string | null), device_code: (string | null), grant_type: GrantType, password: (string | null), redirect_uri: (string | null), refresh_token: (string | null), requested_token_type: (string | null), resource: (string | null), scope: (string | null), subject_token: (string | null), subject_token_type: (string | null), username: (string | null) }>
/**
 * The two success bodies of the token endpoint, for the OpenAPI document
 * only: every grant answers with a `JwtToken`, except token exchange, which
 * answers with the RFC 8693 §2.2.1 body.
 */
export type TokenResponse = (JwtToken | TokenExchangeOutput)
/**
 * The two 401 bodies of the token endpoint, for the OpenAPI document only:
 * token exchange answers every 401 with the RFC 6749 §5.2 body
 * (`invalid_client`), the other grants with the FerrisKey error body.
 */
export type TokenUnauthorizedResponse = (ApiErrorResponse | OAuth2ErrorResponse)
export type UnassignRoleResponse = { message: string, realm_name: string, user_id: string }
export type UpdateClientResponse = { data: Client }
export type UpdateClientScopeValidator = Partial<{ description: (string | null), dynamic_registration_allowed: (boolean | null), is_default: (boolean | null), name: (string | null), protocol: (string | null) }>
export type UpdateClientValidator = Partial<{
  access_token_lifetime: (number | null);
  backchannel_logout_session_required: (boolean | null);
  /**
   * OIDC Back-Channel Logout endpoint of the client. Absent keeps the
   * current value, `null` clears it.
   */
  backchannel_logout_uri: (string | null);
  client_id: (string | null);
  consent_required: (boolean | null);
  direct_access_grants_enabled: (boolean | null);
  enabled: (boolean | null);
  id_token_lifetime: (number | null);
  name: (string | null);
  oauth_device_code_grant_enabled: (boolean | null);
  refresh_token_lifetime: (number | null);
  require_pkce: (boolean | null);
  temporary_token_lifetime: (number | null);
  token_exchange_enabled: (boolean | null);
}>
export type UpdateEmailTemplateResponse = { data: EmailTemplate }
export type UpdateEmailTemplateValidator = { name: string, structure: unknown }
export type UpdateGroupValidator = Partial<{
  description: (string | null);
  name: (string | null);
  /**
   * When present, move the group under this parent.
   */
  parent_group_id: (string | null);
}>
export type UpdateIdentityProviderResponse = { data: IdentityProviderResponse }
export type UpdateIdentityProviderValidator = Partial<{ add_read_token_role_on_create: (boolean | null), authorization_url: (string | null), client_id: (string | null), client_secret: (string | null), config: unknown, display_name: (string | null), enabled: (boolean | null), first_broker_login_flow_alias: (string | null), link_only: (boolean | null), post_broker_login_flow_alias: (string | null), scopes: (string | null), store_token: (boolean | null), token_url: (string | null), trust_email: (boolean | null), use_pkce: (boolean | null), userinfo_url: (string | null) }>
export type UpdateOrganizationValidator = Partial<{ alias: (string | null), description: (string | null), domain: (string | null), enabled: (boolean | null), name: (string | null), redirect_url: (string | null) }>
export type UpdateOwnLocaleResponse = { data: User }
export type UpdateOwnLocaleValidator = Partial<{ locale: (string | null) }>
export type UpdateOwnProfileResponse = { data: User }
export type UpdateOwnProfileValidator = Partial<{ email: (string | null), firstname: (string | null), lastname: (string | null), username: (string | null) }>
export type UpdatePasswordPolicyValidator = Partial<{ check_breached: (boolean | null), forbid_common: (boolean | null), max_age_days: (number | null), min_entropy_bits: (number | null), min_length: (number | null), require_lowercase: (boolean | null), require_number: (boolean | null), require_special: (boolean | null), require_uppercase: (boolean | null) }>
export type UpdatePasswordRequest = Partial<{ value: string }>
export type UpdatePasswordResponse = { message: string, required_actions?: Array<RequiredAction>, url?: (string | null) }
export type UpdatePortalLayoutResponse = { data: PortalLayout }
export type UpdatePortalLayoutValidator = { name: string, tree: unknown }
export type UpdatePostLogoutRedirectUriResponse = { data: RedirectUri }
export type UpdateProtocolMapperValidator = Partial<{ config: unknown, mapper_type: (string | null), name: (string | null) }>
export type UpdateProviderRequest = Partial<{ config: unknown, enabled: (boolean | null), name: (string | null), priority: (number | null), provider_type: (string | null), sync_enabled: (boolean | null), sync_interval_minutes: (number | null), sync_mode: (string | null) }>
export type UpdateProviderResponse = { data: ProviderResponse }
export type UpdateRealmResponse = { data: Realm }
export type UpdateRealmSettingResponse = { data: Realm }
export type UpdateRealmSettingValidator = Partial<{ access_token_lifetime: (number | null), account_chooser_enabled: (boolean | null), account_chooser_show_email: (boolean | null), allowed_resources: (Array<string> | null), cimd_allowed_hosts: (Array<string> | null), cimd_enabled: (boolean | null), compass_enabled: (boolean | null), dcr_enabled: (boolean | null), default_locale: (string | null), default_signing_algorithm: (string | null), edit_username_enabled: (boolean | null), email_verification_enabled: (boolean | null), email_verification_template_id: (string | null), email_verification_ttl_hours: (number | null), forgot_password_enabled: (boolean | null), id_token_lifetime: (number | null), lockout_duration_seconds: (number | null), lockout_threshold: (number | null), login_aliases: (Array<string> | null), magic_link_enabled: (boolean | null), magic_link_template_id: (string | null), magic_link_ttl: (number | null), passkey_enabled: (boolean | null), refresh_token_lifetime: (number | null), remember_me_enabled: (boolean | null), require_mfa: (boolean | null), reset_password_template_id: (string | null), seawatch_pii_mode: (string | null), seawatch_pseudo_key: (string | null), supported_locales: (Array<string> | null), temporary_token_lifetime: (number | null), user_registration_enabled: (boolean | null), webhook_retry_base_delay_ms: (number | null), webhook_retry_max_attempts: (number | null), webhook_retry_max_delay_ms: (number | null), webhook_retry_max_total_delay_ms: (number | null) }>
export type UpdateRealmValidator = { display_name?: (string | null), name: string }
export type UpdateRedirectUriResponse = { data: RedirectUri }
export type UpdateRedirectUriValidator = Partial<{ enabled: boolean }>
export type UpdateRolePermissionsResponse = { data: Role }
export type UpdateRolePermissionsValidator = { permissions: Array<string> }
export type UpdateRoleResponse = { data: Role }
export type UpdateRoleValidator = Partial<{ description: (string | null), name: (string | null), require_mfa: (boolean | null) }>
export type UpdateThemeMetadataResponse = { data: PortalTheme }
export type UpdateThemeMetadataValidator = { config?: PortalThemeConfig, layout_id?: (string | null), name: string }
export type UpdateThemePageResponse = { data: PortalTheme }
export type UpdateThemePageValidator = { tree: unknown }
export type UpdateThemeResponse = { data: PortalTheme }
export type UpdateThemeValidator = { config: PortalThemeConfig }
export type UpdateUserResponse = { data: User }
export type UpdateUserValidator = Partial<{ email: (string | null), email_verified: (boolean | null), enabled: (boolean | null), firstname: (string | null), lastname: (string | null), required_actions: (Array<string> | null) }>
export type UpdateWebhookResponse = { data: Webhook }
export type UpdateWebhookValidator = Partial<{
  description: (string | null);
  endpoint: string;
  /**
   * Omitted keeps the stored headers: the API never returns them, so a
   * client editing another field has no way to send them back and would
   * otherwise wipe them.
   */
  headers: (Record<string, string> | null);
  name: (string | null);
  retry_policy: (RetryPolicyOverride | null);
  subscribers: Array<WebhookTrigger>;
}>
export type UpsertAttributeValidator = { value: string }
export type UpsertSmtpConfigValidator = { encryption: string, from_email: string, from_name: string, host: string, password: string, port: number, username: string }
export type UserAttributesResponse = { data: Array<UserAttribute> }
export type UserInfoResponse = { email?: (string | null), email_verified?: (boolean | null), family_name?: (string | null), given_name?: (string | null), name?: (string | null), preferred_username?: (string | null), sub: string }
export type UserPermissionsResponse = { data: Array<Permissions> }
export type UserResponse = { data: User }
export type ValidatePoliciesValidator = { policies: Array<CandidateValidator> }
export type ValidatePublicKeyResponse = Partial<{ required_actions: Array<RequiredAction>, url: (string | null) }>
export type ValidationErrorResponse = { errors: Array<ValidationError> }
export type ValidationResponse = { issues: Array<IssueBody>, valid: boolean }
export type VerifyEmailRequest = {
  /**
   * Email verification token
   */
  token: string;
}
export type VerifyEmailResult = { user_id: string, verified: boolean }
export type VerifyOtpResponse = { message: string, required_actions?: Array<RequiredAction>, url?: (string | null) }
export type VerifyResetTokenRequest = { token_id: string }
export type VerifyResetTokenResponse = { valid: boolean }
export type WebOriginValue = string
export type WebOrigin = { client_id: string, created_at: string, id: string, updated_at: string, value: WebOriginValue }

    // </Schemas>
    }

  export namespace Endpoints {
  // <Endpoints>

  /**
 * AuthZEN 1.0 policy decision point metadata for a realm. The policy decision point identifier is {root}/realms/{realm}/authzen, so, as the specification requires for an identifier with a path, the well-known segment sits between the host and that path: /.well-known/authzen-configuration{root}/realms/{realm}/authzen.
 */
export type get_Get_authzen_configuration = {
      method: "GET",
      path: "/.well-known/authzen-configuration/realms/{realm_name}/authzen",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.AuthzenConfiguration,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns the available variables for a given email type.
 */
export type get_Get_variables = {
      method: "GET",
      path: "/email-templates/variables/{email_type}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { email_type: string },



          }
      responses: {200: Schemas.GetTemplateVariablesResponse,
400: Schemas.ApiErrorResponse,
},

    }
export type post_Create_realm = {
      method: "POST",
      path: "/realms",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {




        body:  Schemas.CreateRealmValidator,
          }
      responses: {201: Schemas.Realm,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves a realm by its name. This endpoint returns the details of the specified realm.
 */
export type get_Get_realm = {
      method: "GET",
      path: "/realms/{name}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { name: string },



          }
      responses: {200: Schemas.Realm,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Updates the name of an existing realm. This endpoint allows you to change the name of a realm while keeping its associated data intact.
 */
export type put_Update_realm = {
      method: "PUT",
      path: "/realms/{name}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { name: string },


        body:  Schemas.UpdateRealmValidator,
          }
      responses: {200: Schemas.UpdateRealmResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Deletes a realm by its name. This action is irreversible and will remove all associated data.
 */
export type delete_Delete_realm = {
      method: "DELETE",
      path: "/realms/{name}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { name: string },



          }
      responses: {200: Schemas.DeleteRealmResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Get the login settings for a specific realm.
 */
export type get_Get_login_realm_settings_handler = {
      method: "GET",
      path: "/realms/{name}/login-settings",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { name: string },



          }
      responses: {200: Schemas.RealmLoginSetting,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Updates the settings for a specified realm. This endpoint allows modification of various realm settings.
 */
export type put_Update_realm_setting = {
      method: "PUT",
      path: "/realms/{name}/settings",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { name: string },


        body:  Schemas.UpdateRealmSettingValidator,
          }
      responses: {200: Schemas.UpdateRealmSettingResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
422: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * RFC 8414 authorization server metadata of a realm, in the path-insertion form. `registration_endpoint` is present only when dynamic client registration is enabled on the realm, `client_id_metadata_document_supported` only when client ID metadata documents are enabled. `scopes_supported` lists the realm's client scopes.
 */
export type get_Get_authorization_server_metadata = {
      method: "GET",
      path: "/realms/{realm_name}/.well-known/oauth-authorization-server/realms/{realm_name}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.AuthorizationServerMetadata,
401: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves the OpenID Connect configuration for a specific realm. This endpoint provides metadata about the OpenID Connect provider, including endpoints for authorization, token issuance, introspection, user information, and JWKs.
 */
export type get_Get_openid_configuration = {
      method: "GET",
      path: "/realms/{realm_name}/.well-known/openid-configuration",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.GetOpenIdConfigurationResponse,
},

    }
export type get_Get_consent = {
      method: "GET",
      path: "/realms/{realm_name}/auth/consent",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query:  { consent_token: string },
        path:  { realm_name: string },



          }
      responses: {200: Schemas.GetConsentResponse,
404: Schemas.ApiErrorResponse,
},

    }
export type post_Post_consent = {
      method: "POST",
      path: "/realms/{realm_name}/auth/consent",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.PostConsentRequest,
          }
      responses: {200: Schemas.PostConsentResponse,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * With managed_by, only the policies carrying that label. Requires view_authorization.
 */
export type get_List_policies = {
      method: "GET",
      path: "/realms/{realm_name}/authz/policies",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ managed_by: string }>,
        path:  { realm_name: string },



          }
      responses: {200: Array<Schemas.PolicyResponse>,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * The policy is validated with every other enabled policy against the realm's schema. With managed_by, the policy is labelled with that manager. Requires manage_authorization.
 */
export type post_Create_policy = {
      method: "POST",
      path: "/realms/{realm_name}/authz/policies",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ managed_by: string }>,
        path:  { realm_name: string },


        body:  Schemas.PolicyValidator,
          }
      responses: {201: Schemas.PolicyResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
422: Schemas.ApiErrorResponse,
},

    }
/**
 * Parses the Cedar and validates it against the realm's schema. Problems come back as diagnostics with line and column, in a 200; nothing is stored. Requires view_authorization.
 */
export type post_Validate_policies = {
      method: "POST",
      path: "/realms/{realm_name}/authz/policies/validate",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.ValidatePoliciesValidator,
          }
      responses: {200: Schemas.ValidationResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * With managed_by, a policy of another manager or without a label is not found. Requires view_authorization.
 */
export type get_Get_policy = {
      method: "GET",
      path: "/realms/{realm_name}/authz/policies/{policy_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ managed_by: string }>,
        path:  { realm_name: string, policy_id: string },



          }
      responses: {200: Schemas.PolicyResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * With managed_by, only a policy already labelled with that manager can be replaced; without it the existing label is kept. Requires manage_authorization.
 */
export type put_Update_policy = {
      method: "PUT",
      path: "/realms/{realm_name}/authz/policies/{policy_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ managed_by: string }>,
        path:  { realm_name: string, policy_id: string },


        body:  Schemas.PolicyValidator,
          }
      responses: {200: Schemas.PolicyResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
422: Schemas.ApiErrorResponse,
},

    }
/**
 * With managed_by, only a policy labelled with that manager can be deleted. Requires manage_authorization.
 */
export type delete_Delete_policy = {
      method: "DELETE",
      path: "/realms/{realm_name}/authz/policies/{policy_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ managed_by: string }>,
        path:  { realm_name: string, policy_id: string },



          }
      responses: {204: unknown,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
},

    }
/**
 * The realm's own schema, without the built-in FerrisKey types. Requires view_authorization.
 */
export type get_Get_schema = {
      method: "GET",
      path: "/realms/{realm_name}/authz/schema",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.SchemaBody,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * Refused when an enabled policy or template no longer validates against the new schema. Requires manage_authorization.
 */
export type put_Put_schema = {
      method: "PUT",
      path: "/realms/{realm_name}/authz/schema",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.ReplaceSchemaValidator,
          }
      responses: {200: Schemas.SchemaBody,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
422: Schemas.ApiErrorResponse,
},

    }
/**
 * The FerrisKey namespace every realm schema is merged with: users, groups, roles, organizations, and the agent types and actions (call_tool, read_resource, get_prompt). Validate policies client-side against this plus the realm schema. Requires view_authorization.
 */
export type get_Get_builtin_schema = {
      method: "GET",
      path: "/realms/{realm_name}/authz/schema/builtin",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.BuiltinSchemaBody,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * Dry run. The policies in the body, or the realm's enabled ones when absent, are validated and then decide the request. FerrisKey subjects are not looked up: describe them, and their groups and roles as parents, in entities. Requires view_authorization.
 */
export type post_Simulate = {
      method: "POST",
      path: "/realms/{realm_name}/authz/simulate",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.SimulateValidator,
          }
      responses: {200: Schemas.EvaluationResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
422: Schemas.ApiErrorResponse,
},

    }
/**
 * Whether the realm answers authorization questions, and how many times its rules changed. Requires view_authorization.
 */
export type get_Get_state = {
      method: "GET",
      path: "/realms/{realm_name}/authz/state",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.AuthzStateBody,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * Requires manage_authorization.
 */
export type put_Set_state = {
      method: "PUT",
      path: "/realms/{realm_name}/authz/state",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.SetStateValidator,
          }
      responses: {200: Schemas.AuthzStateBody,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * Requires view_authorization.
 */
export type get_List_templates = {
      method: "GET",
      path: "/realms/{realm_name}/authz/templates",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Array<Schemas.TemplateResponse>,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * Requires manage_authorization.
 */
export type post_Create_policy_template = {
      method: "POST",
      path: "/realms/{realm_name}/authz/templates",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.TemplateValidator,
          }
      responses: {201: Schemas.TemplateResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
422: Schemas.ApiErrorResponse,
},

    }
/**
 * Requires view_authorization.
 */
export type get_Get_policy_template = {
      method: "GET",
      path: "/realms/{realm_name}/authz/templates/{template_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, template_id: string },



          }
      responses: {200: Schemas.TemplateResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * Policies linked to the template are validated again. Requires manage_authorization.
 */
export type put_Update_policy_template = {
      method: "PUT",
      path: "/realms/{realm_name}/authz/templates/{template_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, template_id: string },


        body:  Schemas.TemplateValidator,
          }
      responses: {200: Schemas.TemplateResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
422: Schemas.ApiErrorResponse,
},

    }
/**
 * Refused while a policy links it. Requires manage_authorization.
 */
export type delete_Delete_policy_template = {
      method: "DELETE",
      path: "/realms/{realm_name}/authz/templates/{template_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, template_id: string },



          }
      responses: {204: unknown,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
},

    }
/**
 * AuthZEN 1.0 access evaluation. Called by a policy enforcement point authenticated as a service account holding evaluate_authorization. A refusal is a 200 with decision false; any other status carries no decision.
 */
export type post_Evaluate = {
      method: "POST",
      path: "/realms/{realm_name}/authzen/access/v1/evaluation",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.EvaluationValidator,
          }
      responses: {200: Schemas.EvaluationResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * AuthZEN 1.0 boxcarring. Top-level subject, action, resource and context are defaults each item may override. options.evaluations_semantic is execute_all (default), deny_on_first_deny or permit_on_first_permit; a short-circuited response holds the items evaluated so far, in request order. An item the realm's schema rejects is a deny whose context.error says why. Without evaluations the request is a single evaluation. At most 100 items. Called by a service account holding evaluate_authorization.
 */
export type post_Evaluations = {
      method: "POST",
      path: "/realms/{realm_name}/authzen/access/v1/evaluations",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.EvaluationsValidator,
          }
      responses: {200: Schemas.EvaluationsResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Processes the IdP callback, exchanges code for tokens, and redirects to client
 */
export type post_Broker_callback = {
      method: "POST",
      path: "/realms/{realm_name}/broker/{alias}/endpoint",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query:  { code?: string, state: string, error?: string, error_description?: string },
        path:  { realm_name: string, alias: string },



          }
      responses: {302: unknown,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
502: Schemas.ApiErrorResponse,
},

    }
/**
 * Redirects the user to the external identity provider for authentication
 */
export type get_Broker_login = {
      method: "GET",
      path: "/realms/{realm_name}/broker/{alias}/login",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ client_id: string, redirect_uri: string, response_type: string, scope: string, state: string, nonce: string, code_challenge: string, code_challenge_method: string }>,
        path:  { realm_name: string, alias: string },



          }
      responses: {302: unknown,
400: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the realm's client scopes, each with its protocol mappers. Text filters (name, description) match case-insensitively anywhere in the value; search matches the name or the description; protocol and default_scope_type match exactly; has_protocol_mappers keeps scopes with at least one protocol mapper (true) or none (false); not_assigned_to_client takes a client id and keeps the scopes not yet assigned to that client. created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND.
 */
export type get_Get_client_scopes = {
      method: "GET",
      path: "/realms/{realm_name}/client-scopes",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), name: string, description: string, search: string, protocol: string, default_scope_type: ("NONE" | "OPTIONAL" | "DEFAULT"), has_protocol_mappers: boolean, not_assigned_to_client: string, created_from: string, created_to: string, order_by: ("name" | "created_at" | "updated_at") }>,
        path:  { realm_name: string },



          }
      responses: {200: Schemas.Paginated_ClientScope,
400: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Creates a new client scope within the specified realm.
 */
export type post_Create_client_scope = {
      method: "POST",
      path: "/realms/{realm_name}/client-scopes",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.CreateClientScopeValidator,
          }
      responses: {201: Schemas.ClientScope,
400: unknown,
401: unknown,
403: unknown,
},

    }
/**
 * Retrieves a client scope from the specified realm by its ID.
 */
export type get_Get_client_scope = {
      method: "GET",
      path: "/realms/{realm_name}/client-scopes/{scope_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, scope_id: string },



          }
      responses: {200: Schemas.ClientScope,
},

    }
/**
 * Deletes a client scope from the specified realm. This action is irreversible.
 */
export type delete_Delete_client_scope = {
      method: "DELETE",
      path: "/realms/{realm_name}/client-scopes/{scope_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, scope_id: string },



          }
      responses: {200: Schemas.DeleteClientScopeResponse,
},

    }
/**
 * Updates an existing client scope in the specified realm.
 */
export type patch_Update_client_scope = {
      method: "PATCH",
      path: "/realms/{realm_name}/client-scopes/{scope_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, scope_id: string },


        body:  Schemas.UpdateClientScopeValidator,
          }
      responses: {200: Schemas.ClientScope,
},

    }
/**
 * Creates a new protocol mapper for the specified client scope.
 */
export type post_Create_protocol_mapper = {
      method: "POST",
      path: "/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, scope_id: string },


        body:  Schemas.CreateProtocolMapperValidator,
          }
      responses: {201: Schemas.ProtocolMapper,
400: unknown,
401: unknown,
403: unknown,
},

    }
/**
 * Deletes a protocol mapper from the specified client scope.
 */
export type delete_Delete_protocol_mapper = {
      method: "DELETE",
      path: "/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, scope_id: string, mapper_id: string },



          }
      responses: {200: Schemas.DeleteProtocolMapperResponse,
},

    }
/**
 * Updates an existing protocol mapper for the specified client scope.
 */
export type patch_Update_protocol_mapper = {
      method: "PATCH",
      path: "/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, scope_id: string, mapper_id: string },


        body:  Schemas.UpdateProtocolMapperValidator,
          }
      responses: {200: Schemas.ProtocolMapper,
},

    }
/**
 * Returns one page of the realm's clients. search matches case-insensitively anywhere in the name or the client_id; the text filters name and client_id match case-insensitively anywhere in that value; client_id_exact keeps the client whose client_id equals the value exactly, case-sensitive; enabled, public_client, service_account_enabled, protocol and client_type match exactly; oauth_device_code_grant_enabled treats an unset value as false; application_type keeps m2m clients (service account), device clients (device grant and no redirect URI), spa and native clients (public client type, with or without public_client) and web clients (any other client type); has_redirect_uris keeps clients with at least one redirect URI (true) or none (false), disabled redirect URIs included; maintenance_enabled keeps clients in maintenance (true) or not (false); ids takes a comma-separated list of at most 100 client ids. created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND.
 */
export type get_Get_clients = {
      method: "GET",
      path: "/realms/{realm_name}/clients",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, name: string, client_id: string, client_id_exact: string, enabled: boolean, public_client: boolean, service_account_enabled: boolean, oauth_device_code_grant_enabled: boolean, protocol: ("openid-connect" | "saml"), client_type: ("confidential" | "public" | "system"), application_type: ("m2m" | "device" | "spa" | "native" | "web"), has_redirect_uris: boolean, maintenance_enabled: boolean, ids: string, created_from: string, created_to: string, order_by: ("name" | "client_id" | "enabled" | "created_at" | "updated_at") }>,
        path:  { realm_name: string },



          }
      responses: {200: Schemas.Paginated_Client,
400: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Creates a new client within the specified realm. This endpoint allows you to register a new client application that can interact with the realm's resources.
 */
export type post_Create_client = {
      method: "POST",
      path: "/realms/{realm_name}/clients",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.CreateClientValidator,
          }
      responses: {201: Schemas.Client,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
},

    }
/**
 * Anonymous dynamic client registration. Available only when the realm enables it, and rate limited per IP address.
 */
export type post_Register_client = {
      method: "POST",
      path: "/realms/{realm_name}/clients/register",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.ClientRegistrationRequest,
          }
      responses: {201: Schemas.ClientRegistrationResponse,
400: Schemas.RegistrationErrorResponse,
404: Schemas.RegistrationErrorResponse,
429: Schemas.RegistrationErrorResponse,
},

    }
/**
 * Retrieves a client from the specified realm by its ID.
 */
export type get_Get_client = {
      method: "GET",
      path: "/realms/{realm_name}/clients/{client_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },



          }
      responses: {200: Schemas.GetClientResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Deletes a client from the specified realm. This action is irreversible and will remove all associated data.
 */
export type delete_Delete_client = {
      method: "DELETE",
      path: "/realms/{realm_name}/clients/{client_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },



          }
      responses: {200: Schemas.DeleteClientResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Updates an existing client in the specified realm. This endpoint allows you to modify client details such as name, client ID, and enabled status.
 */
export type patch_Update_client = {
      method: "PATCH",
      path: "/realms/{realm_name}/clients/{client_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },


        body:  Schemas.UpdateClientValidator,
          }
      responses: {200: Schemas.UpdateClientResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns all client scopes (default and optional) assigned to the specified client.
 */
export type get_Get_client_client_scopes = {
      method: "GET",
      path: "/realms/{realm_name}/clients/{client_id}/client-scopes",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },



          }
      responses: {200: Array<Schemas.ClientScope>,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns the plaintext secret of a confidential client. Requires ManageClients (or ManageRealm) — strictly more than the ViewClients needed to read the client itself — and records a `client_secret_viewed` security event.
 */
export type get_Get_client_secret = {
      method: "GET",
      path: "/realms/{realm_name}/clients/{client_id}/client-secret",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },



          }
      responses: {200: Schemas.ClientSecretResponse,
403: unknown,
404: unknown,
},

    }
/**
 * Assigns a client scope as a default scope to the specified client. Default scopes are always included in tokens.
 */
export type put_Assign_default_scope = {
      method: "PUT",
      path: "/realms/{realm_name}/clients/{client_id}/default-client-scopes/{scope_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string, scope_id: string },



          }
      responses: {200: Schemas.ClientScopeMapping,
},

    }
/**
 * Removes a client scope from the default scopes of the specified client.
 */
export type delete_Unassign_default_scope = {
      method: "DELETE",
      path: "/realms/{realm_name}/clients/{client_id}/default-client-scopes/{scope_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string, scope_id: string },



          }
      responses: {200: unknown,
},

    }
/**
 * Previews the effective protocol mappers, roles and token claims (access token, ID token, userinfo) a user would receive from this client for a given scope set — without issuing (signing or persisting) a real token.
 */
export type post_Evaluate_client_scopes = {
      method: "POST",
      path: "/realms/{realm_name}/clients/{client_id}/evaluate-scopes",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },


        body:  Schemas.EvaluateScopesValidator,
          }
      responses: {200: Schemas.EvaluateClientScopesResult,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Enables or disables maintenance mode for a client. When enabled, only whitelisted users and roles can authenticate.
 */
export type put_Toggle_maintenance = {
      method: "PUT",
      path: "/realms/{realm_name}/clients/{client_id}/maintenance",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },


        body:  Schemas.ToggleMaintenanceValidator,
          }
      responses: {200: Schemas.ToggleMaintenanceResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns the list of users and roles whitelisted for this client during maintenance mode.
 */
export type get_Get_client_whitelist = {
      method: "GET",
      path: "/realms/{realm_name}/clients/{client_id}/maintenance/whitelist",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },



          }
      responses: {200: Schemas.GetClientWhitelistResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
},

    }
/**
 * Adds a user or role to the client's maintenance whitelist. Exactly one of user_id or role_id must be provided.
 */
export type post_Add_client_whitelist_entry = {
      method: "POST",
      path: "/realms/{realm_name}/clients/{client_id}/maintenance/whitelist",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },


        body:  Schemas.AddWhitelistEntryValidator,
          }
      responses: {201: Schemas.AddClientWhitelistEntryResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
},

    }
/**
 * Removes a user or role from the client's maintenance whitelist.
 */
export type delete_Remove_client_whitelist_entry = {
      method: "DELETE",
      path: "/realms/{realm_name}/clients/{client_id}/maintenance/whitelist/{entry_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string, entry_id: string },



          }
      responses: {200: Schemas.RemoveClientWhitelistEntryResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * Assigns a client scope as an optional scope to the specified client. Optional scopes are included only when explicitly requested.
 */
export type put_Assign_optional_scope = {
      method: "PUT",
      path: "/realms/{realm_name}/clients/{client_id}/optional-client-scopes/{scope_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string, scope_id: string },



          }
      responses: {200: Schemas.ClientScopeMapping,
},

    }
/**
 * Removes a client scope from the optional scopes of the specified client.
 */
export type delete_Unassign_optional_scope = {
      method: "DELETE",
      path: "/realms/{realm_name}/clients/{client_id}/optional-client-scopes/{scope_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string, scope_id: string },



          }
      responses: {200: unknown,
},

    }
/**
 * Retrieves all post-logout redirect URIs associated with a client in a specific realm.
 */
export type get_Get_post_logout_redirect_uris = {
      method: "GET",
      path: "/realms/{realm_name}/clients/{client_id}/post-logout-redirects",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },



          }
      responses: {200: Array<Schemas.RedirectUri>,
},

    }
/**
 * Creates a new post-logout redirect URI for the specified client.
 */
export type post_Create_post_logout_redirect_uri = {
      method: "POST",
      path: "/realms/{realm_name}/clients/{client_id}/post-logout-redirects",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },


        body:  Schemas.CreateRedirectUriValidator,
          }
      responses: {201: Schemas.RedirectUri,
},

    }
/**
 * Updates an existing post-logout redirect URI for a client in a specific realm.
 */
export type put_Update_post_logout_redirect_uri = {
      method: "PUT",
      path: "/realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string, uri_id: string },


        body:  Schemas.UpdateRedirectUriValidator,
          }
      responses: {200: Schemas.UpdatePostLogoutRedirectUriResponse,
},

    }
/**
 * Deletes a specific post-logout redirect URI for a client in a realm.
 */
export type delete_Delete_post_logout_redirect_uri = {
      method: "DELETE",
      path: "/realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string, uri_id: string },



          }
      responses: {200: unknown,
},

    }
/**
 * Retrieves all redirect URIs associated with a client in a specific realm. This endpoint is useful for OAuth2 or OpenID Connect flows where clients need to know their registered redirect URIs.
 */
export type get_Get_redirect_uris = {
      method: "GET",
      path: "/realms/{realm_name}/clients/{client_id}/redirects",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },



          }
      responses: {200: Array<Schemas.RedirectUri>,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Creates a new redirect URI for the specified client. This endpoint allows you to add a redirect URI that the client can use for OAuth2 or OpenID Connect flows.
 */
export type post_Create_redirect_uri = {
      method: "POST",
      path: "/realms/{realm_name}/clients/{client_id}/redirects",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },


        body:  Schemas.CreateRedirectUriValidator,
          }
      responses: {201: Schemas.RedirectUri,
},

    }
/**
 * Updates an existing redirect URI for a client in a specific realm. This endpoint allows you to modify the enabled status of a redirect URI.
 */
export type put_Update_redirect_uri = {
      method: "PUT",
      path: "/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string, uri_id: string },


        body:  Schemas.UpdateRedirectUriValidator,
          }
      responses: {200: Schemas.UpdateRedirectUriResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Deletes a specific redirect URI for a client in a realm. This action is irreversible and will remove the redirect URI from the client's configuration.
 */
export type delete_Delete_redirect_uri = {
      method: "DELETE",
      path: "/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string, uri_id: string },



          }
      responses: {200: unknown,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves all roles associated with a client in a specific realm.
 */
export type get_Get_client_roles = {
      method: "GET",
      path: "/realms/{realm_name}/clients/{client_id}/roles",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },



          }
      responses: {200: Schemas.GetClientRolesResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Creates a new role for a specific client within a realm. This endpoint allows you to define roles that can be assigned to users or groups in the context of a client application.
 */
export type post_Create_client_role = {
      method: "POST",
      path: "/realms/{realm_name}/clients/{client_id}/roles",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },


        body:  Schemas.CreateRoleValidator,
          }
      responses: {201: Schemas.Role,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns every `<saml:Attribute>` the assertion carries for this client, ordered by attribute name.
 */
export type get_Get_saml_attribute_mappers = {
      method: "GET",
      path: "/realms/{realm_name}/clients/{client_id}/saml-attribute-mappers",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },



          }
      responses: {200: Array<Schemas.SamlAttributeMapper>,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Declares one `<saml:Attribute>` the assertion carries for this client. `name` is the attribute name the service provider reads — Chatwoot reads `email`, `first_name` and `last_name`. `source` names the value it is fed from: one of `user:id`, `user:username`, `user:email`, `user:first_name`, `user:last_name`, or `attribute:<key>` for a custom user attribute. A name may be mapped once per client.
 */
export type post_Create_saml_attribute_mapper = {
      method: "POST",
      path: "/realms/{realm_name}/clients/{client_id}/saml-attribute-mappers",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },


        body:  Schemas.CreateSamlAttributeMapperValidator,
          }
      responses: {201: Schemas.SamlAttributeMapper,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Removes one attribute mapper. The lookup is scoped to the owning client, so a mapper identifier belonging to another client is not found rather than deleted.
 */
export type delete_Delete_saml_attribute_mapper = {
      method: "DELETE",
      path: "/realms/{realm_name}/clients/{client_id}/saml-attribute-mappers/{mapper_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string, mapper_id: string },



          }
      responses: {200: unknown,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns the service provider this client stands for: its EntityID, the assertion consumer service URL assertions are posted to, the NameID format identifying the subject, and the signing options. A client that has never been configured for SAML answers 404.
 */
export type get_Get_client_saml_config = {
      method: "GET",
      path: "/realms/{realm_name}/clients/{client_id}/saml-config",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },



          }
      responses: {200: Schemas.ClientSamlConfig,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Declares the service provider this client stands for. A client holds at most one SAML configuration, so this replaces it wholesale rather than creating a second one. The EntityID must be an absolute URI and is stored byte-for-byte, since SAML compares it verbatim against the issuer of an incoming AuthnRequest. The assertion consumer service URL must be an absolute http or https URL carrying no credentials and no fragment; it may carry a query string, which is how Chatwoot passes its account id.
 */
export type put_Set_client_saml_config = {
      method: "PUT",
      path: "/realms/{realm_name}/clients/{client_id}/saml-config",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },


        body:  Schemas.SetClientSamlConfigValidator,
          }
      responses: {201: Schemas.ClientSamlConfig,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns the RFC 8693 token-exchange delegation policies registered for this client, one per target audience.
 */
export type get_Get_token_exchange_policies = {
      method: "GET",
      path: "/realms/{realm_name}/clients/{client_id}/token-exchange-policies",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },



          }
      responses: {200: Array<Schemas.TokenExchangePolicy>,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Registers an RFC 8693 token-exchange delegation policy. `target_audience` is the `client_id` of another client of the same realm. `allowed_scopes`, when set, caps the scopes the exchanged token may carry; each entry is a single scope token without whitespace.
 */
export type post_Create_token_exchange_policy = {
      method: "POST",
      path: "/realms/{realm_name}/clients/{client_id}/token-exchange-policies",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },


        body:  Schemas.CreateTokenExchangePolicyValidator,
          }
      responses: {201: Schemas.TokenExchangePolicy,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Removes an RFC 8693 token-exchange delegation policy. Tokens already issued through it stay valid until they expire.
 */
export type delete_Delete_token_exchange_policy = {
      method: "DELETE",
      path: "/realms/{realm_name}/clients/{client_id}/token-exchange-policies/{policy_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string, policy_id: string },



          }
      responses: {200: unknown,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns the origins this client is allowed to call FerrisKey from in a browser. A `+` entry expands to the origins of the client's literal redirect URIs; anchored regex redirect URIs are skipped, since an origin cannot be derived from a pattern.
 */
export type get_Get_web_origins = {
      method: "GET",
      path: "/realms/{realm_name}/clients/{client_id}/web-origins",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },



          }
      responses: {200: Array<Schemas.WebOrigin>,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Registers an origin the client is allowed to call FerrisKey from in a browser. The value must be a serialized origin (scheme://host[:port], no path), or the `+` sentinel meaning "derive the origins from this client's literal redirect URIs". Enforcement is per realm: the allowlist a browser is answered with is the union of the origins of every enabled client of the realm.
 */
export type post_Create_web_origin = {
      method: "POST",
      path: "/realms/{realm_name}/clients/{client_id}/web-origins",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string },


        body:  Schemas.CreateWebOriginValidator,
          }
      responses: {201: Schemas.WebOrigin,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Removes a registered origin. Browsers may keep honouring a cached preflight until its max-age lapses, and other API replicas may keep serving the origin for up to one cache TTL.
 */
export type delete_Delete_web_origin = {
      method: "DELETE",
      path: "/realms/{realm_name}/clients/{client_id}/web-origins/{web_origin_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, client_id: string, web_origin_id: string },



          }
      responses: {200: unknown,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type get_Get_daily_activity_stats = {
      method: "GET",
      path: "/realms/{realm_name}/compass/v1/activity/daily",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ from: string, to: string, client_id: string, user_id: string, grant_type: string }>,
        path:  { realm_name: string },



          }
      responses: {200: Schemas.GetDailyActivityStatsResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the realm's authentication flows, with their steps. search matches case-insensitively anywhere in the ip_address and never matches a flow without an address. ip_address matches case-insensitively anywhere in the value and never matches a flow without an address. client_id (the OAuth client_id string), user_id, grant_type and status match exactly. identified=true keeps flows tied to a user, identified=false the anonymous ones; completed=true keeps flows with a completion date, completed=false the unfinished ones. from and to bound started_at, both inclusive. Filters combine with AND. Sorting on duration_ms puts unfinished flows last in ascending order and first in descending order.
 */
export type get_Get_flows = {
      method: "GET",
      path: "/realms/{realm_name}/compass/v1/flows",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, client_id: string, user_id: string, grant_type: string, status: ("pending" | "success" | "failure" | "expired"), ip_address: string, identified: boolean, completed: boolean, from: string, to: string, order_by: ("status" | "started_at" | "duration_ms" | "created_at") }>,
        path:  { realm_name: string },



          }
      responses: {200: Schemas.Paginated_CompassFlow,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type get_Get_flow = {
      method: "GET",
      path: "/realms/{realm_name}/compass/v1/flows/{flow_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, flow_id: string },



          }
      responses: {200: Schemas.GetFlowResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type get_Get_stats = {
      method: "GET",
      path: "/realms/{realm_name}/compass/v1/stats",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.GetStatsResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Entry point the user visits (the `verification_uri`) to approve a device. Redirects to the FerrisKey web app, pre-filling the user code when supplied via `?user_code=`.
 */
export type get_Device_verification_page = {
      method: "GET",
      path: "/realms/{realm_name}/device",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ user_code: string }>,
        path:  { realm_name: string },



          }
      responses: {302: unknown,
},

    }
/**
 * Returns the client and the scopes a pending device session is asking for, so the verification page can show what is being approved (RFC 8628 §5.3). Requires the `FERRISKEY_SSO` session cookie and refuses codes belonging to another realm.
 */
export type get_Device_preview = {
      method: "GET",
      path: "/realms/{realm_name}/device/preview",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query:  { user_code: string },
        path:  { realm_name: string },



          }
      responses: {200: Schemas.DeviceVerificationPreview,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
},

    }
/**
 * Called from the verification page once the user is authenticated. Requires the `FERRISKEY_SSO` session cookie; when absent or no longer valid, responds 401 with a `redirect_uri` hint pointing back to the verification page so the front can route to login first.
 */
export type post_Device_verify = {
      method: "POST",
      path: "/realms/{realm_name}/device/verify",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.DeviceVerifyRequest,
          }
      responses: {200: Schemas.DeviceVerifyResponse,
400: Schemas.ApiErrorResponse,
401: unknown,
403: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the realm's email templates, each with its structure and mjml. search and the name filter match case-insensitively anywhere in the name; email_type matches exactly; created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND.
 */
export type get_Fetch_templates = {
      method: "GET",
      path: "/realms/{realm_name}/email-templates",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, name: string, email_type: ("reset_password" | "magic_link" | "email_verification"), created_from: string, created_to: string, order_by: ("name" | "email_type" | "created_at" | "updated_at") }>,
        path:  { realm_name: string },



          }
      responses: {200: Schemas.Paginated_EmailTemplate,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Creates a new email template for the specified realm.
 */
export type post_Create_template = {
      method: "POST",
      path: "/realms/{realm_name}/email-templates",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.CreateEmailTemplateValidator,
          }
      responses: {201: Schemas.CreateEmailTemplateResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Creates an email template from an exported JSON structure or from raw MJML markup. MJML is parsed back into a builder structure so the imported template stays editable; anything outside <mj-body> (such as <mj-head>) is not representable in the builder and is dropped.
 */
export type post_Import_template = {
      method: "POST",
      path: "/realms/{realm_name}/email-templates/import",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.ImportEmailTemplateValidator,
          }
      responses: {201: Schemas.ImportEmailTemplateResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves a single email template by ID.
 */
export type get_Get_template = {
      method: "GET",
      path: "/realms/{realm_name}/email-templates/{template_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, template_id: string },



          }
      responses: {200: Schemas.GetEmailTemplateResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Updates an existing email template.
 */
export type put_Update_template = {
      method: "PUT",
      path: "/realms/{realm_name}/email-templates/{template_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, template_id: string },


        body:  Schemas.UpdateEmailTemplateValidator,
          }
      responses: {200: Schemas.UpdateEmailTemplateResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Deletes an email template.
 */
export type delete_Delete_template = {
      method: "DELETE",
      path: "/realms/{realm_name}/email-templates/{template_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, template_id: string },



          }
      responses: {200: Schemas.DeleteEmailTemplateResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns the template as a downloadable file: the builder JSON envelope (`format=json`, the default) or the rendered MJML markup (`format=mjml`).
 */
export type get_Export_template = {
      method: "GET",
      path: "/realms/{realm_name}/email-templates/{template_id}/export",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ format: Schemas.EmailTemplateExportFormat }>,
        path:  { realm_name: string, template_id: string },



          }
      responses: {200: (Schemas.ExportEnvelope | string),
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the realm's user federation providers. search and the name filter match case-insensitively anywhere in the name; provider_type is an exact, case-sensitive match on the stored type (Ldap, ActiveDirectory, Kerberos), so an unknown or custom value returns an empty page; enabled and sync_enabled match exactly; provider_family=ldap keeps Ldap and ActiveDirectory providers, provider_family=kerberos keeps Kerberos providers; synced=true keeps providers that have synchronized at least once, synced=false keeps the ones that never did; sync_mode is an exact, case-sensitive match on the stored synchronization mode (Import, Force, LinkOnly), any other value is a 400; created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND. Sorting on last_sync_at puts never-synchronized providers last in ascending order and first in descending order. Bind credentials stay masked.
 */
export type get_List_providers = {
      method: "GET",
      path: "/realms/{realm_name}/federation/providers",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, name: string, provider_type: string, provider_family: ("ldap" | "kerberos"), enabled: boolean, sync_enabled: boolean, synced: boolean, sync_mode: ("Import" | "Force" | "LinkOnly"), created_from: string, created_to: string, order_by: ("name" | "priority" | "enabled" | "last_sync_at" | "created_at" | "updated_at") }>,
        path:  { realm_name: string },



          }
      responses: {200: Schemas.Paginated_ProviderResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type post_Create_provider = {
      method: "POST",
      path: "/realms/{realm_name}/federation/providers",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.CreateProviderRequest,
          }
      responses: {201: Schemas.ProviderResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type get_Get_provider = {
      method: "GET",
      path: "/realms/{realm_name}/federation/providers/{id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, id: string },



          }
      responses: {200: Schemas.ProviderResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type put_Update_provider = {
      method: "PUT",
      path: "/realms/{realm_name}/federation/providers/{id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, id: string },


        body:  Schemas.UpdateProviderRequest,
          }
      responses: {200: Schemas.UpdateProviderResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type delete_Delete_provider = {
      method: "DELETE",
      path: "/realms/{realm_name}/federation/providers/{id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, id: string },



          }
      responses: {204: Schemas.DeleteProviderResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Triggers synchronization of users from the external federation provider (LDAP, Kerberos, etc.) to the local IAM database. This performs a comprehensive diff and reconciliation.
 */
export type post_Sync_users = {
      method: "POST",
      path: "/realms/{realm_name}/federation/providers/{id}/sync-users",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, id: string },



          }
      responses: {200: Schemas.SyncUsersResponse,
401: unknown,
403: unknown,
404: unknown,
500: unknown,
},

    }
/**
 * Tests the connection to the external federation provider (LDAP, Kerberos, etc.) to verify configuration and connectivity
 */
export type post_Test_connection = {
      method: "POST",
      path: "/realms/{realm_name}/federation/providers/{id}/test-connection",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, id: string },



          }
      responses: {200: Schemas.TestConnectionResponse,
401: unknown,
403: unknown,
404: unknown,
},

    }
/**
 * Returns one page of the realm's identity providers. search matches case-insensitively a provider whose alias or display_name contains the value; a provider without a display name only matches by alias. Text filters (alias, display_name) match case-insensitively anywhere in the value; provider_id and enabled match exactly; health is the configuration status shown by the console: error when client_id, client_secret, authorization_url or token_url is missing, degraded when scopes is missing, healthy otherwise; created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND. Client secrets stay masked.
 */
export type get_List_identity_providers = {
      method: "GET",
      path: "/realms/{realm_name}/identity-providers",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, alias: string, display_name: string, provider_id: string, enabled: boolean, health: ("healthy" | "degraded" | "error"), created_from: string, created_to: string, order_by: ("alias" | "display_name" | "provider_id" | "enabled" | "created_at" | "updated_at") }>,
        path:  { realm_name: string },



          }
      responses: {200: Schemas.Paginated_IdentityProviderResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * Creates a new identity provider configuration for the specified realm. The identity provider will be used for social login (Google, GitHub, etc.) or OIDC/SAML federation.
 */
export type post_Create_identity_provider = {
      method: "POST",
      path: "/realms/{realm_name}/identity-providers",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.CreateIdentityProviderValidator,
          }
      responses: {201: Schemas.IdentityProviderResponse,
400: unknown,
401: unknown,
403: unknown,
404: unknown,
409: unknown,
},

    }
/**
 * Retrieves the details of a specific identity provider by its alias. Sensitive configuration values (like client secrets) are redacted in the response.
 */
export type get_Get_identity_provider = {
      method: "GET",
      path: "/realms/{realm_name}/identity-providers/{alias}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, alias: string },



          }
      responses: {200: Schemas.IdentityProviderResponse,
401: unknown,
403: unknown,
404: unknown,
},

    }
/**
 * Updates an existing identity provider configuration. Only the fields provided in the request body will be updated. The alias cannot be changed after creation.
 */
export type put_Update_identity_provider = {
      method: "PUT",
      path: "/realms/{realm_name}/identity-providers/{alias}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, alias: string },


        body:  Schemas.UpdateIdentityProviderValidator,
          }
      responses: {200: Schemas.UpdateIdentityProviderResponse,
400: unknown,
401: unknown,
403: unknown,
404: unknown,
},

    }
/**
 * Deletes an identity provider from the realm. This action is irreversible. Users who have linked accounts with this identity provider will no longer be able to use it for authentication.
 */
export type delete_Delete_identity_provider = {
      method: "DELETE",
      path: "/realms/{realm_name}/identity-providers/{alias}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, alias: string },



          }
      responses: {200: Schemas.DeleteIdentityProviderResponse,
401: unknown,
403: unknown,
404: unknown,
409: unknown,
},

    }
/**
 * Returns the accounts the browser previously signed in with, derived only from the FERRISKEY_DEVICE cookie. `client_name` is present only when the FERRISKEY_SESSION cookie belongs to an in-flight authorization request of the queried client. The list is empty, with a 200, when the cookie is absent, unknown, issued for another realm, or the realm has the account chooser disabled.
 */
export type get_List_accounts = {
      method: "GET",
      path: "/realms/{realm_name}/login-actions/accounts",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query:  { client_id: string, max_age?: number },
        path:  { realm_name: string },



          }
      responses: {200: Schemas.AccountListResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Resumes the chosen account's session without credentials and answers like the login endpoint does, with the redirect target in `url` and the FERRISKEY_SSO cookie set. When the session cannot be resumed silently the response is 200 with status requires_credentials and a login_hint, and the credential form re-authenticates into that session.
 */
export type post_Select_account = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/accounts/select",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ max_age: number }>,
        path:  { realm_name: string },


        body:  Schemas.SelectAccountRequest,
          }
      responses: {200: Schemas.SelectAccountResponse,
401: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Detaches the session from the device and stops it from being resumable from this browser. The session row and its tokens are left untouched.
 */
export type delete_Remove_account = {
      method: "DELETE",
      path: "/realms/{realm_name}/login-actions/accounts/{session_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, session_id: string },



          }
      responses: {204: unknown,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type post_Authenticate = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/authenticate",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.AuthenticateRequest,
          }
      responses: {200: Schemas.AuthenticateResponse,
400: Schemas.ApiError,
401: Schemas.ApiError,
500: Schemas.ApiError,
},

    }
/**
 * Using a recovery code allows a user to bypass a MFA challenge
 */
export type post_Burn_recovery_code = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/burn-recovery-code",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.BurnRecoveryCodeRequest,
          }
      responses: {200: Schemas.BurnRecoveryCodeResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Challenges the user to provide a One-Time Password (OTP) for authentication. This is typically used in multi-factor authentication scenarios.
 */
export type post_Challenge_otp = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/challenge-otp",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.ChallengeOtpRequest,
          }
      responses: {200: Schemas.ChallengeOtpResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Sends a password reset email to the user if the email exists in the realm. Always returns 204 to prevent email enumeration.
 */
export type post_Forgot_password = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/forgot-password",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.ForgotPasswordRequest,
          }
      responses: {200: Schemas.ForgotPasswordResponse,
400: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Generates recovery codes that allows the user to bypass a MFA challenge
 */
export type post_Generate_recovery_codes = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/generate-recovery-codes",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.GenerateRecoveryCodesRequest,
          }
      responses: {200: Schemas.GenerateRecoveryCodesResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Complete passkey authentication by submitting the browser's assertion response. On success, returns a login URL with an authorization code.
 */
export type post_Passkey_authenticate = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/passkey-authenticate",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ remember_me: boolean }>,
        path:  { realm_name: string },


        body:  Schemas.PasskeyPublicKeyCredential,
          }
      responses: {200: Schemas.PasskeyAuthenticateResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Initiates a passkey authentication flow. If username is provided, returns a challenge scoped to that user's passkeys. If omitted, returns a discoverable challenge allowing the browser to propose available passkeys.
 */
export type post_Passkey_request_options = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/passkey-request-options",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.PasskeyRequestOptionsRequest,
          }
      responses: {200: Schemas.PasskeyPublicKeyCredentialRequestOptionsJSON,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Resend the email verification link to the user's email address. Requires a valid Bearer token in the Authorization header.
 */
export type post_Resend_verification_email_handler = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/resend-verification-email",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.ResendVerificationEmailResponse,
401: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Completes the password reset flow by verifying the token and setting a new password. Returns authentication tokens to log the user in directly.
 */
export type post_Reset_password_with_token = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/reset-password",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.ResetPasswordRequest,
          }
      responses: {200: Schemas.CompletePasswordResetResponse,
400: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Sends a magic link to the user's email for passwordless authentication. The link contains a unique token that can be used to verify the user's identity.
 */
export type post_Send_magic_link = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/send-magic-link",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.SendMagicLinkRequest,
          }
      responses: {200: Schemas.SendMagicLinkResponse,
400: unknown,
500: unknown,
},

    }
/**
 * Sets up a One-Time Password (OTP) for user authentication. This is typically used in multi-factor authentication scenarios.
 */
export type get_Setup_otp = {
      method: "GET",
      path: "/realms/{realm_name}/login-actions/setup-otp",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.SetupOtpResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type post_Update_password = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/update-password",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.UpdatePasswordRequest,
          }
      responses: {200: Schemas.UpdatePasswordResponse,
400: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Verify a user's email address using the token from the verification email.
 */
export type post_Verify_email_handler = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/verify-email",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.VerifyEmailRequest,
          }
      responses: {200: Schemas.VerifyEmailResult,
400: Schemas.ApiErrorResponse,
422: unknown,
},

    }
/**
 * Verifies the magic link token and completes the authentication flow. Returns authentication status and redirect URL with authorization code.
 */
export type get_Verify_magic_link = {
      method: "GET",
      path: "/realms/{realm_name}/login-actions/verify-magic-link",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query:  { token_id: string, magic_token: string },
        path:  { realm_name: string },



          }
      responses: {200: Schemas.AuthenticateResponse,
400: unknown,
401: unknown,
404: unknown,
500: unknown,
},

    }
/**
 * Verifies the One-Time Password (OTP) provided by the user. This is typically used in multi-factor authentication scenarios.
 */
export type post_Verify_otp = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/verify-otp",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.OtpVerifyRequest,
          }
      responses: {200: Schemas.VerifyOtpResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Checks if a password reset token exists and has not expired, without consuming it.
 */
export type post_Verify_reset_token = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/verify-reset-token",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.VerifyResetTokenRequest,
          }
      responses: {200: Schemas.VerifyResetTokenResponse,
401: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * Attempt authentication using a WebAuthnAssertionResponse payload for webauthn authentication. See https://w3c.github.io/webauthn/#dictdef-authenticationresponsejson and https://w3c.github.io/webauthn/#authenticatorassertionresponse
 */
export type post_Webauthn_public_key_authenticate = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/webauthn-public-key-authenticate",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.PublicKeyCredential,
          }
      responses: {200: Schemas.AuthenticationAttemptResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Saving a webauthn public key to use it for authentication attempts or MFA later.
 */
export type post_Webauthn_public_key_create = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/webauthn-public-key-create",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.PublicKeyCredential,
          }
      responses: {200: Schemas.ValidatePublicKeyResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Provides a full PublicKeyCredentialCreationOption payload for WebAuthn credential creation/authentication. The payload contains the challenge to resolve in B64Url form as described in the specs. The content is described here: https://w3c.github.io/webauthn/#dictdef-publickeycredentialcreationoptions.
 */
export type post_Webauthn_public_key_create_options = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/webauthn-public-key-create-options",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.PublicKeyCredentialCreationOptionsJSON,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Provides a full PublicKeyCredentialRequestOption payload for webauthn authentication. See https://w3c.github.io/webauthn/#dictdef-publickeycredentialrequestoptions and https://w3c.github.io/webauthn/#dictdef-publickeycredentialrequestoptionsjson
 */
export type post_Webauthn_public_key_request_options = {
      method: "POST",
      path: "/realms/{realm_name}/login-actions/webauthn-public-key-request-options",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.PublicKeyCredentialRequestOptionsJSON,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the realm's organizations. Text filters (name, alias, domain, description) match case-insensitively anywhere in the value, and an organization without a description never matches a description filter; search matches case-insensitively an organization whose name, alias, domain or description contains the value, and a missing domain or description never matches; enabled matches exactly; has_domain keeps organizations with a domain (true) or without one (false); without_member takes a user id and keeps the organizations that user has not joined; ids takes a comma-separated list of at most 100 organization ids; created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND.
 */
export type get_List_organizations = {
      method: "GET",
      path: "/realms/{realm_name}/organizations",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, name: string, alias: string, domain: string, description: string, enabled: boolean, has_domain: boolean, without_member: string, ids: string, created_from: string, created_to: string, order_by: ("name" | "alias" | "enabled" | "created_at" | "updated_at") }>,
        path:  { realm_name: string },



          }
      responses: {200: Schemas.Paginated_Organization,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type post_Create_organization = {
      method: "POST",
      path: "/realms/{realm_name}/organizations",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.CreateOrganizationValidator,
          }
      responses: {201: Schemas.Organization,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
422: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type get_Get_organization = {
      method: "GET",
      path: "/realms/{realm_name}/organizations/{organization_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string },



          }
      responses: {200: Schemas.Organization,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type put_Update_organization = {
      method: "PUT",
      path: "/realms/{realm_name}/organizations/{organization_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string },


        body:  Schemas.UpdateOrganizationValidator,
          }
      responses: {200: Schemas.Organization,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
422: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type delete_Delete_organization = {
      method: "DELETE",
      path: "/realms/{realm_name}/organizations/{organization_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string },



          }
      responses: {204: unknown,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type get_List_attributes = {
      method: "GET",
      path: "/realms/{realm_name}/organizations/{organization_id}/attributes",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string },



          }
      responses: {200: Schemas.ListOrganizationAttributesResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type put_Upsert_attribute = {
      method: "PUT",
      path: "/realms/{realm_name}/organizations/{organization_id}/attributes/{key}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, key: string },


        body:  Schemas.UpsertAttributeValidator,
          }
      responses: {200: Schemas.OrganizationAttribute,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
422: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type delete_Delete_attribute = {
      method: "DELETE",
      path: "/realms/{realm_name}/organizations/{organization_id}/attributes/{key}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, key: string },



          }
      responses: {204: unknown,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the organization's groups as a flat list. search matches case-insensitively a group whose name or description contains the value; a group without a description only matches by name. Text filters (name, description) match case-insensitively anywhere in the value; parent_group_id keeps the direct children of that group; is_root keeps top-level groups (true) or nested groups (false), and is_root=true cannot be combined with parent_group_id; child_count is the number of direct sub-groups; ids takes a comma-separated list of at most 100 group ids; created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND.
 */
export type get_List_groups = {
      method: "GET",
      path: "/realms/{realm_name}/organizations/{organization_id}/groups",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, name: string, description: string, parent_group_id: string, is_root: boolean, ids: string, created_from: string, created_to: string, order_by: ("name" | "created_at" | "updated_at") }>,
        path:  { realm_name: string, organization_id: string },



          }
      responses: {200: Schemas.Paginated_GroupListItem,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type post_Create_group = {
      method: "POST",
      path: "/realms/{realm_name}/organizations/{organization_id}/groups",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string },


        body:  Schemas.CreateGroupValidator,
          }
      responses: {201: Schemas.Group,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type get_Get_group = {
      method: "GET",
      path: "/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, group_id: string },



          }
      responses: {200: Schemas.Group,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type put_Update_group = {
      method: "PUT",
      path: "/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, group_id: string },


        body:  Schemas.UpdateGroupValidator,
          }
      responses: {200: Schemas.Group,
400: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type delete_Delete_group = {
      method: "DELETE",
      path: "/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, group_id: string },



          }
      responses: {204: unknown,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type get_List_group_attributes = {
      method: "GET",
      path: "/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/attributes",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, group_id: string },



          }
      responses: {200: Array<Schemas.GroupAttribute>,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type put_Upsert_group_attribute = {
      method: "PUT",
      path: "/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/attributes/{key}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, group_id: string, key: string },


        body:  Schemas.UpsertAttributeValidator,
          }
      responses: {200: Schemas.GroupAttribute,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type delete_Delete_group_attribute = {
      method: "DELETE",
      path: "/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/attributes/{key}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, group_id: string, key: string },



          }
      responses: {204: unknown,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the group's members with their user identity. search matches case-insensitively a member whose username or email contains the value; a member without an email only matches by username. Text filters (username, email) match case-insensitively anywhere in the user's value; enabled matches the user's enabled flag exactly; created_from (inclusive) and created_to (exclusive) bound the membership date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND. created_at is the date the user joined the group.
 */
export type get_List_group_members = {
      method: "GET",
      path: "/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/members",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, username: string, email: string, enabled: boolean, created_from: string, created_to: string, order_by: ("username" | "email" | "created_at") }>,
        path:  { realm_name: string, organization_id: string, group_id: string },



          }
      responses: {200: Schemas.Paginated_GroupMemberDetail,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type post_Add_group_member = {
      method: "POST",
      path: "/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/members",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, group_id: string },


        body:  Schemas.AddGroupMemberValidator,
          }
      responses: {201: Schemas.GroupMember,
404: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type delete_Remove_group_member = {
      method: "DELETE",
      path: "/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/members/{user_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, group_id: string, user_id: string },



          }
      responses: {204: unknown,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type get_List_group_roles = {
      method: "GET",
      path: "/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/roles",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, group_id: string },



          }
      responses: {200: Array<Schemas.Role>,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type post_Assign_group_role = {
      method: "POST",
      path: "/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/roles",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, group_id: string },


        body:  Schemas.AssignGroupRoleValidator,
          }
      responses: {204: unknown,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type delete_Revoke_group_role = {
      method: "DELETE",
      path: "/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/roles/{role_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, group_id: string, role_id: string },



          }
      responses: {204: unknown,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the organization's memberships. search matches case-insensitively a member whose username or email contains the value; a member without an email only matches by username. Text filters (username, email) match case-insensitively anywhere in the member's user value; enabled matches the user's enabled flag exactly; created_from (inclusive) and created_to (exclusive) bound the membership date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND. created_at is the date the user joined the organization.
 */
export type get_List_members = {
      method: "GET",
      path: "/realms/{realm_name}/organizations/{organization_id}/members",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, username: string, email: string, enabled: boolean, created_from: string, created_to: string, order_by: ("username" | "email" | "created_at") }>,
        path:  { realm_name: string, organization_id: string },



          }
      responses: {200: Schemas.Paginated_OrganizationMember,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type post_Add_member = {
      method: "POST",
      path: "/realms/{realm_name}/organizations/{organization_id}/members",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string },


        body:  Schemas.AddMemberValidator,
          }
      responses: {201: Schemas.OrganizationMember,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
422: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type delete_Remove_member = {
      method: "DELETE",
      path: "/realms/{realm_name}/organizations/{organization_id}/members/{user_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, user_id: string },



          }
      responses: {204: unknown,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type get_List_member_roles = {
      method: "GET",
      path: "/realms/{realm_name}/organizations/{organization_id}/members/{user_id}/roles",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, user_id: string },



          }
      responses: {200: Array<Schemas.Role>,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type post_Assign_member_role = {
      method: "POST",
      path: "/realms/{realm_name}/organizations/{organization_id}/members/{user_id}/roles",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, user_id: string },


        body:  Schemas.AssignMemberRoleValidator,
          }
      responses: {204: unknown,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type delete_Revoke_member_role = {
      method: "DELETE",
      path: "/realms/{realm_name}/organizations/{organization_id}/members/{user_id}/roles/{role_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, organization_id: string, user_id: string, role_id: string },



          }
      responses: {204: unknown,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves the password policy for the specified realm.
 */
export type get_Get_password_policy = {
      method: "GET",
      path: "/realms/{realm_name}/password-policy",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.PasswordPolicy,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Updates the password policy for the specified realm.
 */
export type put_Update_password_policy = {
      method: "PUT",
      path: "/realms/{realm_name}/password-policy",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.UpdatePasswordPolicyValidator,
          }
      responses: {200: Schemas.PasswordPolicy,
400: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns the password policy rules for the specified realm. No authentication required.
 */
export type get_Get_public_password_policy = {
      method: "GET",
      path: "/realms/{realm_name}/password-policy/public",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.PublicPasswordPolicy,
401: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the realm's portal layouts, each with its tree and theme_count, the number of the realm's themes built on it. search and the name filter match case-insensitively anywhere in the name; the name filter matches case-insensitively anywhere in the value; is_default matches exactly; in_use=true keeps layouts at least one theme of the realm uses, in_use=false keeps the others; ids takes a comma-separated list of at most 100 layout ids; created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND. Requires manage_realm permission.
 */
export type get_List_layouts = {
      method: "GET",
      path: "/realms/{realm_name}/portal-layouts",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, name: string, is_default: boolean, in_use: boolean, ids: string, created_from: string, created_to: string, order_by: ("name" | "created_at" | "updated_at") }>,
        path:  { realm_name: string },



          }
      responses: {200: Schemas.Paginated_PortalLayoutListItem,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Creates a new portal layout. The first layout in a realm is automatically marked as default.
 */
export type post_Create_layout = {
      method: "POST",
      path: "/realms/{realm_name}/portal-layouts",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.CreatePortalLayoutValidator,
          }
      responses: {201: Schemas.CreatePortalLayoutResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Creates a portal layout from an exported layout tree. The tree is validated before it is stored, unlike the plain create endpoint which is fed by the builder itself.
 */
export type post_Import_layout = {
      method: "POST",
      path: "/realms/{realm_name}/portal-layouts/import",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.ImportPortalLayoutValidator,
          }
      responses: {201: Schemas.ImportPortalLayoutResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
422: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns the realm's default portal layout for the unauthenticated portal pages. No auth required.
 */
export type get_Get_public_default_layout = {
      method: "GET",
      path: "/realms/{realm_name}/portal-layouts/public/default",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.GetPublicDefaultPortalLayoutResponse,
401: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns a portal layout without auth, for the portal renderer. Only the layout named by the realm's active theme, or the realm's default layout, is served; any other layout of the realm answers with null.
 */
export type get_Get_public_layout = {
      method: "GET",
      path: "/realms/{realm_name}/portal-layouts/public/{layout_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, layout_id: string },



          }
      responses: {200: Schemas.GetPublicPortalLayoutResponse,
401: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves a single portal layout by id.
 */
export type get_Get_layout = {
      method: "GET",
      path: "/realms/{realm_name}/portal-layouts/{layout_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, layout_id: string },



          }
      responses: {200: Schemas.GetPortalLayoutResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Updates the name and tree of an existing portal layout.
 */
export type put_Update_layout = {
      method: "PUT",
      path: "/realms/{realm_name}/portal-layouts/{layout_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, layout_id: string },


        body:  Schemas.UpdatePortalLayoutValidator,
          }
      responses: {200: Schemas.UpdatePortalLayoutResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Deletes a portal layout.
 */
export type delete_Delete_layout = {
      method: "DELETE",
      path: "/realms/{realm_name}/portal-layouts/{layout_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, layout_id: string },



          }
      responses: {200: Schemas.DeletePortalLayoutResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Marks the given layout as the default for the realm. Any other default is unset in the same transaction.
 */
export type put_Set_default_layout = {
      method: "PUT",
      path: "/realms/{realm_name}/portal-layouts/{layout_id}/default",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, layout_id: string },



          }
      responses: {200: Schemas.SetDefaultPortalLayoutResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns the layout as a downloadable JSON envelope.
 */
export type get_Export_layout = {
      method: "GET",
      path: "/realms/{realm_name}/portal-layouts/{layout_id}/export",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, layout_id: string },



          }
      responses: {200: Schemas.ExportEnvelope,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Public endpoint used by the portal renderer. Returns the realm's active theme design tokens, the referenced layout ID (if any), and the JSONB component tree for the requested page type. Callers fetch the layout itself via the public portal-layouts endpoint. Falls back to defaults / empty trees when nothing is configured.
 */
export type get_Get_active_theme = {
      method: "GET",
      path: "/realms/{realm_name}/portal/active",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query:  { page_type: Schemas.PortalPageType },
        path:  { realm_name: string },



          }
      responses: {200: Schemas.ActiveThemeResponse,
401: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns the list of block types that must appear in each page's component tree to pass validation.
 */
export type get_Get_page_requirements = {
      method: "GET",
      path: "/realms/{realm_name}/portal/page-requirements",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.PageRequirementsResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves the portal theme configuration for the realm. Requires manage_realm permission. Falls back to defaults when no configuration is stored.
 */
export type get_Get_theme = {
      method: "GET",
      path: "/realms/{realm_name}/portal/theme",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.GetThemeResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Upserts the portal theme configuration for the realm. Requires manage_realm permission.
 */
export type put_Update_theme = {
      method: "PUT",
      path: "/realms/{realm_name}/portal/theme",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.UpdateThemeValidator,
          }
      responses: {200: Schemas.UpdateThemeResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the realm's portal themes, each with its design tokens and page trees. search and the name filter match case-insensitively anywhere in the name; the name filter matches case-insensitively anywhere in the value; layout_id matches exactly; activatable=true keeps themes whose every page holds the blocks listed by page-requirements (the check activation runs), activatable=false keeps the others; created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND. Requires manage_realm permission.
 */
export type get_List_themes = {
      method: "GET",
      path: "/realms/{realm_name}/portal/themes",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, name: string, layout_id: string, activatable: boolean, created_from: string, created_to: string, order_by: ("name" | "created_at" | "updated_at") }>,
        path:  { realm_name: string },



          }
      responses: {200: Schemas.Paginated_PortalTheme,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Creates a new portal theme in the realm with empty page trees. Requires manage_realm permission.
 */
export type post_Create_theme = {
      method: "POST",
      path: "/realms/{realm_name}/portal/themes",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.CreateThemeValidator,
          }
      responses: {201: Schemas.CreateThemeResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Creates a portal theme from an exported JSON envelope: its design tokens, its page trees, and the layout carried inside the file, which is recreated and bound to the new theme.
 */
export type post_Import_theme = {
      method: "POST",
      path: "/realms/{realm_name}/portal/themes/import",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.ImportPortalThemeValidator,
          }
      responses: {201: Schemas.ImportPortalThemeResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves a single portal theme, including design tokens, layout reference, and all seven page trees.
 */
export type get_Get_theme_by_id = {
      method: "GET",
      path: "/realms/{realm_name}/portal/themes/{theme_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, theme_id: string },



          }
      responses: {200: Schemas.GetThemeByIdResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Updates a portal theme's name, layout reference, and design tokens. Page trees are managed via /pages/{page_type}.
 */
export type put_Update_theme_metadata = {
      method: "PUT",
      path: "/realms/{realm_name}/portal/themes/{theme_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, theme_id: string },


        body:  Schemas.UpdateThemeMetadataValidator,
          }
      responses: {200: Schemas.UpdateThemeMetadataResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Deletes a portal theme. Refuses (400) when the theme is currently active for the realm.
 */
export type delete_Delete_theme = {
      method: "DELETE",
      path: "/realms/{realm_name}/portal/themes/{theme_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, theme_id: string },



          }
      responses: {200: Schemas.DeleteThemeResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Marks the given theme as the realm's active theme. Stored in realm_settings.portal_theme_id.
 */
export type post_Activate_theme = {
      method: "POST",
      path: "/realms/{realm_name}/portal/themes/{theme_id}/activate",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, theme_id: string },



          }
      responses: {200: Schemas.ActivateThemeResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns the theme as a downloadable JSON envelope: its design tokens, every page tree, and the layout it is framed by, carried by value so the file can be imported into another realm or deployment.
 */
export type get_Export_theme = {
      method: "GET",
      path: "/realms/{realm_name}/portal/themes/{theme_id}/export",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, theme_id: string },



          }
      responses: {200: Schemas.ThemeExportEnvelope,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Replaces the JSONB component tree for a single page type. The submitted tree must contain every required block type for the page; otherwise 422 is returned.
 */
export type put_Update_theme_page = {
      method: "PUT",
      path: "/realms/{realm_name}/portal/themes/{theme_id}/pages/{page_type}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, theme_id: string, page_type: Schemas.PortalPageType },


        body:  Schemas.UpdateThemePageValidator,
          }
      responses: {200: Schemas.UpdateThemePageResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
422: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Initiates the authentication process for a user in a specific realm.
 */
export type get_Auth_handler = {
      method: "GET",
      path: "/realms/{realm_name}/protocol/openid-connect/auth",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ response_type: string, client_id: string, redirect_uri: string, scope: string, state: string, nonce: string, code_challenge: string, code_challenge_method: Schemas.CodeChallengeMethod, prompt: string, max_age: number, resource: string, login_hint: string }>,
        path:  { realm_name: string },



          }
      responses: {302: Schemas.AuthResponse,
400: unknown,
401: unknown,
500: unknown,
},

    }
/**
 * Initiates the OAuth 2.0 Device Authorization Grant (RFC 8628 §3.1). Public clients pass `client_id` in the form body; confidential clients authenticate with HTTP Basic. Returns a device code, an end-user code, and the verification URIs the device should display.
 */
export type post_Device_authorization = {
      method: "POST",
      path: "/realms/{realm_name}/protocol/openid-connect/auth/device",
      requestFormat: "form-url",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.DeviceAuthorizationRequest,
          }
      responses: {200: Schemas.InitiateDeviceFlowOutput,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves the JSON Web Key (JWK) keys for a specific realm, used for verifying JWT tokens.
 */
export type get_Get_certs = {
      method: "GET",
      path: "/realms/{realm_name}/protocol/openid-connect/certs",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.GetCertsResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves the JSON Web Key Set (JWKS) for a specific realm, used by resource servers to validate JWT signatures.
 */
export type get_Get_jwks_json = {
      method: "GET",
      path: "/realms/{realm_name}/protocol/openid-connect/jwks.json",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.GetCertsResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Ends the user's OP session. Supports id_token_hint, post_logout_redirect_uri, state, and client_id.
 */
export type get_Logout_get = {
      method: "GET",
      path: "/realms/{realm_name}/protocol/openid-connect/logout",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ id_token_hint: string, post_logout_redirect_uri: string, state: string, client_id: string }>,
        path:  { realm_name: string },



          }
      responses: {204: unknown,
307: unknown,
},

    }
/**
 * Ends the user's OP session. Supports id_token_hint, post_logout_redirect_uri, state, and client_id.
 */
export type post_Logout_post = {
      method: "POST",
      path: "/realms/{realm_name}/protocol/openid-connect/logout",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.LogoutRequestValidator,
          }
      responses: {204: unknown,
307: unknown,
},

    }
/**
 * Register a new user in the specified realm. Returns JWT tokens if email verification is disabled, or a pending verification message if enabled.
 */
export type post_Registration_handler = {
      method: "POST",
      path: "/realms/{realm_name}/protocol/openid-connect/registrations",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.RegistrationRequest,
          }
      responses: {201: Schemas.RegistrationResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * OAuth2 token revocation endpoint (RFC 7009). Revokes access or refresh tokens for the requesting client.
 */
export type post_Revoke_token = {
      method: "POST",
      path: "/realms/{realm_name}/protocol/openid-connect/revoke",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.RevokeTokenRequestValidator,
          }
      responses: {200: unknown,
},

    }
/**
 * Exchanges a token for a JWT token. This endpoint allows clients to exchange various types of tokens (like authorization codes, refresh tokens, etc.) for a JWT token. With `grant_type=urn:ietf:params:oauth:grant-type:token-exchange` (RFC 8693), a confidential client swaps a `subject_token` for a narrower access token and gets the RFC 8693 response instead.
 */
export type post_Exchange_token = {
      method: "POST",
      path: "/realms/{realm_name}/protocol/openid-connect/token",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.TokenRequestValidator,
          }
      responses: {200: Schemas.TokenResponse,
400: Schemas.OAuth2ErrorResponse,
401: Schemas.TokenUnauthorizedResponse,
404: Schemas.ApiErrorResponse,
429: Schemas.OAuth2ErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * OAuth2/OIDC Token Introspection (RFC 7662). Only confidential clients may call this endpoint using client_secret_basic or client_secret_post. Authorization requires the caller's service account to have the role `introspect` (treated as the `introspect` scope).
 */
export type post_Introspect_token = {
      method: "POST",
      path: "/realms/{realm_name}/protocol/openid-connect/token/introspect",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.IntrospectRequestValidator,
          }
      responses: {200: Schemas.TokenIntrospectionResponse,
401: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves the user information for the authenticated user in a specific realm.
 */
export type get_Get_userinfo = {
      method: "GET",
      path: "/realms/{realm_name}/protocol/openid-connect/userinfo",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.UserInfoResponse,
401: unknown,
403: unknown,
500: unknown,
},

    }
/**
 * Accepts a deflated, base64 encoded AuthnRequest from a service provider and redirects the browser to the login page.
 */
export type get_Saml_sso_redirect = {
      method: "GET",
      path: "/realms/{realm_name}/protocol/saml",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query:  { SAMLRequest: string, RelayState?: string },
        path:  { realm_name: string },



          }
      responses: {302: unknown,
400: unknown,
500: unknown,
},

    }
/**
 * Accepts a base64 encoded AuthnRequest posted as a form by a service provider and redirects the browser to the login page.
 */
export type post_Saml_sso_post = {
      method: "POST",
      path: "/realms/{realm_name}/protocol/saml",
      requestFormat: "form-url",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.SamlAuthnRequestParams,
          }
      responses: {302: unknown,
400: unknown,
500: unknown,
},

    }
/**
 * Exchanges the authorization code minted by the login flow for a signed SAML response and renders the self-submitting form that posts it to the assertion consumer service.
 */
export type get_Saml_continue = {
      method: "GET",
      path: "/realms/{realm_name}/protocol/saml/continue",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query:  { code: string },
        path:  { realm_name: string },



          }
      responses: {200: unknown,
400: unknown,
500: unknown,
},

    }
/**
 * Returns the IdP entity descriptor a service provider needs to trust this realm: its entity id, its single sign-on endpoints and the certificate that signs assertions.
 */
export type get_Saml_descriptor = {
      method: "GET",
      path: "/realms/{realm_name}/protocol/saml/descriptor",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: unknown,
404: unknown,
500: unknown,
},

    }
/**
 * Lists which confidential client owns each allowed resource of the realm. An owner may exchange tokens whose audience is the resource.
 */
export type get_Get_resource_owners = {
      method: "GET",
      path: "/realms/{realm_name}/resource-owners",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.ResourceOwnersResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Replaces the whole set of resource owners of the realm. Every uri must be in the realm's allowed resources and appear once; every client must be a confidential, admin-registered client of the realm.
 */
export type put_Replace_resource_owners = {
      method: "PUT",
      path: "/realms/{realm_name}/resource-owners",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.ReplaceResourceOwnersValidator,
          }
      responses: {200: Schemas.ResourceOwnersResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the realm's roles. search matches case-insensitively a role whose name or description contains the value (a role without a description never matches it through its description), a client role whose client identifier (client_id string) contains the value, or a client role whose qualified value `<client identifier>.<role name>` contains the value (so `app-web.admin` finds the `admin` role of client `app-web`, and `app-web.ns.admin` finds its `ns.admin` role). Text filters (name, description) match case-insensitively anywhere in the value; qualified_name matches exactly and case-sensitively a realm role whose name equals the value or a client role whose `<client identifier>.<role name>` equals the value, so a realm role `a.b` and the `b` role of client `a` are both returned for `a.b`; require_mfa and client_id match exactly; scope keeps realm roles (no client) or client roles; has_permissions keeps roles granting at least one permission (true) or none (false); ids takes a comma-separated list of at most 100 role ids. created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND.
 */
export type get_Get_roles = {
      method: "GET",
      path: "/realms/{realm_name}/roles",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, name: string, qualified_name: string, description: string, require_mfa: boolean, client_id: string, scope: ("realm" | "client"), has_permissions: boolean, ids: string, created_from: string, created_to: string, order_by: ("name" | "created_at" | "updated_at") }>,
        path:  { realm_name: string },



          }
      responses: {200: Schemas.Paginated_Role,
400: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Creates a new realm-scoped role in the specified realm.
 */
export type post_Create_realm_role = {
      method: "POST",
      path: "/realms/{realm_name}/roles",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.CreateRoleValidator,
          }
      responses: {201: Schemas.CreateRoleResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type get_Get_role = {
      method: "GET",
      path: "/realms/{realm_name}/roles/{role_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, role_id: string },



          }
      responses: {200: Schemas.GetRoleResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type put_Update_role = {
      method: "PUT",
      path: "/realms/{realm_name}/roles/{role_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, role_id: string },


        body:  Schemas.UpdateRoleValidator,
          }
      responses: {200: Schemas.UpdateRoleResponse,
400: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type delete_Delete_role = {
      method: "DELETE",
      path: "/realms/{realm_name}/roles/{role_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, role_id: string },



          }
      responses: {200: Schemas.DeleteRoleResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type patch_Update_role_permissions = {
      method: "PATCH",
      path: "/realms/{realm_name}/roles/{role_id}/permissions",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, role_id: string },


        body:  Schemas.UpdateRolePermissionsValidator,
          }
      responses: {200: Schemas.UpdateRolePermissionsResponse,
400: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the realm's security events. search matches case-insensitively anywhere in the ip_address and never matches an event without an address. ip_address matches case-insensitively anywhere in the value and never matches an event without an address. event_types takes a comma-separated list of event types and keeps the events of any of them. status, target_type, actor_id and client_id (the event's target id) match exactly. from_timestamp and to_timestamp bound timestamp, both inclusive. Filters combine with AND.
 */
export type get_Get_security_events = {
      method: "GET",
      path: "/realms/{realm_name}/seawatch/v1/security-events",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, actor_id: string, client_id: string, event_types: string, status: ("success" | "failure"), target_type: string, from_timestamp: string, to_timestamp: string, ip_address: string, order_by: ("event_type" | "status" | "timestamp" | "created_at") }>,
        path:  { realm_name: string },



          }
      responses: {200: Schemas.Paginated_SecurityEvent,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns the list of users and roles automatically whitelisted on all clients under maintenance in this realm.
 */
export type get_Get_realm_whitelist = {
      method: "GET",
      path: "/realms/{realm_name}/settings/maintenance/whitelist",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.GetRealmWhitelistResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
},

    }
/**
 * Adds a user or role to the realm's default maintenance whitelist. These entries are inherited by all clients under maintenance.
 */
export type post_Add_realm_whitelist_entry = {
      method: "POST",
      path: "/realms/{realm_name}/settings/maintenance/whitelist",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.AddWhitelistEntryValidator,
          }
      responses: {201: Schemas.AddRealmWhitelistEntryResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
},

    }
/**
 * Removes a user or role from the realm's default maintenance whitelist.
 */
export type delete_Remove_realm_whitelist_entry = {
      method: "DELETE",
      path: "/realms/{realm_name}/settings/maintenance/whitelist/{entry_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, entry_id: string },



          }
      responses: {200: Schemas.RemoveRealmWhitelistEntryResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves the SMTP configuration for the specified realm. The password field is omitted from the response.
 */
export type get_Get_smtp_config = {
      method: "GET",
      path: "/realms/{realm_name}/smtp-config",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.SmtpConfig,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Creates or updates the SMTP configuration for the specified realm.
 */
export type put_Upsert_smtp_config = {
      method: "PUT",
      path: "/realms/{realm_name}/smtp-config",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.UpsertSmtpConfigValidator,
          }
      responses: {200: Schemas.SmtpConfig,
400: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Deletes the SMTP configuration for the specified realm.
 */
export type delete_Delete_smtp_config = {
      method: "DELETE",
      path: "/realms/{realm_name}/smtp-config",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {204: unknown,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the realm's users. search matches case-insensitively anywhere in the username, email, firstname or lastname. created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Text filters (username, email, firstname, lastname) match case-insensitively anywhere in the value; enabled, email_verified, service_account and role_id match exactly; ids takes a comma-separated list of at most 100 user ids; not_in_group takes an organization group id and keeps the users that are not members of that group; not_in_organization takes an organization id and keeps the users that are not members of that organization. Filters combine with AND.
 */
export type get_Get_users = {
      method: "GET",
      path: "/realms/{realm_name}/users",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, username: string, email: string, firstname: string, lastname: string, enabled: boolean, email_verified: boolean, service_account: boolean, role_id: string, ids: string, not_in_group: string, not_in_organization: string, created_from: string, created_to: string, order_by: ("username" | "email" | "firstname" | "lastname" | "enabled" | "created_at" | "updated_at") }>,
        path:  { realm_name: string },



          }
      responses: {200: Schemas.Paginated_User,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type post_Create_user = {
      method: "POST",
      path: "/realms/{realm_name}/users",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.CreateUserValidator,
          }
      responses: {201: Schemas.CreateUserResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Stores the locale the currently authenticated user reads the product in. The locale must belong to the realm's supported set; sending null clears it so the realm default applies.
 */
export type put_Update_me_locale = {
      method: "PUT",
      path: "/realms/{realm_name}/users/@me/locale",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.UpdateOwnLocaleValidator,
          }
      responses: {200: Schemas.UpdateOwnLocaleResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
422: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the realms the caller may access. search matches case-insensitively anywhere in the name or display_name. Text filters (name, display_name) match case-insensitively anywhere in the value and combine with AND.
 */
export type get_Get_user_realms = {
      method: "GET",
      path: "/realms/{realm_name}/users/@me/realms",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, name: string, display_name: string, order_by: ("name" | "created_at" | "updated_at") }>,
        path:  { realm_name: string },



          }
      responses: {200: Schemas.Paginated_Realm,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Deletes multiple users in a realm by their IDs. This action is irreversible and will remove all associated data.
 */
export type delete_Bulk_delete_user = {
      method: "DELETE",
      path: "/realms/{realm_name}/users/bulk",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.BulkDeleteUserValidator,
          }
      responses: {200: Schemas.BulkDeleteUserResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves the profile of the currently authenticated user. The user id is taken from the access token, never from the URL.
 */
export type get_Get_own_profile = {
      method: "GET",
      path: "/realms/{realm_name}/users/me",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.OwnProfileResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Updates firstname, lastname and email of the currently authenticated user. Username can only be changed when the realm has username editing enabled.
 */
export type put_Update_own_profile = {
      method: "PUT",
      path: "/realms/{realm_name}/users/me",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.UpdateOwnProfileValidator,
          }
      responses: {200: Schemas.UpdateOwnProfileResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type get_List_own_sessions = {
      method: "GET",
      path: "/realms/{realm_name}/users/me/sessions",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
          query?: Partial<{page: number, limit: number, order: "asc" | "desc", order_by: "last_seen_at" | "expires_at" | "created_at"}>,
          path: {realm_name: string},
      },
      responses: {200: Schemas.Paginated_OwnSessionDto,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},
    }
export type post_Revoke_other_own_sessions = {
      method: "POST",
      path: "/realms/{realm_name}/users/me/sessions/revoke-others",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
          path: {realm_name: string},
      },
      responses: {200: Schemas.RevokeOtherSessionsResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},
    }
export type delete_Revoke_own_session = {
      method: "DELETE",
      path: "/realms/{realm_name}/users/me/sessions/{session_id}",
      requestFormat: "json",
      parameters: {
          path: {realm_name: string, session_id: string},
      },
      responses: {204: unknown,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},
    }
export type get_List_signing_keys = {
      method: "GET",
      path: "/realms/{realm_name}/keys",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
          path: {realm_name: string},
      },
      responses: {200: Schemas.SigningKeysResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},
    }
export type post_Rotate_signing_key = {
      method: "POST",
      path: "/realms/{realm_name}/keys/rotate",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
          path: {realm_name: string},
      },
      responses: {201: Schemas.SigningKeyResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},
    }
export type post_Revoke_signing_key = {
      method: "POST",
      path: "/realms/{realm_name}/keys/{kid}/revoke",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
          path: {realm_name: string, kid: string},
      },
      responses: {200: Schemas.SigningKeyResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},
    }
/**
 * Returns what the caller can sign in with: their password, their authenticator, and each of their passkeys. Never returns secret material. The user id is taken from the access token, never from the URL.
 */
export type get_List_own_credentials = {
      method: "GET",
      path: "/realms/{realm_name}/users/me/credentials",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },



          }
      responses: {200: Schemas.OwnCredentialsResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Verifies a code against the secret recorded by `POST /mfa/otp` and promotes it to a real credential, superseding whatever authenticator the account had. Requires an elevation proved with a password: the code alone must not be able to replace the factor it proves.
 */
export type put_Confirm_own_otp_enrollment = {
      method: "PUT",
      path: "/realms/{realm_name}/users/me/mfa/otp",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.ConfirmOwnOtpEnrollmentValidator,
          }
      responses: {200: Schemas.ConfirmOtpEnrollmentResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Generates a TOTP secret, records it server-side under a short TTL, and returns it with its otpauth URI so the caller can scan it. The secret is never taken back from the caller: `PUT /mfa/otp` reads it from this side of the wire. Requires an elevation proved with a password, refused here rather than at the confirm so the caller is not sent to scan a code they will not be allowed to submit.
 */
export type post_Start_own_otp_enrollment = {
      method: "POST",
      path: "/realms/{realm_name}/users/me/mfa/otp",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.StartOwnOtpEnrollmentValidator,
          }
      responses: {200: Schemas.StartOtpEnrollmentResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Refused when it would leave the account with no way to sign in, or without the second factor the realm or one of the caller's roles requires. Requires an elevation proved with a password.
 */
export type delete_Disable_own_otp = {
      method: "DELETE",
      path: "/realms/{realm_name}/users/me/mfa/otp",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.ElevationOnlyValidator,
          }
      responses: {200: Schemas.DisableOtpResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Answers the challenge issued by `POST /passkeys/options` and stores the resulting passkey on the caller's own account. Requires an elevation proved with a password, for the same reason as the challenge itself.
 */
export type post_Confirm_own_passkey_registration = {
      method: "POST",
      path: "/realms/{realm_name}/users/me/passkeys",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.ConfirmOwnPasskeyRegistrationValidator,
          }
      responses: {200: Schemas.ConfirmPasskeyResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns a WebAuthn creation challenge for the caller's own account, excluding the passkeys they already hold. The challenge is recorded server-side under a short TTL and answered by `POST /passkeys`. Requires an elevation proved with a password: a passkey is a primary sign-in means, so a second factor alone must not be able to add one.
 */
export type post_Start_own_passkey_registration = {
      method: "POST",
      path: "/realms/{realm_name}/users/me/passkeys/options",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.ElevationOnlyValidator,
          }
      responses: {200: Schemas.AccountPublicKeyCredentialCreationOptionsJSON,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Refused when it would leave the account with no way to sign in, or without the second factor the realm or one of the caller's roles requires. A credential that is not a passkey, or belongs to someone else, answers not found. Requires an elevation proved with a password.
 */
export type delete_Delete_own_passkey = {
      method: "DELETE",
      path: "/realms/{realm_name}/users/me/passkeys/{credential_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, credential_id: string },


        body:  Schemas.ElevationOnlyValidator,
          }
      responses: {200: Schemas.DeletePasskeyResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Verifies the current password, applies the realm password policy, and replaces the credential. Every other session of this account is revoked; the caller's own session survives. Any live elevation is accepted here, including one proved with a TOTP code: the current password supplied in the body is itself the primary proof, so demanding a password-proved elevation on top would only make the caller type it twice.
 */
export type put_Change_own_password = {
      method: "PUT",
      path: "/realms/{realm_name}/users/me/password",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.ChangeOwnPasswordValidator,
          }
      responses: {200: Schemas.ChangeOwnPasswordResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
422: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Verifies a credential the caller already holds and returns a short-lived elevation bound to the caller's session. Every sensitive account operation requires one. Supply exactly one of `password` or `otp_code`. Anything that adds, replaces or removes a sign-in credential requires the elevation to have been proved with a password — a second factor must not be able to rotate the factors, including itself. Changing the password is the exception: it verifies the current password in the body, which is the same proof.
 */
export type post_Request_elevation = {
      method: "POST",
      path: "/realms/{realm_name}/users/me/reauthenticate",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.RequestElevationValidator,
          }
      responses: {200: Schemas.ElevationResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves a user by their ID in a specific realm. This endpoint returns detailed information about the user.
 */
export type get_Get_user = {
      method: "GET",
      path: "/realms/{realm_name}/users/{user_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string },



          }
      responses: {200: Schemas.UserResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Updates an existing user in a specific realm. The user must exist and the request must include the necessary fields to update.
 */
export type put_Update_user = {
      method: "PUT",
      path: "/realms/{realm_name}/users/{user_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string },


        body:  Schemas.UpdateUserValidator,
          }
      responses: {200: Schemas.UpdateUserResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Deletes a user in a realm. This action is irreversible and will remove all associated data.
 */
export type delete_Delete_user = {
      method: "DELETE",
      path: "/realms/{realm_name}/users/{user_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string },



          }
      responses: {200: Schemas.DeleteUserResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns all custom key-value attributes attached to the user.
 */
export type get_Get_user_attributes = {
      method: "GET",
      path: "/realms/{realm_name}/users/{user_id}/attributes",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string },



          }
      responses: {200: Schemas.UserAttributesResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Upserts a set of key-value attributes on the user. Existing keys are updated, new keys are created.
 */
export type put_Set_user_attributes = {
      method: "PUT",
      path: "/realms/{realm_name}/users/{user_id}/attributes",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string },


        body:  Schemas.SetUserAttributesValidator,
          }
      responses: {200: Schemas.SetUserAttributesResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Removes the attribute with the given key from the user.
 */
export type delete_Delete_user_attribute = {
      method: "DELETE",
      path: "/realms/{realm_name}/users/{user_id}/attributes/{key}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string, key: string },



          }
      responses: {200: Schemas.DeleteUserAttributeResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves all credentials associated with a user in a specific realm.
 */
export type get_Get_user_credentials = {
      method: "GET",
      path: "/realms/{realm_name}/users/{user_id}/credentials",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string },



          }
      responses: {200: Schemas.GetUserCredentialsResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Stores a password hash produced by another identity provider as the user's password credential. Supported algorithms are bcrypt ($2a$, $2b$, $2y$) and argon2 (PHC string). The hash is re-encoded as argon2id on the user's first successful login.
 */
export type post_Import_password_credential = {
      method: "POST",
      path: "/realms/{realm_name}/users/{user_id}/credentials/import",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string },


        body:  Schemas.ImportPasswordCredentialValidator,
          }
      responses: {201: Schemas.ImportPasswordCredentialResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
422: Schemas.ValidationErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Deletes a specific credential for a user in a realm. This action is irreversible and will remove all associated data.
 */
export type delete_Delete_user_credential = {
      method: "DELETE",
      path: "/realms/{realm_name}/users/{user_id}/credentials/{credential_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string, credential_id: string },



          }
      responses: {200: Schemas.DeleteUserCredentialResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns every external identity provider account currently linked to the given user in the realm. Provider secrets and stored tokens are never returned. Requires the same permission as updating a user.
 */
export type get_List_identity_provider_links = {
      method: "GET",
      path: "/realms/{realm_name}/users/{user_id}/identity-provider-links",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string },



          }
      responses: {200: Schemas.IdentityProviderLinksResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Removes the link between the given user and an external identity provider account. The user can no longer authenticate through that provider until the account is linked again. Requires the same permission as updating a user.
 */
export type delete_Delete_identity_provider_link = {
      method: "DELETE",
      path: "/realms/{realm_name}/users/{user_id}/identity-provider-links/{link_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string, link_id: string },



          }
      responses: {200: Schemas.DeleteIdentityProviderLinkResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
export type get_List_user_organizations = {
      method: "GET",
      path: "/realms/{realm_name}/users/{user_id}/organizations",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string },



          }
      responses: {200: Array<Schemas.OrganizationMember>,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves the permissions assigned to a user by their ID in a specific realm.
 */
export type get_Get_user_permissions = {
      method: "GET",
      path: "/realms/{realm_name}/users/{user_id}/permissions",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string },



          }
      responses: {200: Schemas.UserPermissionsResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Resets the password for a user in a specific realm.
 */
export type put_Reset_password = {
      method: "PUT",
      path: "/realms/{realm_name}/users/{user_id}/reset-password",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string },


        body:  Schemas.ResetPasswordValidator,
          }
      responses: {200: Schemas.ResetPasswordResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves all roles associated with a user in a specific realm.
 */
export type get_Get_user_roles = {
      method: "GET",
      path: "/realms/{realm_name}/users/{user_id}/roles",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string },



          }
      responses: {200: Schemas.GetUserRolesResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Assigns a specified role to a user within a given realm. This endpoint is used to manage user roles in the system.
 */
export type post_Assign_role = {
      method: "POST",
      path: "/realms/{realm_name}/users/{user_id}/roles/{role_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string, role_id: string },



          }
      responses: {200: Schemas.AssignRoleResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Unassigns a specific role from a user in a realm. This action is irreversible and will remove the user's access to the role's permissions.
 */
export type delete_Unassign_role = {
      method: "DELETE",
      path: "/realms/{realm_name}/users/{user_id}/roles/{role_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string, role_id: string },



          }
      responses: {200: Schemas.UnassignRoleResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the user's sessions in the realm, expired ones included. search matches case-insensitively anywhere in the ip_address or user_agent and never matches a session where both are unknown. created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. ip_address and user_agent match case-insensitively anywhere in the value and never match a session where the value is unknown; persistent matches exactly. Filters combine with AND. Sorting on last_seen_at puts never-seen sessions last in ascending order and first in descending order. Requires ManageUsers, ManageRealm or ViewUsers, unless the caller is the user themselves. A user that is not in the realm is a 404.
 */
export type get_List_user_sessions = {
      method: "GET",
      path: "/realms/{realm_name}/users/{user_id}/sessions",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, ip_address: string, user_agent: string, persistent: boolean, created_from: string, created_to: string, order_by: ("last_seen_at" | "expires_at" | "created_at") }>,
        path:  { realm_name: string, user_id: string },



          }
      responses: {200: Schemas.Paginated_UserSessionDto,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Revokes a single session for the given user. The session's tokens are immediately invalidated. Other sessions are unaffected. Requires ManageUsers permission, unless the caller is the user themselves.
 */
export type delete_Revoke_user_session = {
      method: "DELETE",
      path: "/realms/{realm_name}/users/{user_id}/sessions/{session_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string, session_id: string },



          }
      responses: {204: unknown,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Clears the lockout state (failed_login_attempts and locked_until) for the specified user. Requires manage-users permission.
 */
export type post_Unlock_user = {
      method: "POST",
      path: "/realms/{realm_name}/users/{user_id}/unlock",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, user_id: string },



          }
      responses: {204: unknown,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the realm's webhooks, each with its subscribers. Stored secrets and header values are never returned. search matches case-insensitively a webhook whose name or endpoint contains the value; a webhook without a name only matches by its endpoint. Text filters (name, endpoint) match case-insensitively anywhere in the value; triggered keeps webhooks that fired at least once (true) or never (false); has_subscribers keeps webhooks with at least one subscribed trigger (true) or none (false); secure_endpoint keeps endpoints starting with `https://` (true) or not (false). created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND. Webhooks without a name or never triggered sort last in ascending order and first in descending order.
 */
export type get_Fetch_webhooks = {
      method: "GET",
      path: "/realms/{realm_name}/webhooks",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, name: string, endpoint: string, triggered: boolean, has_subscribers: boolean, secure_endpoint: boolean, created_from: string, created_to: string, order_by: ("name" | "endpoint" | "triggered_at" | "created_at" | "updated_at") }>,
        path:  { realm_name: string },



          }
      responses: {200: Schemas.Paginated_Webhook,
400: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Creates a new webhook in the system related to the current realm.
 */
export type post_Create_webhook = {
      method: "POST",
      path: "/realms/{realm_name}/webhooks",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string },


        body:  Schemas.CreateWebhookValidator,
          }
      responses: {200: Schemas.CreateWebhookResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves one webhook in the system related to the current realm.
 */
export type get_Get_webhook = {
      method: "GET",
      path: "/realms/{realm_name}/webhooks/{webhook_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, webhook_id: string },



          }
      responses: {200: Schemas.Webhook,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Updates a webhook in the system related to the current realm.
 */
export type put_Update_webhook = {
      method: "PUT",
      path: "/realms/{realm_name}/webhooks/{webhook_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, webhook_id: string },


        body:  Schemas.UpdateWebhookValidator,
          }
      responses: {200: Schemas.UpdateWebhookResponse,
400: Schemas.ApiErrorResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Deletes a webhook in the system related to the current realm.
 */
export type delete_Delete_webhook = {
      method: "DELETE",
      path: "/realms/{realm_name}/webhooks/{webhook_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, webhook_id: string },



          }
      responses: {200: Schemas.DeleteWebhookResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Returns one page of the recorded delivery attempts of one webhook of the realm. Payloads are not included; read a single delivery to get its payload. search matches case-insensitively a delivery whose resource_id, written as a lowercase hyphenated UUID, contains the value. event matches one webhook trigger exactly; status matches exactly one of pending, delivering, succeeded, failed; resource_id matches the id of the resource the event is about. created_from (inclusive) and created_to (exclusive) bound the creation date and take RFC 3339 date-times with a time and an offset; an inverted range returns an empty page. Filters combine with AND. Deliveries never attempted sort last on last_attempt_at in ascending order and first in descending order.
 */
export type get_Fetch_deliveries = {
      method: "GET",
      path: "/realms/{realm_name}/webhooks/{webhook_id}/deliveries",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {
            query?:  Partial<{ page: number, limit: number, order: ("asc" | "desc"), search: string, event: ("user.created" | "user.email_verified" | "user.updated" | "user.deleted" | "user.role.assigned" | "user.role.unassigned" | "user.bulk_deleted" | "user.credentials.deleted" | "auth.reset_password" | "auth.device_flow.initiated" | "auth.device_flow.denied" | "auth.device_flow.expired" | "client.created" | "client.updated" | "client.deleted" | "client.role.created" | "client.role.updated" | "redirect_uri.created" | "redirect_uri.updated" | "redirect_uri.deleted" | "web_origin.created" | "web_origin.deleted" | "client.saml_config.updated" | "client.saml_attribute_mapper.created" | "client.saml_attribute_mapper.deleted" | "role.created" | "role.updated" | "role.deleted" | "role.permission.updated" | "realm.created" | "realm.updated" | "realm.deleted" | "realm.settings.updated" | "webhook.created" | "webhook.updated" | "webhook.deleted" | "client.maintenance.enabled" | "client.maintenance.disabled"), status: ("pending" | "delivering" | "succeeded" | "failed"), resource_id: string, created_from: string, created_to: string, order_by: ("status" | "attempt_count" | "last_attempt_at" | "created_at" | "updated_at") }>,
        path:  { realm_name: string, webhook_id: string },



          }
      responses: {200: Schemas.Paginated_DeliverySummary,
400: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Retrieves one recorded delivery, including the payload that was sent.
 */
export type get_Get_delivery = {
      method: "GET",
      path: "/realms/{realm_name}/webhooks/{webhook_id}/deliveries/{delivery_id}",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, webhook_id: string, delivery_id: string },



          }
      responses: {200: Schemas.GetDeliveryResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Puts a terminal delivery back in the queue so the worker attempts it again.
 */
export type post_Retry_delivery = {
      method: "POST",
      path: "/realms/{realm_name}/webhooks/{webhook_id}/deliveries/{delivery_id}/retry",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, webhook_id: string, delivery_id: string },



          }
      responses: {202: Schemas.RetryDeliveryResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
409: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }
/**
 * Generates a new signing secret and returns it. This is the only response that ever contains the secret: it cannot be read again afterwards. Deliveries already signed with the previous secret will fail verification.
 */
export type post_Rotate_secret = {
      method: "POST",
      path: "/realms/{realm_name}/webhooks/{webhook_id}/secret/rotate",
      requestFormat: "json",
      responseFormat: "json",
      parameters: {

        path:  { realm_name: string, webhook_id: string },



          }
      responses: {200: Schemas.RotateSecretResponse,
401: Schemas.ApiErrorResponse,
403: Schemas.ApiErrorResponse,
404: Schemas.ApiErrorResponse,
500: Schemas.ApiErrorResponse,
},

    }

  // </Endpoints>
  }



     // <EndpointByMethod>
     export type EndpointByMethod = {
     get: {
           "/.well-known/authzen-configuration/realms/{realm_name}/authzen": Endpoints.get_Get_authzen_configuration,
"/email-templates/variables/{email_type}": Endpoints.get_Get_variables,
"/realms/{name}": Endpoints.get_Get_realm,
"/realms/{name}/login-settings": Endpoints.get_Get_login_realm_settings_handler,
"/realms/{realm_name}/.well-known/oauth-authorization-server/realms/{realm_name}": Endpoints.get_Get_authorization_server_metadata,
"/realms/{realm_name}/.well-known/openid-configuration": Endpoints.get_Get_openid_configuration,
"/realms/{realm_name}/auth/consent": Endpoints.get_Get_consent,
"/realms/{realm_name}/authz/policies": Endpoints.get_List_policies,
"/realms/{realm_name}/authz/policies/{policy_id}": Endpoints.get_Get_policy,
"/realms/{realm_name}/authz/schema": Endpoints.get_Get_schema,
"/realms/{realm_name}/authz/schema/builtin": Endpoints.get_Get_builtin_schema,
"/realms/{realm_name}/authz/state": Endpoints.get_Get_state,
"/realms/{realm_name}/authz/templates": Endpoints.get_List_templates,
"/realms/{realm_name}/authz/templates/{template_id}": Endpoints.get_Get_policy_template,
"/realms/{realm_name}/broker/{alias}/login": Endpoints.get_Broker_login,
"/realms/{realm_name}/client-scopes": Endpoints.get_Get_client_scopes,
"/realms/{realm_name}/client-scopes/{scope_id}": Endpoints.get_Get_client_scope,
"/realms/{realm_name}/clients": Endpoints.get_Get_clients,
"/realms/{realm_name}/clients/{client_id}": Endpoints.get_Get_client,
"/realms/{realm_name}/clients/{client_id}/client-scopes": Endpoints.get_Get_client_client_scopes,
"/realms/{realm_name}/clients/{client_id}/client-secret": Endpoints.get_Get_client_secret,
"/realms/{realm_name}/clients/{client_id}/maintenance/whitelist": Endpoints.get_Get_client_whitelist,
"/realms/{realm_name}/clients/{client_id}/post-logout-redirects": Endpoints.get_Get_post_logout_redirect_uris,
"/realms/{realm_name}/clients/{client_id}/redirects": Endpoints.get_Get_redirect_uris,
"/realms/{realm_name}/clients/{client_id}/roles": Endpoints.get_Get_client_roles,
"/realms/{realm_name}/clients/{client_id}/saml-attribute-mappers": Endpoints.get_Get_saml_attribute_mappers,
"/realms/{realm_name}/clients/{client_id}/saml-config": Endpoints.get_Get_client_saml_config,
"/realms/{realm_name}/clients/{client_id}/token-exchange-policies": Endpoints.get_Get_token_exchange_policies,
"/realms/{realm_name}/clients/{client_id}/web-origins": Endpoints.get_Get_web_origins,
"/realms/{realm_name}/compass/v1/activity/daily": Endpoints.get_Get_daily_activity_stats,
"/realms/{realm_name}/compass/v1/flows": Endpoints.get_Get_flows,
"/realms/{realm_name}/compass/v1/flows/{flow_id}": Endpoints.get_Get_flow,
"/realms/{realm_name}/compass/v1/stats": Endpoints.get_Get_stats,
"/realms/{realm_name}/device": Endpoints.get_Device_verification_page,
"/realms/{realm_name}/device/preview": Endpoints.get_Device_preview,
"/realms/{realm_name}/email-templates": Endpoints.get_Fetch_templates,
"/realms/{realm_name}/email-templates/{template_id}": Endpoints.get_Get_template,
"/realms/{realm_name}/email-templates/{template_id}/export": Endpoints.get_Export_template,
"/realms/{realm_name}/federation/providers": Endpoints.get_List_providers,
"/realms/{realm_name}/federation/providers/{id}": Endpoints.get_Get_provider,
"/realms/{realm_name}/identity-providers": Endpoints.get_List_identity_providers,
"/realms/{realm_name}/identity-providers/{alias}": Endpoints.get_Get_identity_provider,
"/realms/{realm_name}/login-actions/accounts": Endpoints.get_List_accounts,
"/realms/{realm_name}/login-actions/setup-otp": Endpoints.get_Setup_otp,
"/realms/{realm_name}/login-actions/verify-magic-link": Endpoints.get_Verify_magic_link,
"/realms/{realm_name}/organizations": Endpoints.get_List_organizations,
"/realms/{realm_name}/organizations/{organization_id}": Endpoints.get_Get_organization,
"/realms/{realm_name}/organizations/{organization_id}/attributes": Endpoints.get_List_attributes,
"/realms/{realm_name}/organizations/{organization_id}/groups": Endpoints.get_List_groups,
"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}": Endpoints.get_Get_group,
"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/attributes": Endpoints.get_List_group_attributes,
"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/members": Endpoints.get_List_group_members,
"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/roles": Endpoints.get_List_group_roles,
"/realms/{realm_name}/organizations/{organization_id}/members": Endpoints.get_List_members,
"/realms/{realm_name}/organizations/{organization_id}/members/{user_id}/roles": Endpoints.get_List_member_roles,
"/realms/{realm_name}/password-policy": Endpoints.get_Get_password_policy,
"/realms/{realm_name}/password-policy/public": Endpoints.get_Get_public_password_policy,
"/realms/{realm_name}/portal-layouts": Endpoints.get_List_layouts,
"/realms/{realm_name}/portal-layouts/public/default": Endpoints.get_Get_public_default_layout,
"/realms/{realm_name}/portal-layouts/public/{layout_id}": Endpoints.get_Get_public_layout,
"/realms/{realm_name}/portal-layouts/{layout_id}": Endpoints.get_Get_layout,
"/realms/{realm_name}/portal-layouts/{layout_id}/export": Endpoints.get_Export_layout,
"/realms/{realm_name}/portal/active": Endpoints.get_Get_active_theme,
"/realms/{realm_name}/portal/page-requirements": Endpoints.get_Get_page_requirements,
"/realms/{realm_name}/portal/theme": Endpoints.get_Get_theme,
"/realms/{realm_name}/portal/themes": Endpoints.get_List_themes,
"/realms/{realm_name}/portal/themes/{theme_id}": Endpoints.get_Get_theme_by_id,
"/realms/{realm_name}/portal/themes/{theme_id}/export": Endpoints.get_Export_theme,
"/realms/{realm_name}/protocol/openid-connect/auth": Endpoints.get_Auth_handler,
"/realms/{realm_name}/protocol/openid-connect/certs": Endpoints.get_Get_certs,
"/realms/{realm_name}/protocol/openid-connect/jwks.json": Endpoints.get_Get_jwks_json,
"/realms/{realm_name}/protocol/openid-connect/logout": Endpoints.get_Logout_get,
"/realms/{realm_name}/protocol/openid-connect/userinfo": Endpoints.get_Get_userinfo,
"/realms/{realm_name}/protocol/saml": Endpoints.get_Saml_sso_redirect,
"/realms/{realm_name}/protocol/saml/continue": Endpoints.get_Saml_continue,
"/realms/{realm_name}/protocol/saml/descriptor": Endpoints.get_Saml_descriptor,
"/realms/{realm_name}/resource-owners": Endpoints.get_Get_resource_owners,
"/realms/{realm_name}/roles": Endpoints.get_Get_roles,
"/realms/{realm_name}/roles/{role_id}": Endpoints.get_Get_role,
"/realms/{realm_name}/seawatch/v1/security-events": Endpoints.get_Get_security_events,
"/realms/{realm_name}/settings/maintenance/whitelist": Endpoints.get_Get_realm_whitelist,
"/realms/{realm_name}/smtp-config": Endpoints.get_Get_smtp_config,
"/realms/{realm_name}/users": Endpoints.get_Get_users,
"/realms/{realm_name}/users/@me/realms": Endpoints.get_Get_user_realms,
"/realms/{realm_name}/users/me": Endpoints.get_Get_own_profile,
"/realms/{realm_name}/users/me/credentials": Endpoints.get_List_own_credentials,"/realms/{realm_name}/keys": Endpoints.get_List_signing_keys,

"/realms/{realm_name}/users/me/sessions": Endpoints.get_List_own_sessions,
"/realms/{realm_name}/users/{user_id}": Endpoints.get_Get_user,
"/realms/{realm_name}/users/{user_id}/attributes": Endpoints.get_Get_user_attributes,
"/realms/{realm_name}/users/{user_id}/credentials": Endpoints.get_Get_user_credentials,
"/realms/{realm_name}/users/{user_id}/identity-provider-links": Endpoints.get_List_identity_provider_links,
"/realms/{realm_name}/users/{user_id}/organizations": Endpoints.get_List_user_organizations,
"/realms/{realm_name}/users/{user_id}/permissions": Endpoints.get_Get_user_permissions,
"/realms/{realm_name}/users/{user_id}/roles": Endpoints.get_Get_user_roles,
"/realms/{realm_name}/users/{user_id}/sessions": Endpoints.get_List_user_sessions,
"/realms/{realm_name}/webhooks": Endpoints.get_Fetch_webhooks,
"/realms/{realm_name}/webhooks/{webhook_id}": Endpoints.get_Get_webhook,
"/realms/{realm_name}/webhooks/{webhook_id}/deliveries": Endpoints.get_Fetch_deliveries,
"/realms/{realm_name}/webhooks/{webhook_id}/deliveries/{delivery_id}": Endpoints.get_Get_delivery
         },
post: {
           "/realms": Endpoints.post_Create_realm,
"/realms/{realm_name}/auth/consent": Endpoints.post_Post_consent,
"/realms/{realm_name}/authz/policies": Endpoints.post_Create_policy,
"/realms/{realm_name}/authz/policies/validate": Endpoints.post_Validate_policies,
"/realms/{realm_name}/authz/simulate": Endpoints.post_Simulate,
"/realms/{realm_name}/authz/templates": Endpoints.post_Create_policy_template,
"/realms/{realm_name}/authzen/access/v1/evaluation": Endpoints.post_Evaluate,
"/realms/{realm_name}/authzen/access/v1/evaluations": Endpoints.post_Evaluations,
"/realms/{realm_name}/broker/{alias}/endpoint": Endpoints.post_Broker_callback,
"/realms/{realm_name}/client-scopes": Endpoints.post_Create_client_scope,
"/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers": Endpoints.post_Create_protocol_mapper,
"/realms/{realm_name}/clients": Endpoints.post_Create_client,
"/realms/{realm_name}/clients/register": Endpoints.post_Register_client,
"/realms/{realm_name}/clients/{client_id}/evaluate-scopes": Endpoints.post_Evaluate_client_scopes,
"/realms/{realm_name}/clients/{client_id}/maintenance/whitelist": Endpoints.post_Add_client_whitelist_entry,
"/realms/{realm_name}/clients/{client_id}/post-logout-redirects": Endpoints.post_Create_post_logout_redirect_uri,
"/realms/{realm_name}/clients/{client_id}/redirects": Endpoints.post_Create_redirect_uri,
"/realms/{realm_name}/clients/{client_id}/roles": Endpoints.post_Create_client_role,
"/realms/{realm_name}/clients/{client_id}/saml-attribute-mappers": Endpoints.post_Create_saml_attribute_mapper,
"/realms/{realm_name}/clients/{client_id}/token-exchange-policies": Endpoints.post_Create_token_exchange_policy,
"/realms/{realm_name}/clients/{client_id}/web-origins": Endpoints.post_Create_web_origin,
"/realms/{realm_name}/device/verify": Endpoints.post_Device_verify,
"/realms/{realm_name}/email-templates": Endpoints.post_Create_template,
"/realms/{realm_name}/email-templates/import": Endpoints.post_Import_template,
"/realms/{realm_name}/federation/providers": Endpoints.post_Create_provider,
"/realms/{realm_name}/federation/providers/{id}/sync-users": Endpoints.post_Sync_users,
"/realms/{realm_name}/federation/providers/{id}/test-connection": Endpoints.post_Test_connection,
"/realms/{realm_name}/identity-providers": Endpoints.post_Create_identity_provider,
"/realms/{realm_name}/login-actions/accounts/select": Endpoints.post_Select_account,
"/realms/{realm_name}/login-actions/authenticate": Endpoints.post_Authenticate,
"/realms/{realm_name}/login-actions/burn-recovery-code": Endpoints.post_Burn_recovery_code,
"/realms/{realm_name}/login-actions/challenge-otp": Endpoints.post_Challenge_otp,
"/realms/{realm_name}/login-actions/forgot-password": Endpoints.post_Forgot_password,
"/realms/{realm_name}/login-actions/generate-recovery-codes": Endpoints.post_Generate_recovery_codes,
"/realms/{realm_name}/login-actions/passkey-authenticate": Endpoints.post_Passkey_authenticate,
"/realms/{realm_name}/login-actions/passkey-request-options": Endpoints.post_Passkey_request_options,
"/realms/{realm_name}/login-actions/resend-verification-email": Endpoints.post_Resend_verification_email_handler,
"/realms/{realm_name}/login-actions/reset-password": Endpoints.post_Reset_password_with_token,
"/realms/{realm_name}/login-actions/send-magic-link": Endpoints.post_Send_magic_link,
"/realms/{realm_name}/login-actions/update-password": Endpoints.post_Update_password,
"/realms/{realm_name}/login-actions/verify-email": Endpoints.post_Verify_email_handler,
"/realms/{realm_name}/login-actions/verify-otp": Endpoints.post_Verify_otp,
"/realms/{realm_name}/login-actions/verify-reset-token": Endpoints.post_Verify_reset_token,
"/realms/{realm_name}/login-actions/webauthn-public-key-authenticate": Endpoints.post_Webauthn_public_key_authenticate,
"/realms/{realm_name}/login-actions/webauthn-public-key-create": Endpoints.post_Webauthn_public_key_create,
"/realms/{realm_name}/login-actions/webauthn-public-key-create-options": Endpoints.post_Webauthn_public_key_create_options,
"/realms/{realm_name}/login-actions/webauthn-public-key-request-options": Endpoints.post_Webauthn_public_key_request_options,
"/realms/{realm_name}/organizations": Endpoints.post_Create_organization,
"/realms/{realm_name}/organizations/{organization_id}/groups": Endpoints.post_Create_group,
"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/members": Endpoints.post_Add_group_member,
"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/roles": Endpoints.post_Assign_group_role,
"/realms/{realm_name}/organizations/{organization_id}/members": Endpoints.post_Add_member,
"/realms/{realm_name}/organizations/{organization_id}/members/{user_id}/roles": Endpoints.post_Assign_member_role,
"/realms/{realm_name}/portal-layouts": Endpoints.post_Create_layout,
"/realms/{realm_name}/portal-layouts/import": Endpoints.post_Import_layout,
"/realms/{realm_name}/portal/themes": Endpoints.post_Create_theme,
"/realms/{realm_name}/portal/themes/import": Endpoints.post_Import_theme,
"/realms/{realm_name}/portal/themes/{theme_id}/activate": Endpoints.post_Activate_theme,
"/realms/{realm_name}/protocol/openid-connect/auth/device": Endpoints.post_Device_authorization,
"/realms/{realm_name}/protocol/openid-connect/logout": Endpoints.post_Logout_post,
"/realms/{realm_name}/protocol/openid-connect/registrations": Endpoints.post_Registration_handler,
"/realms/{realm_name}/protocol/openid-connect/revoke": Endpoints.post_Revoke_token,
"/realms/{realm_name}/protocol/openid-connect/token": Endpoints.post_Exchange_token,
"/realms/{realm_name}/protocol/openid-connect/token/introspect": Endpoints.post_Introspect_token,
"/realms/{realm_name}/protocol/saml": Endpoints.post_Saml_sso_post,
"/realms/{realm_name}/roles": Endpoints.post_Create_realm_role,
"/realms/{realm_name}/settings/maintenance/whitelist": Endpoints.post_Add_realm_whitelist_entry,
"/realms/{realm_name}/users": Endpoints.post_Create_user,
"/realms/{realm_name}/users/me/mfa/otp": Endpoints.post_Start_own_otp_enrollment,"/realms/{realm_name}/keys/rotate": Endpoints.post_Rotate_signing_key,
"/realms/{realm_name}/keys/{kid}/revoke": Endpoints.post_Revoke_signing_key,

"/realms/{realm_name}/users/me/sessions/revoke-others": Endpoints.post_Revoke_other_own_sessions,
"/realms/{realm_name}/users/me/passkeys": Endpoints.post_Confirm_own_passkey_registration,
"/realms/{realm_name}/users/me/passkeys/options": Endpoints.post_Start_own_passkey_registration,
"/realms/{realm_name}/users/me/reauthenticate": Endpoints.post_Request_elevation,
"/realms/{realm_name}/users/{user_id}/credentials/import": Endpoints.post_Import_password_credential,
"/realms/{realm_name}/users/{user_id}/roles/{role_id}": Endpoints.post_Assign_role,
"/realms/{realm_name}/users/{user_id}/unlock": Endpoints.post_Unlock_user,
"/realms/{realm_name}/webhooks": Endpoints.post_Create_webhook,
"/realms/{realm_name}/webhooks/{webhook_id}/deliveries/{delivery_id}/retry": Endpoints.post_Retry_delivery,
"/realms/{realm_name}/webhooks/{webhook_id}/secret/rotate": Endpoints.post_Rotate_secret
         },
put: {
           "/realms/{name}": Endpoints.put_Update_realm,
"/realms/{name}/settings": Endpoints.put_Update_realm_setting,
"/realms/{realm_name}/authz/policies/{policy_id}": Endpoints.put_Update_policy,
"/realms/{realm_name}/authz/schema": Endpoints.put_Put_schema,
"/realms/{realm_name}/authz/state": Endpoints.put_Set_state,
"/realms/{realm_name}/authz/templates/{template_id}": Endpoints.put_Update_policy_template,
"/realms/{realm_name}/clients/{client_id}/default-client-scopes/{scope_id}": Endpoints.put_Assign_default_scope,
"/realms/{realm_name}/clients/{client_id}/maintenance": Endpoints.put_Toggle_maintenance,
"/realms/{realm_name}/clients/{client_id}/optional-client-scopes/{scope_id}": Endpoints.put_Assign_optional_scope,
"/realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id}": Endpoints.put_Update_post_logout_redirect_uri,
"/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}": Endpoints.put_Update_redirect_uri,
"/realms/{realm_name}/clients/{client_id}/saml-config": Endpoints.put_Set_client_saml_config,
"/realms/{realm_name}/email-templates/{template_id}": Endpoints.put_Update_template,
"/realms/{realm_name}/federation/providers/{id}": Endpoints.put_Update_provider,
"/realms/{realm_name}/identity-providers/{alias}": Endpoints.put_Update_identity_provider,
"/realms/{realm_name}/organizations/{organization_id}": Endpoints.put_Update_organization,
"/realms/{realm_name}/organizations/{organization_id}/attributes/{key}": Endpoints.put_Upsert_attribute,
"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}": Endpoints.put_Update_group,
"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/attributes/{key}": Endpoints.put_Upsert_group_attribute,
"/realms/{realm_name}/password-policy": Endpoints.put_Update_password_policy,
"/realms/{realm_name}/portal-layouts/{layout_id}": Endpoints.put_Update_layout,
"/realms/{realm_name}/portal-layouts/{layout_id}/default": Endpoints.put_Set_default_layout,
"/realms/{realm_name}/portal/theme": Endpoints.put_Update_theme,
"/realms/{realm_name}/portal/themes/{theme_id}": Endpoints.put_Update_theme_metadata,
"/realms/{realm_name}/portal/themes/{theme_id}/pages/{page_type}": Endpoints.put_Update_theme_page,
"/realms/{realm_name}/resource-owners": Endpoints.put_Replace_resource_owners,
"/realms/{realm_name}/roles/{role_id}": Endpoints.put_Update_role,
"/realms/{realm_name}/smtp-config": Endpoints.put_Upsert_smtp_config,
"/realms/{realm_name}/users/@me/locale": Endpoints.put_Update_me_locale,
"/realms/{realm_name}/users/me": Endpoints.put_Update_own_profile,
"/realms/{realm_name}/users/me/mfa/otp": Endpoints.put_Confirm_own_otp_enrollment,
"/realms/{realm_name}/users/me/password": Endpoints.put_Change_own_password,
"/realms/{realm_name}/users/{user_id}": Endpoints.put_Update_user,
"/realms/{realm_name}/users/{user_id}/attributes": Endpoints.put_Set_user_attributes,
"/realms/{realm_name}/users/{user_id}/reset-password": Endpoints.put_Reset_password,
"/realms/{realm_name}/webhooks/{webhook_id}": Endpoints.put_Update_webhook
         },
delete: {
           "/realms/{name}": Endpoints.delete_Delete_realm,
"/realms/{realm_name}/authz/policies/{policy_id}": Endpoints.delete_Delete_policy,
"/realms/{realm_name}/authz/templates/{template_id}": Endpoints.delete_Delete_policy_template,
"/realms/{realm_name}/client-scopes/{scope_id}": Endpoints.delete_Delete_client_scope,
"/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id}": Endpoints.delete_Delete_protocol_mapper,
"/realms/{realm_name}/clients/{client_id}": Endpoints.delete_Delete_client,
"/realms/{realm_name}/clients/{client_id}/default-client-scopes/{scope_id}": Endpoints.delete_Unassign_default_scope,
"/realms/{realm_name}/clients/{client_id}/maintenance/whitelist/{entry_id}": Endpoints.delete_Remove_client_whitelist_entry,
"/realms/{realm_name}/clients/{client_id}/optional-client-scopes/{scope_id}": Endpoints.delete_Unassign_optional_scope,
"/realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id}": Endpoints.delete_Delete_post_logout_redirect_uri,
"/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}": Endpoints.delete_Delete_redirect_uri,
"/realms/{realm_name}/clients/{client_id}/saml-attribute-mappers/{mapper_id}": Endpoints.delete_Delete_saml_attribute_mapper,
"/realms/{realm_name}/clients/{client_id}/token-exchange-policies/{policy_id}": Endpoints.delete_Delete_token_exchange_policy,
"/realms/{realm_name}/clients/{client_id}/web-origins/{web_origin_id}": Endpoints.delete_Delete_web_origin,
"/realms/{realm_name}/email-templates/{template_id}": Endpoints.delete_Delete_template,
"/realms/{realm_name}/federation/providers/{id}": Endpoints.delete_Delete_provider,
"/realms/{realm_name}/identity-providers/{alias}": Endpoints.delete_Delete_identity_provider,
"/realms/{realm_name}/login-actions/accounts/{session_id}": Endpoints.delete_Remove_account,
"/realms/{realm_name}/organizations/{organization_id}": Endpoints.delete_Delete_organization,
"/realms/{realm_name}/organizations/{organization_id}/attributes/{key}": Endpoints.delete_Delete_attribute,
"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}": Endpoints.delete_Delete_group,
"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/attributes/{key}": Endpoints.delete_Delete_group_attribute,
"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/members/{user_id}": Endpoints.delete_Remove_group_member,
"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/roles/{role_id}": Endpoints.delete_Revoke_group_role,
"/realms/{realm_name}/organizations/{organization_id}/members/{user_id}": Endpoints.delete_Remove_member,
"/realms/{realm_name}/organizations/{organization_id}/members/{user_id}/roles/{role_id}": Endpoints.delete_Revoke_member_role,
"/realms/{realm_name}/portal-layouts/{layout_id}": Endpoints.delete_Delete_layout,
"/realms/{realm_name}/portal/themes/{theme_id}": Endpoints.delete_Delete_theme,
"/realms/{realm_name}/roles/{role_id}": Endpoints.delete_Delete_role,
"/realms/{realm_name}/settings/maintenance/whitelist/{entry_id}": Endpoints.delete_Remove_realm_whitelist_entry,
"/realms/{realm_name}/smtp-config": Endpoints.delete_Delete_smtp_config,
"/realms/{realm_name}/users/bulk": Endpoints.delete_Bulk_delete_user,
"/realms/{realm_name}/users/me/mfa/otp": Endpoints.delete_Disable_own_otp,
"/realms/{realm_name}/users/me/passkeys/{credential_id}": Endpoints.delete_Delete_own_passkey,
"/realms/{realm_name}/users/me/sessions/{session_id}": Endpoints.delete_Revoke_own_session,
"/realms/{realm_name}/users/{user_id}": Endpoints.delete_Delete_user,
"/realms/{realm_name}/users/{user_id}/attributes/{key}": Endpoints.delete_Delete_user_attribute,
"/realms/{realm_name}/users/{user_id}/credentials/{credential_id}": Endpoints.delete_Delete_user_credential,
"/realms/{realm_name}/users/{user_id}/identity-provider-links/{link_id}": Endpoints.delete_Delete_identity_provider_link,
"/realms/{realm_name}/users/{user_id}/roles/{role_id}": Endpoints.delete_Unassign_role,
"/realms/{realm_name}/users/{user_id}/sessions/{session_id}": Endpoints.delete_Revoke_user_session,
"/realms/{realm_name}/webhooks/{webhook_id}": Endpoints.delete_Delete_webhook
         },
patch: {
           "/realms/{realm_name}/client-scopes/{scope_id}": Endpoints.patch_Update_client_scope,
"/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id}": Endpoints.patch_Update_protocol_mapper,
"/realms/{realm_name}/clients/{client_id}": Endpoints.patch_Update_client,
"/realms/{realm_name}/roles/{role_id}/permissions": Endpoints.patch_Update_role_permissions
         }
     }

     // </EndpointByMethod>


    // <EndpointByMethod.Shorthands>
    export type GetEndpoints = EndpointByMethod["get"]
export type PostEndpoints = EndpointByMethod["post"]
export type PutEndpoints = EndpointByMethod["put"]
export type DeleteEndpoints = EndpointByMethod["delete"]
export type PatchEndpoints = EndpointByMethod["patch"]
    // </EndpointByMethod.Shorthands>


// <ApiClientTypes>
export type EndpointParameters = {
  body?: unknown;
  query?: unknown;
  header?: unknown;
  path?: unknown;
  cookie?: unknown;
};

export type MutationMethod = "post" | "put" | "patch" | "delete";
export type Method = "get" | "head" | "options" | "trace" | MutationMethod;

export type RequestFormat = "json" | "form-data" | "form-url" | "binary" | "text";
export type ResponseFormat = "json" | "sse";
export type SecurityRequirements = readonly (readonly string[])[];


    // <EndpointRequestFormats>
    /** Non-json request body encodings; missing entries default to `"json"`. */
    export const endpointRequestFormats = {
    post: {
          "/realms/{realm_name}/protocol/openid-connect/auth/device": "form-url",
"/realms/{realm_name}/protocol/saml": "form-url"
        }
    } as Partial<{ [M in keyof EndpointByMethod]: Partial<{ [P in keyof EndpointByMethod[M]]: RequestFormat }> }>;
    // </EndpointRequestFormats>


    // <EndpointParameterStyles>
    export type ParameterSerialization = { style: string; explode: boolean; allowReserved: boolean };
    export type EndpointParameterStyles = Partial<Record<"query" | "path" | "header" | "cookie", Record<string, ParameterSerialization>>>;
    /** OpenAPI parameter styles used by the built-in encoders. */
    export const endpointParameterStyles = {"get":{"/.well-known/authzen-configuration/realms/{realm_name}/authzen":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/email-templates/variables/{email_type}":{"path":{"email_type":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{name}":{"path":{"name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{name}/login-settings":{"path":{"name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/.well-known/oauth-authorization-server/realms/{realm_name}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/.well-known/openid-configuration":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/auth/consent":{"query":{"consent_token":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/policies":{"query":{"managed_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/policies/{policy_id}":{"query":{"managed_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"policy_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/schema":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/schema/builtin":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/state":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/templates":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/templates/{template_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"template_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/broker/{alias}/login":{"query":{"client_id":{"style":"form","explode":true,"allowReserved":false},"redirect_uri":{"style":"form","explode":true,"allowReserved":false},"response_type":{"style":"form","explode":true,"allowReserved":false},"scope":{"style":"form","explode":true,"allowReserved":false},"state":{"style":"form","explode":true,"allowReserved":false},"nonce":{"style":"form","explode":true,"allowReserved":false},"code_challenge":{"style":"form","explode":true,"allowReserved":false},"code_challenge_method":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"alias":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/client-scopes":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"name":{"style":"form","explode":true,"allowReserved":false},"description":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"protocol":{"style":"form","explode":true,"allowReserved":false},"default_scope_type":{"style":"form","explode":true,"allowReserved":false},"has_protocol_mappers":{"style":"form","explode":true,"allowReserved":false},"not_assigned_to_client":{"style":"form","explode":true,"allowReserved":false},"created_from":{"style":"form","explode":true,"allowReserved":false},"created_to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/client-scopes/{scope_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"scope_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"name":{"style":"form","explode":true,"allowReserved":false},"client_id":{"style":"form","explode":true,"allowReserved":false},"client_id_exact":{"style":"form","explode":true,"allowReserved":false},"enabled":{"style":"form","explode":true,"allowReserved":false},"public_client":{"style":"form","explode":true,"allowReserved":false},"service_account_enabled":{"style":"form","explode":true,"allowReserved":false},"oauth_device_code_grant_enabled":{"style":"form","explode":true,"allowReserved":false},"protocol":{"style":"form","explode":true,"allowReserved":false},"client_type":{"style":"form","explode":true,"allowReserved":false},"application_type":{"style":"form","explode":true,"allowReserved":false},"has_redirect_uris":{"style":"form","explode":true,"allowReserved":false},"maintenance_enabled":{"style":"form","explode":true,"allowReserved":false},"ids":{"style":"form","explode":true,"allowReserved":false},"created_from":{"style":"form","explode":true,"allowReserved":false},"created_to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/client-scopes":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/client-secret":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/maintenance/whitelist":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/post-logout-redirects":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/redirects":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/roles":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/saml-attribute-mappers":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/saml-config":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/token-exchange-policies":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/web-origins":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/compass/v1/activity/daily":{"query":{"from":{"style":"form","explode":true,"allowReserved":false},"to":{"style":"form","explode":true,"allowReserved":false},"client_id":{"style":"form","explode":true,"allowReserved":false},"user_id":{"style":"form","explode":true,"allowReserved":false},"grant_type":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/compass/v1/flows":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"client_id":{"style":"form","explode":true,"allowReserved":false},"user_id":{"style":"form","explode":true,"allowReserved":false},"grant_type":{"style":"form","explode":true,"allowReserved":false},"status":{"style":"form","explode":true,"allowReserved":false},"ip_address":{"style":"form","explode":true,"allowReserved":false},"identified":{"style":"form","explode":true,"allowReserved":false},"completed":{"style":"form","explode":true,"allowReserved":false},"from":{"style":"form","explode":true,"allowReserved":false},"to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/compass/v1/flows/{flow_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"flow_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/compass/v1/stats":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/device":{"query":{"user_code":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/device/preview":{"query":{"user_code":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/email-templates":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"name":{"style":"form","explode":true,"allowReserved":false},"email_type":{"style":"form","explode":true,"allowReserved":false},"created_from":{"style":"form","explode":true,"allowReserved":false},"created_to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/email-templates/{template_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"template_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/email-templates/{template_id}/export":{"query":{"format":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"template_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/federation/providers":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"name":{"style":"form","explode":true,"allowReserved":false},"provider_type":{"style":"form","explode":true,"allowReserved":false},"provider_family":{"style":"form","explode":true,"allowReserved":false},"enabled":{"style":"form","explode":true,"allowReserved":false},"sync_enabled":{"style":"form","explode":true,"allowReserved":false},"synced":{"style":"form","explode":true,"allowReserved":false},"sync_mode":{"style":"form","explode":true,"allowReserved":false},"created_from":{"style":"form","explode":true,"allowReserved":false},"created_to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/federation/providers/{id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/identity-providers":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"alias":{"style":"form","explode":true,"allowReserved":false},"display_name":{"style":"form","explode":true,"allowReserved":false},"provider_id":{"style":"form","explode":true,"allowReserved":false},"enabled":{"style":"form","explode":true,"allowReserved":false},"health":{"style":"form","explode":true,"allowReserved":false},"created_from":{"style":"form","explode":true,"allowReserved":false},"created_to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/identity-providers/{alias}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"alias":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/accounts":{"query":{"client_id":{"style":"form","explode":true,"allowReserved":false},"max_age":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/setup-otp":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/verify-magic-link":{"query":{"token_id":{"style":"form","explode":true,"allowReserved":false},"magic_token":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"name":{"style":"form","explode":true,"allowReserved":false},"alias":{"style":"form","explode":true,"allowReserved":false},"domain":{"style":"form","explode":true,"allowReserved":false},"description":{"style":"form","explode":true,"allowReserved":false},"enabled":{"style":"form","explode":true,"allowReserved":false},"has_domain":{"style":"form","explode":true,"allowReserved":false},"without_member":{"style":"form","explode":true,"allowReserved":false},"ids":{"style":"form","explode":true,"allowReserved":false},"created_from":{"style":"form","explode":true,"allowReserved":false},"created_to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/attributes":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/groups":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"name":{"style":"form","explode":true,"allowReserved":false},"description":{"style":"form","explode":true,"allowReserved":false},"parent_group_id":{"style":"form","explode":true,"allowReserved":false},"is_root":{"style":"form","explode":true,"allowReserved":false},"ids":{"style":"form","explode":true,"allowReserved":false},"created_from":{"style":"form","explode":true,"allowReserved":false},"created_to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"group_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/attributes":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"group_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/members":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"username":{"style":"form","explode":true,"allowReserved":false},"email":{"style":"form","explode":true,"allowReserved":false},"enabled":{"style":"form","explode":true,"allowReserved":false},"created_from":{"style":"form","explode":true,"allowReserved":false},"created_to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"group_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/roles":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"group_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/members":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"username":{"style":"form","explode":true,"allowReserved":false},"email":{"style":"form","explode":true,"allowReserved":false},"enabled":{"style":"form","explode":true,"allowReserved":false},"created_from":{"style":"form","explode":true,"allowReserved":false},"created_to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/members/{user_id}/roles":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/password-policy":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/password-policy/public":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal-layouts":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"name":{"style":"form","explode":true,"allowReserved":false},"is_default":{"style":"form","explode":true,"allowReserved":false},"in_use":{"style":"form","explode":true,"allowReserved":false},"ids":{"style":"form","explode":true,"allowReserved":false},"created_from":{"style":"form","explode":true,"allowReserved":false},"created_to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal-layouts/public/default":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal-layouts/public/{layout_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"layout_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal-layouts/{layout_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"layout_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal-layouts/{layout_id}/export":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"layout_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal/active":{"query":{"page_type":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal/page-requirements":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal/theme":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal/themes":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"name":{"style":"form","explode":true,"allowReserved":false},"layout_id":{"style":"form","explode":true,"allowReserved":false},"activatable":{"style":"form","explode":true,"allowReserved":false},"created_from":{"style":"form","explode":true,"allowReserved":false},"created_to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal/themes/{theme_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"theme_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal/themes/{theme_id}/export":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"theme_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/protocol/openid-connect/auth":{"query":{"response_type":{"style":"form","explode":true,"allowReserved":false},"client_id":{"style":"form","explode":true,"allowReserved":false},"redirect_uri":{"style":"form","explode":true,"allowReserved":false},"scope":{"style":"form","explode":true,"allowReserved":false},"state":{"style":"form","explode":true,"allowReserved":false},"nonce":{"style":"form","explode":true,"allowReserved":false},"code_challenge":{"style":"form","explode":true,"allowReserved":false},"code_challenge_method":{"style":"form","explode":true,"allowReserved":false},"prompt":{"style":"form","explode":true,"allowReserved":false},"max_age":{"style":"form","explode":true,"allowReserved":false},"resource":{"style":"form","explode":true,"allowReserved":false},"login_hint":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/protocol/openid-connect/certs":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/protocol/openid-connect/jwks.json":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/protocol/openid-connect/logout":{"query":{"id_token_hint":{"style":"form","explode":true,"allowReserved":false},"post_logout_redirect_uri":{"style":"form","explode":true,"allowReserved":false},"state":{"style":"form","explode":true,"allowReserved":false},"client_id":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/protocol/openid-connect/userinfo":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/protocol/saml":{"query":{"SAMLRequest":{"style":"form","explode":true,"allowReserved":false},"RelayState":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/protocol/saml/continue":{"query":{"code":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/protocol/saml/descriptor":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/resource-owners":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/roles":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"name":{"style":"form","explode":true,"allowReserved":false},"qualified_name":{"style":"form","explode":true,"allowReserved":false},"description":{"style":"form","explode":true,"allowReserved":false},"require_mfa":{"style":"form","explode":true,"allowReserved":false},"client_id":{"style":"form","explode":true,"allowReserved":false},"scope":{"style":"form","explode":true,"allowReserved":false},"has_permissions":{"style":"form","explode":true,"allowReserved":false},"ids":{"style":"form","explode":true,"allowReserved":false},"created_from":{"style":"form","explode":true,"allowReserved":false},"created_to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/roles/{role_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"role_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/seawatch/v1/security-events":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"actor_id":{"style":"form","explode":true,"allowReserved":false},"client_id":{"style":"form","explode":true,"allowReserved":false},"event_types":{"style":"form","explode":true,"allowReserved":false},"status":{"style":"form","explode":true,"allowReserved":false},"target_type":{"style":"form","explode":true,"allowReserved":false},"from_timestamp":{"style":"form","explode":true,"allowReserved":false},"to_timestamp":{"style":"form","explode":true,"allowReserved":false},"ip_address":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/settings/maintenance/whitelist":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/smtp-config":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"username":{"style":"form","explode":true,"allowReserved":false},"email":{"style":"form","explode":true,"allowReserved":false},"firstname":{"style":"form","explode":true,"allowReserved":false},"lastname":{"style":"form","explode":true,"allowReserved":false},"enabled":{"style":"form","explode":true,"allowReserved":false},"email_verified":{"style":"form","explode":true,"allowReserved":false},"service_account":{"style":"form","explode":true,"allowReserved":false},"role_id":{"style":"form","explode":true,"allowReserved":false},"ids":{"style":"form","explode":true,"allowReserved":false},"not_in_group":{"style":"form","explode":true,"allowReserved":false},"not_in_organization":{"style":"form","explode":true,"allowReserved":false},"created_from":{"style":"form","explode":true,"allowReserved":false},"created_to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/@me/realms":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"name":{"style":"form","explode":true,"allowReserved":false},"display_name":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/me":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/me/credentials":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/keys":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/me/sessions":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/attributes":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/credentials":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/identity-provider-links":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/organizations":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/permissions":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/roles":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/sessions":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"ip_address":{"style":"form","explode":true,"allowReserved":false},"user_agent":{"style":"form","explode":true,"allowReserved":false},"persistent":{"style":"form","explode":true,"allowReserved":false},"created_from":{"style":"form","explode":true,"allowReserved":false},"created_to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/webhooks":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"name":{"style":"form","explode":true,"allowReserved":false},"endpoint":{"style":"form","explode":true,"allowReserved":false},"triggered":{"style":"form","explode":true,"allowReserved":false},"has_subscribers":{"style":"form","explode":true,"allowReserved":false},"secure_endpoint":{"style":"form","explode":true,"allowReserved":false},"created_from":{"style":"form","explode":true,"allowReserved":false},"created_to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/webhooks/{webhook_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"webhook_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/webhooks/{webhook_id}/deliveries":{"query":{"page":{"style":"form","explode":true,"allowReserved":false},"limit":{"style":"form","explode":true,"allowReserved":false},"order":{"style":"form","explode":true,"allowReserved":false},"search":{"style":"form","explode":true,"allowReserved":false},"event":{"style":"form","explode":true,"allowReserved":false},"status":{"style":"form","explode":true,"allowReserved":false},"resource_id":{"style":"form","explode":true,"allowReserved":false},"created_from":{"style":"form","explode":true,"allowReserved":false},"created_to":{"style":"form","explode":true,"allowReserved":false},"order_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"webhook_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/webhooks/{webhook_id}/deliveries/{delivery_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"webhook_id":{"style":"simple","explode":false,"allowReserved":false},"delivery_id":{"style":"simple","explode":false,"allowReserved":false}}}},"put":{"/realms/{name}":{"path":{"name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{name}/settings":{"path":{"name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/policies/{policy_id}":{"query":{"managed_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"policy_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/schema":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/state":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/templates/{template_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"template_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/default-client-scopes/{scope_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false},"scope_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/maintenance":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/optional-client-scopes/{scope_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false},"scope_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false},"uri_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false},"uri_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/saml-config":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/email-templates/{template_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"template_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/federation/providers/{id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/identity-providers/{alias}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"alias":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/attributes/{key}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"key":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"group_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/attributes/{key}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"group_id":{"style":"simple","explode":false,"allowReserved":false},"key":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/password-policy":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal-layouts/{layout_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"layout_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal-layouts/{layout_id}/default":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"layout_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal/theme":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal/themes/{theme_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"theme_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal/themes/{theme_id}/pages/{page_type}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"theme_id":{"style":"simple","explode":false,"allowReserved":false},"page_type":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/resource-owners":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/roles/{role_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"role_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/smtp-config":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/@me/locale":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/me":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/me/mfa/otp":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/me/password":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/attributes":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/reset-password":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/webhooks/{webhook_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"webhook_id":{"style":"simple","explode":false,"allowReserved":false}}}},"delete":{"/realms/{name}":{"path":{"name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/policies/{policy_id}":{"query":{"managed_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"policy_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/templates/{template_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"template_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/client-scopes/{scope_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"scope_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"scope_id":{"style":"simple","explode":false,"allowReserved":false},"mapper_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/default-client-scopes/{scope_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false},"scope_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/maintenance/whitelist/{entry_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false},"entry_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/optional-client-scopes/{scope_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false},"scope_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false},"uri_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false},"uri_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/saml-attribute-mappers/{mapper_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false},"mapper_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/token-exchange-policies/{policy_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false},"policy_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/web-origins/{web_origin_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false},"web_origin_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/email-templates/{template_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"template_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/federation/providers/{id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/identity-providers/{alias}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"alias":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/accounts/{session_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"session_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/attributes/{key}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"key":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"group_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/attributes/{key}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"group_id":{"style":"simple","explode":false,"allowReserved":false},"key":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/members/{user_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"group_id":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/roles/{role_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"group_id":{"style":"simple","explode":false,"allowReserved":false},"role_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/members/{user_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/members/{user_id}/roles/{role_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false},"role_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal-layouts/{layout_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"layout_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal/themes/{theme_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"theme_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/roles/{role_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"role_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/settings/maintenance/whitelist/{entry_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"entry_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/smtp-config":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/bulk":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/me/mfa/otp":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/me/sessions/{session_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"session_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/me/passkeys/{credential_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"credential_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/attributes/{key}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false},"key":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/credentials/{credential_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false},"credential_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/identity-provider-links/{link_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false},"link_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/roles/{role_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false},"role_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/sessions/{session_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false},"session_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/webhooks/{webhook_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"webhook_id":{"style":"simple","explode":false,"allowReserved":false}}}},"post":{"/realms/{realm_name}/auth/consent":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/policies":{"query":{"managed_by":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/policies/validate":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/simulate":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authz/templates":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authzen/access/v1/evaluation":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/authzen/access/v1/evaluations":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/broker/{alias}/endpoint":{"query":{"code":{"style":"form","explode":true,"allowReserved":false},"state":{"style":"form","explode":true,"allowReserved":false},"error":{"style":"form","explode":true,"allowReserved":false},"error_description":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"alias":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/client-scopes":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"scope_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/register":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/evaluate-scopes":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/maintenance/whitelist":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/post-logout-redirects":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/redirects":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/roles":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/saml-attribute-mappers":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/token-exchange-policies":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}/web-origins":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/device/verify":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/email-templates":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/email-templates/import":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/federation/providers":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/federation/providers/{id}/sync-users":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/federation/providers/{id}/test-connection":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/identity-providers":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/accounts/select":{"query":{"max_age":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/authenticate":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/burn-recovery-code":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/challenge-otp":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/forgot-password":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/generate-recovery-codes":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/passkey-authenticate":{"query":{"remember_me":{"style":"form","explode":true,"allowReserved":false}},"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/passkey-request-options":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/resend-verification-email":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/reset-password":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/send-magic-link":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/update-password":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/verify-email":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/verify-otp":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/verify-reset-token":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/webauthn-public-key-authenticate":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/webauthn-public-key-create":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/webauthn-public-key-create-options":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/login-actions/webauthn-public-key-request-options":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/groups":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/members":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"group_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/groups/{group_id}/roles":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"group_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/members":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/organizations/{organization_id}/members/{user_id}/roles":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"organization_id":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal-layouts":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal-layouts/import":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal/themes":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal/themes/import":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/portal/themes/{theme_id}/activate":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"theme_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/protocol/openid-connect/auth/device":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/protocol/openid-connect/logout":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/protocol/openid-connect/registrations":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/protocol/openid-connect/revoke":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/protocol/openid-connect/token":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/protocol/openid-connect/token/introspect":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/protocol/saml":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/roles":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/settings/maintenance/whitelist":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/me/mfa/otp":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/keys/rotate":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/keys/{kid}/revoke":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"kid":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/me/sessions/revoke-others":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/me/passkeys":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/me/passkeys/options":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/me/reauthenticate":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/credentials/import":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/roles/{role_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false},"role_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/users/{user_id}/unlock":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"user_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/webhooks":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/webhooks/{webhook_id}/deliveries/{delivery_id}/retry":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"webhook_id":{"style":"simple","explode":false,"allowReserved":false},"delivery_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/webhooks/{webhook_id}/secret/rotate":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"webhook_id":{"style":"simple","explode":false,"allowReserved":false}}}},"patch":{"/realms/{realm_name}/client-scopes/{scope_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"scope_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"scope_id":{"style":"simple","explode":false,"allowReserved":false},"mapper_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/clients/{client_id}":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"client_id":{"style":"simple","explode":false,"allowReserved":false}}},"/realms/{realm_name}/roles/{role_id}/permissions":{"path":{"realm_name":{"style":"simple","explode":false,"allowReserved":false},"role_id":{"style":"simple","explode":false,"allowReserved":false}}}}} as Partial<Record<string, Partial<Record<string, EndpointParameterStyles>>>>;
    // </EndpointParameterStyles>


    // <EndpointResponseFormats>
    /** Non-json response body modes; missing entries default to `"json"`. SSE skips JSON parse + output validation. */
    export const endpointResponseFormats = {

    } as Partial<{ [M in keyof EndpointByMethod]: Partial<{ [P in keyof EndpointByMethod[M]]: ResponseFormat }> }>;
    // </EndpointResponseFormats>


    // <EndpointSecurityRequirements>
    /** OpenAPI security requirements applied when an endpoint has no explicit entry. */
    export const defaultSecurityRequirements = [] as SecurityRequirements;
    /** Endpoint-specific security requirements that differ from the default. */
    export const endpointSecurityRequirements = {
    get: { "/realms/{realm_name}/users/@me/realms": [["Authorization"]] }
    } as Partial<{ [M in keyof EndpointByMethod]: Partial<{ [P in keyof EndpointByMethod[M]]: SecurityRequirements }> }>;
    // </EndpointSecurityRequirements>


export type DefaultEndpoint = {
  parameters?: EndpointParameters | undefined;
  responses?: Record<string, unknown>;
  responseHeaders?: Record<string, unknown>;
};

export type Endpoint<TConfig extends DefaultEndpoint = DefaultEndpoint> = {
  operationId: string;
  method: Method;
  path: string;
  requestFormat: RequestFormat;
  responseFormat: ResponseFormat;
  parameters?: TConfig["parameters"];
  meta: {
    alias: string;
    hasParameters: boolean;
    areParametersRequired: boolean;
  };
  responses?: TConfig["responses"];
  responseHeaders?: TConfig["responseHeaders"]
};

/**
 * Minimal response surface used by ApiClient — avoids depending on the DOM `Response`
 * global (helpful for Node without DOM lib). Structural typing accepts fetch Response.
 */
export interface FetcherResponse {
  ok: boolean;
  status: number;
  statusText: string;
  headers: {
    get(name: string): string | null;
    getSetCookie?: () => string[];
  };
  /** Present on fetch Response; used for SSE / streaming bodies. */
  body?: ReadableStream<Uint8Array> | null;
  json(): Promise<unknown>;
  text(): Promise<string>;
  arrayBuffer(): Promise<ArrayBuffer>;
  clone(): FetcherResponse;
}

export interface Fetcher {
    decodePathParams?: (path: string, pathParams: unknown, styles?: Record<string, ParameterSerialization>) => string
  encodeSearchParams?: (searchParams: unknown, styles?: Record<string, ParameterSerialization>) => URLSearchParams | undefined
  /** Merge cookie params into request headers (default: Cookie header). */
  encodeCookies?: (cookies: unknown, headers: Headers) => void
    //
    fetch: (input: {
      method: Method;
      url: URL;
      urlSearchParams?: URLSearchParams | undefined;
      parameters?: EndpointParameters | undefined;
      path: string;
      /** How to encode `parameters.body` (from OpenAPI requestBody content type). */
      requestFormat: RequestFormat;
      /** OpenAPI parameter serialization metadata for the current endpoint. */
      parameterStyles?: EndpointParameterStyles;
      /** OpenAPI security requirements for this operation. Empty means no credentials are required. */
      security?: SecurityRequirements;
      overrides?: RequestInit;
      throwOnStatusError?: boolean
    }) => Promise<FetcherResponse>;
    parseResponseData?: (response: FetcherResponse) => Promise<unknown>
}

export const successStatusCodes = [200,201,202,203,204,205,206,207,208,226,300,301,302,303,304,305,306,307,308] as const;
export type SuccessStatusCode = typeof successStatusCodes[number];

export const errorStatusCodes = [400,401,402,403,404,405,406,407,408,409,410,411,412,413,414,415,416,417,418,421,422,423,424,425,426,428,429,431,451,500,501,502,503,504,505,506,507,508,510,511] as const;
export type ErrorStatusCode = typeof errorStatusCodes[number];

// Taken from https://github.com/unjs/fetchdts/blob/ec4eaeab5d287116171fc1efd61f4a1ad34e4609/src/fetch.ts#L3
export interface TypedHeaders<TypedHeaderValues = unknown> extends Omit<Headers, 'append' | 'delete' | 'get' | 'getSetCookie' | 'has' | 'set' | 'forEach'> {
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/append) */
  append: <Name extends Extract<keyof TypedHeaderValues, string> | string & {}> (name: Name, value: Lowercase<Name> extends keyof TypedHeaderValues ? TypedHeaderValues[Lowercase<Name>] : string) => void
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/delete) */
  delete: <Name extends Extract<keyof TypedHeaderValues, string> | string & {}> (name: Name) => void
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/get) */
  get: <Name extends Extract<keyof TypedHeaderValues, string> | string & {}> (name: Name) => (Lowercase<Name> extends keyof TypedHeaderValues ? TypedHeaderValues[Lowercase<Name>] : string) | null
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/getSetCookie) */
  getSetCookie: () => string[]
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/has) */
  has: <Name extends Extract<keyof TypedHeaderValues, string> | string & {}> (name: Name) => boolean
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/set) */
  set: <Name extends Extract<keyof TypedHeaderValues, string> | string & {}> (name: Name, value: Lowercase<Name> extends keyof TypedHeaderValues ? TypedHeaderValues[Lowercase<Name>] : string) => void
  forEach: (callbackfn: (value: TypedHeaderValues[keyof TypedHeaderValues] | string & {}, key: Extract<keyof TypedHeaderValues, string> | string & {}, parent: TypedHeaders<TypedHeaderValues>) => void, thisArg?: unknown) => void
}

/** @see https://developer.mozilla.org/en-US/docs/Web/API/Response */
export interface TypedSuccessResponse<TSuccess, TStatusCode, THeaders> extends Omit<FetcherResponse, "ok" | "status" | "json" | "headers"> {
  ok: true;
  status: TStatusCode;
  headers: never extends THeaders ? FetcherResponse["headers"] : TypedHeaders<THeaders>;
  data: TSuccess;
  /** [MDN Reference](https://developer.mozilla.org/en-US/docs/Web/API/Response/json) */
  json: () => Promise<TSuccess>;
}

/** @see https://developer.mozilla.org/en-US/docs/Web/API/Response */
export interface TypedErrorResponse<TData, TStatusCode, THeaders> extends Omit<FetcherResponse, "ok" | "status" | "json" | "headers"> {
  ok: false;
  status: TStatusCode;
  headers: never extends THeaders ? FetcherResponse["headers"] : TypedHeaders<THeaders>;
  data: TData;
  /** [MDN Reference](https://developer.mozilla.org/en-US/docs/Web/API/Response/json) */
  json: () => Promise<TData>;
}

type StatusCodeFromKey<TKey> = TKey extends `${infer TStatusCode extends number}`
  ? TStatusCode
  : TKey extends number
    ? TKey
    : never;

export type TypedApiResponse<TAllResponses = {}, THeaders = {}> = {
  [K in keyof TAllResponses]: StatusCodeFromKey<K> extends infer TStatusCode extends number
    ? TStatusCode extends SuccessStatusCode
      ? TypedSuccessResponse<TAllResponses[K], TStatusCode, K extends keyof THeaders ? THeaders[K] : never>
      : TypedErrorResponse<TAllResponses[K], TStatusCode, K extends keyof THeaders ? THeaders[K] : never>
    : never;
}[keyof TAllResponses];

export type InferSchemaValue<T> = T;
type InferSchemaInput<T> = T;

export type SafeApiResponse<TEndpoint> = TEndpoint extends { responses: infer TResponses }
  ? TResponses extends Record<string | number, unknown>
    ? TypedApiResponse<InferSchemaValue<TResponses>, TEndpoint extends { responseHeaders: infer THeaders } ? InferSchemaValue<THeaders> : never>
    : never
  : never

export type InferResponseByStatus<TEndpoint, TStatusCode> = Extract<SafeApiResponse<TEndpoint>, { status: TStatusCode }>

/**
 * Success-body payload — InferSchemaValue only on success statuses.
 * Filter with extends {} like the old Extract { data: {} } so unknown bodies (e.g. 304) drop out.
 */
export type InferSuccessData<TEndpoint> = TEndpoint extends { responses: infer TResponses }
  ? {
      [K in keyof TResponses]: StatusCodeFromKey<K> extends infer TStatusCode extends number
        ? TStatusCode extends SuccessStatusCode
          ? Extract<InferSchemaValue<TResponses[K]>, {}>
          : never
        : never;
    }[keyof TResponses]
  : never;

type RequiredKeys<T> = {
  [P in keyof T]-?: undefined extends T[P] ? never : P;
}[keyof T];

type MaybeOptionalArg<T> = RequiredKeys<T> extends never ? [config?: T] : [config: T];
type NotNever<T> = [T] extends [never] ? false : true;

export type ApiQueryOptions = {
  /** Override whether a generated TanStack Query consumes TanStack Query's AbortSignal. */
  consumeQuerySignal?: boolean;
};

/** Call options merged onto inferred endpoint parameters. */
type ApiRequestOptions = {
  overrides?: RequestInit;
  queryOptions?: ApiQueryOptions;
  withResponse?: boolean;
  throwOnStatusError?: boolean;
  validate?: ValidateSide;
};

/** Parameter bag for an endpoint + request options. */
export type ApiCallParams<TEndpoint> = TEndpoint extends { parameters: infer UParams }
  ? NotNever<InferSchemaInput<UParams>> extends true
    ? InferSchemaInput<UParams> & ApiRequestOptions
    : ApiRequestOptions
  : ApiRequestOptions;

/** Resolve response type from withResponse flag on the call config. */
export type ApiCallResult<TEndpoint, TParams> = TParams extends { withResponse: true }
  ? SafeApiResponse<TEndpoint>
  : InferSuccessData<TEndpoint>;

export type ValidateSide = "none" | "input" | "output" | "both";
export type OnValidate = (ctx: {
  side: "input" | "output";
  method: string;
  path: string;
  schema: unknown;
  value: unknown;
}) => unknown | Promise<unknown>;

// </ApiClientTypes>

// <TypedStatusError>
export class TypedStatusError<TData = unknown> extends Error {
  response: TypedErrorResponse<TData, ErrorStatusCode, unknown>;
  status: number;
  constructor(response: TypedErrorResponse<TData, ErrorStatusCode, unknown>) {
    super(`HTTP ${response.status}: ${response.statusText}`);
    this.name = 'TypedStatusError';
    this.response = response;
    this.status = response.status;
  }
}
// </TypedStatusError>




// <ApiClient>
const isPlainObject = (value: unknown): value is Record<string, unknown> => {
  if (value === null || typeof value !== "object") return false;
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
};

export class ApiClient {
  baseUrl: string = "";
  successStatusCodes = successStatusCodes;
  errorStatusCodes = errorStatusCodes;
  validate: ValidateSide = "none";
  onValidate?: OnValidate;

  constructor(
    public fetcher: Fetcher,
    options?: { validate?: ValidateSide; onValidate?: OnValidate },
  ) {
    if (options?.validate !== undefined) this.validate = options.validate;
    if (options?.onValidate) this.onValidate = options.onValidate;
  }

  setBaseUrl(baseUrl: string) {
    this.baseUrl = baseUrl;
    return this;
  }

  setValidate(validate: ValidateSide) {
    this.validate = validate;
    return this;
  }

  setOnValidate(onValidate: OnValidate | undefined) {
    if (onValidate === undefined) {
      delete this.onValidate;
    } else {
      this.onValidate = onValidate;
    }
    return this;
  }

  /**
   * Replace path parameters in URL
   * Supports both OpenAPI format {param} and Express format :param
   */
  defaultDecodePathParams = (url: string, params: unknown, styles?: Record<string, ParameterSerialization>): string => {
    const record = (params ?? {}) as Record<string, unknown>;
    const encode = (value: unknown) => encodeURIComponent(String(value));
    const serialize = (key: string, value: unknown): string => {
      const parameterStyle = styles?.[key];
      const style = parameterStyle?.style ?? "simple";
      const explode = parameterStyle?.explode ?? false;
      if (style === "label") {
        if (Array.isArray(value)) return "." + value.filter((item) => item != null).map(encode).join(explode ? "." : ",");
        if (isPlainObject(value)) {
          const entries = Object.entries(value as Record<string, unknown>).filter(([, item]) => item != null);
          return "." + (explode ? entries.map(([name, item]) => encode(name) + "=" + encode(item)).join(".") : entries.flatMap(([name, item]) => [encode(name), encode(item)]).join(","));
        }
        return "." + encode(value);
      }
      if (style === "matrix") {
        if (Array.isArray(value)) return explode ? value.filter((item) => item != null).map((item) => ";" + key + "=" + encode(item)).join("") : ";" + key + "=" + value.filter((item) => item != null).map(encode).join(",");
        if (isPlainObject(value)) {
          const entries = Object.entries(value as Record<string, unknown>).filter(([, item]) => item != null);
          return explode ? entries.map(([name, item]) => ";" + encode(name) + "=" + encode(item)).join("") : ";" + key + "=" + entries.flatMap(([name, item]) => [encode(name), encode(item)]).join(",");
        }
        return ";" + key + "=" + encode(value);
      }
      if (Array.isArray(value)) return value.filter((item) => item != null).map(encode).join(",");
      if (isPlainObject(value)) {
        return Object.entries(value as Record<string, unknown>)
          .filter(([, item]) => item != null)
          .map(([name, item]) => explode ? encode(name) + "=" + encode(item) : [encode(name), encode(item)])
          .flat()
          .join(",");
      }
      return encode(value);
    };
    return url
      .replace(/{([^}]+)}/g, (_, key: string) =>
        record[key] != null ? serialize(key, record[key]) : `{${key}}`,
      )
      .replace(/:([a-zA-Z0-9_]+)/g, (_, key: string) =>
        record[key] != null ? serialize(key, record[key]) : `:${key}`,
      );
  }

  /** Uses URLSearchParams, skips null/undefined values */
  defaultEncodeSearchParams = (queryParams: unknown, styles?: Record<string, ParameterSerialization>): URLSearchParams | undefined => {
    if (!queryParams || typeof queryParams !== "object") return;

    const searchParams = new URLSearchParams();
    const rawEntries: Array<{ key: string; value: string; allowReserved: boolean }> = [];
    const append = (key: string, value: unknown, allowReserved = false) => {
      const stringValue = String(value);
      searchParams.append(key, stringValue);
      rawEntries.push({ key, value: stringValue, allowReserved });
    };
    const encodeQueryComponent = (value: string, allowReserved: boolean) => {
      const encoded = encodeURIComponent(value);
      return allowReserved
        ? encoded.replace(/%3A|%2F|%3F|%40|%21|%24|%26|%27|%28|%29|%2A|%2B|%2C|%3B|%3D|%5B|%5D/gi, (part) => decodeURIComponent(part))
        : encoded;
    };
    Object.defineProperty(searchParams, "toString", {
      value: () => rawEntries.map(({ key, value, allowReserved }) => `${encodeQueryComponent(key, false)}=${encodeQueryComponent(value, allowReserved)}`).join("&"),
    });
    Object.entries(queryParams as Record<string, unknown>).forEach(([key, value]) => {
      if (value != null) {
        // Skip null/undefined values
        const parameterStyle = styles?.[key];
        const style = parameterStyle?.style ?? "form";
        const explode = parameterStyle?.explode ?? true;
        const allowReserved = parameterStyle?.allowReserved === true;
        if (Array.isArray(value)) {
          if (style === "spaceDelimited") append(key, value.filter((item) => item != null).map(String).join(" "), allowReserved);
          else if (style === "pipeDelimited") append(key, value.filter((item) => item != null).map(String).join("|"), allowReserved);
          else if (explode) value.forEach((val) => val != null && append(key, val, allowReserved));
          else append(key, value.filter((item) => item != null).map(String).join(","), allowReserved);
        } else if (isPlainObject(value)) {
          const entries = Object.entries(value as Record<string, unknown>).filter(([, nestedValue]) => nestedValue != null);
          if (style === "deepObject") {
            for (const [nestedKey, nestedValue] of entries) {
              if (Array.isArray(nestedValue)) nestedValue.forEach((item) => item != null && append(`${key}[${nestedKey}]`, item, allowReserved));
              else append(`${key}[${nestedKey}]`, nestedValue, allowReserved);
            }
          } else if (explode) {
            for (const [nestedKey, nestedValue] of entries) {
              if (Array.isArray(nestedValue)) nestedValue.forEach((item) => item != null && append(nestedKey, item, allowReserved));
              else append(nestedKey, nestedValue, allowReserved);
            }
          } else {
            append(key, entries.flatMap(([nestedKey, nestedValue]) => [nestedKey, ...(Array.isArray(nestedValue) ? nestedValue : [nestedValue])]).map(String).join(","), allowReserved);
          }
        } else {
          append(key, value, allowReserved);
        }
      }
    });

    return searchParams;
  }

  /** Append cookie params as a Cookie header (or merge into existing). */
  defaultEncodeCookies = (cookies: unknown, headers: Headers): void => {
    if (!cookies || typeof cookies !== "object") return;
    const parts = Object.entries(cookies as Record<string, unknown>)
      .filter(([, value]) => value != null)
      .map(([key, value]) => `${key}=${String(value)}`);
    if (!parts.length) return;
    const existing = headers.get("cookie");
    headers.set("cookie", existing ? `${existing}; ${parts.join("; ")}` : parts.join("; "));
  }

  defaultParseResponseData = async (response: FetcherResponse): Promise<unknown> => {
    const contentType = response.headers.get("content-type") ?? "";
    const normalizedContentType = contentType.toLowerCase();
    if (normalizedContentType.includes("text/event-stream")) {
      return response.body ?? null;
    }
    if (normalizedContentType.startsWith("text/")) {
      return (await response.text())
    }

    if (normalizedContentType.startsWith("application/octet-stream")) {
      return new Blob([await response.arrayBuffer()])
    }

    if (
      normalizedContentType.includes("application/json") ||
      (normalizedContentType.includes("application/") && normalizedContentType.includes("json")) ||
      normalizedContentType === "*/*"
      ) {
      try {
        return await response.json();
      } catch {
        return undefined
      }
    }

    return
  }

  // <ApiClient.get>
    get<Path extends keyof GetEndpoints, TEndpoint extends GetEndpoints[Path]>(
      path: Path,
      ...params: MaybeOptionalArg<
        (TEndpoint extends { parameters: infer UParams }
          ? NotNever<InferSchemaInput<UParams>> extends true ? InferSchemaInput<UParams> & { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide } : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide }
          : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide })
      >
    ): Promise<SafeApiResponse<TEndpoint>>;

    get<Path extends keyof GetEndpoints, TEndpoint extends GetEndpoints[Path]>(
      path: Path,
      ...params: MaybeOptionalArg<
        (TEndpoint extends { parameters: infer UParams }
          ? NotNever<InferSchemaInput<UParams>> extends true ? InferSchemaInput<UParams> & { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide } : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide }
          : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide })
      >
    ): Promise<InferSuccessData<TEndpoint>>;

    get<Path extends keyof GetEndpoints>(
      path: Path,
      ...params: [config?: unknown]
    ): Promise<unknown> {
        return this.request("get", path, params[0] as never) as Promise<unknown>;
    }
    // </ApiClient.get>

// <ApiClient.post>
    post<Path extends keyof PostEndpoints, TEndpoint extends PostEndpoints[Path]>(
      path: Path,
      ...params: MaybeOptionalArg<
        (TEndpoint extends { parameters: infer UParams }
          ? NotNever<InferSchemaInput<UParams>> extends true ? InferSchemaInput<UParams> & { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide } : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide }
          : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide })
      >
    ): Promise<SafeApiResponse<TEndpoint>>;

    post<Path extends keyof PostEndpoints, TEndpoint extends PostEndpoints[Path]>(
      path: Path,
      ...params: MaybeOptionalArg<
        (TEndpoint extends { parameters: infer UParams }
          ? NotNever<InferSchemaInput<UParams>> extends true ? InferSchemaInput<UParams> & { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide } : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide }
          : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide })
      >
    ): Promise<InferSuccessData<TEndpoint>>;

    post<Path extends keyof PostEndpoints>(
      path: Path,
      ...params: [config?: unknown]
    ): Promise<unknown> {
        return this.request("post", path, params[0] as never) as Promise<unknown>;
    }
    // </ApiClient.post>

// <ApiClient.put>
    put<Path extends keyof PutEndpoints, TEndpoint extends PutEndpoints[Path]>(
      path: Path,
      ...params: MaybeOptionalArg<
        (TEndpoint extends { parameters: infer UParams }
          ? NotNever<InferSchemaInput<UParams>> extends true ? InferSchemaInput<UParams> & { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide } : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide }
          : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide })
      >
    ): Promise<SafeApiResponse<TEndpoint>>;

    put<Path extends keyof PutEndpoints, TEndpoint extends PutEndpoints[Path]>(
      path: Path,
      ...params: MaybeOptionalArg<
        (TEndpoint extends { parameters: infer UParams }
          ? NotNever<InferSchemaInput<UParams>> extends true ? InferSchemaInput<UParams> & { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide } : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide }
          : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide })
      >
    ): Promise<InferSuccessData<TEndpoint>>;

    put<Path extends keyof PutEndpoints>(
      path: Path,
      ...params: [config?: unknown]
    ): Promise<unknown> {
        return this.request("put", path, params[0] as never) as Promise<unknown>;
    }
    // </ApiClient.put>

// <ApiClient.delete>
    delete<Path extends keyof DeleteEndpoints, TEndpoint extends DeleteEndpoints[Path]>(
      path: Path,
      ...params: MaybeOptionalArg<
        (TEndpoint extends { parameters: infer UParams }
          ? NotNever<InferSchemaInput<UParams>> extends true ? InferSchemaInput<UParams> & { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide } : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide }
          : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide })
      >
    ): Promise<SafeApiResponse<TEndpoint>>;

    delete<Path extends keyof DeleteEndpoints, TEndpoint extends DeleteEndpoints[Path]>(
      path: Path,
      ...params: MaybeOptionalArg<
        (TEndpoint extends { parameters: infer UParams }
          ? NotNever<InferSchemaInput<UParams>> extends true ? InferSchemaInput<UParams> & { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide } : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide }
          : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide })
      >
    ): Promise<InferSuccessData<TEndpoint>>;

    delete<Path extends keyof DeleteEndpoints>(
      path: Path,
      ...params: [config?: unknown]
    ): Promise<unknown> {
        return this.request("delete", path, params[0] as never) as Promise<unknown>;
    }
    // </ApiClient.delete>

// <ApiClient.patch>
    patch<Path extends keyof PatchEndpoints, TEndpoint extends PatchEndpoints[Path]>(
      path: Path,
      ...params: MaybeOptionalArg<
        (TEndpoint extends { parameters: infer UParams }
          ? NotNever<InferSchemaInput<UParams>> extends true ? InferSchemaInput<UParams> & { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide } : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide }
          : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide })
      >
    ): Promise<SafeApiResponse<TEndpoint>>;

    patch<Path extends keyof PatchEndpoints, TEndpoint extends PatchEndpoints[Path]>(
      path: Path,
      ...params: MaybeOptionalArg<
        (TEndpoint extends { parameters: infer UParams }
          ? NotNever<InferSchemaInput<UParams>> extends true ? InferSchemaInput<UParams> & { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide } : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide }
          : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide })
      >
    ): Promise<InferSuccessData<TEndpoint>>;

    patch<Path extends keyof PatchEndpoints>(
      path: Path,
      ...params: [config?: unknown]
    ): Promise<unknown> {
        return this.request("patch", path, params[0] as never) as Promise<unknown>;
    }
    // </ApiClient.patch>


    // <ApiClient.request>
    /**
     * Generic request method with full type-safety for any endpoint
     */
    request<
      TMethod extends keyof EndpointByMethod,
      TPath extends keyof EndpointByMethod[TMethod],
      TEndpoint extends EndpointByMethod[TMethod][TPath]
    >(
      method: TMethod,
      path: TPath,
      ...params: MaybeOptionalArg<
        (TEndpoint extends { parameters: infer UParams }
          ? NotNever<InferSchemaInput<UParams>> extends true ? InferSchemaInput<UParams> & { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide } : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide }
          : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse: true; throwOnStatusError?: boolean; validate?: ValidateSide })
      >
    ): Promise<SafeApiResponse<TEndpoint>>;

    request<
      TMethod extends keyof EndpointByMethod,
      TPath extends keyof EndpointByMethod[TMethod],
      TEndpoint extends EndpointByMethod[TMethod][TPath]
    >(
      method: TMethod,
      path: TPath,
      ...params: MaybeOptionalArg<
        (TEndpoint extends { parameters: infer UParams }
          ? NotNever<InferSchemaInput<UParams>> extends true ? InferSchemaInput<UParams> & { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide } : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide }
          : { overrides?: RequestInit; queryOptions?: ApiQueryOptions; withResponse?: false; throwOnStatusError?: boolean; validate?: ValidateSide })
      >
    ): Promise<InferSuccessData<TEndpoint>>;

    request<
      TMethod extends keyof EndpointByMethod,
      TPath extends keyof EndpointByMethod[TMethod],
      TEndpoint extends EndpointByMethod[TMethod][TPath]
    >(
      method: TMethod,
      path: TPath,
      ...params: [config?: unknown]
    ): Promise<unknown> {
      return (async () => {
      const requestParams = params[0] as
        | (EndpointParameters & {
            overrides?: RequestInit;
            queryOptions?: ApiQueryOptions;
            withResponse?: boolean;
            throwOnStatusError?: boolean;
            validate?: ValidateSide;
          })
        | undefined;
      const withResponse = requestParams?.withResponse;
      const throwOnStatusError = requestParams?.throwOnStatusError ?? (withResponse ? false : true);
      let overrides = requestParams?.overrides;


      const parametersToSend: EndpointParameters = {};
      if (requestParams?.body !== undefined) parametersToSend.body = requestParams.body;
      if (requestParams?.query !== undefined) parametersToSend.query = requestParams.query;
      if (requestParams?.header !== undefined) parametersToSend.header = requestParams.header;
      if (requestParams?.path !== undefined) parametersToSend.path = requestParams.path;
      if (requestParams?.cookie !== undefined) parametersToSend.cookie = requestParams.cookie;



      const resolvedPath = (this.fetcher.decodePathParams ?? this.defaultDecodePathParams)(
        this.baseUrl + (path as string),
        parametersToSend.path ?? {},
        endpointParameterStyles[method]?.[path]?.path,
      );
      const url = new URL(resolvedPath);
      const urlSearchParams = (this.fetcher.encodeSearchParams ?? this.defaultEncodeSearchParams)(
        parametersToSend.query,
        endpointParameterStyles[method]?.[path]?.query,
      );

      if (parametersToSend.cookie) {
        const headers = new Headers((overrides as RequestInit | undefined)?.headers);
        (this.fetcher.encodeCookies ?? this.defaultEncodeCookies)(parametersToSend.cookie, headers);
        overrides = { ...overrides, headers };
      }

      const parameterStyles = endpointParameterStyles[method]?.[path as string];
      const response = await this.fetcher.fetch({
        method: method,
        path: (path as string),
        url,
        ...(urlSearchParams ? { urlSearchParams } : {}),
        ...(Object.keys(parametersToSend).length ? { parameters: parametersToSend } : {}),
        requestFormat: endpointRequestFormats[method]?.[path] ?? "json",
        ...(parameterStyles ? { parameterStyles } : {}),
        security: endpointSecurityRequirements[method]?.[path] ?? defaultSecurityRequirements,
        ...(overrides ? { overrides } : {}),
        throwOnStatusError
      });
          const responseFormat = endpointResponseFormats[method]?.[path] ?? "json";
          let data =
            responseFormat === "sse"
              ? (response.body ?? null)
              : await (this.fetcher.parseResponseData ?? this.defaultParseResponseData)(response);

          const typedResponse = Object.assign(response, {
            data: data,
            json: () => Promise.resolve(data)
          }) as SafeApiResponse<TEndpoint>;

          if (throwOnStatusError && (errorStatusCodes as readonly number[]).includes(response.status)) {
            throw new TypedStatusError(typedResponse as TypedErrorResponse<unknown, ErrorStatusCode, unknown>);
          }

          return withResponse ? typedResponse : data;
      })()
    }
    // </ApiClient.request>
}

export function createApiClient(
  fetcher: Fetcher,
  baseUrl?: string,
  options?: { validate?: ValidateSide; onValidate?: OnValidate },
) {
  return new ApiClient(fetcher, options).setBaseUrl(baseUrl ?? "");
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
