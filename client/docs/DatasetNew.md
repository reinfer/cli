# DatasetNew

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**title** | Option<**String**> |  | [optional]
**description** | Option<**String**> |  | [optional]
**source_ids** | Option<**Vec<String>**> | IDs of the sources contained in this dataset | [optional]
**entity_kinds** | Option<**Vec<String>**> |  | [optional]
**entity_defs** | Option<[**Vec<models::EntityDefNew>**](EntityDefNew.md)> |  | [optional]
**general_fields** | Option<[**Vec<models::GeneralFieldDefNew>**](GeneralFieldDefNew.md)> |  | [optional]
**_label_properties** | Option<[**Vec<models::LabelPropertyId>**](LabelPropertyId.md)> |  | [optional]
**debug_config_json** | Option<**String**> |  | [optional]
**model_family** | Option<**String**> |  | [optional]
**_timezone** | Option<**String**> | The name of the IANA time zone (UTC if missing) used by the frontend to display the timestamps in local time | [optional]
**_preferred_locales** | Option<[**models::Locales**](Locales.md)> |  | [optional]
**_dataset_flags** | Option<[**Vec<models::DatasetFlag>**](DatasetFlag.md)> |  | [optional]
**_model_config** | Option<[**models::ModelConfig**](Model_Config.md)> |  | [optional]
**has_sentiment** | Option<**bool**> |  | [optional]
**experimental_async_fork_dataset** | Option<**bool**> |  | [optional]
**label_groups** | Option<[**Vec<models::LabelGroupNew>**](LabelGroupNew.md)> |  | [optional]
**label_defs** | Option<[**Vec<models::LabelDefNew>**](LabelDefNew.md)> |  | [optional]
**copy_annotations_from** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


