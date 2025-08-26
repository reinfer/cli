# Dataset

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | Unique ID of the dataset | 
**owner** | **String** | The project which owns the dataset | 
**name** | **String** | The API name of the dataset which appears in URLs | 
**title** | **String** | A short description of the dataset | 
**description** | **String** | A longer description of the dataset | 
**created** | **String** | The date and time the dataset was created | 
**last_modified** | **String** | The date and time the dataset was last modified | 
**model_family** | [**models::ModelFamily**](ModelFamily.md) | The model family used to train the dataset | 
**source_ids** | **Vec<String>** | IDs of the sources contained in this dataset | 
**has_sentiment** | **bool** | Whether the dataset is sentimentful | 
**limited_access** | **bool** | Limited Access | 
**entity_defs** | [**Vec<models::EntityDef>**](EntityDef.md) | Entity Defs for this Dataset | 
**general_fields** | [**Vec<models::GeneralFieldDef>**](GeneralFieldDef.md) | General Fields for this Dataset | 
**label_groups** | [**Vec<models::LabelGroup>**](LabelGroup.md) | Label Groups for this Dataset | 
**label_defs** | [**Vec<models::LabelDef>**](LabelDef.md) | Label Defs for this Dataset | 
**_label_properties** | [**Vec<models::LabelPropertyId>**](LabelPropertyId.md) | Label Properties | 
**_timezone** | **String** | The name of the IANA time zone used by the frontend to display the timestamps in local time | 
**_preferred_locales** | [**models::PreferredLocale**](PreferredLocale.md) | Preferred Locales | 
**_dataset_flags** | [**Vec<models::DatasetFlag>**](DatasetFlag.md) | Flags for the dataset | 
**debug_config_json** | Option<**String**> | Debug Config JSON | [optional]
**readonly** | Option<**bool**> | Whether the dataset is readonly | [optional]
**_model_config** | [**models::ModelConfig**](Model_Config.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


