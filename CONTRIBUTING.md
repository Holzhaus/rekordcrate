# Contribution Guide

Thank you for investing your time in contributing to *rekordcrate*!
Here's a quick overview of the most important things to keep in mind:

- Be polite and respectful to each other.
- Work in feature branches, and make sure to base your work on the appropriate base branch. If you started from the wrong branch, please rebase and change the PR target.
- Use our [pre-commit](#using-pre-commit-hooks) hooks.
- Write [meaningful commit messages](https://cbea.ms/git-commit/) and use [conventional commit messages](#conventional-commits).
- Open a [draft PR](https://github.blog/2019-02-14-introducing-draft-pull-requests/) to get early feedback.
- Ensure that every commit builds. Please fix broken commits by rebasing.
- Run `cargo test` before committing to ensure that all tests pass.
- Try to split up bigger changes into smaller PRs that are easier to review (which means they will be merged faster!)


## Using pre-commit Hooks

We're using the [pre-commit framework](https://pre-commit.com/) to automatically run some checks at commit time.
This ensures that every commit fulfills the basic requirements to be mergeable.

    $ pre-commit install
    pre-commit installed at .git/hooks/pre-commit
    $ pre-commit install --hook-type commit-msg
    pre-commit installed at .git/hooks/commit-msg


## Conventional Commits

This project follows the [conventional commit specification](https://www.conventionalcommits.org/en/v1.0.0/).
This means that every commit message should have the following form:

    type(optional-scope): description

Each commit should use one of those *types* (sometimes also called *groups*):

  | Type         | Use when the commit...
  | ------------ | ----------------------
  | `build`      | Affects the build system or external dependencies.
  | `chore`      | Does not belong into any other category and should not be listed in the changelog.
  | `ci`         | Changes the CI/pre-commit configuration files and scripts.
  | `docs`       | Only changes the documentation.
  | `feat`       | Adds a new feature.
  | `fix`        | Fixes a bug present in a released version (otherwise use `chore`).
  | `perf`       | Improves the performance.
  | `refactor`   | Changes the code but neither fixes a bug nor adds a feature.
  | `style`      | Only reformats the code (white-space, formatting, missing semi-colons, etc)
  | `test`       | Affects unit/integration tests.

Optionally, commits may also specify a *scope*.
Here's a non-exhaustive list that helps picking the correct scope.

  | Scope          | Use when commit primarily modifies...
  | -------------- | -------------------------------------
  | `setting`      | The `*SETTING.DAT` file parser (`setting.rs`)
  | `anlz`         | The `ANLZ*.DAT` file parser (`anlz.rs`)
  | `pdb`          | The PDB file parser (`pdb.rs`)
  | `contributing` | The contribution guide (i.e. this document - `CONTRIBUTING.md`), usually used with type `docs`
  | `readme`       | The readme file (`README.md`), usually used with types `chore` or `docs`
  | `changelog`    | The changelog (`CHANGELOG.md`), usually used with types `chore` or `docs`
  | `pre-commit`   | The pre-commit hooks configuration, usually used with type `ci`

Thank you very much for your interest in *rekordcrate*, we're looking forward to your pull requests.
