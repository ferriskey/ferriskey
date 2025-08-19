export namespace Schemas {
  // <Schemas>
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
  export type BulkDeleteUserResponse = { count: number }
  export type ChallengeOtpResponse = { url: string }
  export type Client = {
    client_id: string
    client_type: string
    created_at: string
    enabled: boolean
    id: string
    name: string
    protocol: string
    public_client: boolean
    realm_id: string
    redirect_uris?: (Array<RedirectUri> | null) | undefined
    secret?: (string | null) | undefined
    service_account_enabled: boolean
    updated_at: string
  }
  export type ClientsResponse = { data: Array<Client> }
  export type CreateClientValidator = Partial<{
    client_id: string
    client_type: string
    enabled: boolean
    name: string
    protocol: string
    public_client: boolean
    service_account_enabled: boolean
  }>
  export type CreateRealmValidator = Partial<{ name: string }>
  export type CreateRedirectUriValidator = Partial<{ enabled: boolean; value: string }>
  export type CreateRoleValidator = {
    description?: (string | null) | undefined
    name: string
    permissions: Array<string>
  }
  export type Realm = { created_at: string; id: string; name: string; updated_at: string }
  export type RequiredAction = 'configure_otp' | 'verify_email' | 'update_password'
  export type Role = {
    client?: (null | Client) | undefined
    client_id?: (string | null) | undefined
    created_at: string
    description?: (string | null) | undefined
    id: string
    name: string
    permissions: Array<string>
    realm_id: string
    updated_at: string
  }
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
    realm_id: string
    required_actions: Array<RequiredAction>
    roles: Array<Role>
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
  export type CredentialData = { algorithm: string; hash_iterations: number }
  export type CredentialOverview = {
    created_at: string
    credential_data: CredentialData
    credential_type: string
    id: string
    updated_at: string
    user_id: string
    user_label?: (string | null) | undefined
  }
  export type DeleteClientResponse = { message: string }
  export type DeleteRealmResponse = string
  export type DeleteUserCredentialResponse = {
    message: string
    realm_name: string
    user_id: string
  }
  export type DeleteUserResponse = { count: number }
  export type JwkKey = {
    alg: string
    e: string
    kid: string
    kty: string
    n: string
    use_: string
    x5c: string
  }
  export type GetCertsResponse = { keys: Array<JwkKey> }
  export type GetClientResponse = { data: Client }
  export type GetClientRolesResponse = { data: Array<Role> }
  export type GetOpenIdConfigurationResponse = {
    authorization_endpoint: string
    grant_types_supported: Array<string>
    introspection_endpoint: string
    issuer: string
    jwks_uri: string
    token_endpoint: string
    userinfo_endpoint: string
  }
  export type GetRoleResponse = { data: Role }
  export type GetRolesResponse = { data: Array<Role> }
  export type GetUserCredentialsResponse = { data: Array<CredentialOverview> }
  export type GetUserRolesResponse = { data: Array<Role> }
  export type GrantType = 'authorization_code' | 'password' | 'client_credentials' | 'refresh_token'
  export type JwtToken = {
    access_token: string
    expires_in: number
    id_token: string
    refresh_token: string
    token_type: string
  }
  export type OtpVerifyRequest = { code: string; label: string; secret: string }
  export type RedirectUri = {
    client_id: string
    created_at: string
    enabled: boolean
    id: string
    updated_at: string
    value: string
  }
  export type ResetPasswordResponse = { message: string; realm_name: string; user_id: string }
  export type ResetPasswordValidator = Partial<{
    credential_type: string
    temporary: boolean
    value: string
  }>
  export type SetupOtpResponse = { issuer: string; otpauth_url: string; secret: string }
  export type TokenRequestValidator = Partial<{
    client_id: string
    client_secret: string | null
    code: string | null
    grant_type: GrantType
    password: string | null
    refresh_token: string | null
    username: string | null
  }>
  export type UnassignRoleResponse = { message: string; realm_name: string; user_id: string }
  export type UpdateClientValidator = Partial<{
    client_id: string | null
    enabled: boolean | null
    name: string | null
  }>
  export type UpdatePasswordRequest = Partial<{ value: string }>
  export type UpdatePasswordResponse = { message: string }
  export type UpdateRealmSettingValidator = { default_signing_algorithm: string }
  export type UpdateRealmValidator = { name: string }
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
  export type UserRealmsResponse = { data: Array<Realm> }
  export type UserResponse = { data: User }
  export type UsersResponse = { data: Array<User> }
  export type VerifyOtpResponse = { message: string }

  // </Schemas>
}

export namespace Endpoints {
  // <Endpoints>

  export type get_Fetch_realm = {
    method: 'GET'
    path: '/realms'
    requestFormat: 'json'
    parameters: never
    response: Array<Schemas.Realm>
  }
  export type post_Create_realm = {
    method: 'POST'
    path: '/realms'
    requestFormat: 'json'
    parameters: {
      body: Schemas.CreateRealmValidator
    }
    response: Schemas.Realm
  }
  export type get_Get_realm = {
    method: 'GET'
    path: '/realms/{name}'
    requestFormat: 'json'
    parameters: {
      path: { name: string }
    }
    response: Schemas.Realm
  }
  export type put_Update_realm = {
    method: 'PUT'
    path: '/realms/{name}'
    requestFormat: 'json'
    parameters: {
      path: { name: string }

      body: Schemas.UpdateRealmValidator
    }
    response: Schemas.Realm
  }
  export type delete_Delete_realm = {
    method: 'DELETE'
    path: '/realms/{name}'
    requestFormat: 'json'
    parameters: {
      path: { name: string }
    }
    response: Schemas.DeleteRealmResponse
  }
  export type put_Update_realm_setting = {
    method: 'PUT'
    path: '/realms/{name}/settings'
    requestFormat: 'json'
    parameters: {
      path: { name: string }

      body: Schemas.UpdateRealmSettingValidator
    }
    response: Schemas.Realm
  }
  export type get_Get_openid_configuration = {
    method: 'GET'
    path: '/realms/{realm_name}/.well-known/openid-configuration'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    response: Schemas.GetOpenIdConfigurationResponse
  }
  export type get_Get_clients = {
    method: 'GET'
    path: '/realms/{realm_name}/clients'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    response: Schemas.ClientsResponse
  }
  export type post_Create_client = {
    method: 'POST'
    path: '/realms/{realm_name}/clients'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.CreateClientValidator
    }
    response: Schemas.Client
  }
  export type get_Get_client = {
    method: 'GET'
    path: '/realms/{realm_name}/clients/{client_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }
    }
    response: Schemas.GetClientResponse
  }
  export type delete_Delete_client = {
    method: 'DELETE'
    path: '/realms/{realm_name}/clients/{client_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }
    }
    response: Schemas.DeleteClientResponse
  }
  export type patch_Update_client = {
    method: 'PATCH'
    path: '/realms/{realm_name}/clients/{client_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }

      body: Schemas.UpdateClientValidator
    }
    response: Schemas.Client
  }
  export type get_Get_redirect_uris = {
    method: 'GET'
    path: '/realms/{realm_name}/clients/{client_id}/redirects'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }
    }
    response: Array<Schemas.RedirectUri>
  }
  export type post_Create_redirect_uri = {
    method: 'POST'
    path: '/realms/{realm_name}/clients/{client_id}/redirects'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }

      body: Schemas.CreateRedirectUriValidator
    }
    response: Schemas.RedirectUri
  }
  export type put_Update_redirect_uri = {
    method: 'PUT'
    path: '/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string; uri_id: string }

      body: Schemas.UpdateRedirectUriValidator
    }
    response: Schemas.RedirectUri
  }
  export type delete_Delete_redirect_uri = {
    method: 'DELETE'
    path: '/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string; uri_id: string }
    }
    response: unknown
  }
  export type get_Get_client_roles = {
    method: 'GET'
    path: '/realms/{realm_name}/clients/{client_id}/roles'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }
    }
    response: Schemas.GetClientRolesResponse
  }
  export type post_Create_role = {
    method: 'POST'
    path: '/realms/{realm_name}/clients/{client_id}/roles'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; client_id: string }

      body: Schemas.CreateRoleValidator
    }
    response: Schemas.Role
  }
  export type post_Authenticate = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/authenticate'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.AuthenticateRequest
    }
    response: Schemas.AuthenticateResponse
  }
  export type post_Challenge_otp = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/challenge-otp'
    requestFormat: 'json'
    parameters: never
    response: Schemas.ChallengeOtpResponse
  }
  export type get_Setup_otp = {
    method: 'GET'
    path: '/realms/{realm_name}/login-actions/setup-otp'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    response: Schemas.SetupOtpResponse
  }
  export type post_Update_password = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/update-password'
    requestFormat: 'json'
    parameters: {
      body: Schemas.UpdatePasswordRequest
    }
    response: Schemas.UpdatePasswordResponse
  }
  export type post_Verify_otp = {
    method: 'POST'
    path: '/realms/{realm_name}/login-actions/verify-otp'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.OtpVerifyRequest
    }
    response: Schemas.VerifyOtpResponse
  }
  export type get_Auth = {
    method: 'GET'
    path: '/realms/{realm_name}/protocol/openid-connect/auth'
    requestFormat: 'json'
    parameters: {
      path: {
        realm_name: string
        response_type: string
        client_id: string
        redirect_uri: string
        scope: string | null
        state: string | null
      }
    }
    response: unknown
  }
  export type get_Get_certs = {
    method: 'GET'
    path: '/realms/{realm_name}/protocol/openid-connect/certs'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    response: Schemas.GetCertsResponse
  }
  export type post_Exchange_token = {
    method: 'POST'
    path: '/realms/{realm_name}/protocol/openid-connect/token'
    requestFormat: 'json'
    parameters: {
      body: Schemas.TokenRequestValidator
    }
    response: Schemas.JwtToken
  }
  export type get_Get_roles = {
    method: 'GET'
    path: '/realms/{realm_name}/roles'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    response: Schemas.GetRolesResponse
  }
  export type get_Get_role = {
    method: 'GET'
    path: '/realms/{realm_name}/roles/{role_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; role_id: string }
    }
    response: Schemas.GetRoleResponse
  }
  export type put_Update_role = {
    method: 'PUT'
    path: '/realms/{realm_name}/roles/{role_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; role_id: string }

      body: Schemas.UpdateRoleValidator
    }
    response: Schemas.UpdateRoleResponse
  }
  export type patch_Update_role_permissions = {
    method: 'PATCH'
    path: '/realms/{realm_name}/roles/{role_id}/permissions'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; role_id: string }

      body: Schemas.UpdateRolePermissionsValidator
    }
    response: Schemas.UpdateRolePermissionsResponse
  }
  export type get_Get_users = {
    method: 'GET'
    path: '/realms/{realm_name}/users'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    response: Schemas.UsersResponse
  }
  export type post_Create_user = {
    method: 'POST'
    path: '/realms/{realm_name}/users'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }

      body: Schemas.CreateUserValidator
    }
    response: Schemas.CreateUserResponse
  }
  export type get_Get_user_realms = {
    method: 'GET'
    path: '/realms/{realm_name}/users/@me/realms'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string }
    }
    response: Schemas.UserRealmsResponse
  }
  export type delete_Bulk_delete_user = {
    method: 'DELETE'
    path: '/realms/{realm_name}/users/bulk'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; ids: Array<string> }
    }
    response: Schemas.BulkDeleteUserResponse
  }
  export type get_Get_user = {
    method: 'GET'
    path: '/realms/{realm_name}/users/{user_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string }
    }
    response: Schemas.UserResponse
  }
  export type put_Update_user = {
    method: 'PUT'
    path: '/realms/{realm_name}/users/{user_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string }

      body: Schemas.UpdateUserValidator
    }
    response: Schemas.UpdateUserResponse
  }
  export type delete_Delete_user = {
    method: 'DELETE'
    path: '/realms/{realm_name}/users/{user_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string }
    }
    response: Schemas.DeleteUserResponse
  }
  export type get_Get_user_credentials = {
    method: 'GET'
    path: '/realms/{realm_name}/users/{user_id}/credentials'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string }
    }
    response: Schemas.GetUserCredentialsResponse
  }
  export type delete_Delete_user_credential = {
    method: 'DELETE'
    path: '/realms/{realm_name}/users/{user_id}/credentials/{credential_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string; credential_id: string }
    }
    response: Schemas.DeleteUserCredentialResponse
  }
  export type put_Reset_password = {
    method: 'PUT'
    path: '/realms/{realm_name}/users/{user_id}/reset-password'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string }

      body: Schemas.ResetPasswordValidator
    }
    response: Schemas.ResetPasswordResponse
  }
  export type get_Get_user_roles = {
    method: 'GET'
    path: '/realms/{realm_name}/users/{user_id}/roles'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string }
    }
    response: Schemas.GetUserRolesResponse
  }
  export type post_Assign_role = {
    method: 'POST'
    path: '/realms/{realm_name}/users/{user_id}/roles/{role_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string; role_id: string }
    }
    response: Schemas.AssignRoleResponse
  }
  export type delete_Unassign_role = {
    method: 'DELETE'
    path: '/realms/{realm_name}/users/{user_id}/roles/{role_id}'
    requestFormat: 'json'
    parameters: {
      path: { realm_name: string; user_id: string; role_id: string }
    }
    response: Schemas.UnassignRoleResponse
  }

  // </Endpoints>
}

// <EndpointByMethod>
export type EndpointByMethod = {
  get: {
    '/realms': Endpoints.get_Fetch_realm
    '/realms/{name}': Endpoints.get_Get_realm
    '/realms/{realm_name}/.well-known/openid-configuration': Endpoints.get_Get_openid_configuration
    '/realms/{realm_name}/clients': Endpoints.get_Get_clients
    '/realms/{realm_name}/clients/{client_id}': Endpoints.get_Get_client
    '/realms/{realm_name}/clients/{client_id}/redirects': Endpoints.get_Get_redirect_uris
    '/realms/{realm_name}/clients/{client_id}/roles': Endpoints.get_Get_client_roles
    '/realms/{realm_name}/login-actions/setup-otp': Endpoints.get_Setup_otp
    '/realms/{realm_name}/protocol/openid-connect/auth': Endpoints.get_Auth
    '/realms/{realm_name}/protocol/openid-connect/certs': Endpoints.get_Get_certs
    '/realms/{realm_name}/roles': Endpoints.get_Get_roles
    '/realms/{realm_name}/roles/{role_id}': Endpoints.get_Get_role
    '/realms/{realm_name}/users': Endpoints.get_Get_users
    '/realms/{realm_name}/users/@me/realms': Endpoints.get_Get_user_realms
    '/realms/{realm_name}/users/{user_id}': Endpoints.get_Get_user
    '/realms/{realm_name}/users/{user_id}/credentials': Endpoints.get_Get_user_credentials
    '/realms/{realm_name}/users/{user_id}/roles': Endpoints.get_Get_user_roles
  }
  post: {
    '/realms': Endpoints.post_Create_realm
    '/realms/{realm_name}/clients': Endpoints.post_Create_client
    '/realms/{realm_name}/clients/{client_id}/redirects': Endpoints.post_Create_redirect_uri
    '/realms/{realm_name}/clients/{client_id}/roles': Endpoints.post_Create_role
    '/realms/{realm_name}/login-actions/authenticate': Endpoints.post_Authenticate
    '/realms/{realm_name}/login-actions/challenge-otp': Endpoints.post_Challenge_otp
    '/realms/{realm_name}/login-actions/update-password': Endpoints.post_Update_password
    '/realms/{realm_name}/login-actions/verify-otp': Endpoints.post_Verify_otp
    '/realms/{realm_name}/protocol/openid-connect/token': Endpoints.post_Exchange_token
    '/realms/{realm_name}/users': Endpoints.post_Create_user
    '/realms/{realm_name}/users/{user_id}/roles/{role_id}': Endpoints.post_Assign_role
  }
  put: {
    '/realms/{name}': Endpoints.put_Update_realm
    '/realms/{name}/settings': Endpoints.put_Update_realm_setting
    '/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}': Endpoints.put_Update_redirect_uri
    '/realms/{realm_name}/roles/{role_id}': Endpoints.put_Update_role
    '/realms/{realm_name}/users/{user_id}': Endpoints.put_Update_user
    '/realms/{realm_name}/users/{user_id}/reset-password': Endpoints.put_Reset_password
  }
  delete: {
    '/realms/{name}': Endpoints.delete_Delete_realm
    '/realms/{realm_name}/clients/{client_id}': Endpoints.delete_Delete_client
    '/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}': Endpoints.delete_Delete_redirect_uri
    '/realms/{realm_name}/users/bulk': Endpoints.delete_Bulk_delete_user
    '/realms/{realm_name}/users/{user_id}': Endpoints.delete_Delete_user
    '/realms/{realm_name}/users/{user_id}/credentials/{credential_id}': Endpoints.delete_Delete_user_credential
    '/realms/{realm_name}/users/{user_id}/roles/{role_id}': Endpoints.delete_Unassign_role
  }
  patch: {
    '/realms/{realm_name}/clients/{client_id}': Endpoints.patch_Update_client
    '/realms/{realm_name}/roles/{role_id}/permissions': Endpoints.patch_Update_role_permissions
  }
}

// </EndpointByMethod>

// <EndpointByMethod.Shorthands>
export type GetEndpoints = EndpointByMethod['get']
export type PostEndpoints = EndpointByMethod['post']
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
  response: unknown
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
  response: TConfig['response']
  responseHeaders?: TConfig['responseHeaders']
}

export type Fetcher = (
  method: Method,
  url: string,
  parameters?: EndpointParameters | undefined
) => Promise<Response>

type RequiredKeys<T> = {
  [P in keyof T]-?: undefined extends T[P] ? never : P
}[keyof T]

type MaybeOptionalArg<T> = RequiredKeys<T> extends never ? [config?: T] : [config: T]

// </ApiClientTypes>

// <ApiClient>
export class ApiClient {
  baseUrl: string = ''

  constructor(public fetcher: Fetcher) {}

  setBaseUrl(baseUrl: string) {
    this.baseUrl = baseUrl
    return this
  }

  parseResponse = async <T>(response: Response): Promise<T> => {
    const contentType = response.headers.get('content-type')
    if (contentType?.includes('application/json')) {
      return response.json()
    }
    return response.text() as unknown as T
  }

  // <ApiClient.get>
  get<Path extends keyof GetEndpoints, TEndpoint extends GetEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<TEndpoint['parameters']>
  ): Promise<TEndpoint['response']> {
    return this.fetcher('get', this.baseUrl + path, params[0]).then((response) =>
      this.parseResponse(response)
    ) as Promise<TEndpoint['response']>
  }
  // </ApiClient.get>

  // <ApiClient.post>
  post<Path extends keyof PostEndpoints, TEndpoint extends PostEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<TEndpoint['parameters']>
  ): Promise<TEndpoint['response']> {
    return this.fetcher('post', this.baseUrl + path, params[0]).then((response) =>
      this.parseResponse(response)
    ) as Promise<TEndpoint['response']>
  }
  // </ApiClient.post>

  // <ApiClient.put>
  put<Path extends keyof PutEndpoints, TEndpoint extends PutEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<TEndpoint['parameters']>
  ): Promise<TEndpoint['response']> {
    return this.fetcher('put', this.baseUrl + path, params[0]).then((response) =>
      this.parseResponse(response)
    ) as Promise<TEndpoint['response']>
  }
  // </ApiClient.put>

  // <ApiClient.delete>
  delete<Path extends keyof DeleteEndpoints, TEndpoint extends DeleteEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<TEndpoint['parameters']>
  ): Promise<TEndpoint['response']> {
    return this.fetcher('delete', this.baseUrl + path, params[0]).then((response) =>
      this.parseResponse(response)
    ) as Promise<TEndpoint['response']>
  }
  // </ApiClient.delete>

  // <ApiClient.patch>
  patch<Path extends keyof PatchEndpoints, TEndpoint extends PatchEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<TEndpoint['parameters']>
  ): Promise<TEndpoint['response']> {
    return this.fetcher('patch', this.baseUrl + path, params[0]).then((response) =>
      this.parseResponse(response)
    ) as Promise<TEndpoint['response']>
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
    ...params: MaybeOptionalArg<TEndpoint extends { parameters: infer Params } ? Params : never>
  ): Promise<
    Omit<Response, 'json'> & {
      /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/json) */
      json: () => Promise<TEndpoint extends { response: infer Res } ? Res : never>
    }
  > {
    return this.fetcher(method, this.baseUrl + (path as string), params[0] as EndpointParameters)
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
*/

// </ApiClient
