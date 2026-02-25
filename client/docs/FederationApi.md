# \FederationApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_provider**](FederationApi.md#create_provider) | **POST** /realms/{realm_name}/federation/providers | Create a federation provider
[**delete_provider**](FederationApi.md#delete_provider) | **DELETE** /realms/{realm_name}/federation/providers/{id} | Delete a federation provider
[**get_provider**](FederationApi.md#get_provider) | **GET** /realms/{realm_name}/federation/providers/{id} | Get a federation provider details by ID
[**list_providers**](FederationApi.md#list_providers) | **GET** /realms/{realm_name}/federation/providers | List federation providers in a realm
[**sync_users**](FederationApi.md#sync_users) | **POST** /realms/{realm_name}/federation/providers/{id}/sync-users | Sync Users from Federation Provider
[**test_connection**](FederationApi.md#test_connection) | **POST** /realms/{realm_name}/federation/providers/{id}/test-connection | Test Federation Provider Connection
[**update_provider**](FederationApi.md#update_provider) | **PUT** /realms/{realm_name}/federation/providers/{id} | Update a federation provider



## create_provider

> models::ProviderResponse create_provider(realm_name, create_provider_request)
Create a federation provider

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**create_provider_request** | [**CreateProviderRequest**](CreateProviderRequest.md) |  | [required] |

### Return type

[**models::ProviderResponse**](ProviderResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_provider

> models::DeleteProviderResponse delete_provider(realm_name, id)
Delete a federation provider

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**id** | **String** | Provider ID | [required] |

### Return type

[**models::DeleteProviderResponse**](DeleteProviderResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_provider

> models::ProviderResponse get_provider(realm_name, id)
Get a federation provider details by ID

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**id** | **String** | Provider ID | [required] |

### Return type

[**models::ProviderResponse**](ProviderResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_providers

> models::ListProvidersResponse list_providers(realm_name)
List federation providers in a realm

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |

### Return type

[**models::ListProvidersResponse**](ListProvidersResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## sync_users

> models::SyncUsersResponse sync_users(realm_name, id)
Sync Users from Federation Provider

Triggers synchronization of users from the external federation provider (LDAP, Kerberos, etc.) to the local IAM database. This performs a comprehensive diff and reconciliation.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**id** | **String** | Provider ID | [required] |

### Return type

[**models::SyncUsersResponse**](SyncUsersResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## test_connection

> models::TestConnectionResponse test_connection(realm_name, id)
Test Federation Provider Connection

Tests the connection to the external federation provider (LDAP, Kerberos, etc.) to verify configuration and connectivity

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**id** | **String** | Provider ID | [required] |

### Return type

[**models::TestConnectionResponse**](TestConnectionResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_provider

> models::UpdateProviderResponse update_provider(realm_name, id, update_provider_request)
Update a federation provider

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**id** | **String** | Provider ID | [required] |
**update_provider_request** | [**UpdateProviderRequest**](UpdateProviderRequest.md) |  | [required] |

### Return type

[**models::UpdateProviderResponse**](UpdateProviderResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

