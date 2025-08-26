# UserWithLicenseCheck

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
**last_login_at** | **String** |  | 
**is_support_user** | **bool** |  | 
**user_access_token** | Option<**String**> |  | [optional]
**license** | [**models::UserLicense**](UserLicense.md) |  | 
**license_check** | [**models::UserLicenseCheck**](UserLicenseCheck.md) |  | 

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


