# Contributing to Sanctifier

Welcome and thanks for contributing!

## PR Process

- Create an issue or confirm there is already one.
- Fork the repository and create a branch: `git checkout -b issue-###-description`.
- Implement the code and run tests locally:
  - `cargo fmt --all`
  - `cargo test -p sanctifier-core --all-features`
  - `cargo test -p sanctifier-cli --no-default-features`
- Push to your fork and open a PR to `HyperSafeD/Sanctifier:main`.
- Ensure that the PR is checked by CI and that all required status checks pass.
- Seek at least one approving review.

## Branch Protection

This repo uses branch protection for `main`:
- Required status check: `Continuous Integration`
- Require branches to be up to date before merging
- Require at least 1 review approval
- Disallow force pushes

See `BRANCH_PROTECTION.md` for details.

## Code Style

- Use `cargo fmt --all` for formatting.
- Use `cargo clippy` for lint checks.

## QA checklist

- [ ] Branch created for specific issue
- [ ] CI passes on opened PR
- [ ] Peer review completed
- [ ] No direct push to main
