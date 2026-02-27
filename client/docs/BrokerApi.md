# \BrokerApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**broker_callback**](BrokerApi.md#broker_callback) | **POST** /realms/{realm_name}/broker/{alias}/endpoint | Handle SSO callback from identity provider
[**broker_login**](BrokerApi.md#broker_login) | **GET** /realms/{realm_name}/broker/{alias}/login | Initiate SSO login via identity provider



## broker_callback

> broker_callback(realm_name, alias, state, code, error, error_description)
Handle SSO callback from identity provider

Processes the IdP callback, exchanges code for tokens, and redirects to client

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**alias** | **String** | Identity provider alias | [required] |
**state** | **String** | State parameter for CSRF validation (required) | [required] |
**code** | Option<**String**> | Authorization code from IdP |  |
**error** | Option<**String**> | Error code from IdP (if authentication failed) |  |
**error_description** | Option<**String**> | Error description from IdP |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## broker_login

> broker_login(realm_name, alias, client_id, redirect_uri, response_type, scope, state, nonce, session_id)
Initiate SSO login via identity provider

Redirects the user to the external identity provider for authentication

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**alias** | **String** | Identity provider alias | [required] |
**client_id** | Option<**String**> | Client ID initiating the login |  |
**redirect_uri** | Option<**String**> | Redirect URI to return to after authentication |  |
**response_type** | Option<**String**> | OAuth response type (defaults to \"code\") |  |
**scope** | Option<**String**> | Requested scopes |  |
**state** | Option<**String**> | Client's state parameter for CSRF protection |  |
**nonce** | Option<**String**> | OIDC nonce for replay protection |  |
**session_id** | Option<**uuid::Uuid**> | Existing auth session ID (if initiated from login page) |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
