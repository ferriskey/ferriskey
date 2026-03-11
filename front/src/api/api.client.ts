export namespace Schemas {
  // <Schemas>
  export type ApiErrorResponse = { message: string };
  export type AssignRoleResponse = { message: string; realm_name: string; user_id: string };
  export type AuthResponse = { url: string };
  export type AuthenticateRequest = Partial<{ password: string | null; username: string | null }>;
  export type AuthenticationStatus = "Success" | "RequiresActions" | "RequiresOtpChallenge" | "Failed";
  export type AuthenticateResponse = {
    message?: (string | null) | undefined;
    required_actions?: (Array<RequiredAction> | null) | undefined;
    status: AuthenticationStatus;
    token?: (string | null) | undefined;
    url?: (string | null) | undefined;
  };
  export type BulkDeleteUserResponse = { count: number; realm_name: string };
  export type BulkDeleteUserValidator = Partial<{ ids: Array<string> }>;
  export type BurnRecoveryCodeRequest = { recovery_code: string; recovery_code_format: string };
  export type BurnRecoveryCodeResponse = { login_url: string };
  export type ChallengeOtpResponse = { url: string };
  export type Client = {
    client_id: string;
    client_type: string;
    created_at: string;
    direct_access_grants_enabled: boolean;
    enabled: boolean;
    id: string;
    name: string;
    protocol: string;
    public_client: boolean;
    realm_id: string;
    redirect_uris?: (Array<RedirectUri> | null) | undefined;
    secret?: (string | null) | undefined;
    service_account_enabled: boolean;
    updated_at: string;
  };
  export type ClientsResponse = { data: Array<Client> };
  export type CreateClientValidator = Partial<{
    client_id: string;
    client_type: string;
    direct_access_grants_enabled: boolean;
    enabled: boolean;
    name: string;
    protocol: string;
    public_client: boolean;
    service_account_enabled: boolean;
  }>;
  export type CreateRealmValidator = Partial<{ name: string }>;
  export type CreateRedirectUriValidator = Partial<{ enabled: boolean; value: string }>;
  export type CreateRoleValidator = {
    description?: (string | null) | undefined;
    name: string;
    permissions: Array<string>;
  };
  export type Role = {
    client?: (null | Client) | undefined;
    client_id?: (string | null) | undefined;
    created_at: string;
    description?: (string | null) | undefined;
    id: string;
    name: string;
    permissions: Array<string>;
    realm_id: string;
    updated_at: string;
  };
  export type CreateRoleResponse = { data: Role };
  export type RealmSetting = {
    default_signing_algorithm?: (string | null) | undefined;
    forgot_password_enabled: boolean;
    id: string;
    realm_id: string;
    remember_me_enabled: boolean;
    updated_at: string;
    user_registration_enabled: boolean;
  };
  export type Realm = {
    created_at: string;
    id: string;
    name: string;
    settings?: (null | RealmSetting) | undefined;
    updated_at: string;
  };
  export type RequiredAction = "configure_otp" | "verify_email" | "update_password";
  export type User = {
    client_id?: (string | null) | undefined;
    created_at: string;
    email: string;
    email_verified: boolean;
    enabled: boolean;
    firstname: string;
    id: string;
    lastname: string;
    realm?: (null | Realm) | undefined;
    realm_id: string;
    required_actions: Array<RequiredAction>;
    roles: Array<Role>;
    updated_at: string;
    username: string;
  };
  export type CreateUserResponse = { data: User };
  export type CreateUserValidator = Partial<{
    email: string;
    email_verified: boolean | null;
    firstname: string;
    lastname: string;
    username: string;
  }>;
  export type WebhookTrigger =
    | "user.created"
    | "user.updated"
    | "user.deleted"
    | "user.bulk_deleted"
    | "user.assign.role"
    | "user.unassign.role"
    | "user.credentials.deleted"
    | "auth.reset_password"
    | "client.created"
    | "client.updated"
    | "client.deleted"
    | "client.role.created"
    | "client.role.updated"
    | "redirect_uri.created"
    | "redirect_uri.updated"
    | "redirect_uri.deleted"
    | "role.created"
    | "role.updated"
    | "realm.created"
    | "realm.updated"
    | "realm.deleted"
    | "realm.settings.updated"
    | "webhook.created"
    | "webhook.updated"
    | "webhook.deleted";
  export type WebhookSubscriber = { id: string; name: WebhookTrigger; webhook_id: string };
  export type Webhook = {
    created_at: string;
    description?: (string | null) | undefined;
    endpoint: string;
    id: string;
    name?: (string | null) | undefined;
    subscribers: Array<WebhookSubscriber>;
    triggered_at?: (string | null) | undefined;
    updated_at: string;
  };
  export type CreateWebhookResponse = { data: Webhook };
  export type CreateWebhookValidator = Partial<{
    description: string | null;
    endpoint: string;
    name: string | null;
    subscribers: Array<WebhookTrigger>;
  }>;
  export type CredentialData = { algorithm: string; hash_iterations: number };
  export type CredentialOverview = {
    created_at: string;
    credential_data: CredentialData;
    credential_type: string;
    id: string;
    updated_at: string;
    user_id: string;
    user_label?: (string | null) | undefined;
  };
  export type DeleteClientResponse = { message: string; realm_name: string };
  export type DeleteRealmResponse = string;
  export type DeleteRoleResponse = { message: string; realm_name: string; role_id: string };
  export type DeleteUserCredentialResponse = { message: string; realm_name: string; user_id: string };
  export type DeleteUserResponse = { count: number };
  export type DeleteWebhookResponse = { message: string; realm_name: string };
  export type ErrorResponse = { code: string; status: number; message: string };
  export type GenerateRecoveryCodesRequest = { amount: number; code_format: string };
  export type GenerateRecoveryCodesResponse = { codes: Array<string> };
  export type JwkKey = { alg: string; e: string; kid: string; kty: string; n: string; use: string };
  export type GetCertsResponse = { keys: Array<JwkKey> };
  export type GetClientResponse = { data: Client };
  export type GetClientRolesResponse = { data: Array<Role> };
  export type GetOpenIdConfigurationResponse = {
    authorization_endpoint: string;
    grant_types_supported: Array<string>;
    introspection_endpoint: string;
    issuer: string;
    jwks_uri: string;
    token_endpoint: string;
    userinfo_endpoint: string;
  };
  export type GetRoleResponse = { data: Role };
  export type GetRolesResponse = { data: Array<Role> };
  export type GetUserCredentialsResponse = { data: Array<CredentialOverview> };
  export type GetUserRolesResponse = { data: Array<Role> };
  export type GetWebhooksResponse = { data: Array<Webhook> };
  export type GrantType = "authorization_code" | "password" | "client_credentials" | "refresh_token";
  export type JwtToken = {
    access_token: string;
    expires_in: number;
    id_token?: (string | null) | undefined;
    refresh_token: string;
    token_type: string;
  };
  export type OtpVerifyRequest = { code: string; label: string; secret: string };
  export type RealmLoginSetting = {
    forgot_password_enabled: boolean;
    remember_me_enabled: boolean;
    user_registration_enabled: boolean;
  };
  export type RedirectUri = {
    client_id: string;
    created_at: string;
    enabled: boolean;
    id: string;
    updated_at: string;
    value: string;
  };
  export type ResetPasswordResponse = { message: string; realm_name: string; user_id: string };
  export type ResetPasswordValidator = Partial<{ credential_type: string; temporary: boolean; value: string }>;
  export type LogoutRequestValidator = Partial<{
    client_id: string | null;
    id_token_hint: string | null;
    post_logout_redirect_uri: string | null;
    state: string | null;
  }>;
  export type RegistrationRequest = {
    email: string;
    first_name?: (string | null) | undefined;
    last_name?: (string | null) | undefined;
    password: string;
    username: string;
  };
  export type RevokeTokenRequestValidator = {
    client_id: string;
    token: string;
    token_type_hint?: (string | null) | undefined;
  };
  export type SetupOtpResponse = { issuer: string; otpauth_url: string; secret: string };
  export type TokenRequestValidator = Partial<{
    client_id: string;
    client_secret: string | null;
    code: string | null;
    grant_type: GrantType;
    password: string | null;
    refresh_token: string | null;
    scope: string | null;
    username: string | null;
  }>;
  export type UnassignRoleResponse = { message: string; realm_name: string; user_id: string };
  export type UpdateClientValidator = Partial<{
    client_id: string | null;
    direct_access_grants_enabled: boolean | null;
    enabled: boolean | null;
    name: string | null;
  }>;
  export type ForgotPasswordRequest = { email: string };
  export type ResetPasswordWithTokenRequest = { token_id: string; token: string; new_password: string };
  export type ResetPasswordWithTokenResponse = { message: string };
  export type UpdatePasswordRequest = Partial<{ value: string }>;
  export type UpdatePasswordResponse = { message: string };
  export type UpdateRealmSettingValidator = Partial<{
    default_signing_algorithm: string | null;
    forgot_password_enabled: boolean | null;
    remember_me_enabled: boolean | null;
    user_registration_enabled: boolean | null;
  }>;
  export type SmtpConfig = {
    id: string;
    realm_id: string;
    host: string;
    port: number;
    username: string;
    from_email: string;
    from_name: string;
    encryption: "tls" | "starttls" | "none";
    created_at: string;
    updated_at: string;
  };
  export type UpsertSmtpConfigRequest = {
    host: string;
    port: number;
    username: string;
    password: string;
    from_email: string;
    from_name: string;
    encryption: "tls" | "starttls" | "none";
  };
  export type UpdateRealmValidator = { name: string };
  export type UpdateRedirectUriValidator = Partial<{ enabled: boolean }>;
  export type UpdateRolePermissionsResponse = { data: Role };
  export type UpdateRolePermissionsValidator = { permissions: Array<string> };
  export type UpdateRoleResponse = { data: Role };
  export type UpdateRoleValidator = Partial<{ description: string | null; name: string | null }>;
  export type UpdateUserResponse = { data: User };
  export type UpdateUserValidator = Partial<{
    email: string;
    email_verified: boolean | null;
    enabled: boolean | null;
    firstname: string;
    lastname: string;
    required_actions: Array<string> | null;
  }>;
  export type UserRealmsResponse = { data: Array<Realm> };
  export type UserResponse = { data: User };
  export type UsersResponse = { data: Array<User> };
  export type VerifyOtpResponse = { message: string };

  // </Schemas>
}

export namespace Endpoints {
  // <Endpoints>

  export type post_Create_realm = {
    method: "POST";
    path: "/realms";
    requestFormat: "json";
    parameters: {
      body: Schemas.CreateRealmValidator;
    };
    responses: { 201: Schemas.Realm };
  };
  export type get_Get_realm = {
    method: "GET";
    path: "/realms/{name}";
    requestFormat: "json";
    parameters: {
      path: { name: string };
    };
    responses: { 200: Schemas.Realm };
  };
  export type put_Update_realm = {
    method: "PUT";
    path: "/realms/{name}";
    requestFormat: "json";
    parameters: {
      path: { name: string };

      body: Schemas.UpdateRealmValidator;
    };
    responses: { 200: Schemas.Realm };
  };
  export type delete_Delete_realm = {
    method: "DELETE";
    path: "/realms/{name}";
    requestFormat: "json";
    parameters: {
      path: { name: string };
    };
    responses: { 200: Schemas.DeleteRealmResponse };
  };
  export type get_Get_login_realm_settings_handler = {
    method: "GET";
    path: "/realms/{name}/login-settings";
    requestFormat: "json";
    parameters: {
      path: { name: string };
    };
    responses: { 200: Schemas.RealmLoginSetting };
  };
  export type put_Update_realm_setting = {
    method: "PUT";
    path: "/realms/{name}/settings";
    requestFormat: "json";
    parameters: {
      path: { name: string };

      body: Schemas.UpdateRealmSettingValidator;
    };
    responses: { 200: Schemas.Realm };
  };
  export type get_Get_smtp_config = {
    method: "GET";
    path: "/realms/{realm_name}/smtp-config";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };
    };
    responses: { 200: Schemas.SmtpConfig; 404: Schemas.ApiErrorResponse };
  };
  export type put_Upsert_smtp_config = {
    method: "PUT";
    path: "/realms/{realm_name}/smtp-config";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };

      body: Schemas.UpsertSmtpConfigRequest;
    };
    responses: { 200: Schemas.SmtpConfig; 400: Schemas.ApiErrorResponse };
  };
  export type delete_Delete_smtp_config = {
    method: "DELETE";
    path: "/realms/{realm_name}/smtp-config";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };
    };
    responses: { 204: unknown; 403: Schemas.ApiErrorResponse };
  };
  export type get_Get_openid_configuration = {
    method: "GET";
    path: "/realms/{realm_name}/.well-known/openid-configuration";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };
    };
    responses: { 200: Schemas.GetOpenIdConfigurationResponse };
  };
  export type get_Get_clients = {
    method: "GET";
    path: "/realms/{realm_name}/clients";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };
    };
    responses: { 200: Schemas.ClientsResponse };
  };
  export type post_Create_client = {
    method: "POST";
    path: "/realms/{realm_name}/clients";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };

      body: Schemas.CreateClientValidator;
    };
    responses: { 201: Schemas.Client; 400: unknown; 401: unknown; 403: unknown };
  };
  export type get_Get_client = {
    method: "GET";
    path: "/realms/{realm_name}/clients/{client_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; client_id: string };
    };
    responses: { 200: Schemas.GetClientResponse };
  };
  export type delete_Delete_client = {
    method: "DELETE";
    path: "/realms/{realm_name}/clients/{client_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; client_id: string };
    };
    responses: { 200: Schemas.DeleteClientResponse };
  };
  export type patch_Update_client = {
    method: "PATCH";
    path: "/realms/{realm_name}/clients/{client_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; client_id: string };

      body: Schemas.UpdateClientValidator;
    };
    responses: { 200: Schemas.Client };
  };
  export type get_Get_redirect_uris = {
    method: "GET";
    path: "/realms/{realm_name}/clients/{client_id}/redirects";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; client_id: string };
    };
    responses: { 200: Array<Schemas.RedirectUri> };
  };
  export type post_Create_redirect_uri = {
    method: "POST";
    path: "/realms/{realm_name}/clients/{client_id}/redirects";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; client_id: string };

      body: Schemas.CreateRedirectUriValidator;
    };
    responses: { 201: Schemas.RedirectUri };
  };
  export type put_Update_redirect_uri = {
    method: "PUT";
    path: "/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; client_id: string; uri_id: string };

      body: Schemas.UpdateRedirectUriValidator;
    };
    responses: { 200: Schemas.RedirectUri };
  };
  export type delete_Delete_redirect_uri = {
    method: "DELETE";
    path: "/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; client_id: string; uri_id: string };
    };
    responses: { 200: unknown };
  };
  export type get_ListPostLogoutRedirects = {
    method: "GET";
    path: "/realms/{realm_name}/clients/{client_id}/post-logout-redirects";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; client_id: string };
    };
    responses: { 200: Array<Schemas.RedirectUri> };
  };
  export type post_CreatePostLogoutRedirect = {
    method: "POST";
    path: "/realms/{realm_name}/clients/{client_id}/post-logout-redirects";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; client_id: string };

      body: Schemas.CreateRedirectUriValidator;
    };
    responses: { 201: Schemas.RedirectUri };
  };
  export type put_UpdatePostLogoutRedirect = {
    method: "PUT";
    path: "/realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; client_id: string; uri_id: string };

      body: Schemas.UpdateRedirectUriValidator;
    };
    responses: { 200: Schemas.RedirectUri };
  };
  export type delete_DeletePostLogoutRedirect = {
    method: "DELETE";
    path: "/realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; client_id: string; uri_id: string };
    };
    responses: { 200: unknown };
  };
  export type get_Get_client_roles = {
    method: "GET";
    path: "/realms/{realm_name}/clients/{client_id}/roles";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; client_id: string };
    };
    responses: { 200: Schemas.GetClientRolesResponse; 500: unknown };
  };
  export type post_Create_client_role = {
    method: "POST";
    path: "/realms/{realm_name}/clients/{client_id}/roles";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; client_id: string };

      body: Schemas.CreateRoleValidator;
    };
    responses: { 201: Schemas.Role };
  };
  export type post_Authenticate = {
    method: "POST";
    path: "/realms/{realm_name}/login-actions/authenticate";
    requestFormat: "json";
    parameters: {
      query: { client_id: string; session_code: string };
      path: { realm_name: string };

      body: Schemas.AuthenticateRequest;
    };
    responses: { 200: Schemas.AuthenticateResponse };
  };
  export type post_Burn_recovery_code = {
    method: "POST";
    path: "/realms/{realm_name}/login-actions/burn-recovery-code";
    requestFormat: "json";
    parameters: {
      body: Schemas.BurnRecoveryCodeRequest;
    };
    responses: { 200: Schemas.BurnRecoveryCodeResponse; 400: unknown };
  };
  export type post_Challenge_otp = {
    method: "POST";
    path: "/realms/{realm_name}/login-actions/challenge-otp";
    requestFormat: "json";
    parameters: never;
    responses: { 200: Schemas.ChallengeOtpResponse };
  };
  export type post_Generate_recovery_codes = {
    method: "POST";
    path: "/realms/{realm_name}/login-actions/generate-recovery-codes";
    requestFormat: "json";
    parameters: {
      body: Schemas.GenerateRecoveryCodesRequest;
    };
    responses: { 200: Schemas.GenerateRecoveryCodesResponse; 400: unknown };
  };
  export type get_Setup_otp = {
    method: "GET";
    path: "/realms/{realm_name}/login-actions/setup-otp";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };
    };
    responses: { 200: Schemas.SetupOtpResponse; 403: unknown; 500: unknown };
  };
  export type post_Update_password = {
    method: "POST";
    path: "/realms/{realm_name}/login-actions/update-password";
    requestFormat: "json";
    parameters: {
      body: Schemas.UpdatePasswordRequest;
    };
    responses: { 200: Schemas.UpdatePasswordResponse };
  };
  export type post_Forgot_password = {
    method: "POST";
    path: "/realms/{realm_name}/login-actions/forgot-password";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };

      body: Schemas.ForgotPasswordRequest;
    };
    responses: { 204: unknown; 400: unknown; 500: unknown };
  };
  export type post_Reset_password_with_token = {
    method: "POST";
    path: "/realms/{realm_name}/login-actions/reset-password";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };

      body: Schemas.ResetPasswordWithTokenRequest;
    };
    responses: { 200: Schemas.ResetPasswordWithTokenResponse; 400: unknown; 500: unknown };
  };
  export type post_Verify_otp = {
    method: "POST";
    path: "/realms/{realm_name}/login-actions/verify-otp";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };

      body: Schemas.OtpVerifyRequest;
    };
    responses: { 200: Schemas.VerifyOtpResponse };
  };
  export type get_Auth = {
    method: "GET";
    path: "/realms/{realm_name}/protocol/openid-connect/auth";
    requestFormat: "json";
    parameters: {
      query: Partial<{ response_type: string; client_id: string; redirect_uri: string; scope: string; state: string }>;
      path: { realm_name: string };
    };
    responses: { 302: Schemas.AuthResponse; 400: unknown; 401: unknown; 500: unknown };
  };
  export type get_Get_certs = {
    method: "GET";
    path: "/realms/{realm_name}/protocol/openid-connect/certs";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };
    };
    responses: { 200: Schemas.GetCertsResponse };
  };
  export type post_Exchange_token = {
    method: "POST";
    path: "/realms/{realm_name}/protocol/openid-connect/token";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };

      body: Schemas.TokenRequestValidator;
    };
    responses: { 200: Schemas.JwtToken };
  };
  export type get_Logout_get = {
    method: "GET";
    path: "/realms/{realm_name}/protocol/openid-connect/logout";
    requestFormat: "json";
    parameters: {
      query: Partial<{ id_token_hint: string; post_logout_redirect_uri: string; state: string; client_id: string }>;
      path: { realm_name: string };
    };
    responses: { 204: unknown; 307: unknown; 400: Schemas.ErrorResponse };
  };
  export type post_Logout = {
    method: "POST";
    path: "/realms/{realm_name}/protocol/openid-connect/logout";
    requestFormat: "form-url";
    parameters: {
      path: { realm_name: string };

      body: Schemas.LogoutRequestValidator;
    };
    responses: { 204: unknown; 307: unknown; 400: Schemas.ErrorResponse };
  };
  export type post_Revoke_token = {
    method: "POST";
    path: "/realms/{realm_name}/protocol/openid-connect/revoke";
    requestFormat: "form-url";
    parameters: {
      path: { realm_name: string };

      body: Schemas.RevokeTokenRequestValidator;
    };
    responses: { 200: unknown; 400: Schemas.ErrorResponse; 503: Schemas.ErrorResponse };
  };
  export type post_Registration_handler = {
    method: "POST";
    path: "/realms/{realm_name}/protocol/openid-connect/registrations";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };

      body: Schemas.RegistrationRequest;
    };
    responses: { 201: Schemas.JwtToken; 403: unknown };
  };
  export type get_Get_roles = {
    method: "GET";
    path: "/realms/{realm_name}/roles";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };
    };
    responses: { 200: Schemas.GetRolesResponse };
  };
  export type post_Create_realm_role = {
    method: "POST";
    path: "/realms/{realm_name}/roles";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };

      body: Schemas.CreateRoleValidator;
    };
    responses: {
      201: Schemas.CreateRoleResponse;
      400: Schemas.ApiErrorResponse;
      403: Schemas.ApiErrorResponse;
      500: Schemas.ApiErrorResponse;
    };
  };
  export type get_Get_role = {
    method: "GET";
    path: "/realms/{realm_name}/roles/{role_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; role_id: string };
    };
    responses: { 200: Schemas.GetRoleResponse; 404: unknown };
  };
  export type put_Update_role = {
    method: "PUT";
    path: "/realms/{realm_name}/roles/{role_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; role_id: string };

      body: Schemas.UpdateRoleValidator;
    };
    responses: { 200: Schemas.UpdateRoleResponse; 400: unknown; 404: unknown };
  };
  export type delete_Delete_role = {
    method: "DELETE";
    path: "/realms/{realm_name}/roles/{role_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; role_id: string };
    };
    responses: { 200: Schemas.DeleteRoleResponse; 400: unknown; 404: unknown };
  };
  export type patch_Update_role_permissions = {
    method: "PATCH";
    path: "/realms/{realm_name}/roles/{role_id}/permissions";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; role_id: string };

      body: Schemas.UpdateRolePermissionsValidator;
    };
    responses: { 200: Schemas.UpdateRolePermissionsResponse; 400: unknown; 404: unknown };
  };
  export type get_Get_users = {
    method: "GET";
    path: "/realms/{realm_name}/users";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };
    };
    responses: { 200: Schemas.UsersResponse };
  };
  export type post_Create_user = {
    method: "POST";
    path: "/realms/{realm_name}/users";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };

      body: Schemas.CreateUserValidator;
    };
    responses: { 201: Schemas.CreateUserResponse; 400: unknown; 401: unknown; 403: unknown };
  };
  export type get_Get_user_realms = {
    method: "GET";
    path: "/realms/{realm_name}/users/@me/realms";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };
    };
    responses: { 200: Schemas.UserRealmsResponse; 401: unknown; 403: unknown };
  };
  export type delete_Bulk_delete_user = {
    method: "DELETE";
    path: "/realms/{realm_name}/users/bulk";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };

      body: Schemas.BulkDeleteUserValidator;
    };
    responses: { 200: Schemas.BulkDeleteUserResponse };
  };
  export type get_Get_user = {
    method: "GET";
    path: "/realms/{realm_name}/users/{user_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; user_id: string };
    };
    responses: { 200: Schemas.UserResponse; 403: unknown; 404: unknown };
  };
  export type put_Update_user = {
    method: "PUT";
    path: "/realms/{realm_name}/users/{user_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; user_id: string };

      body: Schemas.UpdateUserValidator;
    };
    responses: { 200: Schemas.UpdateUserResponse; 400: unknown; 401: unknown; 403: unknown; 404: unknown };
  };
  export type delete_Delete_user = {
    method: "DELETE";
    path: "/realms/{realm_name}/users/{user_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; user_id: string };
    };
    responses: { 200: Schemas.DeleteUserResponse; 403: unknown; 404: unknown };
  };
  export type get_Get_user_credentials = {
    method: "GET";
    path: "/realms/{realm_name}/users/{user_id}/credentials";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; user_id: string };
    };
    responses: { 200: Schemas.GetUserCredentialsResponse };
  };
  export type delete_Delete_user_credential = {
    method: "DELETE";
    path: "/realms/{realm_name}/users/{user_id}/credentials/{credential_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; user_id: string; credential_id: string };
    };
    responses: { 200: Schemas.DeleteUserCredentialResponse };
  };
  export type put_Reset_password = {
    method: "PUT";
    path: "/realms/{realm_name}/users/{user_id}/reset-password";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; user_id: string };

      body: Schemas.ResetPasswordValidator;
    };
    responses: { 200: Schemas.ResetPasswordResponse };
  };
  export type get_Get_user_roles = {
    method: "GET";
    path: "/realms/{realm_name}/users/{user_id}/roles";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; user_id: string };
    };
    responses: { 200: Schemas.GetUserRolesResponse; 404: unknown };
  };
  export type post_Assign_role = {
    method: "POST";
    path: "/realms/{realm_name}/users/{user_id}/roles/{role_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; user_id: string; role_id: string };
    };
    responses: { 200: Schemas.AssignRoleResponse; 403: unknown; 404: unknown };
  };
  export type delete_Unassign_role = {
    method: "DELETE";
    path: "/realms/{realm_name}/users/{user_id}/roles/{role_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; user_id: string; role_id: string };
    };
    responses: { 200: Schemas.UnassignRoleResponse; 403: unknown; 404: unknown };
  };
  export type get_Fetch_webhooks = {
    method: "GET";
    path: "/realms/{realm_name}/webhooks";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };
    };
    responses: { 200: Schemas.GetWebhooksResponse };
  };
  export type put_Update_webhook = {
    method: "PUT";
    path: "/realms/{realm_name}/webhooks";
    requestFormat: "json";
    parameters: never;
    responses: { 200: Schemas.Webhook };
  };
  export type post_Create_webhook = {
    method: "POST";
    path: "/realms/{realm_name}/webhooks";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string };

      body: Schemas.CreateWebhookValidator;
    };
    responses: { 200: Schemas.CreateWebhookResponse };
  };
  export type get_Get_webhook = {
    method: "GET";
    path: "/realms/{realm_name}/webhooks/{webhook_id}";
    requestFormat: "json";
    parameters: {
      path: { webhook_id: string };
    };
    responses: { 200: Schemas.Webhook };
  };
  export type delete_Delete_webhook = {
    method: "DELETE";
    path: "/realms/{realm_name}/webhooks/{webhook_id}";
    requestFormat: "json";
    parameters: {
      path: { realm_name: string; webhook_id: string };
    };
    responses: { 200: Schemas.DeleteWebhookResponse };
  };

  // </Endpoints>
}

// <EndpointByMethod>
export type EndpointByMethod = {
  post: {
    "/realms": Endpoints.post_Create_realm;
    "/realms/{realm_name}/clients": Endpoints.post_Create_client;
    "/realms/{realm_name}/clients/{client_id}/redirects": Endpoints.post_Create_redirect_uri;
    "/realms/{realm_name}/clients/{client_id}/post-logout-redirects": Endpoints.post_CreatePostLogoutRedirect;
    "/realms/{realm_name}/clients/{client_id}/roles": Endpoints.post_Create_client_role;
    "/realms/{realm_name}/login-actions/authenticate": Endpoints.post_Authenticate;
    "/realms/{realm_name}/login-actions/burn-recovery-code": Endpoints.post_Burn_recovery_code;
    "/realms/{realm_name}/login-actions/challenge-otp": Endpoints.post_Challenge_otp;
    "/realms/{realm_name}/login-actions/generate-recovery-codes": Endpoints.post_Generate_recovery_codes;
    "/realms/{realm_name}/login-actions/update-password": Endpoints.post_Update_password;
    "/realms/{realm_name}/login-actions/forgot-password": Endpoints.post_Forgot_password;
    "/realms/{realm_name}/login-actions/reset-password": Endpoints.post_Reset_password_with_token;
    "/realms/{realm_name}/login-actions/verify-otp": Endpoints.post_Verify_otp;
    "/realms/{realm_name}/protocol/openid-connect/token": Endpoints.post_Exchange_token;
    "/realms/{realm_name}/protocol/openid-connect/logout": Endpoints.post_Logout;
    "/realms/{realm_name}/protocol/openid-connect/revoke": Endpoints.post_Revoke_token;
    "/realms/{realm_name}/protocol/openid-connect/registrations": Endpoints.post_Registration_handler;
    "/realms/{realm_name}/roles": Endpoints.post_Create_realm_role;
    "/realms/{realm_name}/users": Endpoints.post_Create_user;
    "/realms/{realm_name}/users/{user_id}/roles/{role_id}": Endpoints.post_Assign_role;
    "/realms/{realm_name}/webhooks": Endpoints.post_Create_webhook;
  };
  get: {
    "/realms/{name}": Endpoints.get_Get_realm;
    "/realms/{name}/login-settings": Endpoints.get_Get_login_realm_settings_handler;
    "/realms/{realm_name}/smtp-config": Endpoints.get_Get_smtp_config;
    "/realms/{realm_name}/.well-known/openid-configuration": Endpoints.get_Get_openid_configuration;
    "/realms/{realm_name}/clients": Endpoints.get_Get_clients;
    "/realms/{realm_name}/clients/{client_id}": Endpoints.get_Get_client;
    "/realms/{realm_name}/clients/{client_id}/redirects": Endpoints.get_Get_redirect_uris;
    "/realms/{realm_name}/clients/{client_id}/post-logout-redirects": Endpoints.get_ListPostLogoutRedirects;
    "/realms/{realm_name}/clients/{client_id}/roles": Endpoints.get_Get_client_roles;
    "/realms/{realm_name}/login-actions/setup-otp": Endpoints.get_Setup_otp;
    "/realms/{realm_name}/protocol/openid-connect/auth": Endpoints.get_Auth;
    "/realms/{realm_name}/protocol/openid-connect/certs": Endpoints.get_Get_certs;
    "/realms/{realm_name}/protocol/openid-connect/logout": Endpoints.get_Logout_get;
    "/realms/{realm_name}/roles": Endpoints.get_Get_roles;
    "/realms/{realm_name}/roles/{role_id}": Endpoints.get_Get_role;
    "/realms/{realm_name}/users": Endpoints.get_Get_users;
    "/realms/{realm_name}/users/@me/realms": Endpoints.get_Get_user_realms;
    "/realms/{realm_name}/users/{user_id}": Endpoints.get_Get_user;
    "/realms/{realm_name}/users/{user_id}/credentials": Endpoints.get_Get_user_credentials;
    "/realms/{realm_name}/users/{user_id}/roles": Endpoints.get_Get_user_roles;
    "/realms/{realm_name}/webhooks": Endpoints.get_Fetch_webhooks;
    "/realms/{realm_name}/webhooks/{webhook_id}": Endpoints.get_Get_webhook;
  };
  put: {
    "/realms/{name}": Endpoints.put_Update_realm;
    "/realms/{name}/settings": Endpoints.put_Update_realm_setting;
    "/realms/{realm_name}/smtp-config": Endpoints.put_Upsert_smtp_config;
    "/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}": Endpoints.put_Update_redirect_uri;
    "/realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id}": Endpoints.put_UpdatePostLogoutRedirect;
    "/realms/{realm_name}/roles/{role_id}": Endpoints.put_Update_role;
    "/realms/{realm_name}/users/{user_id}": Endpoints.put_Update_user;
    "/realms/{realm_name}/users/{user_id}/reset-password": Endpoints.put_Reset_password;
    "/realms/{realm_name}/webhooks": Endpoints.put_Update_webhook;
  };
  delete: {
    "/realms/{name}": Endpoints.delete_Delete_realm;
    "/realms/{realm_name}/smtp-config": Endpoints.delete_Delete_smtp_config;
    "/realms/{realm_name}/clients/{client_id}": Endpoints.delete_Delete_client;
    "/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}": Endpoints.delete_Delete_redirect_uri;
    "/realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id}": Endpoints.delete_DeletePostLogoutRedirect;
    "/realms/{realm_name}/roles/{role_id}": Endpoints.delete_Delete_role;
    "/realms/{realm_name}/users/bulk": Endpoints.delete_Bulk_delete_user;
    "/realms/{realm_name}/users/{user_id}": Endpoints.delete_Delete_user;
    "/realms/{realm_name}/users/{user_id}/credentials/{credential_id}": Endpoints.delete_Delete_user_credential;
    "/realms/{realm_name}/users/{user_id}/roles/{role_id}": Endpoints.delete_Unassign_role;
    "/realms/{realm_name}/webhooks/{webhook_id}": Endpoints.delete_Delete_webhook;
  };
  patch: {
    "/realms/{realm_name}/clients/{client_id}": Endpoints.patch_Update_client;
    "/realms/{realm_name}/roles/{role_id}/permissions": Endpoints.patch_Update_role_permissions;
  };
};

// </EndpointByMethod>

// <EndpointByMethod.Shorthands>
export type PostEndpoints = EndpointByMethod["post"];
export type GetEndpoints = EndpointByMethod["get"];
export type PutEndpoints = EndpointByMethod["put"];
export type DeleteEndpoints = EndpointByMethod["delete"];
export type PatchEndpoints = EndpointByMethod["patch"];
// </EndpointByMethod.Shorthands>

// <ApiClientTypes>
export type EndpointParameters = {
  body?: unknown;
  query?: Record<string, unknown>;
  header?: Record<string, unknown>;
  path?: Record<string, unknown>;
};

export type MutationMethod = "post" | "put" | "patch" | "delete";
export type Method = "get" | "head" | "options" | MutationMethod;

type RequestFormat = "json" | "form-data" | "form-url" | "binary" | "text";

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
  parameters?: TConfig["parameters"];
  meta: {
    alias: string;
    hasParameters: boolean;
    areParametersRequired: boolean;
  };
  responses?: TConfig["responses"];
  responseHeaders?: TConfig["responseHeaders"];
};

export interface Fetcher {
  decodePathParams?: (path: string, pathParams: Record<string, string>) => string;
  encodeSearchParams?: (searchParams: Record<string, unknown> | undefined) => URLSearchParams;
  //
  fetch: (input: {
    method: Method;
    url: URL;
    urlSearchParams?: URLSearchParams | undefined;
    parameters?: EndpointParameters | undefined;
    path: string;
    overrides?: RequestInit;
    throwOnStatusError?: boolean;
  }) => Promise<Response>;
  parseResponseData?: (response: Response) => Promise<unknown>;
}

export const successStatusCodes = [
  200, 201, 202, 203, 204, 205, 206, 207, 208, 226, 300, 301, 302, 303, 304, 305, 306, 307, 308,
] as const;
export type SuccessStatusCode = (typeof successStatusCodes)[number];

export const errorStatusCodes = [
  400, 401, 402, 403, 404, 405, 406, 407, 408, 409, 410, 411, 412, 413, 414, 415, 416, 417, 418, 421, 422, 423, 424,
  425, 426, 428, 429, 431, 451, 500, 501, 502, 503, 504, 505, 506, 507, 508, 510, 511,
] as const;
export type ErrorStatusCode = (typeof errorStatusCodes)[number];

// Taken from https://github.com/unjs/fetchdts/blob/ec4eaeab5d287116171fc1efd61f4a1ad34e4609/src/fetch.ts#L3
export interface TypedHeaders<TypedHeaderValues extends Record<string, string> | unknown>
  extends Omit<Headers, "append" | "delete" | "get" | "getSetCookie" | "has" | "set" | "forEach"> {
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/append) */
  append: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(
    name: Name,
    value: Lowercase<Name> extends keyof TypedHeaderValues ? TypedHeaderValues[Lowercase<Name>] : string,
  ) => void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/delete) */
  delete: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(name: Name) => void;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/get) */
  get: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(
    name: Name,
  ) => (Lowercase<Name> extends keyof TypedHeaderValues ? TypedHeaderValues[Lowercase<Name>] : string) | null;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/getSetCookie) */
  getSetCookie: () => string[];
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/has) */
  has: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(name: Name) => boolean;
  /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/set) */
  set: <Name extends Extract<keyof TypedHeaderValues, string> | (string & {})>(
    name: Name,
    value: Lowercase<Name> extends keyof TypedHeaderValues ? TypedHeaderValues[Lowercase<Name>] : string,
  ) => void;
  forEach: (
    callbackfn: (
      value: TypedHeaderValues[keyof TypedHeaderValues] | (string & {}),
      key: Extract<keyof TypedHeaderValues, string> | (string & {}),
      parent: TypedHeaders<TypedHeaderValues>,
    ) => void,
    thisArg?: any,
  ) => void;
}

/** @see https://developer.mozilla.org/en-US/docs/Web/API/Response */
export interface TypedSuccessResponse<TSuccess, TStatusCode, THeaders>
  extends Omit<Response, "ok" | "status" | "json" | "headers"> {
  ok: true;
  status: TStatusCode;
  headers: never extends THeaders ? Headers : TypedHeaders<THeaders>;
  data: TSuccess;
  /** [MDN Reference](https://developer.mozilla.org/en-US/docs/Web/API/Response/json) */
  json: () => Promise<TSuccess>;
}

/** @see https://developer.mozilla.org/en-US/docs/Web/API/Response */
export interface TypedErrorResponse<TData, TStatusCode, THeaders>
  extends Omit<Response, "ok" | "status" | "json" | "headers"> {
  ok: false;
  status: TStatusCode;
  headers: never extends THeaders ? Headers : TypedHeaders<THeaders>;
  data: TData;
  /** [MDN Reference](https://developer.mozilla.org/en-US/docs/Web/API/Response/json) */
  json: () => Promise<TData>;
}

export type TypedApiResponse<TAllResponses extends Record<string | number, unknown> = {}, THeaders = {}> = {
  [K in keyof TAllResponses]: K extends string
    ? K extends `${infer TStatusCode extends number}`
      ? TStatusCode extends SuccessStatusCode
        ? TypedSuccessResponse<TAllResponses[K], TStatusCode, K extends keyof THeaders ? THeaders[K] : never>
        : TypedErrorResponse<TAllResponses[K], TStatusCode, K extends keyof THeaders ? THeaders[K] : never>
      : never
    : K extends number
      ? K extends SuccessStatusCode
        ? TypedSuccessResponse<TAllResponses[K], K, K extends keyof THeaders ? THeaders[K] : never>
        : TypedErrorResponse<TAllResponses[K], K, K extends keyof THeaders ? THeaders[K] : never>
      : never;
}[keyof TAllResponses];

export type SafeApiResponse<TEndpoint> = TEndpoint extends { responses: infer TResponses }
  ? TResponses extends Record<string, unknown>
    ? TypedApiResponse<TResponses, TEndpoint extends { responseHeaders: infer THeaders } ? THeaders : never>
    : never
  : never;

export type InferResponseByStatus<TEndpoint, TStatusCode> = Extract<
  SafeApiResponse<TEndpoint>,
  { status: TStatusCode }
>;

type RequiredKeys<T> = {
  [P in keyof T]-?: undefined extends T[P] ? never : P;
}[keyof T];

type MaybeOptionalArg<T> = RequiredKeys<T> extends never ? [config?: T] : [config: T];
type NotNever<T> = [T] extends [never] ? false : true;

// </ApiClientTypes>

// <TypedStatusError>
export class TypedStatusError<TData = unknown> extends Error {
  response: TypedErrorResponse<TData, ErrorStatusCode, unknown>;
  status: number;
  constructor(response: TypedErrorResponse<TData, ErrorStatusCode, unknown>) {
    super(`HTTP ${response.status}: ${response.statusText}`);
    this.name = "TypedStatusError";
    this.response = response;
    this.status = response.status;
  }
}
// </TypedStatusError>

// <ApiClient>
export class ApiClient {
  baseUrl: string = "";
  successStatusCodes = successStatusCodes;
  errorStatusCodes = errorStatusCodes;

  constructor(public fetcher: Fetcher) {}

  setBaseUrl(baseUrl: string) {
    this.baseUrl = baseUrl;
    return this;
  }

  /**
   * Replace path parameters in URL
   * Supports both OpenAPI format {param} and Express format :param
   */
  defaultDecodePathParams = (url: string, params: Record<string, string>): string => {
    return url
      .replace(/{(\w+)}/g, (_, key: string) => params[key] || `{${key}}`)
      .replace(/:([a-zA-Z0-9_]+)/g, (_, key: string) => params[key] || `:${key}`);
  };

  /** Uses URLSearchParams, skips null/undefined values */
  defaultEncodeSearchParams = (queryParams: Record<string, unknown> | undefined): URLSearchParams | undefined => {
    if (!queryParams) return;

    const searchParams = new URLSearchParams();
    Object.entries(queryParams).forEach(([key, value]) => {
      if (value != null) {
        // Skip null/undefined values
        if (Array.isArray(value)) {
          value.forEach((val) => val != null && searchParams.append(key, String(val)));
        } else {
          searchParams.append(key, String(value));
        }
      }
    });

    return searchParams;
  };

  defaultParseResponseData = async (response: Response): Promise<unknown> => {
    const contentType = response.headers.get("content-type") ?? "";
    if (contentType.startsWith("text/")) {
      return await response.text();
    }

    if (contentType === "application/octet-stream") {
      return await response.arrayBuffer();
    }

    if (
      contentType.includes("application/json") ||
      (contentType.includes("application/") && contentType.includes("json")) ||
      contentType === "*/*"
    ) {
      try {
        return await response.json();
      } catch {
        return undefined;
      }
    }

    return;
  };

  // <ApiClient.post>
  post<Path extends keyof PostEndpoints, TEndpoint extends PostEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>["data"]>;

  post<Path extends keyof PostEndpoints, TEndpoint extends PostEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>;

  post<Path extends keyof PostEndpoints, _TEndpoint extends PostEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<any>
  ): Promise<any> {
    return this.request("post", path, ...params);
  }
  // </ApiClient.post>

  // <ApiClient.get>
  get<Path extends keyof GetEndpoints, TEndpoint extends GetEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>["data"]>;

  get<Path extends keyof GetEndpoints, TEndpoint extends GetEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>;

  get<Path extends keyof GetEndpoints, _TEndpoint extends GetEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<any>
  ): Promise<any> {
    return this.request("get", path, ...params);
  }
  // </ApiClient.get>

  // <ApiClient.put>
  put<Path extends keyof PutEndpoints, TEndpoint extends PutEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>["data"]>;

  put<Path extends keyof PutEndpoints, TEndpoint extends PutEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>;

  put<Path extends keyof PutEndpoints, _TEndpoint extends PutEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<any>
  ): Promise<any> {
    return this.request("put", path, ...params);
  }
  // </ApiClient.put>

  // <ApiClient.delete>
  delete<Path extends keyof DeleteEndpoints, TEndpoint extends DeleteEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>["data"]>;

  delete<Path extends keyof DeleteEndpoints, TEndpoint extends DeleteEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>;

  delete<Path extends keyof DeleteEndpoints, _TEndpoint extends DeleteEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<any>
  ): Promise<any> {
    return this.request("delete", path, ...params);
  }
  // </ApiClient.delete>

  // <ApiClient.patch>
  patch<Path extends keyof PatchEndpoints, TEndpoint extends PatchEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>["data"]>;

  patch<Path extends keyof PatchEndpoints, TEndpoint extends PatchEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<
      TEndpoint extends { parameters: infer UParams }
        ? NotNever<UParams> extends true
          ? UParams & { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: true; throwOnStatusError?: boolean }
    >
  ): Promise<SafeApiResponse<TEndpoint>>;

  patch<Path extends keyof PatchEndpoints, _TEndpoint extends PatchEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<any>
  ): Promise<any> {
    return this.request("patch", path, ...params);
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
          ? UParams & { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
          : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
        : { overrides?: RequestInit; withResponse?: false; throwOnStatusError?: boolean }
    >
  ): Promise<Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>["data"]>;

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
  ): Promise<SafeApiResponse<TEndpoint>>;

  request<
    TMethod extends keyof EndpointByMethod,
    TPath extends keyof EndpointByMethod[TMethod],
    TEndpoint extends EndpointByMethod[TMethod][TPath],
  >(method: TMethod, path: TPath, ...params: MaybeOptionalArg<any>): Promise<any> {
    const requestParams = params[0];
    const withResponse = requestParams?.withResponse;
    const {
      withResponse: _,
      throwOnStatusError = withResponse ? false : true,
      overrides,
      ...fetchParams
    } = requestParams || {};

    const parametersToSend: EndpointParameters = {};
    if (requestParams?.body !== undefined) (parametersToSend as any).body = requestParams.body;
    if (requestParams?.query !== undefined) (parametersToSend as any).query = requestParams.query;
    if (requestParams?.header !== undefined) (parametersToSend as any).header = requestParams.header;
    if (requestParams?.path !== undefined) (parametersToSend as any).path = requestParams.path;

    const resolvedPath = (this.fetcher.decodePathParams ?? this.defaultDecodePathParams)(
      this.baseUrl + (path as string),
      (parametersToSend.path ?? {}) as Record<string, string>,
    );
    const url = new URL(resolvedPath);
    const urlSearchParams = (this.fetcher.encodeSearchParams ?? this.defaultEncodeSearchParams)(parametersToSend.query);

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
        const data = await (this.fetcher.parseResponseData ?? this.defaultParseResponseData)(response);
        const typedResponse = Object.assign(response, {
          data: data,
          json: () => Promise.resolve(data),
        }) as SafeApiResponse<TEndpoint>;

        if (throwOnStatusError && errorStatusCodes.includes(response.status as never)) {
          throw new TypedStatusError(typedResponse as never);
        }

        return withResponse ? typedResponse : data;
      });

    return promise as Extract<InferResponseByStatus<TEndpoint, SuccessStatusCode>, { data: {} }>["data"];
  }
  // </ApiClient.request>
}

export function createApiClient(fetcher: Fetcher, baseUrl?: string) {
  return new ApiClient(fetcher).setBaseUrl(baseUrl ?? "");
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
