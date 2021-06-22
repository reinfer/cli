# Unreleased

## Breaking

- All API resources with floats now use `ordered_float::NotNan`

## Changed

- More public types implement `Serialize`, `Eq` and `Hash` for downstream use.

## Added

- `get comment`: get a single comment by source and id

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
