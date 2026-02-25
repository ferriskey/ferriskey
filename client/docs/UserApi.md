# \UserApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**assign_role**](UserApi.md#assign_role) | **POST** /realms/{realm_name}/users/{user_id}/roles/{role_id} | Assign a role to a user in a realm
[**bulk_delete_user**](UserApi.md#bulk_delete_user) | **DELETE** /realms/{realm_name}/users/bulk | Bulk delete users in a realm
[**create_user**](UserApi.md#create_user) | **POST** /realms/{realm_name}/users | Create a new user in a realm
[**delete_user**](UserApi.md#delete_user) | **DELETE** /realms/{realm_name}/users/{user_id} | Delete a user in a realm
[**delete_user_credential**](UserApi.md#delete_user_credential) | **DELETE** /realms/{realm_name}/users/{user_id}/credentials/{credential_id} | Delete a user credential in a realm
[**get_user**](UserApi.md#get_user) | **GET** /realms/{realm_name}/users/{user_id} | Get a user by ID in a realm
[**get_user_credentials**](UserApi.md#get_user_credentials) | **GET** /realms/{realm_name}/users/{user_id}/credentials | Get user credentials in a realm
[**get_user_permissions**](UserApi.md#get_user_permissions) | **GET** /realms/{realm_name}/users/{user_id}/permissions | Get user permissions by ID in a realm
[**get_user_roles**](UserApi.md#get_user_roles) | **GET** /realms/{realm_name}/users/{user_id}/roles | Get all roles for a specific user
[**get_users**](UserApi.md#get_users) | **GET** /realms/{realm_name}/users | Get all users in a realm
[**reset_password**](UserApi.md#reset_password) | **PUT** /realms/{realm_name}/users/{user_id}/reset-password | Reset user password
[**unassign_role**](UserApi.md#unassign_role) | **DELETE** /realms/{realm_name}/users/{user_id}/roles/{role_id} | Unassign a role from a user in a realm
[**update_user**](UserApi.md#update_user) | **PUT** /realms/{realm_name}/users/{user_id} | Update a user in a realm



## assign_role

> models::AssignRoleResponse assign_role(realm_name, user_id, role_id)
Assign a role to a user in a realm

Assigns a specified role to a user within a given realm. This endpoint is used to manage user roles in the system.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**user_id** | **uuid::Uuid** | User ID | [required] |
**role_id** | **uuid::Uuid** | Role ID | [required] |

### Return type

[**models::AssignRoleResponse**](AssignRoleResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## bulk_delete_user

> models::BulkDeleteUserResponse bulk_delete_user(realm_name, bulk_delete_user_validator)
Bulk delete users in a realm

Deletes multiple users in a realm by their IDs. This action is irreversible and will remove all associated data.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**bulk_delete_user_validator** | [**BulkDeleteUserValidator**](BulkDeleteUserValidator.md) |  | [required] |

### Return type

[**models::BulkDeleteUserResponse**](BulkDeleteUserResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_user

> models::CreateUserResponse create_user(realm_name, create_user_validator)
Create a new user in a realm

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**create_user_validator** | [**CreateUserValidator**](CreateUserValidator.md) | User to create | [required] |

### Return type

[**models::CreateUserResponse**](CreateUserResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_user

> models::DeleteUserResponse delete_user(realm_name, user_id)
Delete a user in a realm

Deletes a user in a realm. This action is irreversible and will remove all associated data.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**user_id** | **String** | User ID | [required] |

### Return type

[**models::DeleteUserResponse**](DeleteUserResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_user_credential

> models::DeleteUserCredentialResponse delete_user_credential(realm_name, user_id, credential_id)
Delete a user credential in a realm

Deletes a specific credential for a user in a realm. This action is irreversible and will remove all associated data.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**user_id** | **uuid::Uuid** | User ID | [required] |
**credential_id** | **uuid::Uuid** | Credential ID | [required] |

### Return type

[**models::DeleteUserCredentialResponse**](DeleteUserCredentialResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user

> models::UserResponse get_user(realm_name, user_id)
Get a user by ID in a realm

Retrieves a user by their ID in a specific realm. This endpoint returns detailed information about the user.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**user_id** | **String** | User ID | [required] |

### Return type

[**models::UserResponse**](UserResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_credentials

> models::GetUserCredentialsResponse get_user_credentials(realm_name, user_id)
Get user credentials in a realm

Retrieves all credentials associated with a user in a specific realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**user_id** | **uuid::Uuid** | User ID | [required] |

### Return type

[**models::GetUserCredentialsResponse**](GetUserCredentialsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_permissions

> models::UserPermissionsResponse get_user_permissions(realm_name, user_id)
Get user permissions by ID in a realm

Retrieves the permissions assigned to a user by their ID in a specific realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**user_id** | **String** | User ID | [required] |

### Return type

[**models::UserPermissionsResponse**](UserPermissionsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_roles

> models::GetUserRolesResponse get_user_roles(realm_name, user_id)
Get all roles for a specific user

Retrieves all roles associated with a user in a specific realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**user_id** | **uuid::Uuid** | User ID | [required] |

### Return type

[**models::GetUserRolesResponse**](GetUserRolesResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_users

> models::UsersResponse get_users(realm_name)
Get all users in a realm

Retrieves all users associated with a specific realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |

### Return type

[**models::UsersResponse**](UsersResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## reset_password

> models::ResetPasswordResponse reset_password(realm_name, user_id, reset_password_validator)
Reset user password

Resets the password for a user in a specific realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**user_id** | **uuid::Uuid** | User ID | [required] |
**reset_password_validator** | [**ResetPasswordValidator**](ResetPasswordValidator.md) | New password for the user | [required] |

### Return type

[**models::ResetPasswordResponse**](ResetPasswordResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unassign_role

> models::UnassignRoleResponse unassign_role(realm_name, user_id, role_id)
Unassign a role from a user in a realm

Unassigns a specific role from a user in a realm. This action is irreversible and will remove the user's access to the role's permissions.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**user_id** | **uuid::Uuid** | User ID | [required] |
**role_id** | **uuid::Uuid** | Role ID | [required] |

### Return type

[**models::UnassignRoleResponse**](UnassignRoleResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_user

> models::UpdateUserResponse update_user(realm_name, user_id, update_user_validator)
Update a user in a realm

Updates an existing user in a specific realm. The user must exist and the request must include the necessary fields to update.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**user_id** | **String** | User ID | [required] |
**update_user_validator** | [**UpdateUserValidator**](UpdateUserValidator.md) | User to update | [required] |

### Return type

[**models::UpdateUserResponse**](UpdateUserResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

