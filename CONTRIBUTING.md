# Contributing

Pull-request titles must follow [Conventional Commits](https://www.conventionalcommits.org/), for example `feat(order): add typed market orders` or `fix(client): bound response bodies`. CI checks the pull-request title.

The release workflow assumes squash merging with GitHub configured to use the pull-request title as the default squash-commit subject. This makes the title validated by CI the Conventional Commit consumed by Release Please.

Before opening a pull request, run:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-features --locked
cargo package --locked
```
