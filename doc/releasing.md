# Release process

Target audience: crate maintainers who wish to release `embedded-graphics-simulator`.

> Please take a cautious approach to this. If any step doesn't feel right or doesn't succeed smoothly, stop and rectify any issues before continuing.

## On GitHub

- Check that all desired PRs are merged and all desired issues are closed/resolved.
- Check that the latest master build passed in CircleCI.

## On your local machine

- `cd` to the repository root
- Check that `cargo-release` is installed and available in `$PATH`:

  ```bash
  cargo release --version
  ```

- Ensure you have the latest changes with `git switch master` and `git pull --rebase`
- Check that your local repository is clean with no uncommitted changes and no unpushed commits. Ideally, use `git reset --hard origin/master` to ensure your local state is up to date with `origin/master`. You may need to change `origin` to the name of the remote pointing to <https://github.com/embedded-graphics/simulator>.
- Before a **stable** release:
  - Search the repository for any `TODO` or `FIXME` comments. If any need resolving before release, stop this process and fix them with one or more PRs.
- Check that the crate version in `Cargo.toml` matches the latest released versions on <https://crates.io/crates/embedded-graphics-simulator>.
- Run `just build` to ensure the build passes locally.
  - If the build fails for any reason, stop the release process and fix any issues by creating PRs. The upstream master branch must remain the source of truth. Restart this checklist once `just build` passes.
- Double check the release level (major, minor, patch)
- Release the crate:

  ```bash
  cargo release --push-remote <push-remote> <level>
  ```

  Where `<level>` is `major`, `minor`, `patch`, or a specific SemVer version number, and where `<push-remote>` is the git remote for the upstream repository `embedded-graphics/simulator`.

## Post release

- Check that the release command pushed a Git tag when the crate was published, something like `v0.3.0-beta.1` or `v0.3.1`.
- For the new tag, go to its page at e.g. <https://github.com/embedded-graphics/simulator/releases/tag/v0.3.0-beta.1>, click <kbd>Edit tag</kbd> and draft a release:

  - Copy and paste the tag into the `Release title` field.
  - Copy and paste the latest released section out of the crate's `CHANGELOG.md` file into the `Describe this release` field. Do not include the version header, e.g.:

    ```markdown
    ### Added

    - [#111](https://github.com/embedded-graphics/simulator/pull/111) Added something

    ### Removed

    - [#222](https://github.com/embedded-graphics/simulator/pull/222) Removed a thing
    ```

  - For `alpha` or `beta` releases, check the `This is a pre-release` checkbox.
  - Hit <kbd>Publish release</kbd>

- Check that the release is displayed on the [repository homepage](https://github.com/embedded-graphics/simulator).
- Post a link to the released tag (e.g. <https://github.com/embedded-graphics/simulator/releases/tag/v0.3.0-beta.1>) to the embedded-graphics Matrix room at <https://matrix.to/#/!SfJCDXZbMHXkPovtKL:matrix.org>
- If you are @jamwaffles, post a Tweet tagging @rustembedded with a happy announcement message.

- Check the other repositories in the [embedded-graphics organization](https://github.com/embedded-graphics) for dependencies on `embedded-graphics-simulator`. The version should be updated to the latest releases made whilst following this guide.
