# \IdentityProviderApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_identity_provider**](IdentityProviderApi.md#create_identity_provider) | **POST** /realms/{realm_name}/identity-providers | Create a new identity provider in a realm
[**delete_identity_provider**](IdentityProviderApi.md#delete_identity_provider) | **DELETE** /realms/{realm_name}/identity-providers/{alias} | Delete an identity provider
[**get_identity_provider**](IdentityProviderApi.md#get_identity_provider) | **GET** /realms/{realm_name}/identity-providers/{alias} | Get an identity provider by alias
[**list_identity_providers**](IdentityProviderApi.md#list_identity_providers) | **GET** /realms/{realm_name}/identity-providers | List all identity providers in a realm
[**update_identity_provider**](IdentityProviderApi.md#update_identity_provider) | **PUT** /realms/{realm_name}/identity-providers/{alias} | Update an identity provider



## create_identity_provider

> models::IdentityProviderResponse create_identity_provider(realm_name, create_identity_provider_validator)
Create a new identity provider in a realm

Creates a new identity provider configuration for the specified realm. The identity provider will be used for social login (Google, GitHub, etc.) or OIDC/SAML federation.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | The name of the realm | [required] |
**create_identity_provider_validator** | [**CreateIdentityProviderValidator**](CreateIdentityProviderValidator.md) |  | [required] |

### Return type

[**models::IdentityProviderResponse**](IdentityProviderResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_identity_provider

> models::DeleteIdentityProviderResponse delete_identity_provider(realm_name, alias)
Delete an identity provider

Deletes an identity provider from the realm. This action is irreversible. Users who have linked accounts with this identity provider will no longer be able to use it for authentication.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | The name of the realm | [required] |
**alias** | **String** | The unique alias of the identity provider to delete | [required] |

### Return type

[**models::DeleteIdentityProviderResponse**](DeleteIdentityProviderResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_identity_provider

> models::IdentityProviderResponse get_identity_provider(realm_name, alias)
Get an identity provider by alias

Retrieves the details of a specific identity provider by its alias. Sensitive configuration values (like client secrets) are redacted in the response.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | The name of the realm | [required] |
**alias** | **String** | The unique alias of the identity provider | [required] |

### Return type

[**models::IdentityProviderResponse**](IdentityProviderResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_identity_providers

> models::IdentityProvidersResponse list_identity_providers(realm_name, brief_representation)
List all identity providers in a realm

Retrieves all identity providers configured for the specified realm. Optionally returns a brief representation with fewer fields.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | The name of the realm | [required] |
**brief_representation** | Option<**bool**> |  |  |

### Return type

[**models::IdentityProvidersResponse**](IdentityProvidersResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_identity_provider

> models::UpdateIdentityProviderResponse update_identity_provider(realm_name, alias, update_identity_provider_validator)
Update an identity provider

Updates an existing identity provider configuration. Only the fields provided in the request body will be updated. The alias cannot be changed after creation.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | The name of the realm | [required] |
**alias** | **String** | The unique alias of the identity provider to update | [required] |
**update_identity_provider_validator** | [**UpdateIdentityProviderValidator**](UpdateIdentityProviderValidator.md) |  | [required] |

### Return type

[**models::UpdateIdentityProviderResponse**](UpdateIdentityProviderResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
