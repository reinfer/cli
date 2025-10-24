# User

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** |  | 
**created** | **String** |  | 
**username** | **String** |  | 
**email** | **String** |  | 
**verified** | **bool** |  | 
**organisation_permissions** | [**std::collections::HashMap<String, Vec<models::ProjectPermission>>**](Vec.md) |  | 
**derived_organisation_permissions** | [**std::collections::HashMap<String, Vec<models::ProjectPermission>>**](Vec.md) |  | 
**global_permissions** | [**Vec<models::GlobalPermission>**](GlobalPermission.md) |  | 
**sso_global_permissions** | [**Vec<models::GlobalPermission>**](GlobalPermission.md) |  | 
**derived_user_global_permissions** | [**Vec<models::GlobalPermission>**](GlobalPermission.md) |  | 
**derived_global_permissions** | [**Vec<models::GlobalPermission>**](GlobalPermission.md) |  | 
**last_login_at** | Option<**String**> |  | 
**is_support_user** | **bool** |  | 
**user_access_token** | Option<**String**> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


