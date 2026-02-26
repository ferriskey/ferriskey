# \RoleApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_role**](RoleApi.md#delete_role) | **DELETE** /realms/{realm_name}/roles/{role_id} | Delete a role in a realm
[**get_role**](RoleApi.md#get_role) | **GET** /realms/{realm_name}/roles/{role_id} | Get a role by ID in a realm
[**get_roles**](RoleApi.md#get_roles) | **GET** /realms/{realm_name}/roles | Get all roles for a realm
[**update_role**](RoleApi.md#update_role) | **PUT** /realms/{realm_name}/roles/{role_id} | Update a role in a realm
[**update_role_permissions**](RoleApi.md#update_role_permissions) | **PATCH** /realms/{realm_name}/roles/{role_id}/permissions | Update a role in a realm



## delete_role

> models::DeleteRoleResponse delete_role(realm_name, role_id)
Delete a role in a realm

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**role_id** | **uuid::Uuid** | Role ID | [required] |

### Return type

[**models::DeleteRoleResponse**](DeleteRoleResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_role

> models::GetRoleResponse get_role(realm_name, role_id)
Get a role by ID in a realm

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**role_id** | **uuid::Uuid** | Role ID | [required] |

### Return type

[**models::GetRoleResponse**](GetRoleResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_roles

> models::GetRolesResponse get_roles(realm_name)
Get all roles for a realm

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |

### Return type

[**models::GetRolesResponse**](GetRolesResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_role

> models::UpdateRoleResponse update_role(realm_name, role_id, update_role_validator)
Update a role in a realm

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**role_id** | **uuid::Uuid** | Role ID | [required] |
**update_role_validator** | [**UpdateRoleValidator**](UpdateRoleValidator.md) |  | [required] |

### Return type

[**models::UpdateRoleResponse**](UpdateRoleResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_role_permissions

> models::UpdateRolePermissionsResponse update_role_permissions(realm_name, role_id, update_role_permissions_validator)
Update a role in a realm

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**role_id** | **uuid::Uuid** | Role ID | [required] |
**update_role_permissions_validator** | [**UpdateRolePermissionsValidator**](UpdateRolePermissionsValidator.md) |  | [required] |

### Return type

[**models::UpdateRolePermissionsResponse**](UpdateRolePermissionsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
