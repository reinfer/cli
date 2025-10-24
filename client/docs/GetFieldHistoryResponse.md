# GetFieldHistoryResponse

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**status** | Option<**String**> |  | [optional][default to Ok]
**versions** | [**Vec<models::FieldVersionEntry>**](FieldVersionEntry.md) | Version history entries | 
**oldest_version** | Option<**i32**> | To be used with older_than_version for next page. If None, there are no more versions. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


