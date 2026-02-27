# \AuthApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**auth_handler**](AuthApi.md#auth_handler) | **GET** /realms/{realm_name}/protocol/openid-connect/auth | Authenticate a user
[**authenticate**](AuthApi.md#authenticate) | **POST** /realms/{realm_name}/login-actions/authenticate | Authenticate a user in a realm
[**burn_recovery_code**](AuthApi.md#burn_recovery_code) | **POST** /realms/{realm_name}/login-actions/burn-recovery-code | Burn a recovery code to authenticate
[**challenge_otp**](AuthApi.md#challenge_otp) | **POST** /realms/{realm_name}/login-actions/challenge-otp | Challenge OTP for user authentication
[**exchange_token**](AuthApi.md#exchange_token) | **POST** /realms/{realm_name}/protocol/openid-connect/token | Exchange token
[**generate_recovery_codes**](AuthApi.md#generate_recovery_codes) | **POST** /realms/{realm_name}/login-actions/generate-recovery-codes | Generate recovery codes
[**get_certs**](AuthApi.md#get_certs) | **GET** /realms/{realm_name}/protocol/openid-connect/certs | Get JWK keys for a realm
[**get_jwks_json**](AuthApi.md#get_jwks_json) | **GET** /realms/{realm_name}/protocol/openid-connect/jwks.json | Get JWKS for a realm
[**get_openid_configuration**](AuthApi.md#get_openid_configuration) | **GET** /realms/{realm_name}/.well-known/openid-configuration | Get OpenID Connect configuration
[**get_userinfo**](AuthApi.md#get_userinfo) | **GET** /realms/{realm_name}/protocol/openid-connect/userinfo | Get user info
[**introspect_token**](AuthApi.md#introspect_token) | **POST** /realms/{realm_name}/protocol/openid-connect/token/introspect | Token introspection
[**logout_get**](AuthApi.md#logout_get) | **GET** /realms/{realm_name}/protocol/openid-connect/logout | OIDC RP-Initiated Logout
[**logout_post**](AuthApi.md#logout_post) | **POST** /realms/{realm_name}/protocol/openid-connect/logout | OIDC RP-Initiated Logout
[**registration_handler**](AuthApi.md#registration_handler) | **POST** /realms/{realm_name}/protocol/openid-connect/registrations | Register a new user
[**revoke_token**](AuthApi.md#revoke_token) | **POST** /realms/{realm_name}/protocol/openid-connect/revoke | Token revocation
[**send_magic_link**](AuthApi.md#send_magic_link) | **POST** /realms/{realm_name}/login-actions/send-magic-link | Send magic link for passwordless authentication
[**setup_otp**](AuthApi.md#setup_otp) | **GET** /realms/{realm_name}/login-actions/setup-otp | Setup OTP for user authentication
[**update_password**](AuthApi.md#update_password) | **POST** /realms/{realm_name}/login-actions/update-password | Update Password
[**verify_magic_link**](AuthApi.md#verify_magic_link) | **GET** /realms/{realm_name}/login-actions/verify-magic-link | Verify magic link and complete authentication
[**verify_otp**](AuthApi.md#verify_otp) | **POST** /realms/{realm_name}/login-actions/verify-otp | Verify OTP for user authentication
[**webauthn_public_key_authenticate**](AuthApi.md#webauthn_public_key_authenticate) | **POST** /realms/{realm_name}/login-actions/webauthn-public-key-authenticate | Authenticate using webauthn
[**webauthn_public_key_create**](AuthApi.md#webauthn_public_key_create) | **POST** /realms/{realm_name}/login-actions/webauthn-public-key-create | Validate and save a webauthn public key
[**webauthn_public_key_create_options**](AuthApi.md#webauthn_public_key_create_options) | **POST** /realms/{realm_name}/login-actions/webauthn-public-key-create-options | Create a webauthn public key
[**webauthn_public_key_request_options**](AuthApi.md#webauthn_public_key_request_options) | **POST** /realms/{realm_name}/login-actions/webauthn-public-key-request-options | Request webauthn challenge



## auth_handler

> auth_handler(realm_name, response_type, client_id, redirect_uri, scope, state)
Authenticate a user

Initiates the authentication process for a user in a specific realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**response_type** | Option<**String**> |  |  |
**client_id** | Option<**String**> |  |  |
**redirect_uri** | Option<**String**> |  |  |
**scope** | Option<**String**> |  |  |
**state** | Option<**String**> |  |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## authenticate

> models::AuthenticateResponse authenticate(realm_name, authenticate_request)
Authenticate a user in a realm

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**authenticate_request** | [**AuthenticateRequest**](AuthenticateRequest.md) |  | [required] |

### Return type

[**models::AuthenticateResponse**](AuthenticateResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## burn_recovery_code

> models::BurnRecoveryCodeResponse burn_recovery_code(realm_name, burn_recovery_code_request)
Burn a recovery code to authenticate

Using a recovery code allows a user to bypass a MFA challenge

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Name of the realm | [required] |
**burn_recovery_code_request** | [**BurnRecoveryCodeRequest**](BurnRecoveryCodeRequest.md) |  | [required] |

### Return type

[**models::BurnRecoveryCodeResponse**](BurnRecoveryCodeResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## challenge_otp

> models::ChallengeOtpResponse challenge_otp(realm_name, challenge_otp_request)
Challenge OTP for user authentication

Challenges the user to provide a One-Time Password (OTP) for authentication. This is typically used in multi-factor authentication scenarios.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Name of the realm | [required] |
**challenge_otp_request** | [**ChallengeOtpRequest**](ChallengeOtpRequest.md) |  | [required] |

### Return type

[**models::ChallengeOtpResponse**](ChallengeOtpResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## exchange_token

> models::JwtToken exchange_token(realm_name, token_request_validator)
Exchange token

Exchanges a token for a JWT token. This endpoint allows clients to exchange various types of tokens (like authorization codes, refresh tokens, etc.) for a JWT token.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**token_request_validator** | [**TokenRequestValidator**](TokenRequestValidator.md) |  | [required] |

### Return type

[**models::JwtToken**](JwtToken.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## generate_recovery_codes

> models::GenerateRecoveryCodesResponse generate_recovery_codes(realm_name, generate_recovery_codes_request)
Generate recovery codes

Generates recovery codes that allows the user to bypass a MFA challenge

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Name of the realm | [required] |
**generate_recovery_codes_request** | [**GenerateRecoveryCodesRequest**](GenerateRecoveryCodesRequest.md) |  | [required] |

### Return type

[**models::GenerateRecoveryCodesResponse**](GenerateRecoveryCodesResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_certs

> models::GetCertsResponse get_certs(realm_name)
Get JWK keys for a realm

Retrieves the JSON Web Key (JWK) keys for a specific realm, used for verifying JWT tokens.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |

### Return type

[**models::GetCertsResponse**](GetCertsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_jwks_json

> models::GetCertsResponse get_jwks_json(realm_name)
Get JWKS for a realm

Retrieves the JSON Web Key Set (JWKS) for a specific realm, used by resource servers to validate JWT signatures.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |

### Return type

[**models::GetCertsResponse**](GetCertsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_openid_configuration

> models::GetOpenIdConfigurationResponse get_openid_configuration(realm_name)
Get OpenID Connect configuration

Retrieves the OpenID Connect configuration for a specific realm. This endpoint provides metadata about the OpenID Connect provider, including endpoints for authorization, token issuance, introspection, user information, and JWKs.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |

### Return type

[**models::GetOpenIdConfigurationResponse**](GetOpenIdConfigurationResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_userinfo

> models::UserInfoResponse get_userinfo(realm_name)
Get user info

Retrieves the user information for the authenticated user in a specific realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |

### Return type

[**models::UserInfoResponse**](UserInfoResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## introspect_token

> models::TokenIntrospectionResponse introspect_token(realm_name, introspect_request_validator)
Token introspection

OAuth2/OIDC Token Introspection (RFC 7662). Only confidential clients may call this endpoint using client_secret_basic or client_secret_post. Authorization requires the caller's service account to have the role `introspect` (treated as the `introspect` scope).

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**introspect_request_validator** | [**IntrospectRequestValidator**](IntrospectRequestValidator.md) |  | [required] |

### Return type

[**models::TokenIntrospectionResponse**](TokenIntrospectionResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## logout_get

> logout_get(realm_name, id_token_hint, post_logout_redirect_uri, state, client_id)
OIDC RP-Initiated Logout

Ends the user's OP session. Supports id_token_hint, post_logout_redirect_uri, state, and client_id.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**id_token_hint** | Option<**String**> |  |  |
**post_logout_redirect_uri** | Option<**String**> |  |  |
**state** | Option<**String**> |  |  |
**client_id** | Option<**String**> |  |  |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## logout_post

> logout_post(realm_name, logout_request_validator)
OIDC RP-Initiated Logout

Ends the user's OP session. Supports id_token_hint, post_logout_redirect_uri, state, and client_id.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**logout_request_validator** | [**LogoutRequestValidator**](LogoutRequestValidator.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## registration_handler

> models::JwtToken registration_handler(realm_name, registration_request)
Register a new user

Register a new user in the specified realm

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | The realm name | [required] |
**registration_request** | [**RegistrationRequest**](RegistrationRequest.md) |  | [required] |

### Return type

[**models::JwtToken**](JwtToken.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## revoke_token

> revoke_token(realm_name, revoke_token_request_validator)
Token revocation

OAuth2 token revocation endpoint (RFC 7009). Revokes access or refresh tokens for the requesting client.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**revoke_token_request_validator** | [**RevokeTokenRequestValidator**](RevokeTokenRequestValidator.md) |  | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## send_magic_link

> models::SendMagicLinkResponse send_magic_link(realm_name, send_magic_link_request)
Send magic link for passwordless authentication

Sends a magic link to the user's email for passwordless authentication. The link contains a unique token that can be used to verify the user's identity.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | The realm name | [required] |
**send_magic_link_request** | [**SendMagicLinkRequest**](SendMagicLinkRequest.md) |  | [required] |

### Return type

[**models::SendMagicLinkResponse**](SendMagicLinkResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## setup_otp

> models::SetupOtpResponse setup_otp(realm_name)
Setup OTP for user authentication

Sets up a One-Time Password (OTP) for user authentication. This is typically used in multi-factor authentication scenarios.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |

### Return type

[**models::SetupOtpResponse**](SetupOtpResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_password

> models::UpdatePasswordResponse update_password(realm_name, update_password_request)
Update Password

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Name of the realm | [required] |
**update_password_request** | [**UpdatePasswordRequest**](UpdatePasswordRequest.md) |  | [required] |

### Return type

[**models::UpdatePasswordResponse**](UpdatePasswordResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## verify_magic_link

> models::AuthenticateResponse verify_magic_link(realm_name, token_id, magic_token)
Verify magic link and complete authentication

Verifies the magic link token and completes the authentication flow. Returns authentication status and redirect URL with authorization code.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Name of the realm | [required] |
**token_id** | **String** | The unique token identifier from the magic link | [required] |
**magic_token** | **String** | The secret verification token from the magic link | [required] |

### Return type

[**models::AuthenticateResponse**](AuthenticateResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## verify_otp

> models::VerifyOtpResponse verify_otp(realm_name, otp_verify_request)
Verify OTP for user authentication

Verifies the One-Time Password (OTP) provided by the user. This is typically used in multi-factor authentication scenarios.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**otp_verify_request** | [**OtpVerifyRequest**](OtpVerifyRequest.md) |  | [required] |

### Return type

[**models::VerifyOtpResponse**](VerifyOtpResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## webauthn_public_key_authenticate

> models::AuthenticationAttemptResponse webauthn_public_key_authenticate(realm_name, body)
Authenticate using webauthn

Attempt authentication using a WebAuthnAssertionResponse payload for webauthn authentication. See https://w3c.github.io/webauthn/#dictdef-authenticationresponsejson and https://w3c.github.io/webauthn/#authenticatorassertionresponse

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Name of the realm | [required] |
**body** | **serde_json::Value** |  | [required] |

### Return type

[**models::AuthenticationAttemptResponse**](AuthenticationAttemptResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## webauthn_public_key_create

> serde_json::Value webauthn_public_key_create(realm_name, body)
Validate and save a webauthn public key

Saving a webauthn public key to use it for authentication attempts or MFA later.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Name of the realm | [required] |
**body** | **serde_json::Value** |  | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## webauthn_public_key_create_options

> serde_json::Value webauthn_public_key_create_options(realm_name)
Create a webauthn public key

Provides a full PublicKeyCredentialCreationOption payload for WebAuthn credential creation/authentication. The payload contains the challenge to resolve in B64Url form as described in the specs. The content is described here: https://w3c.github.io/webauthn/#dictdef-publickeycredentialcreationoptions.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Name of the realm | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## webauthn_public_key_request_options

> serde_json::Value webauthn_public_key_request_options(realm_name)
Request webauthn challenge

Provides a full PublicKeyCredentialRequestOption payload for webauthn authentication. See https://w3c.github.io/webauthn/#dictdef-publickeycredentialrequestoptions and https://w3c.github.io/webauthn/#dictdef-publickeycredentialrequestoptionsjson

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Name of the realm | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
