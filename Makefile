.PHONY: generate-kotlin-bindings rust-fmt-check rust-clippy rust-test rust-quality

generate-kotlin-bindings:
	powershell -ExecutionPolicy Bypass -File scripts/generate-kotlin-bindings.ps1

rust-fmt-check:
	cargo fmt --all -- --check

rust-clippy:
	cargo clippy --workspace --all-targets -- -D warnings

rust-test:
	cargo test --workspace --all-targets

rust-quality: rust-fmt-check rust-clippy rust-test
