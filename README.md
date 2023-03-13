## Before Your First Commit

This crate relies on SQLx's offline mode in order to compile in CI. As a result, every time a SQLx query is created, deleted, or modified in some way, `cargo sqlx prepare -- --all-targets --all-features` has to be run to create or update the `sqlx-data.json` file at the top-level directory.

(Naturally, this requires that `sqlx-cli` is installed. Head to https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md to do so.)

Since that's easy to forget, I made a Git hook that runs the command before every commit. Unfortunately, client-side hooks are difficult to share, so if you've just cloned the repository, you'll have to manually create the file. Add the following code to `.git/hooks/pre-commit`:

```sh
#!/bin/sh

# Prepare SQLx queries for "offline mode", which is necessary to compile the
# crate in CI, since migrations are run at runtime.
#
# This could have been done in `pre-push` (since it concerns the remote
# repository), but it would necessitate an extra commit. `git add` is cleaner.
#
# Note: For this to work, the "offline" feature has to be enabled for the SQLx
# dependency in `Cargo.toml` and `sqlx-cli` has to be installed.
cargo sqlx prepare -- --all-targets --all-features
git add sqlx-data.json
```

Git hooks need to be executable, so don't forget to run `chmod +x .git/hooks/pre-commit` (or whatever your OS's equivalent is).
