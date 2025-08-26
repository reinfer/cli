# CommentFilter

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**entities** | Option<[**models::EntitiesRules**](EntitiesRules.md)> |  | [optional]
**thread_properties** | Option<[**models::ThreadPropertyFilter**](ThreadPropertyFilter.md)> |  | [optional]
**timestamp** | Option<[**models::TimestampRangeFilter**](TimestampRangeFilter.md)> |  | [optional]
**reviewed** | Option<[**models::Reviewed**](Reviewed.md)> |  | [optional]
**sources** | Option<**Vec<String>**> |  | [optional]
**messages** | Option<[**models::MessageFilter**](MessageFilter.md)> |  | [optional]
**user_properties** | Option<[**serde_json::Value**](.md)> |  | [optional]
**trigger_exceptions** | Option<[**std::collections::HashMap<String, models::NullableStringArrayFilter>**](NullableStringArrayFilter.md)> |  | [optional]
**annotations** | Option<[**models::Annotations**](Annotations.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


