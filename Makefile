.PHONY: clean build validator local test docs release

clean:
	@rm -rf test-ledger

build:
	@cd program && cargo build-sbf

test:
	@cd program && cargo test-sbf

docs:
	cargo doc --workspace --no-deps --open

release:
ifndef VERSION
	$(error VERSION is not set. Usage: make release VERSION=0.1.6)
endif
	cargo release $(VERSION) --workspace --execute

validator:
	solana-test-validator \
	  --bpf-program usdfcP2V1bh1Lz7Y87pxR4zJd3wnVtssJ6GeSHFeZeu target/deploy/usdf_swap.so

local: clean build validator
