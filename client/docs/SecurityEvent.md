# SecurityEvent

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**actor_id** | Option<**uuid::Uuid**> |  | [optional]
**actor_type** | Option<[**models::ActorType**](ActorType.md)> |  | [optional]
**details** | Option<**serde_json::Value**> |  | [optional]
**event_type** | [**models::SecurityEventType**](SecurityEventType.md) |  |
**id** | **uuid::Uuid** |  |
**ip_address** | Option<**String**> |  | [optional]
**realm_id** | **uuid::Uuid** |  |
**resource** | Option<**String**> |  | [optional]
**status** | [**models::EventStatus**](EventStatus.md) |  |
**target_id** | Option<**uuid::Uuid**> |  | [optional]
**target_type** | Option<**String**> |  | [optional]
**timestamp** | **String** |  |
**trace_id** | Option<**String**> |  | [optional]
**user_agent** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
