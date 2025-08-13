import { z } from "zod";

export type AssignRoleResponse = z.infer<typeof AssignRoleResponse>;
export const AssignRoleResponse = z.object({
  message: z.string(),
});

export type AuthResponse = z.infer<typeof AuthResponse>;
export const AuthResponse = z.object({
  url: z.string(),
});

export type AuthenticateRequest = z.infer<typeof AuthenticateRequest>;
export const AuthenticateRequest = z.object({
  password: z.union([z.string(), z.null()]).optional(),
  username: z.union([z.string(), z.null()]).optional(),
});

export type AuthenticationStatus = z.infer<typeof AuthenticationStatus>;
export const AuthenticationStatus = z.union([
  z.literal("Success"),
  z.literal("RequiresActions"),
  z.literal("RequiresOtpChallenge"),
  z.literal("Failed"),
]);

export type AuthenticateResponse_properties_message = z.infer<typeof AuthenticateResponse_properties_message>;
export const AuthenticateResponse_properties_message = z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type AuthenticateResponse_properties_required_actions_anyOf_0_anyOf_1 = z.infer<
  typeof AuthenticateResponse_properties_required_actions_anyOf_0_anyOf_1
>;
export const AuthenticateResponse_properties_required_actions_anyOf_0_anyOf_1 = z.null();

export type AuthenticateResponse_properties_required_actions_anyOf_1 = z.infer<
  typeof AuthenticateResponse_properties_required_actions_anyOf_1
>;
export const AuthenticateResponse_properties_required_actions_anyOf_1 = z.undefined();

export type AuthenticateResponse_properties_status = z.infer<typeof AuthenticateResponse_properties_status>;
export const AuthenticateResponse_properties_status = z.union([
  z.literal("Success"),
  z.literal("RequiresActions"),
  z.literal("RequiresOtpChallenge"),
  z.literal("Failed"),
]);

export type AuthenticateResponse_properties_token = z.infer<typeof AuthenticateResponse_properties_token>;
export const AuthenticateResponse_properties_token = z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type AuthenticateResponse_properties_url = z.infer<typeof AuthenticateResponse_properties_url>;
export const AuthenticateResponse_properties_url = z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type BulkDeleteUserResponse = z.infer<typeof BulkDeleteUserResponse>;
export const BulkDeleteUserResponse = z.object({
  count: z.number(),
});

export type ChallengeOtpResponse = z.infer<typeof ChallengeOtpResponse>;
export const ChallengeOtpResponse = z.object({
  url: z.string(),
});

export type Client_properties_client_id = z.infer<typeof Client_properties_client_id>;
export const Client_properties_client_id = z.string();

export type Client_properties_client_type = z.infer<typeof Client_properties_client_type>;
export const Client_properties_client_type = z.string();

export type Client_properties_created_at = z.infer<typeof Client_properties_created_at>;
export const Client_properties_created_at = z.string();

export type Client_properties_enabled = z.infer<typeof Client_properties_enabled>;
export const Client_properties_enabled = z.boolean();

export type Client_properties_id = z.infer<typeof Client_properties_id>;
export const Client_properties_id = z.string();

export type Client_properties_name = z.infer<typeof Client_properties_name>;
export const Client_properties_name = z.string();

export type Client_properties_protocol = z.infer<typeof Client_properties_protocol>;
export const Client_properties_protocol = z.string();

export type Client_properties_public_client = z.infer<typeof Client_properties_public_client>;
export const Client_properties_public_client = z.boolean();

export type Client_properties_realm_id = z.infer<typeof Client_properties_realm_id>;
export const Client_properties_realm_id = z.string();

export type Client_properties_redirect_uris_anyOf_0_anyOf_1 = z.infer<
  typeof Client_properties_redirect_uris_anyOf_0_anyOf_1
>;
export const Client_properties_redirect_uris_anyOf_0_anyOf_1 = z.null();

export type Client_properties_redirect_uris_anyOf_1 = z.infer<typeof Client_properties_redirect_uris_anyOf_1>;
export const Client_properties_redirect_uris_anyOf_1 = z.undefined();

export type Client_properties_secret = z.infer<typeof Client_properties_secret>;
export const Client_properties_secret = z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type Client_properties_service_account_enabled = z.infer<typeof Client_properties_service_account_enabled>;
export const Client_properties_service_account_enabled = z.boolean();

export type Client_properties_updated_at = z.infer<typeof Client_properties_updated_at>;
export const Client_properties_updated_at = z.string();

export type ClientsResponse_properties_data_items_properties_client_id = z.infer<
  typeof ClientsResponse_properties_data_items_properties_client_id
>;
export const ClientsResponse_properties_data_items_properties_client_id = z.string();

export type ClientsResponse_properties_data_items_properties_client_type = z.infer<
  typeof ClientsResponse_properties_data_items_properties_client_type
>;
export const ClientsResponse_properties_data_items_properties_client_type = z.string();

export type ClientsResponse_properties_data_items_properties_created_at = z.infer<
  typeof ClientsResponse_properties_data_items_properties_created_at
>;
export const ClientsResponse_properties_data_items_properties_created_at = z.string();

export type ClientsResponse_properties_data_items_properties_enabled = z.infer<
  typeof ClientsResponse_properties_data_items_properties_enabled
>;
export const ClientsResponse_properties_data_items_properties_enabled = z.boolean();

export type ClientsResponse_properties_data_items_properties_id = z.infer<
  typeof ClientsResponse_properties_data_items_properties_id
>;
export const ClientsResponse_properties_data_items_properties_id = z.string();

export type ClientsResponse_properties_data_items_properties_name = z.infer<
  typeof ClientsResponse_properties_data_items_properties_name
>;
export const ClientsResponse_properties_data_items_properties_name = z.string();

export type ClientsResponse_properties_data_items_properties_protocol = z.infer<
  typeof ClientsResponse_properties_data_items_properties_protocol
>;
export const ClientsResponse_properties_data_items_properties_protocol = z.string();

export type ClientsResponse_properties_data_items_properties_public_client = z.infer<
  typeof ClientsResponse_properties_data_items_properties_public_client
>;
export const ClientsResponse_properties_data_items_properties_public_client = z.boolean();

export type ClientsResponse_properties_data_items_properties_realm_id = z.infer<
  typeof ClientsResponse_properties_data_items_properties_realm_id
>;
export const ClientsResponse_properties_data_items_properties_realm_id = z.string();

export type ClientsResponse_properties_data_items_properties_redirect_uris_anyOf_0_anyOf_1 = z.infer<
  typeof ClientsResponse_properties_data_items_properties_redirect_uris_anyOf_0_anyOf_1
>;
export const ClientsResponse_properties_data_items_properties_redirect_uris_anyOf_0_anyOf_1 = z.null();

export type ClientsResponse_properties_data_items_properties_redirect_uris_anyOf_1 = z.infer<
  typeof ClientsResponse_properties_data_items_properties_redirect_uris_anyOf_1
>;
export const ClientsResponse_properties_data_items_properties_redirect_uris_anyOf_1 = z.undefined();

export type ClientsResponse_properties_data_items_properties_secret = z.infer<
  typeof ClientsResponse_properties_data_items_properties_secret
>;
export const ClientsResponse_properties_data_items_properties_secret = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type ClientsResponse_properties_data_items_properties_service_account_enabled = z.infer<
  typeof ClientsResponse_properties_data_items_properties_service_account_enabled
>;
export const ClientsResponse_properties_data_items_properties_service_account_enabled = z.boolean();

export type ClientsResponse_properties_data_items_properties_updated_at = z.infer<
  typeof ClientsResponse_properties_data_items_properties_updated_at
>;
export const ClientsResponse_properties_data_items_properties_updated_at = z.string();

export type CreateClientValidator = z.infer<typeof CreateClientValidator>;
export const CreateClientValidator = z.object({
  client_id: z.string().optional(),
  client_type: z.string().optional(),
  enabled: z.boolean().optional(),
  name: z.string().optional(),
  protocol: z.string().optional(),
  public_client: z.boolean().optional(),
  service_account_enabled: z.boolean().optional(),
});

export type CreateRealmValidator = z.infer<typeof CreateRealmValidator>;
export const CreateRealmValidator = z.object({
  name: z.string().optional(),
});

export type CreateRedirectUriValidator = z.infer<typeof CreateRedirectUriValidator>;
export const CreateRedirectUriValidator = z.object({
  enabled: z.boolean().optional(),
  value: z.string().optional(),
});

export type CreateRoleValidator = z.infer<typeof CreateRoleValidator>;
export const CreateRoleValidator = z.object({
  description: z.union([z.union([z.string(), z.null()]), z.undefined()]).optional(),
  name: z.string(),
  permissions: z.array(z.string()),
});

export type Realm = z.infer<typeof Realm>;
export const Realm = z.object({
  created_at: z.string(),
  id: z.string(),
  name: z.string(),
  updated_at: z.string(),
});

export type RequiredAction = z.infer<typeof RequiredAction>;
export const RequiredAction = z.union([
  z.literal("configure_otp"),
  z.literal("verify_email"),
  z.literal("update_password"),
]);

export type Role_properties_client_anyOf_0_anyOf_0 = z.infer<typeof Role_properties_client_anyOf_0_anyOf_0>;
export const Role_properties_client_anyOf_0_anyOf_0 = z.null();

export type Role_properties_client_anyOf_0_anyOf_1_properties_client_id = z.infer<
  typeof Role_properties_client_anyOf_0_anyOf_1_properties_client_id
>;
export const Role_properties_client_anyOf_0_anyOf_1_properties_client_id = z.string();

export type Role_properties_client_anyOf_0_anyOf_1_properties_client_type = z.infer<
  typeof Role_properties_client_anyOf_0_anyOf_1_properties_client_type
>;
export const Role_properties_client_anyOf_0_anyOf_1_properties_client_type = z.string();

export type Role_properties_client_anyOf_0_anyOf_1_properties_created_at = z.infer<
  typeof Role_properties_client_anyOf_0_anyOf_1_properties_created_at
>;
export const Role_properties_client_anyOf_0_anyOf_1_properties_created_at = z.string();

export type Role_properties_client_anyOf_0_anyOf_1_properties_enabled = z.infer<
  typeof Role_properties_client_anyOf_0_anyOf_1_properties_enabled
>;
export const Role_properties_client_anyOf_0_anyOf_1_properties_enabled = z.boolean();

export type Role_properties_client_anyOf_0_anyOf_1_properties_id = z.infer<
  typeof Role_properties_client_anyOf_0_anyOf_1_properties_id
>;
export const Role_properties_client_anyOf_0_anyOf_1_properties_id = z.string();

export type Role_properties_client_anyOf_0_anyOf_1_properties_name = z.infer<
  typeof Role_properties_client_anyOf_0_anyOf_1_properties_name
>;
export const Role_properties_client_anyOf_0_anyOf_1_properties_name = z.string();

export type Role_properties_client_anyOf_0_anyOf_1_properties_protocol = z.infer<
  typeof Role_properties_client_anyOf_0_anyOf_1_properties_protocol
>;
export const Role_properties_client_anyOf_0_anyOf_1_properties_protocol = z.string();

export type Role_properties_client_anyOf_0_anyOf_1_properties_public_client = z.infer<
  typeof Role_properties_client_anyOf_0_anyOf_1_properties_public_client
>;
export const Role_properties_client_anyOf_0_anyOf_1_properties_public_client = z.boolean();

export type Role_properties_client_anyOf_0_anyOf_1_properties_realm_id = z.infer<
  typeof Role_properties_client_anyOf_0_anyOf_1_properties_realm_id
>;
export const Role_properties_client_anyOf_0_anyOf_1_properties_realm_id = z.string();

export type Role_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 = z.infer<
  typeof Role_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1
>;
export const Role_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 = z.null();

export type Role_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 = z.infer<
  typeof Role_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1
>;
export const Role_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 = z.undefined();

export type Role_properties_client_anyOf_0_anyOf_1_properties_secret = z.infer<
  typeof Role_properties_client_anyOf_0_anyOf_1_properties_secret
>;
export const Role_properties_client_anyOf_0_anyOf_1_properties_secret = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type Role_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled = z.infer<
  typeof Role_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled
>;
export const Role_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled = z.boolean();

export type Role_properties_client_anyOf_0_anyOf_1_properties_updated_at = z.infer<
  typeof Role_properties_client_anyOf_0_anyOf_1_properties_updated_at
>;
export const Role_properties_client_anyOf_0_anyOf_1_properties_updated_at = z.string();

export type Role_properties_client_anyOf_1 = z.infer<typeof Role_properties_client_anyOf_1>;
export const Role_properties_client_anyOf_1 = z.undefined();

export type Role_properties_client_id = z.infer<typeof Role_properties_client_id>;
export const Role_properties_client_id = z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type Role_properties_created_at = z.infer<typeof Role_properties_created_at>;
export const Role_properties_created_at = z.string();

export type Role_properties_description = z.infer<typeof Role_properties_description>;
export const Role_properties_description = z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type Role_properties_id = z.infer<typeof Role_properties_id>;
export const Role_properties_id = z.string();

export type Role_properties_name = z.infer<typeof Role_properties_name>;
export const Role_properties_name = z.string();

export type Role_properties_permissions = z.infer<typeof Role_properties_permissions>;
export const Role_properties_permissions = z.array(z.string());

export type Role_properties_realm_id = z.infer<typeof Role_properties_realm_id>;
export const Role_properties_realm_id = z.string();

export type Role_properties_updated_at = z.infer<typeof Role_properties_updated_at>;
export const Role_properties_updated_at = z.string();

export type User_properties_client_id = z.infer<typeof User_properties_client_id>;
export const User_properties_client_id = z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type User_properties_created_at = z.infer<typeof User_properties_created_at>;
export const User_properties_created_at = z.string();

export type User_properties_email = z.infer<typeof User_properties_email>;
export const User_properties_email = z.string();

export type User_properties_email_verified = z.infer<typeof User_properties_email_verified>;
export const User_properties_email_verified = z.boolean();

export type User_properties_enabled = z.infer<typeof User_properties_enabled>;
export const User_properties_enabled = z.boolean();

export type User_properties_firstname = z.infer<typeof User_properties_firstname>;
export const User_properties_firstname = z.string();

export type User_properties_id = z.infer<typeof User_properties_id>;
export const User_properties_id = z.string();

export type User_properties_lastname = z.infer<typeof User_properties_lastname>;
export const User_properties_lastname = z.string();

export type User_properties_realm = z.infer<typeof User_properties_realm>;
export const User_properties_realm = z.union([z.union([z.null(), Realm]), z.undefined()]);

export type User_properties_realm_id = z.infer<typeof User_properties_realm_id>;
export const User_properties_realm_id = z.string();

export type User_properties_required_actions = z.infer<typeof User_properties_required_actions>;
export const User_properties_required_actions = z.array(RequiredAction);

export type User_properties_roles_items_properties_client_anyOf_0_anyOf_0 = z.infer<
  typeof User_properties_roles_items_properties_client_anyOf_0_anyOf_0
>;
export const User_properties_roles_items_properties_client_anyOf_0_anyOf_0 = z.null();

export type User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id = z.infer<
  typeof User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id
>;
export const User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id = z.string();

export type User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type = z.infer<
  typeof User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type
>;
export const User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type = z.string();

export type User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at = z.infer<
  typeof User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at
>;
export const User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at = z.string();

export type User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled = z.infer<
  typeof User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled
>;
export const User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled = z.boolean();

export type User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id = z.infer<
  typeof User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id
>;
export const User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id = z.string();

export type User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name = z.infer<
  typeof User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name
>;
export const User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name = z.string();

export type User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol = z.infer<
  typeof User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol
>;
export const User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol = z.string();

export type User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client = z.infer<
  typeof User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client
>;
export const User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client = z.boolean();

export type User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id = z.infer<
  typeof User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id
>;
export const User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id = z.string();

export type User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.infer<
    typeof User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1
  >;
export const User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 = z.infer<
  typeof User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1
>;
export const User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret = z.infer<
  typeof User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret
>;
export const User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled = z.infer<
  typeof User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled
>;
export const User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at = z.infer<
  typeof User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at
>;
export const User_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at = z.string();

export type User_properties_roles_items_properties_client_anyOf_1 = z.infer<
  typeof User_properties_roles_items_properties_client_anyOf_1
>;
export const User_properties_roles_items_properties_client_anyOf_1 = z.undefined();

export type User_properties_roles_items_properties_client_id = z.infer<
  typeof User_properties_roles_items_properties_client_id
>;
export const User_properties_roles_items_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type User_properties_roles_items_properties_created_at = z.infer<
  typeof User_properties_roles_items_properties_created_at
>;
export const User_properties_roles_items_properties_created_at = z.string();

export type User_properties_roles_items_properties_description = z.infer<
  typeof User_properties_roles_items_properties_description
>;
export const User_properties_roles_items_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type User_properties_roles_items_properties_id = z.infer<typeof User_properties_roles_items_properties_id>;
export const User_properties_roles_items_properties_id = z.string();

export type User_properties_roles_items_properties_name = z.infer<typeof User_properties_roles_items_properties_name>;
export const User_properties_roles_items_properties_name = z.string();

export type User_properties_roles_items_properties_permissions = z.infer<
  typeof User_properties_roles_items_properties_permissions
>;
export const User_properties_roles_items_properties_permissions = z.array(z.string());

export type User_properties_roles_items_properties_realm_id = z.infer<
  typeof User_properties_roles_items_properties_realm_id
>;
export const User_properties_roles_items_properties_realm_id = z.string();

export type User_properties_roles_items_properties_updated_at = z.infer<
  typeof User_properties_roles_items_properties_updated_at
>;
export const User_properties_roles_items_properties_updated_at = z.string();

export type User_properties_updated_at = z.infer<typeof User_properties_updated_at>;
export const User_properties_updated_at = z.string();

export type User_properties_username = z.infer<typeof User_properties_username>;
export const User_properties_username = z.string();

export type CreateUserResponse_properties_data_properties_client_id = z.infer<
  typeof CreateUserResponse_properties_data_properties_client_id
>;
export const CreateUserResponse_properties_data_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type CreateUserResponse_properties_data_properties_created_at = z.infer<
  typeof CreateUserResponse_properties_data_properties_created_at
>;
export const CreateUserResponse_properties_data_properties_created_at = z.string();

export type CreateUserResponse_properties_data_properties_email = z.infer<
  typeof CreateUserResponse_properties_data_properties_email
>;
export const CreateUserResponse_properties_data_properties_email = z.string();

export type CreateUserResponse_properties_data_properties_email_verified = z.infer<
  typeof CreateUserResponse_properties_data_properties_email_verified
>;
export const CreateUserResponse_properties_data_properties_email_verified = z.boolean();

export type CreateUserResponse_properties_data_properties_enabled = z.infer<
  typeof CreateUserResponse_properties_data_properties_enabled
>;
export const CreateUserResponse_properties_data_properties_enabled = z.boolean();

export type CreateUserResponse_properties_data_properties_firstname = z.infer<
  typeof CreateUserResponse_properties_data_properties_firstname
>;
export const CreateUserResponse_properties_data_properties_firstname = z.string();

export type CreateUserResponse_properties_data_properties_id = z.infer<
  typeof CreateUserResponse_properties_data_properties_id
>;
export const CreateUserResponse_properties_data_properties_id = z.string();

export type CreateUserResponse_properties_data_properties_lastname = z.infer<
  typeof CreateUserResponse_properties_data_properties_lastname
>;
export const CreateUserResponse_properties_data_properties_lastname = z.string();

export type CreateUserResponse_properties_data_properties_realm = z.infer<
  typeof CreateUserResponse_properties_data_properties_realm
>;
export const CreateUserResponse_properties_data_properties_realm = z.union([z.union([z.null(), Realm]), z.undefined()]);

export type CreateUserResponse_properties_data_properties_realm_id = z.infer<
  typeof CreateUserResponse_properties_data_properties_realm_id
>;
export const CreateUserResponse_properties_data_properties_realm_id = z.string();

export type CreateUserResponse_properties_data_properties_required_actions = z.infer<
  typeof CreateUserResponse_properties_data_properties_required_actions
>;
export const CreateUserResponse_properties_data_properties_required_actions = z.array(RequiredAction);

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0 = z.infer<
  typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0
>;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0 = z.null();

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.infer<
    typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id
  >;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.infer<
    typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type
  >;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.infer<
    typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at
  >;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.infer<
    typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled
  >;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id =
  z.infer<
    typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id
  >;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id =
  z.string();

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name =
  z.infer<
    typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name
  >;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name =
  z.string();

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.infer<
    typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol
  >;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.infer<
    typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client
  >;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.infer<
    typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id
  >;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.infer<
    typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1
  >;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.infer<
    typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1
  >;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.infer<
    typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret
  >;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.infer<
    typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled
  >;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.infer<
    typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at
  >;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_1 = z.infer<
  typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_1
>;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_1 = z.undefined();

export type CreateUserResponse_properties_data_properties_roles_items_properties_client_id = z.infer<
  typeof CreateUserResponse_properties_data_properties_roles_items_properties_client_id
>;
export const CreateUserResponse_properties_data_properties_roles_items_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type CreateUserResponse_properties_data_properties_roles_items_properties_created_at = z.infer<
  typeof CreateUserResponse_properties_data_properties_roles_items_properties_created_at
>;
export const CreateUserResponse_properties_data_properties_roles_items_properties_created_at = z.string();

export type CreateUserResponse_properties_data_properties_roles_items_properties_description = z.infer<
  typeof CreateUserResponse_properties_data_properties_roles_items_properties_description
>;
export const CreateUserResponse_properties_data_properties_roles_items_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type CreateUserResponse_properties_data_properties_roles_items_properties_id = z.infer<
  typeof CreateUserResponse_properties_data_properties_roles_items_properties_id
>;
export const CreateUserResponse_properties_data_properties_roles_items_properties_id = z.string();

export type CreateUserResponse_properties_data_properties_roles_items_properties_name = z.infer<
  typeof CreateUserResponse_properties_data_properties_roles_items_properties_name
>;
export const CreateUserResponse_properties_data_properties_roles_items_properties_name = z.string();

export type CreateUserResponse_properties_data_properties_roles_items_properties_permissions = z.infer<
  typeof CreateUserResponse_properties_data_properties_roles_items_properties_permissions
>;
export const CreateUserResponse_properties_data_properties_roles_items_properties_permissions = z.array(z.string());

export type CreateUserResponse_properties_data_properties_roles_items_properties_realm_id = z.infer<
  typeof CreateUserResponse_properties_data_properties_roles_items_properties_realm_id
>;
export const CreateUserResponse_properties_data_properties_roles_items_properties_realm_id = z.string();

export type CreateUserResponse_properties_data_properties_roles_items_properties_updated_at = z.infer<
  typeof CreateUserResponse_properties_data_properties_roles_items_properties_updated_at
>;
export const CreateUserResponse_properties_data_properties_roles_items_properties_updated_at = z.string();

export type CreateUserResponse_properties_data_properties_updated_at = z.infer<
  typeof CreateUserResponse_properties_data_properties_updated_at
>;
export const CreateUserResponse_properties_data_properties_updated_at = z.string();

export type CreateUserResponse_properties_data_properties_username = z.infer<
  typeof CreateUserResponse_properties_data_properties_username
>;
export const CreateUserResponse_properties_data_properties_username = z.string();

export type CreateUserValidator = z.infer<typeof CreateUserValidator>;
export const CreateUserValidator = z.object({
  email: z.string().optional(),
  email_verified: z.union([z.boolean(), z.null()]).optional(),
  firstname: z.string().optional(),
  lastname: z.string().optional(),
  username: z.string().optional(),
});

export type CredentialData = z.infer<typeof CredentialData>;
export const CredentialData = z.object({
  algorithm: z.string(),
  hash_iterations: z.number(),
});

export type CredentialOverview = z.infer<typeof CredentialOverview>;
export const CredentialOverview = z.object({
  created_at: z.string(),
  credential_data: CredentialData,
  credential_type: z.string(),
  id: z.string(),
  updated_at: z.string(),
  user_id: z.string(),
  user_label: z.union([z.union([z.string(), z.null()]), z.undefined()]).optional(),
});

export type DeleteClientResponse = z.infer<typeof DeleteClientResponse>;
export const DeleteClientResponse = z.object({
  message: z.string(),
});

export type DeleteRealmResponse = z.infer<typeof DeleteRealmResponse>;
export const DeleteRealmResponse = z.string();

export type DeleteUserCredentialResponse = z.infer<typeof DeleteUserCredentialResponse>;
export const DeleteUserCredentialResponse = z.object({
  message: z.string(),
});

export type DeleteUserResponse = z.infer<typeof DeleteUserResponse>;
export const DeleteUserResponse = z.object({
  count: z.number(),
});

export type JwkKey = z.infer<typeof JwkKey>;
export const JwkKey = z.object({
  alg: z.string(),
  e: z.string(),
  kid: z.string(),
  kty: z.string(),
  n: z.string(),
  use_: z.string(),
  x5c: z.string(),
});

export type GetCertsResponse = z.infer<typeof GetCertsResponse>;
export const GetCertsResponse = z.object({
  keys: z.array(JwkKey),
});

export type GetClientResponse_properties_data_properties_client_id = z.infer<
  typeof GetClientResponse_properties_data_properties_client_id
>;
export const GetClientResponse_properties_data_properties_client_id = z.string();

export type GetClientResponse_properties_data_properties_client_type = z.infer<
  typeof GetClientResponse_properties_data_properties_client_type
>;
export const GetClientResponse_properties_data_properties_client_type = z.string();

export type GetClientResponse_properties_data_properties_created_at = z.infer<
  typeof GetClientResponse_properties_data_properties_created_at
>;
export const GetClientResponse_properties_data_properties_created_at = z.string();

export type GetClientResponse_properties_data_properties_enabled = z.infer<
  typeof GetClientResponse_properties_data_properties_enabled
>;
export const GetClientResponse_properties_data_properties_enabled = z.boolean();

export type GetClientResponse_properties_data_properties_id = z.infer<
  typeof GetClientResponse_properties_data_properties_id
>;
export const GetClientResponse_properties_data_properties_id = z.string();

export type GetClientResponse_properties_data_properties_name = z.infer<
  typeof GetClientResponse_properties_data_properties_name
>;
export const GetClientResponse_properties_data_properties_name = z.string();

export type GetClientResponse_properties_data_properties_protocol = z.infer<
  typeof GetClientResponse_properties_data_properties_protocol
>;
export const GetClientResponse_properties_data_properties_protocol = z.string();

export type GetClientResponse_properties_data_properties_public_client = z.infer<
  typeof GetClientResponse_properties_data_properties_public_client
>;
export const GetClientResponse_properties_data_properties_public_client = z.boolean();

export type GetClientResponse_properties_data_properties_realm_id = z.infer<
  typeof GetClientResponse_properties_data_properties_realm_id
>;
export const GetClientResponse_properties_data_properties_realm_id = z.string();

export type GetClientResponse_properties_data_properties_redirect_uris_anyOf_0_anyOf_1 = z.infer<
  typeof GetClientResponse_properties_data_properties_redirect_uris_anyOf_0_anyOf_1
>;
export const GetClientResponse_properties_data_properties_redirect_uris_anyOf_0_anyOf_1 = z.null();

export type GetClientResponse_properties_data_properties_redirect_uris_anyOf_1 = z.infer<
  typeof GetClientResponse_properties_data_properties_redirect_uris_anyOf_1
>;
export const GetClientResponse_properties_data_properties_redirect_uris_anyOf_1 = z.undefined();

export type GetClientResponse_properties_data_properties_secret = z.infer<
  typeof GetClientResponse_properties_data_properties_secret
>;
export const GetClientResponse_properties_data_properties_secret = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type GetClientResponse_properties_data_properties_service_account_enabled = z.infer<
  typeof GetClientResponse_properties_data_properties_service_account_enabled
>;
export const GetClientResponse_properties_data_properties_service_account_enabled = z.boolean();

export type GetClientResponse_properties_data_properties_updated_at = z.infer<
  typeof GetClientResponse_properties_data_properties_updated_at
>;
export const GetClientResponse_properties_data_properties_updated_at = z.string();

export type GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_0 = z.infer<
  typeof GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_0
>;
export const GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_0 = z.null();

export type GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.infer<typeof GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id>;
export const GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.infer<typeof GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type>;
export const GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.infer<typeof GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at>;
export const GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled = z.infer<
  typeof GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled
>;
export const GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id = z.infer<
  typeof GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id
>;
export const GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id = z.string();

export type GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name = z.infer<
  typeof GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name
>;
export const GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name =
  z.string();

export type GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.infer<typeof GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol>;
export const GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.infer<
    typeof GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client
  >;
export const GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.infer<typeof GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id>;
export const GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.infer<
    typeof GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1
  >;
export const GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.infer<
    typeof GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1
  >;
export const GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret = z.infer<
  typeof GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret
>;
export const GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret = z.union(
  [z.union([z.string(), z.null()]), z.undefined()],
);

export type GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.infer<
    typeof GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled
  >;
export const GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.infer<typeof GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at>;
export const GetClientRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type GetClientRolesResponse_properties_data_items_properties_client_anyOf_1 = z.infer<
  typeof GetClientRolesResponse_properties_data_items_properties_client_anyOf_1
>;
export const GetClientRolesResponse_properties_data_items_properties_client_anyOf_1 = z.undefined();

export type GetClientRolesResponse_properties_data_items_properties_client_id = z.infer<
  typeof GetClientRolesResponse_properties_data_items_properties_client_id
>;
export const GetClientRolesResponse_properties_data_items_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type GetClientRolesResponse_properties_data_items_properties_created_at = z.infer<
  typeof GetClientRolesResponse_properties_data_items_properties_created_at
>;
export const GetClientRolesResponse_properties_data_items_properties_created_at = z.string();

export type GetClientRolesResponse_properties_data_items_properties_description = z.infer<
  typeof GetClientRolesResponse_properties_data_items_properties_description
>;
export const GetClientRolesResponse_properties_data_items_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type GetClientRolesResponse_properties_data_items_properties_id = z.infer<
  typeof GetClientRolesResponse_properties_data_items_properties_id
>;
export const GetClientRolesResponse_properties_data_items_properties_id = z.string();

export type GetClientRolesResponse_properties_data_items_properties_name = z.infer<
  typeof GetClientRolesResponse_properties_data_items_properties_name
>;
export const GetClientRolesResponse_properties_data_items_properties_name = z.string();

export type GetClientRolesResponse_properties_data_items_properties_permissions = z.infer<
  typeof GetClientRolesResponse_properties_data_items_properties_permissions
>;
export const GetClientRolesResponse_properties_data_items_properties_permissions = z.array(z.string());

export type GetClientRolesResponse_properties_data_items_properties_realm_id = z.infer<
  typeof GetClientRolesResponse_properties_data_items_properties_realm_id
>;
export const GetClientRolesResponse_properties_data_items_properties_realm_id = z.string();

export type GetClientRolesResponse_properties_data_items_properties_updated_at = z.infer<
  typeof GetClientRolesResponse_properties_data_items_properties_updated_at
>;
export const GetClientRolesResponse_properties_data_items_properties_updated_at = z.string();

export type GetOpenIdConfigurationResponse = z.infer<typeof GetOpenIdConfigurationResponse>;
export const GetOpenIdConfigurationResponse = z.object({
  authorization_endpoint: z.string(),
  grant_types_supported: z.array(z.string()),
  introspection_endpoint: z.string(),
  issuer: z.string(),
  jwks_uri: z.string(),
  token_endpoint: z.string(),
  userinfo_endpoint: z.string(),
});

export type GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_0 = z.infer<
  typeof GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_0
>;
export const GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_0 = z.null();

export type GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id = z.infer<
  typeof GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id
>;
export const GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id = z.string();

export type GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type = z.infer<
  typeof GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type
>;
export const GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type = z.string();

export type GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at = z.infer<
  typeof GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at
>;
export const GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at = z.string();

export type GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled = z.infer<
  typeof GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled
>;
export const GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled = z.boolean();

export type GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_id = z.infer<
  typeof GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_id
>;
export const GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_id = z.string();

export type GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_name = z.infer<
  typeof GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_name
>;
export const GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_name = z.string();

export type GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol = z.infer<
  typeof GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol
>;
export const GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol = z.string();

export type GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client = z.infer<
  typeof GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client
>;
export const GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client = z.boolean();

export type GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id = z.infer<
  typeof GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id
>;
export const GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id = z.string();

export type GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.infer<
    typeof GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1
  >;
export const GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.infer<typeof GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1>;
export const GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret = z.infer<
  typeof GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret
>;
export const GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.infer<typeof GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled>;
export const GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at = z.infer<
  typeof GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at
>;
export const GetRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at = z.string();

export type GetRoleResponse_properties_data_properties_client_anyOf_1 = z.infer<
  typeof GetRoleResponse_properties_data_properties_client_anyOf_1
>;
export const GetRoleResponse_properties_data_properties_client_anyOf_1 = z.undefined();

export type GetRoleResponse_properties_data_properties_client_id = z.infer<
  typeof GetRoleResponse_properties_data_properties_client_id
>;
export const GetRoleResponse_properties_data_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type GetRoleResponse_properties_data_properties_created_at = z.infer<
  typeof GetRoleResponse_properties_data_properties_created_at
>;
export const GetRoleResponse_properties_data_properties_created_at = z.string();

export type GetRoleResponse_properties_data_properties_description = z.infer<
  typeof GetRoleResponse_properties_data_properties_description
>;
export const GetRoleResponse_properties_data_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type GetRoleResponse_properties_data_properties_id = z.infer<
  typeof GetRoleResponse_properties_data_properties_id
>;
export const GetRoleResponse_properties_data_properties_id = z.string();

export type GetRoleResponse_properties_data_properties_name = z.infer<
  typeof GetRoleResponse_properties_data_properties_name
>;
export const GetRoleResponse_properties_data_properties_name = z.string();

export type GetRoleResponse_properties_data_properties_permissions = z.infer<
  typeof GetRoleResponse_properties_data_properties_permissions
>;
export const GetRoleResponse_properties_data_properties_permissions = z.array(z.string());

export type GetRoleResponse_properties_data_properties_realm_id = z.infer<
  typeof GetRoleResponse_properties_data_properties_realm_id
>;
export const GetRoleResponse_properties_data_properties_realm_id = z.string();

export type GetRoleResponse_properties_data_properties_updated_at = z.infer<
  typeof GetRoleResponse_properties_data_properties_updated_at
>;
export const GetRoleResponse_properties_data_properties_updated_at = z.string();

export type GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_0 = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_0
>;
export const GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_0 = z.null();

export type GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id
>;
export const GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id = z.string();

export type GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type
>;
export const GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at
>;
export const GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled
>;
export const GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled = z.boolean();

export type GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id
>;
export const GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id = z.string();

export type GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name
>;
export const GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name = z.string();

export type GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol
>;
export const GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol = z.string();

export type GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client
>;
export const GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id
>;
export const GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id = z.string();

export type GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.infer<
    typeof GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1
  >;
export const GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.infer<
    typeof GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1
  >;
export const GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret
>;
export const GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.infer<
    typeof GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled
  >;
export const GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at
>;
export const GetRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type GetRolesResponse_properties_data_items_properties_client_anyOf_1 = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_client_anyOf_1
>;
export const GetRolesResponse_properties_data_items_properties_client_anyOf_1 = z.undefined();

export type GetRolesResponse_properties_data_items_properties_client_id = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_client_id
>;
export const GetRolesResponse_properties_data_items_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type GetRolesResponse_properties_data_items_properties_created_at = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_created_at
>;
export const GetRolesResponse_properties_data_items_properties_created_at = z.string();

export type GetRolesResponse_properties_data_items_properties_description = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_description
>;
export const GetRolesResponse_properties_data_items_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type GetRolesResponse_properties_data_items_properties_id = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_id
>;
export const GetRolesResponse_properties_data_items_properties_id = z.string();

export type GetRolesResponse_properties_data_items_properties_name = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_name
>;
export const GetRolesResponse_properties_data_items_properties_name = z.string();

export type GetRolesResponse_properties_data_items_properties_permissions = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_permissions
>;
export const GetRolesResponse_properties_data_items_properties_permissions = z.array(z.string());

export type GetRolesResponse_properties_data_items_properties_realm_id = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_realm_id
>;
export const GetRolesResponse_properties_data_items_properties_realm_id = z.string();

export type GetRolesResponse_properties_data_items_properties_updated_at = z.infer<
  typeof GetRolesResponse_properties_data_items_properties_updated_at
>;
export const GetRolesResponse_properties_data_items_properties_updated_at = z.string();

export type GetUserCredentialsResponse = z.infer<typeof GetUserCredentialsResponse>;
export const GetUserCredentialsResponse = z.object({
  data: z.array(CredentialOverview),
});

export type GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_0 = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_0
>;
export const GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_0 = z.null();

export type GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id
>;
export const GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.infer<typeof GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type>;
export const GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.infer<typeof GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at>;
export const GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled
>;
export const GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id
>;
export const GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id = z.string();

export type GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name
>;
export const GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name = z.string();

export type GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol
>;
export const GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.infer<typeof GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client>;
export const GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id
>;
export const GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.infer<
    typeof GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1
  >;
export const GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.infer<
    typeof GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1
  >;
export const GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret
>;
export const GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.infer<
    typeof GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled
  >;
export const GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.infer<typeof GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at>;
export const GetUserRolesResponse_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type GetUserRolesResponse_properties_data_items_properties_client_anyOf_1 = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_client_anyOf_1
>;
export const GetUserRolesResponse_properties_data_items_properties_client_anyOf_1 = z.undefined();

export type GetUserRolesResponse_properties_data_items_properties_client_id = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_client_id
>;
export const GetUserRolesResponse_properties_data_items_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type GetUserRolesResponse_properties_data_items_properties_created_at = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_created_at
>;
export const GetUserRolesResponse_properties_data_items_properties_created_at = z.string();

export type GetUserRolesResponse_properties_data_items_properties_description = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_description
>;
export const GetUserRolesResponse_properties_data_items_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type GetUserRolesResponse_properties_data_items_properties_id = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_id
>;
export const GetUserRolesResponse_properties_data_items_properties_id = z.string();

export type GetUserRolesResponse_properties_data_items_properties_name = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_name
>;
export const GetUserRolesResponse_properties_data_items_properties_name = z.string();

export type GetUserRolesResponse_properties_data_items_properties_permissions = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_permissions
>;
export const GetUserRolesResponse_properties_data_items_properties_permissions = z.array(z.string());

export type GetUserRolesResponse_properties_data_items_properties_realm_id = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_realm_id
>;
export const GetUserRolesResponse_properties_data_items_properties_realm_id = z.string();

export type GetUserRolesResponse_properties_data_items_properties_updated_at = z.infer<
  typeof GetUserRolesResponse_properties_data_items_properties_updated_at
>;
export const GetUserRolesResponse_properties_data_items_properties_updated_at = z.string();

export type GrantType = z.infer<typeof GrantType>;
export const GrantType = z.union([
  z.literal("authorization_code"),
  z.literal("password"),
  z.literal("client_credentials"),
  z.literal("refresh_token"),
]);

export type JwtToken = z.infer<typeof JwtToken>;
export const JwtToken = z.object({
  access_token: z.string(),
  expires_in: z.number(),
  id_token: z.string(),
  refresh_token: z.string(),
  token_type: z.string(),
});

export type OtpVerifyRequest = z.infer<typeof OtpVerifyRequest>;
export const OtpVerifyRequest = z.object({
  code: z.string(),
  label: z.string(),
  secret: z.string(),
});

export type RedirectUri = z.infer<typeof RedirectUri>;
export const RedirectUri = z.object({
  client_id: z.string(),
  created_at: z.string(),
  enabled: z.boolean(),
  id: z.string(),
  updated_at: z.string(),
  value: z.string(),
});

export type ResetPasswordValidator = z.infer<typeof ResetPasswordValidator>;
export const ResetPasswordValidator = z.object({
  credential_type: z.string().optional(),
  temporary: z.boolean().optional(),
  value: z.string().optional(),
});

export type SetupOtpResponse = z.infer<typeof SetupOtpResponse>;
export const SetupOtpResponse = z.object({
  issuer: z.string(),
  otpauth_url: z.string(),
  secret: z.string(),
});

export type TokenRequestValidator = z.infer<typeof TokenRequestValidator>;
export const TokenRequestValidator = z.object({
  client_id: z.string().optional(),
  client_secret: z.union([z.string(), z.null()]).optional(),
  code: z.union([z.string(), z.null()]).optional(),
  grant_type: GrantType.optional(),
  password: z.union([z.string(), z.null()]).optional(),
  refresh_token: z.union([z.string(), z.null()]).optional(),
  username: z.union([z.string(), z.null()]).optional(),
});

export type UnassignRoleResponse = z.infer<typeof UnassignRoleResponse>;
export const UnassignRoleResponse = z.object({
  message: z.string(),
});

export type UpdateClientValidator = z.infer<typeof UpdateClientValidator>;
export const UpdateClientValidator = z.object({
  client_id: z.union([z.string(), z.null()]).optional(),
  enabled: z.union([z.boolean(), z.null()]).optional(),
  name: z.union([z.string(), z.null()]).optional(),
});

export type UpdateRealmSettingValidator = z.infer<typeof UpdateRealmSettingValidator>;
export const UpdateRealmSettingValidator = z.object({
  default_signing_algorithm: z.string(),
});

export type UpdateRealmValidator = z.infer<typeof UpdateRealmValidator>;
export const UpdateRealmValidator = z.object({
  name: z.string(),
});

export type UpdateRedirectUriValidator = z.infer<typeof UpdateRedirectUriValidator>;
export const UpdateRedirectUriValidator = z.object({
  enabled: z.boolean().optional(),
});

export type UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_0 = z.infer<
  typeof UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_0
>;
export const UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_0 = z.null();

export type UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.infer<typeof UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id>;
export const UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.infer<
    typeof UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type
  >;
export const UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.infer<typeof UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at>;
export const UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.infer<typeof UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled>;
export const UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_id = z.infer<
  typeof UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_id
>;
export const UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_id = z.string();

export type UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_name = z.infer<
  typeof UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_name
>;
export const UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_name =
  z.string();

export type UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.infer<typeof UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol>;
export const UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.infer<
    typeof UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client
  >;
export const UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.infer<typeof UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id>;
export const UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.infer<
    typeof UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1
  >;
export const UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.infer<
    typeof UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1
  >;
export const UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret = z.infer<
  typeof UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret
>;
export const UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.infer<
    typeof UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled
  >;
export const UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.infer<typeof UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at>;
export const UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_1 = z.infer<
  typeof UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_1
>;
export const UpdateRolePermissionsResponse_properties_data_properties_client_anyOf_1 = z.undefined();

export type UpdateRolePermissionsResponse_properties_data_properties_client_id = z.infer<
  typeof UpdateRolePermissionsResponse_properties_data_properties_client_id
>;
export const UpdateRolePermissionsResponse_properties_data_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type UpdateRolePermissionsResponse_properties_data_properties_created_at = z.infer<
  typeof UpdateRolePermissionsResponse_properties_data_properties_created_at
>;
export const UpdateRolePermissionsResponse_properties_data_properties_created_at = z.string();

export type UpdateRolePermissionsResponse_properties_data_properties_description = z.infer<
  typeof UpdateRolePermissionsResponse_properties_data_properties_description
>;
export const UpdateRolePermissionsResponse_properties_data_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type UpdateRolePermissionsResponse_properties_data_properties_id = z.infer<
  typeof UpdateRolePermissionsResponse_properties_data_properties_id
>;
export const UpdateRolePermissionsResponse_properties_data_properties_id = z.string();

export type UpdateRolePermissionsResponse_properties_data_properties_name = z.infer<
  typeof UpdateRolePermissionsResponse_properties_data_properties_name
>;
export const UpdateRolePermissionsResponse_properties_data_properties_name = z.string();

export type UpdateRolePermissionsResponse_properties_data_properties_permissions = z.infer<
  typeof UpdateRolePermissionsResponse_properties_data_properties_permissions
>;
export const UpdateRolePermissionsResponse_properties_data_properties_permissions = z.array(z.string());

export type UpdateRolePermissionsResponse_properties_data_properties_realm_id = z.infer<
  typeof UpdateRolePermissionsResponse_properties_data_properties_realm_id
>;
export const UpdateRolePermissionsResponse_properties_data_properties_realm_id = z.string();

export type UpdateRolePermissionsResponse_properties_data_properties_updated_at = z.infer<
  typeof UpdateRolePermissionsResponse_properties_data_properties_updated_at
>;
export const UpdateRolePermissionsResponse_properties_data_properties_updated_at = z.string();

export type UpdateRolePermissionsValidator = z.infer<typeof UpdateRolePermissionsValidator>;
export const UpdateRolePermissionsValidator = z.object({
  permissions: z.array(z.string()),
});

export type UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_0 = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_0
>;
export const UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_0 = z.null();

export type UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id
>;
export const UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id = z.string();

export type UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type
>;
export const UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type = z.string();

export type UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at
>;
export const UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at = z.string();

export type UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled
>;
export const UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled = z.boolean();

export type UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_id = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_id
>;
export const UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_id = z.string();

export type UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_name = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_name
>;
export const UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_name = z.string();

export type UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol
>;
export const UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol = z.string();

export type UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client
>;
export const UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id
>;
export const UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id = z.string();

export type UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.infer<
    typeof UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1
  >;
export const UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.infer<typeof UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1>;
export const UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret
>;
export const UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.infer<
    typeof UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled
  >;
export const UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at
>;
export const UpdateRoleResponse_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at = z.string();

export type UpdateRoleResponse_properties_data_properties_client_anyOf_1 = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_client_anyOf_1
>;
export const UpdateRoleResponse_properties_data_properties_client_anyOf_1 = z.undefined();

export type UpdateRoleResponse_properties_data_properties_client_id = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_client_id
>;
export const UpdateRoleResponse_properties_data_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type UpdateRoleResponse_properties_data_properties_created_at = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_created_at
>;
export const UpdateRoleResponse_properties_data_properties_created_at = z.string();

export type UpdateRoleResponse_properties_data_properties_description = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_description
>;
export const UpdateRoleResponse_properties_data_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type UpdateRoleResponse_properties_data_properties_id = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_id
>;
export const UpdateRoleResponse_properties_data_properties_id = z.string();

export type UpdateRoleResponse_properties_data_properties_name = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_name
>;
export const UpdateRoleResponse_properties_data_properties_name = z.string();

export type UpdateRoleResponse_properties_data_properties_permissions = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_permissions
>;
export const UpdateRoleResponse_properties_data_properties_permissions = z.array(z.string());

export type UpdateRoleResponse_properties_data_properties_realm_id = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_realm_id
>;
export const UpdateRoleResponse_properties_data_properties_realm_id = z.string();

export type UpdateRoleResponse_properties_data_properties_updated_at = z.infer<
  typeof UpdateRoleResponse_properties_data_properties_updated_at
>;
export const UpdateRoleResponse_properties_data_properties_updated_at = z.string();

export type UpdateRoleValidator = z.infer<typeof UpdateRoleValidator>;
export const UpdateRoleValidator = z.object({
  description: z.union([z.string(), z.null()]).optional(),
  name: z.union([z.string(), z.null()]).optional(),
});

export type UpdateUserResponse_properties_data_properties_client_id = z.infer<
  typeof UpdateUserResponse_properties_data_properties_client_id
>;
export const UpdateUserResponse_properties_data_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type UpdateUserResponse_properties_data_properties_created_at = z.infer<
  typeof UpdateUserResponse_properties_data_properties_created_at
>;
export const UpdateUserResponse_properties_data_properties_created_at = z.string();

export type UpdateUserResponse_properties_data_properties_email = z.infer<
  typeof UpdateUserResponse_properties_data_properties_email
>;
export const UpdateUserResponse_properties_data_properties_email = z.string();

export type UpdateUserResponse_properties_data_properties_email_verified = z.infer<
  typeof UpdateUserResponse_properties_data_properties_email_verified
>;
export const UpdateUserResponse_properties_data_properties_email_verified = z.boolean();

export type UpdateUserResponse_properties_data_properties_enabled = z.infer<
  typeof UpdateUserResponse_properties_data_properties_enabled
>;
export const UpdateUserResponse_properties_data_properties_enabled = z.boolean();

export type UpdateUserResponse_properties_data_properties_firstname = z.infer<
  typeof UpdateUserResponse_properties_data_properties_firstname
>;
export const UpdateUserResponse_properties_data_properties_firstname = z.string();

export type UpdateUserResponse_properties_data_properties_id = z.infer<
  typeof UpdateUserResponse_properties_data_properties_id
>;
export const UpdateUserResponse_properties_data_properties_id = z.string();

export type UpdateUserResponse_properties_data_properties_lastname = z.infer<
  typeof UpdateUserResponse_properties_data_properties_lastname
>;
export const UpdateUserResponse_properties_data_properties_lastname = z.string();

export type UpdateUserResponse_properties_data_properties_realm = z.infer<
  typeof UpdateUserResponse_properties_data_properties_realm
>;
export const UpdateUserResponse_properties_data_properties_realm = z.union([z.union([z.null(), Realm]), z.undefined()]);

export type UpdateUserResponse_properties_data_properties_realm_id = z.infer<
  typeof UpdateUserResponse_properties_data_properties_realm_id
>;
export const UpdateUserResponse_properties_data_properties_realm_id = z.string();

export type UpdateUserResponse_properties_data_properties_required_actions = z.infer<
  typeof UpdateUserResponse_properties_data_properties_required_actions
>;
export const UpdateUserResponse_properties_data_properties_required_actions = z.array(RequiredAction);

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0 = z.infer<
  typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0
>;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0 = z.null();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.infer<
    typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id
  >;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.infer<
    typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type
  >;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.infer<
    typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at
  >;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.infer<
    typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled
  >;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id =
  z.infer<
    typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id
  >;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id =
  z.string();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name =
  z.infer<
    typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name
  >;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name =
  z.string();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.infer<
    typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol
  >;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.infer<
    typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client
  >;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.infer<
    typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id
  >;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.infer<
    typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1
  >;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.infer<
    typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1
  >;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.infer<
    typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret
  >;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.infer<
    typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled
  >;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.infer<
    typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at
  >;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_1 = z.infer<
  typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_1
>;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_anyOf_1 = z.undefined();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_client_id = z.infer<
  typeof UpdateUserResponse_properties_data_properties_roles_items_properties_client_id
>;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type UpdateUserResponse_properties_data_properties_roles_items_properties_created_at = z.infer<
  typeof UpdateUserResponse_properties_data_properties_roles_items_properties_created_at
>;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_created_at = z.string();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_description = z.infer<
  typeof UpdateUserResponse_properties_data_properties_roles_items_properties_description
>;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type UpdateUserResponse_properties_data_properties_roles_items_properties_id = z.infer<
  typeof UpdateUserResponse_properties_data_properties_roles_items_properties_id
>;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_id = z.string();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_name = z.infer<
  typeof UpdateUserResponse_properties_data_properties_roles_items_properties_name
>;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_name = z.string();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_permissions = z.infer<
  typeof UpdateUserResponse_properties_data_properties_roles_items_properties_permissions
>;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_permissions = z.array(z.string());

export type UpdateUserResponse_properties_data_properties_roles_items_properties_realm_id = z.infer<
  typeof UpdateUserResponse_properties_data_properties_roles_items_properties_realm_id
>;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_realm_id = z.string();

export type UpdateUserResponse_properties_data_properties_roles_items_properties_updated_at = z.infer<
  typeof UpdateUserResponse_properties_data_properties_roles_items_properties_updated_at
>;
export const UpdateUserResponse_properties_data_properties_roles_items_properties_updated_at = z.string();

export type UpdateUserResponse_properties_data_properties_updated_at = z.infer<
  typeof UpdateUserResponse_properties_data_properties_updated_at
>;
export const UpdateUserResponse_properties_data_properties_updated_at = z.string();

export type UpdateUserResponse_properties_data_properties_username = z.infer<
  typeof UpdateUserResponse_properties_data_properties_username
>;
export const UpdateUserResponse_properties_data_properties_username = z.string();

export type UpdateUserValidator = z.infer<typeof UpdateUserValidator>;
export const UpdateUserValidator = z.object({
  email: z.string().optional(),
  email_verified: z.union([z.boolean(), z.null()]).optional(),
  enabled: z.union([z.boolean(), z.null()]).optional(),
  firstname: z.string().optional(),
  lastname: z.string().optional(),
  required_actions: z.union([z.array(z.string()), z.null()]).optional(),
});

export type UserRealmsResponse = z.infer<typeof UserRealmsResponse>;
export const UserRealmsResponse = z.object({
  data: z.array(Realm),
});

export type UserResponse_properties_data_properties_client_id = z.infer<
  typeof UserResponse_properties_data_properties_client_id
>;
export const UserResponse_properties_data_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type UserResponse_properties_data_properties_created_at = z.infer<
  typeof UserResponse_properties_data_properties_created_at
>;
export const UserResponse_properties_data_properties_created_at = z.string();

export type UserResponse_properties_data_properties_email = z.infer<
  typeof UserResponse_properties_data_properties_email
>;
export const UserResponse_properties_data_properties_email = z.string();

export type UserResponse_properties_data_properties_email_verified = z.infer<
  typeof UserResponse_properties_data_properties_email_verified
>;
export const UserResponse_properties_data_properties_email_verified = z.boolean();

export type UserResponse_properties_data_properties_enabled = z.infer<
  typeof UserResponse_properties_data_properties_enabled
>;
export const UserResponse_properties_data_properties_enabled = z.boolean();

export type UserResponse_properties_data_properties_firstname = z.infer<
  typeof UserResponse_properties_data_properties_firstname
>;
export const UserResponse_properties_data_properties_firstname = z.string();

export type UserResponse_properties_data_properties_id = z.infer<typeof UserResponse_properties_data_properties_id>;
export const UserResponse_properties_data_properties_id = z.string();

export type UserResponse_properties_data_properties_lastname = z.infer<
  typeof UserResponse_properties_data_properties_lastname
>;
export const UserResponse_properties_data_properties_lastname = z.string();

export type UserResponse_properties_data_properties_realm = z.infer<
  typeof UserResponse_properties_data_properties_realm
>;
export const UserResponse_properties_data_properties_realm = z.union([z.union([z.null(), Realm]), z.undefined()]);

export type UserResponse_properties_data_properties_realm_id = z.infer<
  typeof UserResponse_properties_data_properties_realm_id
>;
export const UserResponse_properties_data_properties_realm_id = z.string();

export type UserResponse_properties_data_properties_required_actions = z.infer<
  typeof UserResponse_properties_data_properties_required_actions
>;
export const UserResponse_properties_data_properties_required_actions = z.array(RequiredAction);

export type UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0 = z.infer<
  typeof UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0
>;
export const UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0 = z.null();

export type UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.infer<
    typeof UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id
  >;
export const UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.infer<
    typeof UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type
  >;
export const UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.infer<
    typeof UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at
  >;
export const UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.infer<
    typeof UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled
  >;
export const UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id =
  z.infer<typeof UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id>;
export const UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id =
  z.string();

export type UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name =
  z.infer<typeof UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name>;
export const UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name =
  z.string();

export type UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.infer<
    typeof UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol
  >;
export const UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.infer<
    typeof UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client
  >;
export const UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.infer<
    typeof UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id
  >;
export const UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.infer<
    typeof UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1
  >;
export const UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.infer<
    typeof UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1
  >;
export const UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.infer<
    typeof UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret
  >;
export const UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.infer<
    typeof UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled
  >;
export const UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.infer<
    typeof UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at
  >;
export const UserResponse_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type UserResponse_properties_data_properties_roles_items_properties_client_anyOf_1 = z.infer<
  typeof UserResponse_properties_data_properties_roles_items_properties_client_anyOf_1
>;
export const UserResponse_properties_data_properties_roles_items_properties_client_anyOf_1 = z.undefined();

export type UserResponse_properties_data_properties_roles_items_properties_client_id = z.infer<
  typeof UserResponse_properties_data_properties_roles_items_properties_client_id
>;
export const UserResponse_properties_data_properties_roles_items_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type UserResponse_properties_data_properties_roles_items_properties_created_at = z.infer<
  typeof UserResponse_properties_data_properties_roles_items_properties_created_at
>;
export const UserResponse_properties_data_properties_roles_items_properties_created_at = z.string();

export type UserResponse_properties_data_properties_roles_items_properties_description = z.infer<
  typeof UserResponse_properties_data_properties_roles_items_properties_description
>;
export const UserResponse_properties_data_properties_roles_items_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type UserResponse_properties_data_properties_roles_items_properties_id = z.infer<
  typeof UserResponse_properties_data_properties_roles_items_properties_id
>;
export const UserResponse_properties_data_properties_roles_items_properties_id = z.string();

export type UserResponse_properties_data_properties_roles_items_properties_name = z.infer<
  typeof UserResponse_properties_data_properties_roles_items_properties_name
>;
export const UserResponse_properties_data_properties_roles_items_properties_name = z.string();

export type UserResponse_properties_data_properties_roles_items_properties_permissions = z.infer<
  typeof UserResponse_properties_data_properties_roles_items_properties_permissions
>;
export const UserResponse_properties_data_properties_roles_items_properties_permissions = z.array(z.string());

export type UserResponse_properties_data_properties_roles_items_properties_realm_id = z.infer<
  typeof UserResponse_properties_data_properties_roles_items_properties_realm_id
>;
export const UserResponse_properties_data_properties_roles_items_properties_realm_id = z.string();

export type UserResponse_properties_data_properties_roles_items_properties_updated_at = z.infer<
  typeof UserResponse_properties_data_properties_roles_items_properties_updated_at
>;
export const UserResponse_properties_data_properties_roles_items_properties_updated_at = z.string();

export type UserResponse_properties_data_properties_updated_at = z.infer<
  typeof UserResponse_properties_data_properties_updated_at
>;
export const UserResponse_properties_data_properties_updated_at = z.string();

export type UserResponse_properties_data_properties_username = z.infer<
  typeof UserResponse_properties_data_properties_username
>;
export const UserResponse_properties_data_properties_username = z.string();

export type UsersResponse_properties_data_items_properties_client_id = z.infer<
  typeof UsersResponse_properties_data_items_properties_client_id
>;
export const UsersResponse_properties_data_items_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type UsersResponse_properties_data_items_properties_created_at = z.infer<
  typeof UsersResponse_properties_data_items_properties_created_at
>;
export const UsersResponse_properties_data_items_properties_created_at = z.string();

export type UsersResponse_properties_data_items_properties_email = z.infer<
  typeof UsersResponse_properties_data_items_properties_email
>;
export const UsersResponse_properties_data_items_properties_email = z.string();

export type UsersResponse_properties_data_items_properties_email_verified = z.infer<
  typeof UsersResponse_properties_data_items_properties_email_verified
>;
export const UsersResponse_properties_data_items_properties_email_verified = z.boolean();

export type UsersResponse_properties_data_items_properties_enabled = z.infer<
  typeof UsersResponse_properties_data_items_properties_enabled
>;
export const UsersResponse_properties_data_items_properties_enabled = z.boolean();

export type UsersResponse_properties_data_items_properties_firstname = z.infer<
  typeof UsersResponse_properties_data_items_properties_firstname
>;
export const UsersResponse_properties_data_items_properties_firstname = z.string();

export type UsersResponse_properties_data_items_properties_id = z.infer<
  typeof UsersResponse_properties_data_items_properties_id
>;
export const UsersResponse_properties_data_items_properties_id = z.string();

export type UsersResponse_properties_data_items_properties_lastname = z.infer<
  typeof UsersResponse_properties_data_items_properties_lastname
>;
export const UsersResponse_properties_data_items_properties_lastname = z.string();

export type UsersResponse_properties_data_items_properties_realm = z.infer<
  typeof UsersResponse_properties_data_items_properties_realm
>;
export const UsersResponse_properties_data_items_properties_realm = z.union([
  z.union([z.null(), Realm]),
  z.undefined(),
]);

export type UsersResponse_properties_data_items_properties_realm_id = z.infer<
  typeof UsersResponse_properties_data_items_properties_realm_id
>;
export const UsersResponse_properties_data_items_properties_realm_id = z.string();

export type UsersResponse_properties_data_items_properties_required_actions = z.infer<
  typeof UsersResponse_properties_data_items_properties_required_actions
>;
export const UsersResponse_properties_data_items_properties_required_actions = z.array(RequiredAction);

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_0 = z.infer<
  typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_0
>;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_0 = z.null();

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.infer<
    typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id
  >;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.infer<
    typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type
  >;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.infer<
    typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at
  >;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.infer<
    typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled
  >;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id =
  z.infer<
    typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id
  >;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id =
  z.string();

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name =
  z.infer<
    typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name
  >;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name =
  z.string();

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.infer<
    typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol
  >;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.infer<
    typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client
  >;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.infer<
    typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id
  >;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.infer<
    typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1
  >;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.infer<
    typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1
  >;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.infer<
    typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret
  >;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.infer<
    typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled
  >;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.infer<
    typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at
  >;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_1 = z.infer<
  typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_1
>;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_anyOf_1 = z.undefined();

export type UsersResponse_properties_data_items_properties_roles_items_properties_client_id = z.infer<
  typeof UsersResponse_properties_data_items_properties_roles_items_properties_client_id
>;
export const UsersResponse_properties_data_items_properties_roles_items_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type UsersResponse_properties_data_items_properties_roles_items_properties_created_at = z.infer<
  typeof UsersResponse_properties_data_items_properties_roles_items_properties_created_at
>;
export const UsersResponse_properties_data_items_properties_roles_items_properties_created_at = z.string();

export type UsersResponse_properties_data_items_properties_roles_items_properties_description = z.infer<
  typeof UsersResponse_properties_data_items_properties_roles_items_properties_description
>;
export const UsersResponse_properties_data_items_properties_roles_items_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type UsersResponse_properties_data_items_properties_roles_items_properties_id = z.infer<
  typeof UsersResponse_properties_data_items_properties_roles_items_properties_id
>;
export const UsersResponse_properties_data_items_properties_roles_items_properties_id = z.string();

export type UsersResponse_properties_data_items_properties_roles_items_properties_name = z.infer<
  typeof UsersResponse_properties_data_items_properties_roles_items_properties_name
>;
export const UsersResponse_properties_data_items_properties_roles_items_properties_name = z.string();

export type UsersResponse_properties_data_items_properties_roles_items_properties_permissions = z.infer<
  typeof UsersResponse_properties_data_items_properties_roles_items_properties_permissions
>;
export const UsersResponse_properties_data_items_properties_roles_items_properties_permissions = z.array(z.string());

export type UsersResponse_properties_data_items_properties_roles_items_properties_realm_id = z.infer<
  typeof UsersResponse_properties_data_items_properties_roles_items_properties_realm_id
>;
export const UsersResponse_properties_data_items_properties_roles_items_properties_realm_id = z.string();

export type UsersResponse_properties_data_items_properties_roles_items_properties_updated_at = z.infer<
  typeof UsersResponse_properties_data_items_properties_roles_items_properties_updated_at
>;
export const UsersResponse_properties_data_items_properties_roles_items_properties_updated_at = z.string();

export type UsersResponse_properties_data_items_properties_updated_at = z.infer<
  typeof UsersResponse_properties_data_items_properties_updated_at
>;
export const UsersResponse_properties_data_items_properties_updated_at = z.string();

export type UsersResponse_properties_data_items_properties_username = z.infer<
  typeof UsersResponse_properties_data_items_properties_username
>;
export const UsersResponse_properties_data_items_properties_username = z.string();

export type VerifyOtpResponse = z.infer<typeof VerifyOtpResponse>;
export const VerifyOtpResponse = z.object({
  message: z.string(),
});

export type get_Fetch_realm = typeof get_Fetch_realm;
export const get_Fetch_realm = {
  method: z.literal("GET"),
  path: z.literal("/realms"),
  requestFormat: z.literal("json"),
  parameters: z.never(),
  response: z.array(Realm),
};

export type post_Create_realm = typeof post_Create_realm;
export const post_Create_realm = {
  method: z.literal("POST"),
  path: z.literal("/realms"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    body: CreateRealmValidator,
  }),
  response: Realm,
};

export type get_Get_user_realms = typeof get_Get_user_realms;
export const get_Get_user_realms = {
  method: z.literal("GET"),
  path: z.literal("/realms/users/@me/realms"),
  requestFormat: z.literal("json"),
  parameters: z.never(),
  response: UserRealmsResponse,
};

export type get_Get_realm = typeof get_Get_realm;
export const get_Get_realm = {
  method: z.literal("GET"),
  path: z.literal("/realms/{name}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      name: z.string(),
    }),
  }),
  response: Realm,
};

export type put_Update_realm = typeof put_Update_realm;
export const put_Update_realm = {
  method: z.literal("PUT"),
  path: z.literal("/realms/{name}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      name: z.string(),
    }),
    body: UpdateRealmValidator,
  }),
  response: Realm,
};

export type delete_Delete_realm = typeof delete_Delete_realm;
export const delete_Delete_realm = {
  method: z.literal("DELETE"),
  path: z.literal("/realms/{name}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      name: z.string(),
    }),
  }),
  response: DeleteRealmResponse,
};

export type put_Update_realm_setting = typeof put_Update_realm_setting;
export const put_Update_realm_setting = {
  method: z.literal("PUT"),
  path: z.literal("/realms/{name}/settings"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      name: z.string(),
    }),
    body: UpdateRealmSettingValidator,
  }),
  response: Realm,
};

export type get_Get_openid_configuration = typeof get_Get_openid_configuration;
export const get_Get_openid_configuration = {
  method: z.literal("GET"),
  path: z.literal("/realms/{realm_name}/.well-known/openid-configuration"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
    }),
  }),
  response: GetOpenIdConfigurationResponse,
};

export type get_Get_clients_properties_method = typeof get_Get_clients_properties_method;
export const get_Get_clients_properties_method = z.literal("GET");

export type get_Get_clients_properties_path = typeof get_Get_clients_properties_path;
export const get_Get_clients_properties_path = z.literal("/realms/{realm_name}/clients");

export type get_Get_clients_properties_requestFormat = typeof get_Get_clients_properties_requestFormat;
export const get_Get_clients_properties_requestFormat = z.literal("json");

export type get_Get_clients_properties_parameters = typeof get_Get_clients_properties_parameters;
export const get_Get_clients_properties_parameters = {
  path: z.object({
    realm_name: z.string(),
  }),
};

export type get_Get_clients_properties_response_properties_data_items_properties_client_id =
  typeof get_Get_clients_properties_response_properties_data_items_properties_client_id;
export const get_Get_clients_properties_response_properties_data_items_properties_client_id = z.string();

export type get_Get_clients_properties_response_properties_data_items_properties_client_type =
  typeof get_Get_clients_properties_response_properties_data_items_properties_client_type;
export const get_Get_clients_properties_response_properties_data_items_properties_client_type = z.string();

export type get_Get_clients_properties_response_properties_data_items_properties_created_at =
  typeof get_Get_clients_properties_response_properties_data_items_properties_created_at;
export const get_Get_clients_properties_response_properties_data_items_properties_created_at = z.string();

export type get_Get_clients_properties_response_properties_data_items_properties_enabled =
  typeof get_Get_clients_properties_response_properties_data_items_properties_enabled;
export const get_Get_clients_properties_response_properties_data_items_properties_enabled = z.boolean();

export type get_Get_clients_properties_response_properties_data_items_properties_id =
  typeof get_Get_clients_properties_response_properties_data_items_properties_id;
export const get_Get_clients_properties_response_properties_data_items_properties_id = z.string();

export type get_Get_clients_properties_response_properties_data_items_properties_name =
  typeof get_Get_clients_properties_response_properties_data_items_properties_name;
export const get_Get_clients_properties_response_properties_data_items_properties_name = z.string();

export type get_Get_clients_properties_response_properties_data_items_properties_protocol =
  typeof get_Get_clients_properties_response_properties_data_items_properties_protocol;
export const get_Get_clients_properties_response_properties_data_items_properties_protocol = z.string();

export type get_Get_clients_properties_response_properties_data_items_properties_public_client =
  typeof get_Get_clients_properties_response_properties_data_items_properties_public_client;
export const get_Get_clients_properties_response_properties_data_items_properties_public_client = z.boolean();

export type get_Get_clients_properties_response_properties_data_items_properties_realm_id =
  typeof get_Get_clients_properties_response_properties_data_items_properties_realm_id;
export const get_Get_clients_properties_response_properties_data_items_properties_realm_id = z.string();

export type get_Get_clients_properties_response_properties_data_items_properties_redirect_uris_anyOf_0_anyOf_1 =
  typeof get_Get_clients_properties_response_properties_data_items_properties_redirect_uris_anyOf_0_anyOf_1;
export const get_Get_clients_properties_response_properties_data_items_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type get_Get_clients_properties_response_properties_data_items_properties_redirect_uris_anyOf_1 =
  typeof get_Get_clients_properties_response_properties_data_items_properties_redirect_uris_anyOf_1;
export const get_Get_clients_properties_response_properties_data_items_properties_redirect_uris_anyOf_1 = z.undefined();

export type get_Get_clients_properties_response_properties_data_items_properties_secret =
  typeof get_Get_clients_properties_response_properties_data_items_properties_secret;
export const get_Get_clients_properties_response_properties_data_items_properties_secret = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type get_Get_clients_properties_response_properties_data_items_properties_service_account_enabled =
  typeof get_Get_clients_properties_response_properties_data_items_properties_service_account_enabled;
export const get_Get_clients_properties_response_properties_data_items_properties_service_account_enabled = z.boolean();

export type get_Get_clients_properties_response_properties_data_items_properties_updated_at =
  typeof get_Get_clients_properties_response_properties_data_items_properties_updated_at;
export const get_Get_clients_properties_response_properties_data_items_properties_updated_at = z.string();

export type post_Create_client_properties_method = typeof post_Create_client_properties_method;
export const post_Create_client_properties_method = z.literal("POST");

export type post_Create_client_properties_path = typeof post_Create_client_properties_path;
export const post_Create_client_properties_path = z.literal("/realms/{realm_name}/clients");

export type post_Create_client_properties_requestFormat = typeof post_Create_client_properties_requestFormat;
export const post_Create_client_properties_requestFormat = z.literal("json");

export type post_Create_client_properties_parameters = typeof post_Create_client_properties_parameters;
export const post_Create_client_properties_parameters = {
  path: z.object({
    realm_name: z.string(),
  }),
  body: CreateClientValidator,
};

export type post_Create_client_properties_response_properties_client_id =
  typeof post_Create_client_properties_response_properties_client_id;
export const post_Create_client_properties_response_properties_client_id = z.string();

export type post_Create_client_properties_response_properties_client_type =
  typeof post_Create_client_properties_response_properties_client_type;
export const post_Create_client_properties_response_properties_client_type = z.string();

export type post_Create_client_properties_response_properties_created_at =
  typeof post_Create_client_properties_response_properties_created_at;
export const post_Create_client_properties_response_properties_created_at = z.string();

export type post_Create_client_properties_response_properties_enabled =
  typeof post_Create_client_properties_response_properties_enabled;
export const post_Create_client_properties_response_properties_enabled = z.boolean();

export type post_Create_client_properties_response_properties_id =
  typeof post_Create_client_properties_response_properties_id;
export const post_Create_client_properties_response_properties_id = z.string();

export type post_Create_client_properties_response_properties_name =
  typeof post_Create_client_properties_response_properties_name;
export const post_Create_client_properties_response_properties_name = z.string();

export type post_Create_client_properties_response_properties_protocol =
  typeof post_Create_client_properties_response_properties_protocol;
export const post_Create_client_properties_response_properties_protocol = z.string();

export type post_Create_client_properties_response_properties_public_client =
  typeof post_Create_client_properties_response_properties_public_client;
export const post_Create_client_properties_response_properties_public_client = z.boolean();

export type post_Create_client_properties_response_properties_realm_id =
  typeof post_Create_client_properties_response_properties_realm_id;
export const post_Create_client_properties_response_properties_realm_id = z.string();

export type post_Create_client_properties_response_properties_redirect_uris_anyOf_0_anyOf_1 =
  typeof post_Create_client_properties_response_properties_redirect_uris_anyOf_0_anyOf_1;
export const post_Create_client_properties_response_properties_redirect_uris_anyOf_0_anyOf_1 = z.null();

export type post_Create_client_properties_response_properties_redirect_uris_anyOf_1 =
  typeof post_Create_client_properties_response_properties_redirect_uris_anyOf_1;
export const post_Create_client_properties_response_properties_redirect_uris_anyOf_1 = z.undefined();

export type post_Create_client_properties_response_properties_secret =
  typeof post_Create_client_properties_response_properties_secret;
export const post_Create_client_properties_response_properties_secret = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type post_Create_client_properties_response_properties_service_account_enabled =
  typeof post_Create_client_properties_response_properties_service_account_enabled;
export const post_Create_client_properties_response_properties_service_account_enabled = z.boolean();

export type post_Create_client_properties_response_properties_updated_at =
  typeof post_Create_client_properties_response_properties_updated_at;
export const post_Create_client_properties_response_properties_updated_at = z.string();

export type get_Get_client_properties_method = typeof get_Get_client_properties_method;
export const get_Get_client_properties_method = z.literal("GET");

export type get_Get_client_properties_path = typeof get_Get_client_properties_path;
export const get_Get_client_properties_path = z.literal("/realms/{realm_name}/clients/{client_id}");

export type get_Get_client_properties_requestFormat = typeof get_Get_client_properties_requestFormat;
export const get_Get_client_properties_requestFormat = z.literal("json");

export type get_Get_client_properties_parameters = typeof get_Get_client_properties_parameters;
export const get_Get_client_properties_parameters = {
  path: z.object({
    realm_name: z.string(),
    client_id: z.string(),
  }),
};

export type get_Get_client_properties_response_properties_data_properties_client_id =
  typeof get_Get_client_properties_response_properties_data_properties_client_id;
export const get_Get_client_properties_response_properties_data_properties_client_id = z.string();

export type get_Get_client_properties_response_properties_data_properties_client_type =
  typeof get_Get_client_properties_response_properties_data_properties_client_type;
export const get_Get_client_properties_response_properties_data_properties_client_type = z.string();

export type get_Get_client_properties_response_properties_data_properties_created_at =
  typeof get_Get_client_properties_response_properties_data_properties_created_at;
export const get_Get_client_properties_response_properties_data_properties_created_at = z.string();

export type get_Get_client_properties_response_properties_data_properties_enabled =
  typeof get_Get_client_properties_response_properties_data_properties_enabled;
export const get_Get_client_properties_response_properties_data_properties_enabled = z.boolean();

export type get_Get_client_properties_response_properties_data_properties_id =
  typeof get_Get_client_properties_response_properties_data_properties_id;
export const get_Get_client_properties_response_properties_data_properties_id = z.string();

export type get_Get_client_properties_response_properties_data_properties_name =
  typeof get_Get_client_properties_response_properties_data_properties_name;
export const get_Get_client_properties_response_properties_data_properties_name = z.string();

export type get_Get_client_properties_response_properties_data_properties_protocol =
  typeof get_Get_client_properties_response_properties_data_properties_protocol;
export const get_Get_client_properties_response_properties_data_properties_protocol = z.string();

export type get_Get_client_properties_response_properties_data_properties_public_client =
  typeof get_Get_client_properties_response_properties_data_properties_public_client;
export const get_Get_client_properties_response_properties_data_properties_public_client = z.boolean();

export type get_Get_client_properties_response_properties_data_properties_realm_id =
  typeof get_Get_client_properties_response_properties_data_properties_realm_id;
export const get_Get_client_properties_response_properties_data_properties_realm_id = z.string();

export type get_Get_client_properties_response_properties_data_properties_redirect_uris_anyOf_0_anyOf_1 =
  typeof get_Get_client_properties_response_properties_data_properties_redirect_uris_anyOf_0_anyOf_1;
export const get_Get_client_properties_response_properties_data_properties_redirect_uris_anyOf_0_anyOf_1 = z.null();

export type get_Get_client_properties_response_properties_data_properties_redirect_uris_anyOf_1 =
  typeof get_Get_client_properties_response_properties_data_properties_redirect_uris_anyOf_1;
export const get_Get_client_properties_response_properties_data_properties_redirect_uris_anyOf_1 = z.undefined();

export type get_Get_client_properties_response_properties_data_properties_secret =
  typeof get_Get_client_properties_response_properties_data_properties_secret;
export const get_Get_client_properties_response_properties_data_properties_secret = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type get_Get_client_properties_response_properties_data_properties_service_account_enabled =
  typeof get_Get_client_properties_response_properties_data_properties_service_account_enabled;
export const get_Get_client_properties_response_properties_data_properties_service_account_enabled = z.boolean();

export type get_Get_client_properties_response_properties_data_properties_updated_at =
  typeof get_Get_client_properties_response_properties_data_properties_updated_at;
export const get_Get_client_properties_response_properties_data_properties_updated_at = z.string();

export type delete_Delete_client = typeof delete_Delete_client;
export const delete_Delete_client = {
  method: z.literal("DELETE"),
  path: z.literal("/realms/{realm_name}/clients/{client_id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
      client_id: z.string(),
    }),
  }),
  response: DeleteClientResponse,
};

export type patch_Update_client_properties_method = typeof patch_Update_client_properties_method;
export const patch_Update_client_properties_method = z.literal("PATCH");

export type patch_Update_client_properties_path = typeof patch_Update_client_properties_path;
export const patch_Update_client_properties_path = z.literal("/realms/{realm_name}/clients/{client_id}");

export type patch_Update_client_properties_requestFormat = typeof patch_Update_client_properties_requestFormat;
export const patch_Update_client_properties_requestFormat = z.literal("json");

export type patch_Update_client_properties_parameters = typeof patch_Update_client_properties_parameters;
export const patch_Update_client_properties_parameters = {
  path: z.object({
    realm_name: z.string(),
    client_id: z.string(),
  }),
  body: UpdateClientValidator,
};

export type patch_Update_client_properties_response_properties_client_id =
  typeof patch_Update_client_properties_response_properties_client_id;
export const patch_Update_client_properties_response_properties_client_id = z.string();

export type patch_Update_client_properties_response_properties_client_type =
  typeof patch_Update_client_properties_response_properties_client_type;
export const patch_Update_client_properties_response_properties_client_type = z.string();

export type patch_Update_client_properties_response_properties_created_at =
  typeof patch_Update_client_properties_response_properties_created_at;
export const patch_Update_client_properties_response_properties_created_at = z.string();

export type patch_Update_client_properties_response_properties_enabled =
  typeof patch_Update_client_properties_response_properties_enabled;
export const patch_Update_client_properties_response_properties_enabled = z.boolean();

export type patch_Update_client_properties_response_properties_id =
  typeof patch_Update_client_properties_response_properties_id;
export const patch_Update_client_properties_response_properties_id = z.string();

export type patch_Update_client_properties_response_properties_name =
  typeof patch_Update_client_properties_response_properties_name;
export const patch_Update_client_properties_response_properties_name = z.string();

export type patch_Update_client_properties_response_properties_protocol =
  typeof patch_Update_client_properties_response_properties_protocol;
export const patch_Update_client_properties_response_properties_protocol = z.string();

export type patch_Update_client_properties_response_properties_public_client =
  typeof patch_Update_client_properties_response_properties_public_client;
export const patch_Update_client_properties_response_properties_public_client = z.boolean();

export type patch_Update_client_properties_response_properties_realm_id =
  typeof patch_Update_client_properties_response_properties_realm_id;
export const patch_Update_client_properties_response_properties_realm_id = z.string();

export type patch_Update_client_properties_response_properties_redirect_uris_anyOf_0_anyOf_1 =
  typeof patch_Update_client_properties_response_properties_redirect_uris_anyOf_0_anyOf_1;
export const patch_Update_client_properties_response_properties_redirect_uris_anyOf_0_anyOf_1 = z.null();

export type patch_Update_client_properties_response_properties_redirect_uris_anyOf_1 =
  typeof patch_Update_client_properties_response_properties_redirect_uris_anyOf_1;
export const patch_Update_client_properties_response_properties_redirect_uris_anyOf_1 = z.undefined();

export type patch_Update_client_properties_response_properties_secret =
  typeof patch_Update_client_properties_response_properties_secret;
export const patch_Update_client_properties_response_properties_secret = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type patch_Update_client_properties_response_properties_service_account_enabled =
  typeof patch_Update_client_properties_response_properties_service_account_enabled;
export const patch_Update_client_properties_response_properties_service_account_enabled = z.boolean();

export type patch_Update_client_properties_response_properties_updated_at =
  typeof patch_Update_client_properties_response_properties_updated_at;
export const patch_Update_client_properties_response_properties_updated_at = z.string();

export type get_Get_redirect_uris = typeof get_Get_redirect_uris;
export const get_Get_redirect_uris = {
  method: z.literal("GET"),
  path: z.literal("/realms/{realm_name}/clients/{client_id}/redirects"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
      client_id: z.string(),
    }),
  }),
  response: z.array(RedirectUri),
};

export type post_Create_redirect_uri = typeof post_Create_redirect_uri;
export const post_Create_redirect_uri = {
  method: z.literal("POST"),
  path: z.literal("/realms/{realm_name}/clients/{client_id}/redirects"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
      client_id: z.string(),
    }),
    body: CreateRedirectUriValidator,
  }),
  response: RedirectUri,
};

export type put_Update_redirect_uri = typeof put_Update_redirect_uri;
export const put_Update_redirect_uri = {
  method: z.literal("PUT"),
  path: z.literal("/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
      client_id: z.string(),
      uri_id: z.string(),
    }),
    body: UpdateRedirectUriValidator,
  }),
  response: RedirectUri,
};

export type delete_Delete_redirect_uri = typeof delete_Delete_redirect_uri;
export const delete_Delete_redirect_uri = {
  method: z.literal("DELETE"),
  path: z.literal("/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
      client_id: z.string(),
      uri_id: z.string(),
    }),
  }),
  response: z.unknown(),
};

export type get_Get_client_roles_properties_method = typeof get_Get_client_roles_properties_method;
export const get_Get_client_roles_properties_method = z.literal("GET");

export type get_Get_client_roles_properties_path = typeof get_Get_client_roles_properties_path;
export const get_Get_client_roles_properties_path = z.literal("/realms/{realm_name}/clients/{client_id}/roles");

export type get_Get_client_roles_properties_requestFormat = typeof get_Get_client_roles_properties_requestFormat;
export const get_Get_client_roles_properties_requestFormat = z.literal("json");

export type get_Get_client_roles_properties_parameters = typeof get_Get_client_roles_properties_parameters;
export const get_Get_client_roles_properties_parameters = {
  path: z.object({
    realm_name: z.string(),
    client_id: z.string(),
  }),
};

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_0 =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_0;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_0 =
  z.null();

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id =
  z.string();

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name =
  z.string();

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_1 =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_1;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_anyOf_1 = z.undefined();

export type get_Get_client_roles_properties_response_properties_data_items_properties_client_id =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_client_id;
export const get_Get_client_roles_properties_response_properties_data_items_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type get_Get_client_roles_properties_response_properties_data_items_properties_created_at =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_created_at;
export const get_Get_client_roles_properties_response_properties_data_items_properties_created_at = z.string();

export type get_Get_client_roles_properties_response_properties_data_items_properties_description =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_description;
export const get_Get_client_roles_properties_response_properties_data_items_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type get_Get_client_roles_properties_response_properties_data_items_properties_id =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_id;
export const get_Get_client_roles_properties_response_properties_data_items_properties_id = z.string();

export type get_Get_client_roles_properties_response_properties_data_items_properties_name =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_name;
export const get_Get_client_roles_properties_response_properties_data_items_properties_name = z.string();

export type get_Get_client_roles_properties_response_properties_data_items_properties_permissions =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_permissions;
export const get_Get_client_roles_properties_response_properties_data_items_properties_permissions = z.array(
  z.string(),
);

export type get_Get_client_roles_properties_response_properties_data_items_properties_realm_id =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_realm_id;
export const get_Get_client_roles_properties_response_properties_data_items_properties_realm_id = z.string();

export type get_Get_client_roles_properties_response_properties_data_items_properties_updated_at =
  typeof get_Get_client_roles_properties_response_properties_data_items_properties_updated_at;
export const get_Get_client_roles_properties_response_properties_data_items_properties_updated_at = z.string();

export type post_Create_role_properties_method = typeof post_Create_role_properties_method;
export const post_Create_role_properties_method = z.literal("POST");

export type post_Create_role_properties_path = typeof post_Create_role_properties_path;
export const post_Create_role_properties_path = z.literal("/realms/{realm_name}/clients/{client_id}/roles");

export type post_Create_role_properties_requestFormat = typeof post_Create_role_properties_requestFormat;
export const post_Create_role_properties_requestFormat = z.literal("json");

export type post_Create_role_properties_parameters = typeof post_Create_role_properties_parameters;
export const post_Create_role_properties_parameters = {
  path: z.object({
    realm_name: z.string(),
    client_id: z.string(),
  }),
  body: CreateRoleValidator,
};

export type post_Create_role_properties_response_properties_client_anyOf_0_anyOf_0 =
  typeof post_Create_role_properties_response_properties_client_anyOf_0_anyOf_0;
export const post_Create_role_properties_response_properties_client_anyOf_0_anyOf_0 = z.null();

export type post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_client_id =
  typeof post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_client_id;
export const post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_client_id = z.string();

export type post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_client_type =
  typeof post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_client_type;
export const post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_client_type = z.string();

export type post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_created_at =
  typeof post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_created_at;
export const post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_created_at = z.string();

export type post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_enabled =
  typeof post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_enabled;
export const post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_enabled = z.boolean();

export type post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_id =
  typeof post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_id;
export const post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_id = z.string();

export type post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_name =
  typeof post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_name;
export const post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_name = z.string();

export type post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_protocol =
  typeof post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_protocol;
export const post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_protocol = z.string();

export type post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_public_client =
  typeof post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_public_client;
export const post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  typeof post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_realm_id;
export const post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_realm_id = z.string();

export type post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  typeof post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1;
export const post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  typeof post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1;
export const post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_secret =
  typeof post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_secret;
export const post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_secret = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  typeof post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled;
export const post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  typeof post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_updated_at;
export const post_Create_role_properties_response_properties_client_anyOf_0_anyOf_1_properties_updated_at = z.string();

export type post_Create_role_properties_response_properties_client_anyOf_1 =
  typeof post_Create_role_properties_response_properties_client_anyOf_1;
export const post_Create_role_properties_response_properties_client_anyOf_1 = z.undefined();

export type post_Create_role_properties_response_properties_client_id =
  typeof post_Create_role_properties_response_properties_client_id;
export const post_Create_role_properties_response_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type post_Create_role_properties_response_properties_created_at =
  typeof post_Create_role_properties_response_properties_created_at;
export const post_Create_role_properties_response_properties_created_at = z.string();

export type post_Create_role_properties_response_properties_description =
  typeof post_Create_role_properties_response_properties_description;
export const post_Create_role_properties_response_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type post_Create_role_properties_response_properties_id =
  typeof post_Create_role_properties_response_properties_id;
export const post_Create_role_properties_response_properties_id = z.string();

export type post_Create_role_properties_response_properties_name =
  typeof post_Create_role_properties_response_properties_name;
export const post_Create_role_properties_response_properties_name = z.string();

export type post_Create_role_properties_response_properties_permissions =
  typeof post_Create_role_properties_response_properties_permissions;
export const post_Create_role_properties_response_properties_permissions = z.array(z.string());

export type post_Create_role_properties_response_properties_realm_id =
  typeof post_Create_role_properties_response_properties_realm_id;
export const post_Create_role_properties_response_properties_realm_id = z.string();

export type post_Create_role_properties_response_properties_updated_at =
  typeof post_Create_role_properties_response_properties_updated_at;
export const post_Create_role_properties_response_properties_updated_at = z.string();

export type post_Authenticate_properties_method = typeof post_Authenticate_properties_method;
export const post_Authenticate_properties_method = z.literal("POST");

export type post_Authenticate_properties_path = typeof post_Authenticate_properties_path;
export const post_Authenticate_properties_path = z.literal("/realms/{realm_name}/login-actions/authenticate");

export type post_Authenticate_properties_requestFormat = typeof post_Authenticate_properties_requestFormat;
export const post_Authenticate_properties_requestFormat = z.literal("json");

export type post_Authenticate_properties_parameters = typeof post_Authenticate_properties_parameters;
export const post_Authenticate_properties_parameters = {
  path: z.object({
    realm_name: z.string(),
  }),
  body: AuthenticateRequest,
};

export type post_Authenticate_properties_response_properties_message =
  typeof post_Authenticate_properties_response_properties_message;
export const post_Authenticate_properties_response_properties_message = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type post_Authenticate_properties_response_properties_required_actions_anyOf_0_anyOf_1 =
  typeof post_Authenticate_properties_response_properties_required_actions_anyOf_0_anyOf_1;
export const post_Authenticate_properties_response_properties_required_actions_anyOf_0_anyOf_1 = z.null();

export type post_Authenticate_properties_response_properties_required_actions_anyOf_1 =
  typeof post_Authenticate_properties_response_properties_required_actions_anyOf_1;
export const post_Authenticate_properties_response_properties_required_actions_anyOf_1 = z.undefined();

export type post_Authenticate_properties_response_properties_status =
  typeof post_Authenticate_properties_response_properties_status;
export const post_Authenticate_properties_response_properties_status = z.union([
  z.literal("Success"),
  z.literal("RequiresActions"),
  z.literal("RequiresOtpChallenge"),
  z.literal("Failed"),
]);

export type post_Authenticate_properties_response_properties_token =
  typeof post_Authenticate_properties_response_properties_token;
export const post_Authenticate_properties_response_properties_token = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type post_Authenticate_properties_response_properties_url =
  typeof post_Authenticate_properties_response_properties_url;
export const post_Authenticate_properties_response_properties_url = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type post_Challenge_otp = typeof post_Challenge_otp;
export const post_Challenge_otp = {
  method: z.literal("POST"),
  path: z.literal("/realms/{realm_name}/login-actions/challenge-otp"),
  requestFormat: z.literal("json"),
  parameters: z.never(),
  response: ChallengeOtpResponse,
};

export type get_Setup_otp = typeof get_Setup_otp;
export const get_Setup_otp = {
  method: z.literal("GET"),
  path: z.literal("/realms/{realm_name}/login-actions/setup-otp"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
    }),
  }),
  response: SetupOtpResponse,
};

export type post_Verify_otp = typeof post_Verify_otp;
export const post_Verify_otp = {
  method: z.literal("POST"),
  path: z.literal("/realms/{realm_name}/login-actions/verify-otp"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
    }),
    body: OtpVerifyRequest,
  }),
  response: VerifyOtpResponse,
};

export type get_Auth = typeof get_Auth;
export const get_Auth = {
  method: z.literal("GET"),
  path: z.literal("/realms/{realm_name}/protocol/openid-connect/auth"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
      response_type: z.string(),
      client_id: z.string(),
      redirect_uri: z.string(),
      scope: z.union([z.string(), z.null()]),
      state: z.union([z.string(), z.null()]),
    }),
  }),
  response: z.unknown(),
};

export type get_Get_certs = typeof get_Get_certs;
export const get_Get_certs = {
  method: z.literal("GET"),
  path: z.literal("/realms/{realm_name}/protocol/openid-connect/certs"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
    }),
  }),
  response: GetCertsResponse,
};

export type post_Exchange_token = typeof post_Exchange_token;
export const post_Exchange_token = {
  method: z.literal("POST"),
  path: z.literal("/realms/{realm_name}/protocol/openid-connect/token"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    body: TokenRequestValidator,
  }),
  response: JwtToken,
};

export type get_Get_roles_properties_method = typeof get_Get_roles_properties_method;
export const get_Get_roles_properties_method = z.literal("GET");

export type get_Get_roles_properties_path = typeof get_Get_roles_properties_path;
export const get_Get_roles_properties_path = z.literal("/realms/{realm_name}/roles");

export type get_Get_roles_properties_requestFormat = typeof get_Get_roles_properties_requestFormat;
export const get_Get_roles_properties_requestFormat = z.literal("json");

export type get_Get_roles_properties_parameters = typeof get_Get_roles_properties_parameters;
export const get_Get_roles_properties_parameters = {
  path: z.object({
    realm_name: z.string(),
  }),
};

export type get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_0 =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_0;
export const get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_0 = z.null();

export type get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id;
export const get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type;
export const get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at;
export const get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled;
export const get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id;
export const get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id =
  z.string();

export type get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name;
export const get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name =
  z.string();

export type get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol;
export const get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client;
export const get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id;
export const get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1;
export const get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1;
export const get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret;
export const get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled;
export const get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at;
export const get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_1 =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_1;
export const get_Get_roles_properties_response_properties_data_items_properties_client_anyOf_1 = z.undefined();

export type get_Get_roles_properties_response_properties_data_items_properties_client_id =
  typeof get_Get_roles_properties_response_properties_data_items_properties_client_id;
export const get_Get_roles_properties_response_properties_data_items_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type get_Get_roles_properties_response_properties_data_items_properties_created_at =
  typeof get_Get_roles_properties_response_properties_data_items_properties_created_at;
export const get_Get_roles_properties_response_properties_data_items_properties_created_at = z.string();

export type get_Get_roles_properties_response_properties_data_items_properties_description =
  typeof get_Get_roles_properties_response_properties_data_items_properties_description;
export const get_Get_roles_properties_response_properties_data_items_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type get_Get_roles_properties_response_properties_data_items_properties_id =
  typeof get_Get_roles_properties_response_properties_data_items_properties_id;
export const get_Get_roles_properties_response_properties_data_items_properties_id = z.string();

export type get_Get_roles_properties_response_properties_data_items_properties_name =
  typeof get_Get_roles_properties_response_properties_data_items_properties_name;
export const get_Get_roles_properties_response_properties_data_items_properties_name = z.string();

export type get_Get_roles_properties_response_properties_data_items_properties_permissions =
  typeof get_Get_roles_properties_response_properties_data_items_properties_permissions;
export const get_Get_roles_properties_response_properties_data_items_properties_permissions = z.array(z.string());

export type get_Get_roles_properties_response_properties_data_items_properties_realm_id =
  typeof get_Get_roles_properties_response_properties_data_items_properties_realm_id;
export const get_Get_roles_properties_response_properties_data_items_properties_realm_id = z.string();

export type get_Get_roles_properties_response_properties_data_items_properties_updated_at =
  typeof get_Get_roles_properties_response_properties_data_items_properties_updated_at;
export const get_Get_roles_properties_response_properties_data_items_properties_updated_at = z.string();

export type get_Get_role_properties_method = typeof get_Get_role_properties_method;
export const get_Get_role_properties_method = z.literal("GET");

export type get_Get_role_properties_path = typeof get_Get_role_properties_path;
export const get_Get_role_properties_path = z.literal("/realms/{realm_name}/roles/{role_id}");

export type get_Get_role_properties_requestFormat = typeof get_Get_role_properties_requestFormat;
export const get_Get_role_properties_requestFormat = z.literal("json");

export type get_Get_role_properties_parameters = typeof get_Get_role_properties_parameters;
export const get_Get_role_properties_parameters = {
  path: z.object({
    realm_name: z.string(),
    role_id: z.string(),
  }),
};

export type get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_0 =
  typeof get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_0;
export const get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_0 = z.null();

export type get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id =
  typeof get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id;
export const get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type =
  typeof get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type;
export const get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at =
  typeof get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at;
export const get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled =
  typeof get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled;
export const get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_id =
  typeof get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_id;
export const get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_id =
  z.string();

export type get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_name =
  typeof get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_name;
export const get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_name =
  z.string();

export type get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol =
  typeof get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol;
export const get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client =
  typeof get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client;
export const get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  typeof get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id;
export const get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  typeof get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1;
export const get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  typeof get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1;
export const get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret =
  typeof get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret;
export const get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  typeof get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled;
export const get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  typeof get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at;
export const get_Get_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type get_Get_role_properties_response_properties_data_properties_client_anyOf_1 =
  typeof get_Get_role_properties_response_properties_data_properties_client_anyOf_1;
export const get_Get_role_properties_response_properties_data_properties_client_anyOf_1 = z.undefined();

export type get_Get_role_properties_response_properties_data_properties_client_id =
  typeof get_Get_role_properties_response_properties_data_properties_client_id;
export const get_Get_role_properties_response_properties_data_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type get_Get_role_properties_response_properties_data_properties_created_at =
  typeof get_Get_role_properties_response_properties_data_properties_created_at;
export const get_Get_role_properties_response_properties_data_properties_created_at = z.string();

export type get_Get_role_properties_response_properties_data_properties_description =
  typeof get_Get_role_properties_response_properties_data_properties_description;
export const get_Get_role_properties_response_properties_data_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type get_Get_role_properties_response_properties_data_properties_id =
  typeof get_Get_role_properties_response_properties_data_properties_id;
export const get_Get_role_properties_response_properties_data_properties_id = z.string();

export type get_Get_role_properties_response_properties_data_properties_name =
  typeof get_Get_role_properties_response_properties_data_properties_name;
export const get_Get_role_properties_response_properties_data_properties_name = z.string();

export type get_Get_role_properties_response_properties_data_properties_permissions =
  typeof get_Get_role_properties_response_properties_data_properties_permissions;
export const get_Get_role_properties_response_properties_data_properties_permissions = z.array(z.string());

export type get_Get_role_properties_response_properties_data_properties_realm_id =
  typeof get_Get_role_properties_response_properties_data_properties_realm_id;
export const get_Get_role_properties_response_properties_data_properties_realm_id = z.string();

export type get_Get_role_properties_response_properties_data_properties_updated_at =
  typeof get_Get_role_properties_response_properties_data_properties_updated_at;
export const get_Get_role_properties_response_properties_data_properties_updated_at = z.string();

export type put_Update_role_properties_method = typeof put_Update_role_properties_method;
export const put_Update_role_properties_method = z.literal("PUT");

export type put_Update_role_properties_path = typeof put_Update_role_properties_path;
export const put_Update_role_properties_path = z.literal("/realms/{realm_name}/roles/{role_id}");

export type put_Update_role_properties_requestFormat = typeof put_Update_role_properties_requestFormat;
export const put_Update_role_properties_requestFormat = z.literal("json");

export type put_Update_role_properties_parameters = typeof put_Update_role_properties_parameters;
export const put_Update_role_properties_parameters = {
  path: z.object({
    realm_name: z.string(),
    role_id: z.string(),
  }),
  body: UpdateRoleValidator,
};

export type put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_0 =
  typeof put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_0;
export const put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_0 = z.null();

export type put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id =
  typeof put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id;
export const put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type =
  typeof put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type;
export const put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at =
  typeof put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at;
export const put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled =
  typeof put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled;
export const put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_id =
  typeof put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_id;
export const put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_id =
  z.string();

export type put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_name =
  typeof put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_name;
export const put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_name =
  z.string();

export type put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol =
  typeof put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol;
export const put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client =
  typeof put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client;
export const put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  typeof put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id;
export const put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  typeof put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1;
export const put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  typeof put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1;
export const put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret =
  typeof put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret;
export const put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  typeof put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled;
export const put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  typeof put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at;
export const put_Update_role_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type put_Update_role_properties_response_properties_data_properties_client_anyOf_1 =
  typeof put_Update_role_properties_response_properties_data_properties_client_anyOf_1;
export const put_Update_role_properties_response_properties_data_properties_client_anyOf_1 = z.undefined();

export type put_Update_role_properties_response_properties_data_properties_client_id =
  typeof put_Update_role_properties_response_properties_data_properties_client_id;
export const put_Update_role_properties_response_properties_data_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type put_Update_role_properties_response_properties_data_properties_created_at =
  typeof put_Update_role_properties_response_properties_data_properties_created_at;
export const put_Update_role_properties_response_properties_data_properties_created_at = z.string();

export type put_Update_role_properties_response_properties_data_properties_description =
  typeof put_Update_role_properties_response_properties_data_properties_description;
export const put_Update_role_properties_response_properties_data_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type put_Update_role_properties_response_properties_data_properties_id =
  typeof put_Update_role_properties_response_properties_data_properties_id;
export const put_Update_role_properties_response_properties_data_properties_id = z.string();

export type put_Update_role_properties_response_properties_data_properties_name =
  typeof put_Update_role_properties_response_properties_data_properties_name;
export const put_Update_role_properties_response_properties_data_properties_name = z.string();

export type put_Update_role_properties_response_properties_data_properties_permissions =
  typeof put_Update_role_properties_response_properties_data_properties_permissions;
export const put_Update_role_properties_response_properties_data_properties_permissions = z.array(z.string());

export type put_Update_role_properties_response_properties_data_properties_realm_id =
  typeof put_Update_role_properties_response_properties_data_properties_realm_id;
export const put_Update_role_properties_response_properties_data_properties_realm_id = z.string();

export type put_Update_role_properties_response_properties_data_properties_updated_at =
  typeof put_Update_role_properties_response_properties_data_properties_updated_at;
export const put_Update_role_properties_response_properties_data_properties_updated_at = z.string();

export type patch_Update_role_permissions_properties_method = typeof patch_Update_role_permissions_properties_method;
export const patch_Update_role_permissions_properties_method = z.literal("PATCH");

export type patch_Update_role_permissions_properties_path = typeof patch_Update_role_permissions_properties_path;
export const patch_Update_role_permissions_properties_path = z.literal(
  "/realms/{realm_name}/roles/{role_id}/permissions",
);

export type patch_Update_role_permissions_properties_requestFormat =
  typeof patch_Update_role_permissions_properties_requestFormat;
export const patch_Update_role_permissions_properties_requestFormat = z.literal("json");

export type patch_Update_role_permissions_properties_parameters =
  typeof patch_Update_role_permissions_properties_parameters;
export const patch_Update_role_permissions_properties_parameters = {
  path: z.object({
    realm_name: z.string(),
    role_id: z.string(),
  }),
  body: UpdateRolePermissionsValidator,
};

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_0 =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_0;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_0 =
  z.null();

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_id =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_id;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_id =
  z.string();

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_name =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_name;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_name =
  z.string();

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_1 =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_1;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_anyOf_1 =
  z.undefined();

export type patch_Update_role_permissions_properties_response_properties_data_properties_client_id =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_client_id;
export const patch_Update_role_permissions_properties_response_properties_data_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type patch_Update_role_permissions_properties_response_properties_data_properties_created_at =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_created_at;
export const patch_Update_role_permissions_properties_response_properties_data_properties_created_at = z.string();

export type patch_Update_role_permissions_properties_response_properties_data_properties_description =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_description;
export const patch_Update_role_permissions_properties_response_properties_data_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type patch_Update_role_permissions_properties_response_properties_data_properties_id =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_id;
export const patch_Update_role_permissions_properties_response_properties_data_properties_id = z.string();

export type patch_Update_role_permissions_properties_response_properties_data_properties_name =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_name;
export const patch_Update_role_permissions_properties_response_properties_data_properties_name = z.string();

export type patch_Update_role_permissions_properties_response_properties_data_properties_permissions =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_permissions;
export const patch_Update_role_permissions_properties_response_properties_data_properties_permissions = z.array(
  z.string(),
);

export type patch_Update_role_permissions_properties_response_properties_data_properties_realm_id =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_realm_id;
export const patch_Update_role_permissions_properties_response_properties_data_properties_realm_id = z.string();

export type patch_Update_role_permissions_properties_response_properties_data_properties_updated_at =
  typeof patch_Update_role_permissions_properties_response_properties_data_properties_updated_at;
export const patch_Update_role_permissions_properties_response_properties_data_properties_updated_at = z.string();

export type get_Get_users_properties_method = typeof get_Get_users_properties_method;
export const get_Get_users_properties_method = z.literal("GET");

export type get_Get_users_properties_path = typeof get_Get_users_properties_path;
export const get_Get_users_properties_path = z.literal("/realms/{realm_name}/users");

export type get_Get_users_properties_requestFormat = typeof get_Get_users_properties_requestFormat;
export const get_Get_users_properties_requestFormat = z.literal("json");

export type get_Get_users_properties_parameters = typeof get_Get_users_properties_parameters;
export const get_Get_users_properties_parameters = {
  path: z.object({
    realm_name: z.string(),
  }),
};

export type get_Get_users_properties_response_properties_data_items_properties_client_id =
  typeof get_Get_users_properties_response_properties_data_items_properties_client_id;
export const get_Get_users_properties_response_properties_data_items_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type get_Get_users_properties_response_properties_data_items_properties_created_at =
  typeof get_Get_users_properties_response_properties_data_items_properties_created_at;
export const get_Get_users_properties_response_properties_data_items_properties_created_at = z.string();

export type get_Get_users_properties_response_properties_data_items_properties_email =
  typeof get_Get_users_properties_response_properties_data_items_properties_email;
export const get_Get_users_properties_response_properties_data_items_properties_email = z.string();

export type get_Get_users_properties_response_properties_data_items_properties_email_verified =
  typeof get_Get_users_properties_response_properties_data_items_properties_email_verified;
export const get_Get_users_properties_response_properties_data_items_properties_email_verified = z.boolean();

export type get_Get_users_properties_response_properties_data_items_properties_enabled =
  typeof get_Get_users_properties_response_properties_data_items_properties_enabled;
export const get_Get_users_properties_response_properties_data_items_properties_enabled = z.boolean();

export type get_Get_users_properties_response_properties_data_items_properties_firstname =
  typeof get_Get_users_properties_response_properties_data_items_properties_firstname;
export const get_Get_users_properties_response_properties_data_items_properties_firstname = z.string();

export type get_Get_users_properties_response_properties_data_items_properties_id =
  typeof get_Get_users_properties_response_properties_data_items_properties_id;
export const get_Get_users_properties_response_properties_data_items_properties_id = z.string();

export type get_Get_users_properties_response_properties_data_items_properties_lastname =
  typeof get_Get_users_properties_response_properties_data_items_properties_lastname;
export const get_Get_users_properties_response_properties_data_items_properties_lastname = z.string();

export type get_Get_users_properties_response_properties_data_items_properties_realm =
  typeof get_Get_users_properties_response_properties_data_items_properties_realm;
export const get_Get_users_properties_response_properties_data_items_properties_realm = z.union([
  z.union([z.null(), Realm]),
  z.undefined(),
]);

export type get_Get_users_properties_response_properties_data_items_properties_realm_id =
  typeof get_Get_users_properties_response_properties_data_items_properties_realm_id;
export const get_Get_users_properties_response_properties_data_items_properties_realm_id = z.string();

export type get_Get_users_properties_response_properties_data_items_properties_required_actions =
  typeof get_Get_users_properties_response_properties_data_items_properties_required_actions;
export const get_Get_users_properties_response_properties_data_items_properties_required_actions =
  z.array(RequiredAction);

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_0 =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_0;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_0 =
  z.null();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id =
  z.string();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name =
  z.string();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_1 =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_1;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_anyOf_1 =
  z.undefined();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_id =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_id;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_client_id =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_created_at =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_created_at;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_created_at =
  z.string();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_description =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_description;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_description =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_id =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_id;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_id = z.string();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_name =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_name;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_name =
  z.string();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_permissions =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_permissions;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_permissions =
  z.array(z.string());

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_realm_id =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_realm_id;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_realm_id =
  z.string();

export type get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_updated_at =
  typeof get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_updated_at;
export const get_Get_users_properties_response_properties_data_items_properties_roles_items_properties_updated_at =
  z.string();

export type get_Get_users_properties_response_properties_data_items_properties_updated_at =
  typeof get_Get_users_properties_response_properties_data_items_properties_updated_at;
export const get_Get_users_properties_response_properties_data_items_properties_updated_at = z.string();

export type get_Get_users_properties_response_properties_data_items_properties_username =
  typeof get_Get_users_properties_response_properties_data_items_properties_username;
export const get_Get_users_properties_response_properties_data_items_properties_username = z.string();

export type post_Create_user_properties_method = typeof post_Create_user_properties_method;
export const post_Create_user_properties_method = z.literal("POST");

export type post_Create_user_properties_path = typeof post_Create_user_properties_path;
export const post_Create_user_properties_path = z.literal("/realms/{realm_name}/users");

export type post_Create_user_properties_requestFormat = typeof post_Create_user_properties_requestFormat;
export const post_Create_user_properties_requestFormat = z.literal("json");

export type post_Create_user_properties_parameters = typeof post_Create_user_properties_parameters;
export const post_Create_user_properties_parameters = {
  path: z.object({
    realm_name: z.string(),
  }),
  body: CreateUserValidator,
};

export type post_Create_user_properties_response_properties_data_properties_client_id =
  typeof post_Create_user_properties_response_properties_data_properties_client_id;
export const post_Create_user_properties_response_properties_data_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type post_Create_user_properties_response_properties_data_properties_created_at =
  typeof post_Create_user_properties_response_properties_data_properties_created_at;
export const post_Create_user_properties_response_properties_data_properties_created_at = z.string();

export type post_Create_user_properties_response_properties_data_properties_email =
  typeof post_Create_user_properties_response_properties_data_properties_email;
export const post_Create_user_properties_response_properties_data_properties_email = z.string();

export type post_Create_user_properties_response_properties_data_properties_email_verified =
  typeof post_Create_user_properties_response_properties_data_properties_email_verified;
export const post_Create_user_properties_response_properties_data_properties_email_verified = z.boolean();

export type post_Create_user_properties_response_properties_data_properties_enabled =
  typeof post_Create_user_properties_response_properties_data_properties_enabled;
export const post_Create_user_properties_response_properties_data_properties_enabled = z.boolean();

export type post_Create_user_properties_response_properties_data_properties_firstname =
  typeof post_Create_user_properties_response_properties_data_properties_firstname;
export const post_Create_user_properties_response_properties_data_properties_firstname = z.string();

export type post_Create_user_properties_response_properties_data_properties_id =
  typeof post_Create_user_properties_response_properties_data_properties_id;
export const post_Create_user_properties_response_properties_data_properties_id = z.string();

export type post_Create_user_properties_response_properties_data_properties_lastname =
  typeof post_Create_user_properties_response_properties_data_properties_lastname;
export const post_Create_user_properties_response_properties_data_properties_lastname = z.string();

export type post_Create_user_properties_response_properties_data_properties_realm =
  typeof post_Create_user_properties_response_properties_data_properties_realm;
export const post_Create_user_properties_response_properties_data_properties_realm = z.union([
  z.union([z.null(), Realm]),
  z.undefined(),
]);

export type post_Create_user_properties_response_properties_data_properties_realm_id =
  typeof post_Create_user_properties_response_properties_data_properties_realm_id;
export const post_Create_user_properties_response_properties_data_properties_realm_id = z.string();

export type post_Create_user_properties_response_properties_data_properties_required_actions =
  typeof post_Create_user_properties_response_properties_data_properties_required_actions;
export const post_Create_user_properties_response_properties_data_properties_required_actions = z.array(RequiredAction);

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0 =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0 =
  z.null();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id =
  z.string();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name =
  z.string();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_1 =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_1;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_1 =
  z.undefined();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_id =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_id;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_client_id = z.union(
  [z.union([z.string(), z.null()]), z.undefined()],
);

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_created_at =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_created_at;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_created_at =
  z.string();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_description =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_description;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_description =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_id =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_id;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_id = z.string();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_name =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_name;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_name = z.string();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_permissions =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_permissions;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_permissions =
  z.array(z.string());

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_realm_id =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_realm_id;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_realm_id =
  z.string();

export type post_Create_user_properties_response_properties_data_properties_roles_items_properties_updated_at =
  typeof post_Create_user_properties_response_properties_data_properties_roles_items_properties_updated_at;
export const post_Create_user_properties_response_properties_data_properties_roles_items_properties_updated_at =
  z.string();

export type post_Create_user_properties_response_properties_data_properties_updated_at =
  typeof post_Create_user_properties_response_properties_data_properties_updated_at;
export const post_Create_user_properties_response_properties_data_properties_updated_at = z.string();

export type post_Create_user_properties_response_properties_data_properties_username =
  typeof post_Create_user_properties_response_properties_data_properties_username;
export const post_Create_user_properties_response_properties_data_properties_username = z.string();

export type delete_Bulk_delete_user = typeof delete_Bulk_delete_user;
export const delete_Bulk_delete_user = {
  method: z.literal("DELETE"),
  path: z.literal("/realms/{realm_name}/users/bulk"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
      ids: z.array(z.string()),
    }),
  }),
  response: BulkDeleteUserResponse,
};

export type get_Get_user_properties_method = typeof get_Get_user_properties_method;
export const get_Get_user_properties_method = z.literal("GET");

export type get_Get_user_properties_path = typeof get_Get_user_properties_path;
export const get_Get_user_properties_path = z.literal("/realms/{realm_name}/users/{user_id}");

export type get_Get_user_properties_requestFormat = typeof get_Get_user_properties_requestFormat;
export const get_Get_user_properties_requestFormat = z.literal("json");

export type get_Get_user_properties_parameters = typeof get_Get_user_properties_parameters;
export const get_Get_user_properties_parameters = {
  path: z.object({
    realm_name: z.string(),
    user_id: z.string(),
  }),
};

export type get_Get_user_properties_response_properties_data_properties_client_id =
  typeof get_Get_user_properties_response_properties_data_properties_client_id;
export const get_Get_user_properties_response_properties_data_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type get_Get_user_properties_response_properties_data_properties_created_at =
  typeof get_Get_user_properties_response_properties_data_properties_created_at;
export const get_Get_user_properties_response_properties_data_properties_created_at = z.string();

export type get_Get_user_properties_response_properties_data_properties_email =
  typeof get_Get_user_properties_response_properties_data_properties_email;
export const get_Get_user_properties_response_properties_data_properties_email = z.string();

export type get_Get_user_properties_response_properties_data_properties_email_verified =
  typeof get_Get_user_properties_response_properties_data_properties_email_verified;
export const get_Get_user_properties_response_properties_data_properties_email_verified = z.boolean();

export type get_Get_user_properties_response_properties_data_properties_enabled =
  typeof get_Get_user_properties_response_properties_data_properties_enabled;
export const get_Get_user_properties_response_properties_data_properties_enabled = z.boolean();

export type get_Get_user_properties_response_properties_data_properties_firstname =
  typeof get_Get_user_properties_response_properties_data_properties_firstname;
export const get_Get_user_properties_response_properties_data_properties_firstname = z.string();

export type get_Get_user_properties_response_properties_data_properties_id =
  typeof get_Get_user_properties_response_properties_data_properties_id;
export const get_Get_user_properties_response_properties_data_properties_id = z.string();

export type get_Get_user_properties_response_properties_data_properties_lastname =
  typeof get_Get_user_properties_response_properties_data_properties_lastname;
export const get_Get_user_properties_response_properties_data_properties_lastname = z.string();

export type get_Get_user_properties_response_properties_data_properties_realm =
  typeof get_Get_user_properties_response_properties_data_properties_realm;
export const get_Get_user_properties_response_properties_data_properties_realm = z.union([
  z.union([z.null(), Realm]),
  z.undefined(),
]);

export type get_Get_user_properties_response_properties_data_properties_realm_id =
  typeof get_Get_user_properties_response_properties_data_properties_realm_id;
export const get_Get_user_properties_response_properties_data_properties_realm_id = z.string();

export type get_Get_user_properties_response_properties_data_properties_required_actions =
  typeof get_Get_user_properties_response_properties_data_properties_required_actions;
export const get_Get_user_properties_response_properties_data_properties_required_actions = z.array(RequiredAction);

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0 =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0 =
  z.null();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id =
  z.string();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name =
  z.string();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_1 =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_1;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_1 =
  z.undefined();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_id =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_id;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_created_at =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_created_at;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_created_at = z.string();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_description =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_description;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_id =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_id;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_id = z.string();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_name =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_name;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_name = z.string();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_permissions =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_permissions;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_permissions = z.array(
  z.string(),
);

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_realm_id =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_realm_id;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_realm_id = z.string();

export type get_Get_user_properties_response_properties_data_properties_roles_items_properties_updated_at =
  typeof get_Get_user_properties_response_properties_data_properties_roles_items_properties_updated_at;
export const get_Get_user_properties_response_properties_data_properties_roles_items_properties_updated_at = z.string();

export type get_Get_user_properties_response_properties_data_properties_updated_at =
  typeof get_Get_user_properties_response_properties_data_properties_updated_at;
export const get_Get_user_properties_response_properties_data_properties_updated_at = z.string();

export type get_Get_user_properties_response_properties_data_properties_username =
  typeof get_Get_user_properties_response_properties_data_properties_username;
export const get_Get_user_properties_response_properties_data_properties_username = z.string();

export type post_Update_user_properties_method = typeof post_Update_user_properties_method;
export const post_Update_user_properties_method = z.literal("POST");

export type post_Update_user_properties_path = typeof post_Update_user_properties_path;
export const post_Update_user_properties_path = z.literal("/realms/{realm_name}/users/{user_id}");

export type post_Update_user_properties_requestFormat = typeof post_Update_user_properties_requestFormat;
export const post_Update_user_properties_requestFormat = z.literal("json");

export type post_Update_user_properties_parameters = typeof post_Update_user_properties_parameters;
export const post_Update_user_properties_parameters = {
  path: z.object({
    realm_name: z.string(),
    user_id: z.string(),
  }),
  body: UpdateUserValidator,
};

export type post_Update_user_properties_response_properties_data_properties_client_id =
  typeof post_Update_user_properties_response_properties_data_properties_client_id;
export const post_Update_user_properties_response_properties_data_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type post_Update_user_properties_response_properties_data_properties_created_at =
  typeof post_Update_user_properties_response_properties_data_properties_created_at;
export const post_Update_user_properties_response_properties_data_properties_created_at = z.string();

export type post_Update_user_properties_response_properties_data_properties_email =
  typeof post_Update_user_properties_response_properties_data_properties_email;
export const post_Update_user_properties_response_properties_data_properties_email = z.string();

export type post_Update_user_properties_response_properties_data_properties_email_verified =
  typeof post_Update_user_properties_response_properties_data_properties_email_verified;
export const post_Update_user_properties_response_properties_data_properties_email_verified = z.boolean();

export type post_Update_user_properties_response_properties_data_properties_enabled =
  typeof post_Update_user_properties_response_properties_data_properties_enabled;
export const post_Update_user_properties_response_properties_data_properties_enabled = z.boolean();

export type post_Update_user_properties_response_properties_data_properties_firstname =
  typeof post_Update_user_properties_response_properties_data_properties_firstname;
export const post_Update_user_properties_response_properties_data_properties_firstname = z.string();

export type post_Update_user_properties_response_properties_data_properties_id =
  typeof post_Update_user_properties_response_properties_data_properties_id;
export const post_Update_user_properties_response_properties_data_properties_id = z.string();

export type post_Update_user_properties_response_properties_data_properties_lastname =
  typeof post_Update_user_properties_response_properties_data_properties_lastname;
export const post_Update_user_properties_response_properties_data_properties_lastname = z.string();

export type post_Update_user_properties_response_properties_data_properties_realm =
  typeof post_Update_user_properties_response_properties_data_properties_realm;
export const post_Update_user_properties_response_properties_data_properties_realm = z.union([
  z.union([z.null(), Realm]),
  z.undefined(),
]);

export type post_Update_user_properties_response_properties_data_properties_realm_id =
  typeof post_Update_user_properties_response_properties_data_properties_realm_id;
export const post_Update_user_properties_response_properties_data_properties_realm_id = z.string();

export type post_Update_user_properties_response_properties_data_properties_required_actions =
  typeof post_Update_user_properties_response_properties_data_properties_required_actions;
export const post_Update_user_properties_response_properties_data_properties_required_actions = z.array(RequiredAction);

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0 =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_0 =
  z.null();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_id =
  z.string();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_name =
  z.string();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_1 =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_1;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_anyOf_1 =
  z.undefined();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_id =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_id;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_client_id = z.union(
  [z.union([z.string(), z.null()]), z.undefined()],
);

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_created_at =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_created_at;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_created_at =
  z.string();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_description =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_description;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_description =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_id =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_id;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_id = z.string();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_name =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_name;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_name = z.string();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_permissions =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_permissions;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_permissions =
  z.array(z.string());

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_realm_id =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_realm_id;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_realm_id =
  z.string();

export type post_Update_user_properties_response_properties_data_properties_roles_items_properties_updated_at =
  typeof post_Update_user_properties_response_properties_data_properties_roles_items_properties_updated_at;
export const post_Update_user_properties_response_properties_data_properties_roles_items_properties_updated_at =
  z.string();

export type post_Update_user_properties_response_properties_data_properties_updated_at =
  typeof post_Update_user_properties_response_properties_data_properties_updated_at;
export const post_Update_user_properties_response_properties_data_properties_updated_at = z.string();

export type post_Update_user_properties_response_properties_data_properties_username =
  typeof post_Update_user_properties_response_properties_data_properties_username;
export const post_Update_user_properties_response_properties_data_properties_username = z.string();

export type delete_Delete_user = typeof delete_Delete_user;
export const delete_Delete_user = {
  method: z.literal("DELETE"),
  path: z.literal("/realms/{realm_name}/users/{user_id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
      user_id: z.string(),
    }),
  }),
  response: DeleteUserResponse,
};

export type get_Get_user_credentials = typeof get_Get_user_credentials;
export const get_Get_user_credentials = {
  method: z.literal("GET"),
  path: z.literal("/realms/{realm_name}/users/{user_id}/credentials"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
      user_id: z.string(),
    }),
  }),
  response: GetUserCredentialsResponse,
};

export type delete_Delete_user_credential = typeof delete_Delete_user_credential;
export const delete_Delete_user_credential = {
  method: z.literal("DELETE"),
  path: z.literal("/realms/{realm_name}/users/{user_id}/credentials/{credential_id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
      user_id: z.string(),
      credential_id: z.string(),
    }),
  }),
  response: DeleteUserCredentialResponse,
};

export type put_Reset_password = typeof put_Reset_password;
export const put_Reset_password = {
  method: z.literal("PUT"),
  path: z.literal("/realms/{realm_name}/users/{user_id}/reset-password"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
      user_id: z.string(),
    }),
    body: ResetPasswordValidator,
  }),
  response: z.unknown(),
};

export type get_Get_user_roles_properties_method = typeof get_Get_user_roles_properties_method;
export const get_Get_user_roles_properties_method = z.literal("GET");

export type get_Get_user_roles_properties_path = typeof get_Get_user_roles_properties_path;
export const get_Get_user_roles_properties_path = z.literal("/realms/{realm_name}/users/{user_id}/roles");

export type get_Get_user_roles_properties_requestFormat = typeof get_Get_user_roles_properties_requestFormat;
export const get_Get_user_roles_properties_requestFormat = z.literal("json");

export type get_Get_user_roles_properties_parameters = typeof get_Get_user_roles_properties_parameters;
export const get_Get_user_roles_properties_parameters = {
  path: z.object({
    realm_name: z.string(),
    user_id: z.string(),
  }),
};

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_0 =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_0;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_0 = z.null();

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_id =
  z.string();

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_client_type =
  z.string();

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_created_at =
  z.string();

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_enabled =
  z.boolean();

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_id =
  z.string();

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_name =
  z.string();

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_protocol =
  z.string();

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_public_client =
  z.boolean();

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_realm_id =
  z.string();

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_0_anyOf_1 =
  z.null();

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_redirect_uris_anyOf_1 =
  z.undefined();

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_secret =
  z.union([z.union([z.string(), z.null()]), z.undefined()]);

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_service_account_enabled =
  z.boolean();

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_0_anyOf_1_properties_updated_at =
  z.string();

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_1 =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_1;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_anyOf_1 = z.undefined();

export type get_Get_user_roles_properties_response_properties_data_items_properties_client_id =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_client_id;
export const get_Get_user_roles_properties_response_properties_data_items_properties_client_id = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type get_Get_user_roles_properties_response_properties_data_items_properties_created_at =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_created_at;
export const get_Get_user_roles_properties_response_properties_data_items_properties_created_at = z.string();

export type get_Get_user_roles_properties_response_properties_data_items_properties_description =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_description;
export const get_Get_user_roles_properties_response_properties_data_items_properties_description = z.union([
  z.union([z.string(), z.null()]),
  z.undefined(),
]);

export type get_Get_user_roles_properties_response_properties_data_items_properties_id =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_id;
export const get_Get_user_roles_properties_response_properties_data_items_properties_id = z.string();

export type get_Get_user_roles_properties_response_properties_data_items_properties_name =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_name;
export const get_Get_user_roles_properties_response_properties_data_items_properties_name = z.string();

export type get_Get_user_roles_properties_response_properties_data_items_properties_permissions =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_permissions;
export const get_Get_user_roles_properties_response_properties_data_items_properties_permissions = z.array(z.string());

export type get_Get_user_roles_properties_response_properties_data_items_properties_realm_id =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_realm_id;
export const get_Get_user_roles_properties_response_properties_data_items_properties_realm_id = z.string();

export type get_Get_user_roles_properties_response_properties_data_items_properties_updated_at =
  typeof get_Get_user_roles_properties_response_properties_data_items_properties_updated_at;
export const get_Get_user_roles_properties_response_properties_data_items_properties_updated_at = z.string();

export type post_Assign_role = typeof post_Assign_role;
export const post_Assign_role = {
  method: z.literal("POST"),
  path: z.literal("/realms/{realm_name}/users/{user_id}/roles/{role_id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
      user_id: z.string(),
      role_id: z.string(),
    }),
  }),
  response: AssignRoleResponse,
};

export type delete_Unassign_role = typeof delete_Unassign_role;
export const delete_Unassign_role = {
  method: z.literal("DELETE"),
  path: z.literal("/realms/{realm_name}/users/{user_id}/roles/{role_id}"),
  requestFormat: z.literal("json"),
  parameters: z.object({
    path: z.object({
      realm_name: z.string(),
      user_id: z.string(),
      role_id: z.string(),
    }),
  }),
  response: UnassignRoleResponse,
};

// <EndpointByMethod>
export const EndpointByMethod = {
  get: {
    "/realms": get_Fetch_realm,
    "/realms/users/@me/realms": get_Get_user_realms,
    "/realms/{name}": get_Get_realm,
    "/realms/{realm_name}/.well-known/openid-configuration": get_Get_openid_configuration,
    "/realms/{realm_name}/clients": get_Get_clients,
    "/realms/{realm_name}/clients/{client_id}": get_Get_client,
    "/realms/{realm_name}/clients/{client_id}/redirects": get_Get_redirect_uris,
    "/realms/{realm_name}/clients/{client_id}/roles": get_Get_client_roles,
    "/realms/{realm_name}/login-actions/setup-otp": get_Setup_otp,
    "/realms/{realm_name}/protocol/openid-connect/auth": get_Auth,
    "/realms/{realm_name}/protocol/openid-connect/certs": get_Get_certs,
    "/realms/{realm_name}/roles": get_Get_roles,
    "/realms/{realm_name}/roles/{role_id}": get_Get_role,
    "/realms/{realm_name}/users": get_Get_users,
    "/realms/{realm_name}/users/{user_id}": get_Get_user,
    "/realms/{realm_name}/users/{user_id}/credentials": get_Get_user_credentials,
    "/realms/{realm_name}/users/{user_id}/roles": get_Get_user_roles,
  },
  post: {
    "/realms": post_Create_realm,
    "/realms/{realm_name}/clients": post_Create_client,
    "/realms/{realm_name}/clients/{client_id}/redirects": post_Create_redirect_uri,
    "/realms/{realm_name}/clients/{client_id}/roles": post_Create_role,
    "/realms/{realm_name}/login-actions/authenticate": post_Authenticate,
    "/realms/{realm_name}/login-actions/challenge-otp": post_Challenge_otp,
    "/realms/{realm_name}/login-actions/verify-otp": post_Verify_otp,
    "/realms/{realm_name}/protocol/openid-connect/token": post_Exchange_token,
    "/realms/{realm_name}/users": post_Create_user,
    "/realms/{realm_name}/users/{user_id}": post_Update_user,
    "/realms/{realm_name}/users/{user_id}/roles/{role_id}": post_Assign_role,
  },
  put: {
    "/realms/{name}": put_Update_realm,
    "/realms/{name}/settings": put_Update_realm_setting,
    "/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}": put_Update_redirect_uri,
    "/realms/{realm_name}/roles/{role_id}": put_Update_role,
    "/realms/{realm_name}/users/{user_id}/reset-password": put_Reset_password,
  },
  delete: {
    "/realms/{name}": delete_Delete_realm,
    "/realms/{realm_name}/clients/{client_id}": delete_Delete_client,
    "/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}": delete_Delete_redirect_uri,
    "/realms/{realm_name}/users/bulk": delete_Bulk_delete_user,
    "/realms/{realm_name}/users/{user_id}": delete_Delete_user,
    "/realms/{realm_name}/users/{user_id}/credentials/{credential_id}": delete_Delete_user_credential,
    "/realms/{realm_name}/users/{user_id}/roles/{role_id}": delete_Unassign_role,
  },
  patch: {
    "/realms/{realm_name}/clients/{client_id}": patch_Update_client,
    "/realms/{realm_name}/roles/{role_id}/permissions": patch_Update_role_permissions,
  },
};
export type EndpointByMethod = typeof EndpointByMethod;
// </EndpointByMethod>

// <EndpointByMethod.Shorthands>
export type GetEndpoints = EndpointByMethod["get"];
export type PostEndpoints = EndpointByMethod["post"];
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
  response: unknown;
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
  response: TConfig["response"];
  responseHeaders?: TConfig["responseHeaders"];
};

export type Fetcher = (method: Method, url: string, parameters?: EndpointParameters | undefined) => Promise<Response>;

type RequiredKeys<T> = {
  [P in keyof T]-?: undefined extends T[P] ? never : P;
}[keyof T];

type MaybeOptionalArg<T> = RequiredKeys<T> extends never ? [config?: T] : [config: T];

// </ApiClientTypes>

// <ApiClient>
export class ApiClient {
  baseUrl: string = "";

  constructor(public fetcher: Fetcher) {}

  setBaseUrl(baseUrl: string) {
    this.baseUrl = baseUrl;
    return this;
  }

  parseResponse = async <T,>(response: Response): Promise<T> => {
    const contentType = response.headers.get("content-type");
    if (contentType?.includes("application/json")) {
      return response.json();
    }
    return response.text() as unknown as T;
  };

  // <ApiClient.get>
  get<Path extends keyof GetEndpoints, TEndpoint extends GetEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<z.infer<TEndpoint["parameters"]>>
  ): Promise<z.infer<TEndpoint["response"]>> {
    return this.fetcher("get", this.baseUrl + path, params[0]).then((response) =>
      this.parseResponse(response),
    ) as Promise<z.infer<TEndpoint["response"]>>;
  }
  // </ApiClient.get>

  // <ApiClient.post>
  post<Path extends keyof PostEndpoints, TEndpoint extends PostEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<z.infer<TEndpoint["parameters"]>>
  ): Promise<z.infer<TEndpoint["response"]>> {
    return this.fetcher("post", this.baseUrl + path, params[0]).then((response) =>
      this.parseResponse(response),
    ) as Promise<z.infer<TEndpoint["response"]>>;
  }
  // </ApiClient.post>

  // <ApiClient.put>
  put<Path extends keyof PutEndpoints, TEndpoint extends PutEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<z.infer<TEndpoint["parameters"]>>
  ): Promise<z.infer<TEndpoint["response"]>> {
    return this.fetcher("put", this.baseUrl + path, params[0]).then((response) =>
      this.parseResponse(response),
    ) as Promise<z.infer<TEndpoint["response"]>>;
  }
  // </ApiClient.put>

  // <ApiClient.delete>
  delete<Path extends keyof DeleteEndpoints, TEndpoint extends DeleteEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<z.infer<TEndpoint["parameters"]>>
  ): Promise<z.infer<TEndpoint["response"]>> {
    return this.fetcher("delete", this.baseUrl + path, params[0]).then((response) =>
      this.parseResponse(response),
    ) as Promise<z.infer<TEndpoint["response"]>>;
  }
  // </ApiClient.delete>

  // <ApiClient.patch>
  patch<Path extends keyof PatchEndpoints, TEndpoint extends PatchEndpoints[Path]>(
    path: Path,
    ...params: MaybeOptionalArg<z.infer<TEndpoint["parameters"]>>
  ): Promise<z.infer<TEndpoint["response"]>> {
    return this.fetcher("patch", this.baseUrl + path, params[0]).then((response) =>
      this.parseResponse(response),
    ) as Promise<z.infer<TEndpoint["response"]>>;
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
    ...params: MaybeOptionalArg<z.infer<TEndpoint extends { parameters: infer Params } ? Params : never>>
  ): Promise<
    Omit<Response, "json"> & {
      /** [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/json) */
      json: () => Promise<TEndpoint extends { response: infer Res } ? Res : never>;
    }
  > {
    return this.fetcher(method, this.baseUrl + (path as string), params[0] as EndpointParameters);
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
*/

// </ApiClient
