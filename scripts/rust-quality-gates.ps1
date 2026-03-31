$ErrorActionPreference = "Stop"

Write-Host "[rust-quality] fmt check"
cargo fmt --all -- --check

Write-Host "[rust-quality] clippy"
cargo clippy --workspace --all-targets -- -D warnings

Write-Host "[rust-quality] tests"
cargo test --workspace --all-targets

Write-Host "[rust-quality] all checks passed"
