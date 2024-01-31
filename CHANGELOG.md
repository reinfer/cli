# v0.22.2
- Fix a bug where some label annotations cannot be applied

# v0.22.1
- minor api improvements 

# v0.22.0
- Add integration commands

# v0.21.5 
- Fix a bug where stream responses were not correctly parsed 
- Fix a bug where streams were not correctly advanced 

# v0.21.4
- Add messages filters
- Fixes `required` field error when interacting with datasets

## v0.21.3 
- Reduce batch size for parse emls

## v0.21.2

- Add get audit events command
- Add ability to parse .emls

## v0.21.1

- Add more stream stats

## v0.21.0

- Fix url used for fetching streams
- Return `is_end_sequence` on stream fetch
- Make `transform_tag` optional on `create bucket`
- Retry `put_emails` requests
- Add `get stream-stats` to expose and compare model validation

## v0.20.0

- Add ability to get dataset stats
- Show Global Permissions in `get users`
- Upgrade `ordered-float` version, which is exposed in the public crate api.
- Add ability to filter users by project and permission
- Add feature to parse unicode msg files

## v0.19.0

- Add create streams command
- Show source statistics in table when getting sources

## v0.18.2

-Add ability to filter on user properties when getting comments

## v0.18.1

- Add comment id to document object in api

## v0.18.0

- Add label filter when downloading comments with predictions
- Retry requests on request error

## v0.17.2

- Retry TOO_MANY_REQUESTS

## v0.17.1

- Support markup in signatures
- Fix bug where annotations may have been uploaded before comments, causing a failure

## v0.17.0

- Always retry on connection issues
- Upload annotations in parallel

## v0.16.1

- Add attachments to `sync-raw-email`

## v0.16.0

- Add command to list quotas for current tenant
- Show correct statistics when downloading comments
- Add `sync-raw-emails` to api

## v0.15.0

- Add support for markup on comments

## v0.14.0

- Add a warning for UiPath cloud users when an operation will charge ai units

## v0.13.4

- Add user property filters to the query api

## v0.13.3

- Add recent as an option for the query api

## v0.13.2

- Skip serialization of continuation on `None`

## v0.13.1

- Add `no-charge` flag to create comment/email commands
- Add comment and label filters to `get_statistics`
- Add timeseries to `get_statistics`
- Add `query_dataset` to api

## Added

- `re get comments` returns label properties

# v0.12.3

## Added

- `re create quota` to set a quota in a tenant

# v0.12.2

- Rename "triggers" to "streams" following the rename in the API
- Removed semantic url joins to support deployments within a subdirectory
- Added functionality to use moon forms both in `LabelDef`s and in `AnnotatedComments`s

## Added

- `re get comments` will now return auto-thresholds for predicted labels if provided with a `--model-version` parameter
- `re update users` for bulk user permission updates
- Option to send welcome email on create user

# v0.12.1

## Added

- `re update source` can now update the source's transform tag
- `re get source` and `re get sources` will show bucket name if exists.
- `re get comments` can now download predictions within a given timerange

# v0.12.0

## Added

- Display project ids when listing projects
- Add support for getting or deleting a single user
- Upgrade all dependencies to their latest released version
- Enable retry logic for uploading annotations
- Add support for optionally setting a transform tag on a source

# v0.11.0

## Breaking

- Renames organisation -> project throughout, including in the CLI command line arguments for consistency with the new API
- `re create dataset` will default to sentiment disabled if `--has-sentiment` is not provided.
- Changed `--source-type` parameter to `--kind`.

## Added

- `re create trigger-exception` to tag a comment exception within a trigger.

## Bug Fixes

- Fix serialization of sources after api change of internal parameter `_kind`.

# v0.10.2

## Bug Fixes

- Fixes serialization issue where statistics expected `usize` not `f64`

# v0.10.1

## Added

- Add an optional `--source-type` parameter to `create source`. Only for internal use.

# v0.10.0

## Added

- New `re create annotations` command for uploading annotations (labels and
  entities) to existing comments in a dataset, without having to use `re create comments`.
  This avoids potentially - and unknowingly - modifying the underlying comments in the source.
- Add support to `--force` delete projects with existing resources.
- Print comment `uid` when a comment upload fails due to bad annotations.

## Bug Fixes

- Failure when uploading comments with thread properties

# v0.9.0

## Breaking

- Added support for new labellings api. Old jsonl files can still be uploaded with `re` but newly downloaded jsonl files will be in the new format.

## Added

- Deserialize thread properties when downloading comments for a dataset (the `-d dataset` option for `re get comments`). This limitation exists as only the
  /labellings API route returns thread properties.
- Added `re config get-token [context]` which dumps the auth token for the
  current or a different, given context.
- Added CRUD commands for projects.
- Added option for `--label-groups` in `re create dataset`.

# v0.8.0

## Breaking

- All API resources with floats now use `ordered_float::NotNan`
- A new top level flag `-o/--output` has been added. This replaces all previous `-o/--output` flags in the `re get *` subcommands.
- The `EntityDefs` wrapper has been removed in favour of `Vec<EntityDef>`. This impacts the `NewDataset` and `Dataset` structs
- `EntityDef` has added fields to accurately reflect the api return type
- Added `metadata` field to the `Label` struct

## Changed

- More public types implement `Serialize`, `Eq` and `Hash` for downstream use.

## Added

- `get comment`: get a single comment by source and id
- Created or updated resources will be returned via stdout. The format of the output can be changed with the global `-o/--output` flag.
  - This excludes creation of the `comments` and `emails` resources.
- Added `entity_defs` and `label_defs` to the `reinfer_api::Dataset` struct, and `create dataset` command
- Added `LabelDef`, `NewLabelDef`, `NewEntity` and associated structs

# v0.7.0

## Breaking

- `NewDataset`'s `entity_defs` field is now an `Option` for consistency

## Changed

- When uploading annotated comments, empty lists of assigned / dismissed labels
  are serialized in the request. Previously empty lists were skipped which
  meant it was not possible to remove labellings (N.B. the API distinguishes
  between missing field -- labellings are unmodified -- or and empty list --
  labellings are removed).
- All `*Id` types now implement `Hash`, `PartialEq`, and `Eq`
- `NewDataset` and `NewSource` now implement `Default`

## Added

- `update source`: update an existing source
- `update dataset`: update an existing dataset

# v0.6.0

## Breaking

- The `create bucket` flag `--transform-tag` is now required.

## Changed

- `delete bulk`: slight performance optimisations.
- `create dataset`: Accept an optional `--model-family` and `--copy-annotations-from` for the new dataset.

# v0.5.3

- `delete bulk`: For deleting multiple comments by source id. When run with `--include-annotated=false`, skips annotated comments. When run with `--include-annotated=true`, deletes all comments.

# v0.5.2

- Add support for using an HTTP proxy for all requests. The proxy configuration is saved as part of the context.
  Additionally, the proxy can be overridden / specified as a one off using a global command line argument
  `--proxy https://proxy.example` (e.g. similar to `--endpoint`).

# v0.5.1

## Breaking

- Updated error types and handling throughout. This changes the publicly visible `reinfer_client::errors` module.

- The `-e` flag used to pass in entity kinds at dataset creation has been re-purposed. One now needs to pass in a `json` object containing the corresponding `EntityDef` to be added to the new dataset. Example:

```
re create dataset org/example-dataset -s org/example-source --has-sentiment false -e '[{"name":"trainable_org","title":"Custom Organisation","inherits_from":["org"],"trainable":true}','{"name":"non_trainable_person","title":"Basic Person","inherits_from":["person"],"trainable":false}]'
```

## Added

- `delete comments`: For deleting multiple comments by comment id from a source.

## Changed

- `create bucket`: Accept an optional `--transform-tag` value for the new bucket.
- `get buckets`: Display transform tag for retrieved data.

# v0.4.1

## Changed

- `create source`: Improve error message when specifying an invalid source name.
- Commands which make multiple API requests will now retry on timeout for requests after the first.
- Bump all dependencies to latest versions.

# v0.4.0

## Added

- `create comments`: Add check for duplicate comment IDs before attempting upload of a comment file. Use `--allow-duplicates` to skip this check.

## Changed

- `create comments`, `get comments`, `create emails`: Replace `--progress` flag with `--no-progress`.
- `create comments`: Stop overwriting existing comments by default. Use `--overwrite` to use the previous behaviour.

# v0.3.2

This release is identical to 0.3.1, but was republished due to a packaging bug.

# v0.3.1

## Bugfixes

- Fixes downloading predictions for comments in sentimentless datasets (#6).
