# UnboundOS top-level Makefile. Convenience wrappers around cargo,
# scripts/, and the QEMU smoke test. The authoritative build commands
# live in CLAUDE.md §5; this file just makes the common ones one
# keystroke away.

CARGO       ?= cargo
KERNEL      := target/x86_64-unboundos/release/kernel
IMAGE       := /tmp/unboundos.img
SERIAL_LOG  := /tmp/unboundos-serial.log

.PHONY: help
help:
	@echo "UnboundOS — top-level make targets:"
	@echo "  make build           # build kernel + crates"
	@echo "  make kernel          # build only the kernel for the custom target"
	@echo "  make test            # run host-side tests"
	@echo "  make qemu            # build + boot under QEMU (with display)"
	@echo "  make qemu-headless   # build + boot under QEMU, headless"
	@echo "  make qemu-no-serial  # exercise no-UART boot fallback"
	@echo "  make fidelity        # run scripts/fidelity_check.sh"
	@echo "  make address-scan    # scan persistent fixtures"
	@echo "  make fmt             # cargo fmt --check"
	@echo "  make clippy          # cargo clippy -D warnings"
	@echo "  make gates           # run all gates sequentially (fmt/clippy/test/scan/fidelity/qemu)"
	@echo "  make repo-state      # JSON verdict from scripts/milestone_state.py"
	@echo "  make mission-preflight # repo-state then gates (the /go preflight)"
	@echo "  make clean           # cargo clean"

.PHONY: build
build:
	$(CARGO) build --workspace --exclude kernel
	$(MAKE) kernel

.PHONY: kernel
kernel:
	$(CARGO) build -p kernel \
		--target x86_64-unboundos.json \
		-Z json-target-spec \
		-Z build-std=core,alloc \
		-Z build-std-features=compiler-builtins-mem \
		-Z json-target-spec \
		--release

.PHONY: image
image: kernel
	./scripts/make_image.sh $(KERNEL) $(IMAGE)

.PHONY: qemu
qemu: image
	./scripts/qemu.sh

.PHONY: qemu-headless
qemu-headless: image
	./scripts/qemu.sh --headless

.PHONY: qemu-no-serial
qemu-no-serial: image
	./scripts/qemu.sh --headless --no-serial

.PHONY: test
test:
	$(CARGO) test --workspace --exclude kernel

.PHONY: fidelity
fidelity:
	./scripts/fidelity_check.sh

.PHONY: address-scan
address-scan:
	python3 scripts/address_scan.py tests/golden_graphs tests/golden_models

.PHONY: fmt
fmt:
	$(CARGO) fmt --check

.PHONY: clippy
clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

.PHONY: gates
gates:
	./scripts/gates.sh

.PHONY: repo-state
repo-state:
	python3 scripts/milestone_state.py

.PHONY: mission-preflight
mission-preflight:
	@$(MAKE) -s repo-state
	@$(MAKE) -s gates

.PHONY: clean
clean:
	$(CARGO) clean
	rm -f $(IMAGE) $(SERIAL_LOG)
