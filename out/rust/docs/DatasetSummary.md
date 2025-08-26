# DatasetSummary

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**user_properties** | [**models::DatasetUserPropertiesSummary**](DatasetUserPropertiesSummary.md) | User properties | 
**label_properties** | **Vec<String>** | List of label properties | 
**source_kinds** | **Vec<String>** | List of label values | 
**source_names** | **Vec<String>** | List of source names | 
**annotation_metadata** | [**models::AnnotationMetadataCount**](AnnotationMetadataCount.md) | Annotation metadata | 
**labels** | [**Vec<models::SingleLabelSummary>**](SingleLabelSummary.md) | Labels | 
**entity_defs** | [**Vec<models::EntityDefSummary>**](EntityDefSummary.md) | Entity definitions | 
**email_properties** | Option<[**models::EmailPropertiesSummaryByProperty**](EmailPropertiesSummaryByProperty.md)> | Email properties | [optional]
**trigger_exceptions** | Option<[**std::collections::HashMap<String, models::TriggerExceptionsSummary>**](TriggerExceptionsSummary.md)> | Trigger exceptions | [optional]
**attachments_common_types_extensions** | [**std::collections::HashMap<String, Vec<String>>**](Vec.md) | Mapping between common types and file extensions to be used in attachment property filters | 
**attachments_file_extensions** | [**std::collections::HashMap<String, Vec<String>>**](Vec.md) | List of file extensions to be shown in attachment property filters | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


