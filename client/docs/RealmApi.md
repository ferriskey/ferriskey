# \RealmApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_realm**](RealmApi.md#create_realm) | **POST** /realms | Create a new realm
[**delete_realm**](RealmApi.md#delete_realm) | **DELETE** /realms/{name} | Delete a realm by name
[**get_login_realm_settings_handler**](RealmApi.md#get_login_realm_settings_handler) | **GET** /realms/{name}/login-settings | Get login settings
[**get_realm**](RealmApi.md#get_realm) | **GET** /realms/{name} | Get a realm by name
[**get_user_realms**](RealmApi.md#get_user_realms) | **GET** /realms/{realm_name}/users/@me/realms | Get user realms
[**update_realm**](RealmApi.md#update_realm) | **PUT** /realms/{name} | Update a realm by name
[**update_realm_setting**](RealmApi.md#update_realm_setting) | **PUT** /realms/{name}/settings | Update settings for a realm by name



## create_realm

> models::Realm create_realm(create_realm_validator)
Create a new realm

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_realm_validator** | [**CreateRealmValidator**](CreateRealmValidator.md) |  | [required] |

### Return type

[**models::Realm**](Realm.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_realm

> String delete_realm(name)
Delete a realm by name

Deletes a realm by its name. This action is irreversible and will remove all associated data.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | Realm name | [required] |

### Return type

**String**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_login_realm_settings_handler

> models::RealmLoginSetting get_login_realm_settings_handler(name)
Get login settings

Get the login settings for a specific realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | The name of the realm | [required] |

### Return type

[**models::RealmLoginSetting**](RealmLoginSetting.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_realm

> models::Realm get_realm(name)
Get a realm by name

Retrieves a realm by its name. This endpoint returns the details of the specified realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | Realm name | [required] |

### Return type

[**models::Realm**](Realm.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_user_realms

> models::UserRealmsResponse get_user_realms(realm_name)
Get user realms

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Name of the realm | [required] |

### Return type

[**models::UserRealmsResponse**](UserRealmsResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_realm

> models::UpdateRealmResponse update_realm(name, update_realm_validator)
Update a realm by name

Updates the name of an existing realm. This endpoint allows you to change the name of a realm while keeping its associated data intact.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | Realm name | [required] |
**update_realm_validator** | [**UpdateRealmValidator**](UpdateRealmValidator.md) |  | [required] |

### Return type

[**models::UpdateRealmResponse**](UpdateRealmResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_realm_setting

> models::UpdateRealmSettingResponse update_realm_setting(name, update_realm_setting_validator)
Update settings for a realm by name

Updates the settings for a specified realm. This endpoint allows modification of various realm settings.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**name** | **String** | Realm name | [required] |
**update_realm_setting_validator** | [**UpdateRealmSettingValidator**](UpdateRealmSettingValidator.md) |  | [required] |

### Return type

[**models::UpdateRealmSettingResponse**](UpdateRealmSettingResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)
