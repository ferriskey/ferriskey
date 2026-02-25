# \WebhookApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_webhook**](WebhookApi.md#create_webhook) | **POST** /realms/{realm_name}/webhooks | Create webhook
[**delete_webhook**](WebhookApi.md#delete_webhook) | **DELETE** /realms/{realm_name}/webhooks/{webhook_id} | Delete webhook
[**fetch_webhooks**](WebhookApi.md#fetch_webhooks) | **GET** /realms/{realm_name}/webhooks | Fetch all webhooks
[**get_webhook**](WebhookApi.md#get_webhook) | **GET** /realms/{realm_name}/webhooks/{webhook_id} | Get webhook
[**update_webhook**](WebhookApi.md#update_webhook) | **PUT** /realms/{realm_name}/webhooks/{webhook_id} | Update webhook



## create_webhook

> models::CreateWebhookResponse create_webhook(realm_name, create_webhook_validator)
Create webhook

Creates a new webhook in the system related to the current realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**create_webhook_validator** | [**CreateWebhookValidator**](CreateWebhookValidator.md) |  | [required] |

### Return type

[**models::CreateWebhookResponse**](CreateWebhookResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_webhook

> models::DeleteWebhookResponse delete_webhook(realm_name, webhook_id)
Delete webhook

Deletes a webhook in the system related to the current realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Realm name | [required] |
**webhook_id** | **uuid::Uuid** | Webhook ID | [required] |

### Return type

[**models::DeleteWebhookResponse**](DeleteWebhookResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_webhooks

> models::GetWebhooksResponse fetch_webhooks(realm_name)
Fetch all webhooks

Retrieves a list of all webhooks available in the system related to the current realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Name of the realm | [required] |

### Return type

[**models::GetWebhooksResponse**](GetWebhooksResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_webhook

> models::Webhook get_webhook(realm_name, webhook_id)
Get webhook

Retrieves one webhook in the system related to the current realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Name of the realm | [required] |
**webhook_id** | **uuid::Uuid** | Webhook ID | [required] |

### Return type

[**models::Webhook**](Webhook.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_webhook

> models::UpdateWebhookResponse update_webhook(realm_name, webhook_id)
Update webhook

Updates a webhook in the system related to the current realm.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**realm_name** | **String** | Name of the realm | [required] |
**webhook_id** | **uuid::Uuid** | Webhook ID | [required] |

### Return type

[**models::UpdateWebhookResponse**](UpdateWebhookResponse.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

