# next

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
