/*
 Generated by typeshare 1.13.2
*/

export interface AuthResponse {
	url: string;
}

export interface AuthenticateQueryParams {
	client_id: string;
}

export interface AuthenticateRequest {
	username?: string;
	password?: string;
}

export interface AuthenticateResponse {
	url: string;
}

export interface BulkDeleteUserResponse {
	count: number;
}

export interface RedirectUri {
	id: string;
	client_id: string;
	value: string;
	enabled: boolean;
	created_at: Date;
	updated_at: Date;
}

export interface Client {
	id: string;
	enabled: boolean;
	client_id: string;
	secret?: string;
	realm_id: string;
	protocol: string;
	public_client: boolean;
	service_account_enabled: boolean;
	client_type: string;
	name: string;
	redirect_uris?: RedirectUri[];
	created_at: Date;
	updated_at: Date;
}

export interface ClientsResponse {
	data: Client[];
}

export interface CreateRoleDto {
	name: string;
	description?: string;
	permissions: string[];
	realm_id: string;
	client_id: string;
}

export interface CreateRoleValidator {
	name: string;
	description?: string;
	permissions: string[];
}

export interface Role {
	id: string;
	name: string;
	description?: string;
	permissions: string[];
	realm_id: string;
	client_id: string;
	client?: Client;
	created_at: Date;
	updated_at: Date;
}

export interface Realm {
	id: string;
	name: string;
	created_at: Date;
	updated_at: Date;
}

export interface User {
	id: string;
	realm_id: string;
	client_id: string;
	username: string;
	firstname: string;
	lastname: string;
	email: string;
	email_verified: boolean;
	enabled: boolean;
	roles: Role[];
	realm?: Realm;
	created_at: Date;
	updated_at: Date;
}

export interface CreateUserResponse {
	data: User;
}

export interface CredentialData {
	hash_iterations: number;
	algorithm: string;
}

export interface CredentialOverview {
	id: string;
	user_id: string;
	credential_type: string;
	user_label?: string;
	credential_data: CredentialData;
	created_at: Date;
	updated_at: Date;
}

export interface DeleteClientResponse {
	message: string;
}

export interface DeleteUserCredentialResponse {
	message: string;
}

export interface DeleteUserResponse {
	count: number;
}

export interface JwkKey {
	kid: string;
	kty: string;
	use_: string;
	alg: string;
	x5c: string;
	n: string;
	e: string;
}

export interface GetCertsResponse {
	keys: JwkKey[];
}

export interface GetClientResponse {
	data: Client;
}

export interface GetConfigResponse {
	app_version: string;
	environment: string;
}

export interface GetOpenIdConfigurationResponse {
	issuer: string;
	authorization_endpoint: string;
	token_endpoint: string;
	introspection_endpoint: string;
	userinfo_endpoint: string;
	jwks_uri: string;
	grant_types_supported: string[];
}

export interface GetRoleResponse {
	data: Role;
}

export interface GetRolesResponse {
	data: Role[];
}

export interface GetUserCredentialsResponse {
	data: CredentialOverview[];
}

export interface GetUserRolesResponse {
	data: Role[];
}

export interface JwtToken {
	access_token: string;
	token_type: string;
	refresh_token: string;
	expires_in: number;
	id_token: string;
}

export enum GrantType {
	Code = "authorization_code",
	Password = "password",
	Credentials = "client_credentials",
	RefreshToken = "refresh_token",
}

export interface TokenRequestValidator {
	grant_type?: GrantType;
	client_id?: string;
	client_secret?: string;
	code?: string;
	username?: string;
	password?: string;
	refresh_token?: string;
}

export interface UnassignRoleResponse {
	message: string;
}

export interface UpdateRoleResponse {
	data: Role;
}

export interface UpdateRoleValidator {
	name?: string;
	description?: string;
}

export interface UpdateUserResponse {
	data: User;
}

export interface UserRealmsResponse {
	data: Realm[];
}

export interface UserResponse {
	data: User;
}

export interface UsersResponse {
	data: User[];
}

export enum AppEnv {
	Development = "Development",
	Production = "Production",
}

export enum Permissions {
	CreateClient = "create_client",
	ManageAuthorization = "manage_authorization",
	ManageClients = "manage_clients",
	ManageEvents = "manage_events",
	ManageIdentityProviders = "manage_identity_providers",
	ManageRealm = "manage_realm",
	ManageUsers = "manage_users",
	ManageRoles = "manage_roles",
	QueryClients = "query_clients",
	QueryGroups = "query_groups",
	QueryRealms = "query_realms",
	QueryUsers = "query_users",
	ViewAuthorization = "view_authorization",
	ViewClients = "view_clients",
	ViewEvents = "view_events",
	ViewIdentityProviders = "view_identity_providers",
	ViewRealm = "view_realm",
	ViewUsers = "view_users",
	ViewRoles = "view_roles",
}

