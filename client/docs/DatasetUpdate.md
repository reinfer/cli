# DatasetUpdate

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**title** | Option<**String**> |  | [optional]
**description** | Option<**String**> |  | [optional]
**source_ids** | Option<**Vec<String>**> | IDs of the sources contained in this dataset | [optional]
**entity_kinds** | Option<**Vec<String>**> |  | [optional]
**entity_defs** | Option<[**Vec<models::DatasetUpdateEntityDefsInner>**](DatasetUpdate_entity_defs_inner.md)> |  | [optional]
**general_fields** | Option<[**Vec<models::GeneralFieldDefUpdate>**](GeneralFieldDefUpdate.md)> |  | [optional]
**_label_properties** | Option<[**Vec<models::LabelPropertyId>**](LabelPropertyId.md)> |  | [optional]
**debug_config_json** | Option<**String**> |  | [optional]
**model_family** | Option<[**models::ModelFamily**](ModelFamily.md)> |  | [optional]
**_timezone** | Option<**String**> | The name of the IANA time zone used by the frontend to display the timestamps in local time | [optional]
**_preferred_locales** | Option<[**models::Locales**](Locales.md)> |  | [optional]
**_dataset_flags** | Option<[**Vec<models::DatasetFlag>**](DatasetFlag.md)> |  | [optional]
**_model_config** | Option<[**models::ModelConfig**](Model_Config.md)> |  | [optional]
**_default_label_group_instructions** | Option<**String**> | The instructions to apply to the default label group | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


