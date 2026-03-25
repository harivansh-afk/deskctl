.PHONY: fmt fmt-check lint test-unit test-integration site-format-check validate

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

lint:
	cargo clippy --all-targets -- -D warnings

test-unit:
	cargo test --lib

test-integration:
	@if [ "$$(uname -s)" != "Linux" ]; then \
		echo "Integration tests require Linux and xvfb-run."; \
		exit 1; \
	fi
	@if ! command -v xvfb-run >/dev/null 2>&1; then \
		echo "xvfb-run is required to execute integration tests."; \
		exit 1; \
	fi
	XDG_SESSION_TYPE=x11 xvfb-run -a cargo test --test x11_runtime -- --test-threads=1

site-format-check:
	@if ! command -v pnpm >/dev/null 2>&1; then \
		echo "pnpm is required for site formatting checks."; \
		exit 1; \
	fi
	pnpm --dir site format:check

validate: fmt-check lint test-unit test-integration site-format-check
