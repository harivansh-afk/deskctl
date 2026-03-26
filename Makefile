.PHONY: fmt fmt-check lint test-unit test-integration site-format-check cargo-publish-dry-run npm-package-check nix-flake-check dist-validate validate

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

cargo-publish-dry-run:
	cargo publish --dry-run --allow-dirty --locked

npm-package-check:
	@if ! command -v npm >/dev/null 2>&1; then \
		echo "npm is required for npm packaging validation."; \
		exit 1; \
	fi
	node npm/deskctl-cli/scripts/validate-package.js
	rm -rf tmp/npm-pack tmp/npm-install
	mkdir -p tmp/npm-pack tmp/npm-install/bin
	npm pack ./npm/deskctl-cli --pack-destination ./tmp/npm-pack >/dev/null
	@if [ "$$(uname -s)" != "Linux" ]; then \
		echo "Skipping npm package runtime smoke test on non-Linux host."; \
	else \
		cargo build && \
		PACK_TGZ=$$(ls ./tmp/npm-pack/*.tgz | head -n 1) && \
		DESKCTL_BINARY_PATH="$$(pwd)/target/debug/deskctl" npm install --prefix ./tmp/npm-install "$${PACK_TGZ}" && \
		./tmp/npm-install/node_modules/.bin/deskctl --version; \
	fi

nix-flake-check:
	@if ! command -v nix >/dev/null 2>&1; then \
		echo "nix is required for flake validation."; \
		exit 1; \
	fi
	nix flake check

dist-validate: test-unit cargo-publish-dry-run npm-package-check nix-flake-check

validate: fmt-check lint test-unit test-integration site-format-check
