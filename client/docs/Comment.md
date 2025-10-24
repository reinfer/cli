# Comment

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**uid** | **String** |  | 
**id** | **String** | ID of the comment unique within a source | 
**timestamp** | **String** | User supplied comment timestamp | 
**thread_id** | Option<**String**> |  | [optional]
**user_properties** | [**std::collections::HashMap<String, models::UserPropertiesValue>**](User_Properties_value.md) |  | 
**messages** | [**Vec<models::Message>**](Message.md) |  | 
**text_format** | Option<[**models::TextFormat**](TextFormat.md)> |  | [optional]
**attachments** | [**Vec<models::Attachment>**](Attachment.md) |  | 
**source_id** | **String** | ID of the source containing the comment | 
**last_modified** | **String** | Timestamp of the last modification of the comment | 
**created_at** | **String** | Timestamp of the creation of the comment | 
**context** | Option<**String**> |  | [optional]
**has_annotations** | Option<**bool**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


