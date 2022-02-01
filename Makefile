all:

precommit: fmt fix

fmt:
	cargo fmt

fix:
	cargo fix --allow-dirty --allow-staged

.PHONY: precommit fmt fix