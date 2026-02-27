# \ClientScopeApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**assign_default_scope**](ClientScopeApi.md#assign_default_scope) | **PUT** /realms/{realm_name}/client-scopes/clients/{client_id}/default-client-scopes/{scope_id} | Assign a default client scope to a client
[**assign_optional_scope**](ClientScopeApi.md#assign_optional_scope) | **PUT** /realms/{realm_name}/client-scopes/clients/{client_id}/optional-client-scopes/{scope_id} | Assign an optional client scope to a client
[**create_client_scope**](ClientScopeApi.md#create_client_scope) | **POST** /realms/{realm_name}/client-scopes | Create a new client scope
[**create_protocol_mapper**](ClientScopeApi.md#create_protocol_mapper) | **POST** /realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers | Create a protocol mapper
[**delete_client_scope**](ClientScopeApi.md#delete_client_scope) | **DELETE** /realms/{realm_name}/client-scopes/{scope_id} | Delete a client scope
[**delete_protocol_mapper**](ClientScopeApi.md#delete_protocol_mapper) | **DELETE** /realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id} | Delete a protocol mapper
[**get_client_client_scopes**](ClientScopeApi.md#get_client_client_scopes) | **GET** /realms/{realm_name}/client-scopes/clients/{client_id}/client-scopes | Get client scopes assigned to a client
[**get_client_scope**](ClientScopeApi.md#get_client_scope) | **GET** /realms/{realm_name}/client-scopes/{scope_id} | Get a client scope
[**get_client_scopes**](ClientScopeApi.md#get_client_scopes) | **GET** /realms/{realm_name}/client-scopes | Get client scopes in a realm
[**unassign_default_scope**](ClientScopeApi.md#unassign_default_scope) | **DELETE** /realms/{realm_name}/client-scopes/clients/{client_id}/default-client-scopes/{scope_id} | Remove a default client scope from a client
[**unassign_optional_scope**](ClientScopeApi.md#unassign_optional_scope) | **DELETE** /realms/{realm_name}/client-scopes/clients/{client_id}/optional-client-scopes/{scope_id} | Remove an optional client scope from a client
[**update_client_scope**](ClientScopeApi.md#update_client_scope) | **PATCH** /realms/{realm_name}/client-scopes/{scope_id} | Update a client scope
[**update_protocol_mapper**](ClientScopeApi.md#update_protocol_mapper) | **PATCH** /realms/{realm_name}/client-scopes/{scope_id}/protocol-mappers/{mapper_id} | Update a protocol mapper



## assign_default_scope

> models::ClientScopeMapping assign_default_scope(realm_name, client_id, scope_id)
Assign a default client scope to a client

Assigns a client scope as a default scope to the specified client. Default scopes are always included in tokens.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |
**scope_id** | **uuid::Uuid** | Client scope ID | [required] |

### Return type

[**models::ClientScopeMapping**](ClientScopeMapping.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## assign_optional_scope

> models::ClientScopeMapping assign_optional_scope(realm_name, client_id, scope_id)
Assign an optional client scope to a client

Assigns a client scope as an optional scope to the specified client. Optional scopes are included only when explicitly requested.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |
**scope_id** | **uuid::Uuid** | Client scope ID | [required] |

### Return type

[**models::ClientScopeMapping**](ClientScopeMapping.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_client_scope

> models::ClientScope create_client_scope(realm_name, create_client_scope_validator)
Create a new client scope

Creates a new client scope within the specified realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**create_client_scope_validator** | [**CreateClientScopeValidator**](CreateClientScopeValidator.md) |  | [required] |

### Return type

[**models::ClientScope**](ClientScope.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_protocol_mapper

> models::ProtocolMapper create_protocol_mapper(realm_name, scope_id, create_protocol_mapper_validator)
Create a protocol mapper

Creates a new protocol mapper for the specified client scope.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**scope_id** | **uuid::Uuid** | Client scope ID | [required] |
**create_protocol_mapper_validator** | [**CreateProtocolMapperValidator**](CreateProtocolMapperValidator.md) |  | [required] |

### Return type

[**models::ProtocolMapper**](ProtocolMapper.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_client_scope

> models::DeleteClientScopeResponse delete_client_scope(realm_name, scope_id)
Delete a client scope

Deletes a client scope from the specified realm. This action is irreversible.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**scope_id** | **uuid::Uuid** | Client scope ID | [required] |

### Return type

[**models::DeleteClientScopeResponse**](DeleteClientScopeResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_protocol_mapper

> models::DeleteProtocolMapperResponse delete_protocol_mapper(realm_name, scope_id, mapper_id)
Delete a protocol mapper

Deletes a protocol mapper from the specified client scope.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**scope_id** | **uuid::Uuid** | Client scope ID | [required] |
**mapper_id** | **uuid::Uuid** | Protocol mapper ID | [required] |

### Return type

[**models::DeleteProtocolMapperResponse**](DeleteProtocolMapperResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_client_client_scopes

> Vec<models::ClientScope> get_client_client_scopes(realm_name, client_id)
Get client scopes assigned to a client

Returns all client scopes (default and optional) assigned to the specified client.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |

### Return type

[**Vec<models::ClientScope>**](ClientScope.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_client_scope

> models::ClientScope get_client_scope(realm_name, scope_id)
Get a client scope

Retrieves a client scope from the specified realm by its ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**scope_id** | **uuid::Uuid** | Client scope ID | [required] |

### Return type

[**models::ClientScope**](ClientScope.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_client_scopes

> models::ClientScopesResponse get_client_scopes(realm_name)
Get client scopes in a realm

Retrieves all client scopes associated with a specific realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |

### Return type

[**models::ClientScopesResponse**](ClientScopesResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unassign_default_scope

> unassign_default_scope(realm_name, client_id, scope_id)
Remove a default client scope from a client

Removes a client scope from the default scopes of the specified client.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |
**scope_id** | **uuid::Uuid** | Client scope ID | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## unassign_optional_scope

> unassign_optional_scope(realm_name, client_id, scope_id)
Remove an optional client scope from a client

Removes a client scope from the optional scopes of the specified client.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |
**scope_id** | **uuid::Uuid** | Client scope ID | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_client_scope

> models::ClientScope update_client_scope(realm_name, scope_id, update_client_scope_validator)
Update a client scope

Updates an existing client scope in the specified realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**scope_id** | **uuid::Uuid** | Client scope ID | [required] |
**update_client_scope_validator** | [**UpdateClientScopeValidator**](UpdateClientScopeValidator.md) |  | [required] |

### Return type

[**models::ClientScope**](ClientScope.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_protocol_mapper

> models::ProtocolMapper update_protocol_mapper(realm_name, scope_id, mapper_id, update_protocol_mapper_validator)
Update a protocol mapper

Updates an existing protocol mapper for the specified client scope.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**scope_id** | **uuid::Uuid** | Client scope ID | [required] |
**mapper_id** | **uuid::Uuid** | Protocol mapper ID | [required] |
**update_protocol_mapper_validator** | [**UpdateProtocolMapperValidator**](UpdateProtocolMapperValidator.md) |  | [required] |

### Return type

[**models::ProtocolMapper**](ProtocolMapper.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
