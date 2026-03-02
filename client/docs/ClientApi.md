# \ClientApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_client**](ClientApi.md#create_client) | **POST** /realms/{realm_name}/clients | Create a new client in a realm
[**create_post_logout_redirect_uri**](ClientApi.md#create_post_logout_redirect_uri) | **POST** /realms/{realm_name}/clients/{client_id}/post-logout-redirects | Create a new post-logout redirect URI for a client
[**create_redirect_uri**](ClientApi.md#create_redirect_uri) | **POST** /realms/{realm_name}/clients/{client_id}/redirects | Create a new redirect URI for a client
[**create_role**](ClientApi.md#create_role) | **POST** /realms/{realm_name}/clients/{client_id}/roles | Create a new role
[**delete_client**](ClientApi.md#delete_client) | **DELETE** /realms/{realm_name}/clients/{client_id} | Delete a client
[**delete_post_logout_redirect_uri**](ClientApi.md#delete_post_logout_redirect_uri) | **DELETE** /realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id} | Delete a post-logout redirect URI for a client
[**delete_redirect_uri**](ClientApi.md#delete_redirect_uri) | **DELETE** /realms/{realm_name}/clients/{client_id}/redirects/{uri_id} | Delete a redirect URI for a client
[**get_client**](ClientApi.md#get_client) | **GET** /realms/{realm_name}/clients/{client_id} | Get a client
[**get_client_roles**](ClientApi.md#get_client_roles) | **GET** /realms/{realm_name}/clients/{client_id}/roles | Get client roles
[**get_clients**](ClientApi.md#get_clients) | **GET** /realms/{realm_name}/clients | Get clients in a realm
[**get_post_logout_redirect_uris**](ClientApi.md#get_post_logout_redirect_uris) | **GET** /realms/{realm_name}/clients/{client_id}/post-logout-redirects | Get post-logout redirect URIs for a client
[**get_redirect_uris**](ClientApi.md#get_redirect_uris) | **GET** /realms/{realm_name}/clients/{client_id}/redirects | Get redirect URIs for a client
[**update_client**](ClientApi.md#update_client) | **PATCH** /realms/{realm_name}/clients/{client_id} | Update a client
[**update_post_logout_redirect_uri**](ClientApi.md#update_post_logout_redirect_uri) | **PUT** /realms/{realm_name}/clients/{client_id}/post-logout-redirects/{uri_id} | Update a post-logout redirect URI for a client
[**update_redirect_uri**](ClientApi.md#update_redirect_uri) | **PUT** /realms/{realm_name}/clients/{client_id}/redirects/{uri_id} | Update a redirect URI for a client



## create_client

> models::Client create_client(realm_name, create_client_validator)
Create a new client in a realm

Creates a new client within the specified realm. This endpoint allows you to register a new client application that can interact with the realm's resources.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**create_client_validator** | [**CreateClientValidator**](CreateClientValidator.md) |  | [required] |

### Return type

[**models::Client**](Client.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_post_logout_redirect_uri

> models::RedirectUri create_post_logout_redirect_uri(realm_name, client_id, create_redirect_uri_validator)
Create a new post-logout redirect URI for a client

Creates a new post-logout redirect URI for the specified client.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |
**create_redirect_uri_validator** | [**CreateRedirectUriValidator**](CreateRedirectUriValidator.md) |  | [required] |

### Return type

[**models::RedirectUri**](RedirectUri.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_redirect_uri

> models::RedirectUri create_redirect_uri(realm_name, client_id, create_redirect_uri_validator)
Create a new redirect URI for a client

Creates a new redirect URI for the specified client. This endpoint allows you to add a redirect URI that the client can use for OAuth2 or OpenID Connect flows.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |
**create_redirect_uri_validator** | [**CreateRedirectUriValidator**](CreateRedirectUriValidator.md) |  | [required] |

### Return type

[**models::RedirectUri**](RedirectUri.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_role

> models::Role create_role(realm_name, client_id, create_role_validator)
Create a new role

Creates a new role for a specific client within a realm. This endpoint allows you to define roles that can be assigned to users or groups in the context of a client application.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |
**create_role_validator** | [**CreateRoleValidator**](CreateRoleValidator.md) |  | [required] |

### Return type

[**models::Role**](Role.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_client

> models::DeleteClientResponse delete_client(realm_name, client_id)
Delete a client

Deletes a client from the specified realm. This action is irreversible and will remove all associated data.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |

### Return type

[**models::DeleteClientResponse**](DeleteClientResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_post_logout_redirect_uri

> delete_post_logout_redirect_uri(realm_name, client_id, uri_id)
Delete a post-logout redirect URI for a client

Deletes a specific post-logout redirect URI for a client in a realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |
**uri_id** | **uuid::Uuid** | Post-logout redirect URI ID | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_redirect_uri

> delete_redirect_uri(realm_name, client_id, uri_id)
Delete a redirect URI for a client

Deletes a specific redirect URI for a client in a realm. This action is irreversible and will remove the redirect URI from the client's configuration.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |
**uri_id** | **uuid::Uuid** | Redirect URI ID | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_client

> models::GetClientResponse get_client(realm_name, client_id)
Get a client

Retrieves a client from the specified realm by its ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |

### Return type

[**models::GetClientResponse**](GetClientResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_client_roles

> models::GetClientRolesResponse get_client_roles(realm_name, client_id)
Get client roles

Retrieves all roles associated with a client in a specific realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |

### Return type

[**models::GetClientRolesResponse**](GetClientRolesResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_clients

> models::ClientsResponse get_clients(realm_name)
Get clients in a realm

Retrieves all clients associated with a specific realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |

### Return type

[**models::ClientsResponse**](ClientsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_post_logout_redirect_uris

> Vec<models::RedirectUri> get_post_logout_redirect_uris(realm_name, client_id)
Get post-logout redirect URIs for a client

Retrieves all post-logout redirect URIs associated with a client in a specific realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |

### Return type

[**Vec<models::RedirectUri>**](RedirectUri.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_redirect_uris

> Vec<models::RedirectUri> get_redirect_uris(realm_name, client_id)
Get redirect URIs for a client

Retrieves all redirect URIs associated with a client in a specific realm. This endpoint is useful for OAuth2 or OpenID Connect flows where clients need to know their registered redirect URIs.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |

### Return type

[**Vec<models::RedirectUri>**](RedirectUri.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_client

> models::UpdateClientResponse update_client(realm_name, client_id, update_client_validator)
Update a client

Updates an existing client in the specified realm. This endpoint allows you to modify client details such as name, client ID, and enabled status.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |
**update_client_validator** | [**UpdateClientValidator**](UpdateClientValidator.md) |  | [required] |

### Return type

[**models::UpdateClientResponse**](UpdateClientResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_post_logout_redirect_uri

> models::UpdatePostLogoutRedirectUriResponse update_post_logout_redirect_uri(realm_name, client_id, uri_id, update_redirect_uri_validator)
Update a post-logout redirect URI for a client

Updates an existing post-logout redirect URI for a client in a specific realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |
**uri_id** | **uuid::Uuid** | Post-logout redirect URI ID | [required] |
**update_redirect_uri_validator** | [**UpdateRedirectUriValidator**](UpdateRedirectUriValidator.md) |  | [required] |

### Return type

[**models::UpdatePostLogoutRedirectUriResponse**](UpdatePostLogoutRedirectUriResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_redirect_uri

> models::UpdateRedirectUriResponse update_redirect_uri(realm_name, client_id, uri_id, update_redirect_uri_validator)
Update a redirect URI for a client

Updates an existing redirect URI for a client in a specific realm. This endpoint allows you to modify the enabled status of a redirect URI.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**client_id** | **uuid::Uuid** | Client ID | [required] |
**uri_id** | **uuid::Uuid** | Redirect URI ID | [required] |
**update_redirect_uri_validator** | [**UpdateRedirectUriValidator**](UpdateRedirectUriValidator.md) |  | [required] |

### Return type

[**models::UpdateRedirectUriResponse**](UpdateRedirectUriResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
