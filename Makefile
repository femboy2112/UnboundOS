# UnboundOS top-level Makefile. Convenience wrappers around cargo,
# scripts/, and the QEMU smoke test. The authoritative build commands
# live in CLAUDE.md §5; this file just makes the common ones one
# keystroke away.

CARGO       ?= cargo
KERNEL      := target/x86_64-unboundos/release/kernel
IMAGE       := /tmp/unboundos.img
SERIAL_LOG  := /tmp/unboundos-serial.log
FORCE_IMAGE := /tmp/unboundos-forced-fault.img
NO_SERIAL_IMAGE := /tmp/unboundos-no-serial.img
STORAGE_SMOKE_IMAGE := /tmp/unboundos-storage-smoke.img
STORAGE_FIXTURE := /tmp/unboundos-storage-sector0.bin

.PHONY: help
help:
	@echo "UnboundOS — top-level make targets:"
	@echo "  make build           # build kernel + crates"
	@echo "  make kernel          # build only the kernel for the custom target"
	@echo "  make test            # run host-side tests"
	@echo "  make qemu            # build + boot under QEMU (with display)"
	@echo "  make qemu-headless   # build + boot under QEMU, headless"
	@echo "  make qemu-interactive-smoke # boot QEMU and exercise serial shell commands"
	@echo "  make qemu-no-serial  # exercise no-UART boot fallback"
	@echo "  make qemu-storage-smoke # assert M6 raw sector read under QEMU"
	@echo "  make ui-smoke        # source-level M5 framebuffer/graph-state smoke"
	@echo "  make tokenizer-smoke # source-level M7 tokenizer smoke"
	@echo "  make toy-transformer-smoke # source-level M8 toy transformer smoke"
	@echo "  make umdl-smoke      # source-level M9 UMDL loader smoke"
	@echo "  make quantized-smoke # source-level M10 quantized inference smoke"
	@echo "  make assistant-smoke # source-level M11 assistant explanation smoke"
	@echo "  make qemu-fault-de   # assert divide-by-zero SSOD path"
	@echo "  make qemu-fault-ud   # assert invalid-opcode SSOD path"
	@echo "  make qemu-fault-pf   # assert page-fault SSOD path"
	@echo "  make qemu-m2-dump    # assert M2 memory/arena diagnostic dump"
	@echo "  make qemu-graph-boot # assert initial graph loads during boot"
	@echo "  make qemu-framebuffer-smoke # capture framebuffer diagnostics from QEMU"
	@echo "  make qemu-stress     # repeat live QEMU milestone smoke paths"
	@echo "  make qemu-matrix     # live QEMU CPU/RAM profile matrix"
	@echo "  make fidelity        # run scripts/fidelity_check.sh"
	@echo "  make address-scan    # scan persistent fixtures"
	@echo "  make fmt             # cargo fmt --check"
	@echo "  make clippy          # cargo clippy -D warnings"
	@echo "  make gates           # run spec gates sequentially (build/test/static/smoke/qemu)"
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
	./scripts/qemu.sh --headless --assert-heartbeat

.PHONY: qemu-interactive-smoke
qemu-interactive-smoke:
	python3 scripts/check_qemu_interactive.py

.PHONY: qemu-no-serial
qemu-no-serial:
	UNBOUNDOS_QEMU_EXIT_ON_BOOT_OK=1 $(MAKE) image IMAGE=$(NO_SERIAL_IMAGE)
	./scripts/qemu.sh --headless --no-serial --image $(NO_SERIAL_IMAGE)

.PHONY: qemu-m2-dump
qemu-m2-dump: image
	./scripts/qemu.sh --headless --assert-m2-dump

.PHONY: qemu-graph-boot
qemu-graph-boot: image
	./scripts/qemu.sh --headless --assert-graph-boot

.PHONY: qemu-framebuffer-smoke
qemu-framebuffer-smoke:
	python3 scripts/check_qemu_framebuffer.py

.PHONY: qemu-stress
qemu-stress:
	python3 scripts/check_qemu_stress.py

.PHONY: qemu-matrix
qemu-matrix:
	python3 scripts/check_qemu_matrix.py

.PHONY: qemu-storage-smoke
qemu-storage-smoke:
	python3 scripts/make_storage_fixture.py $(STORAGE_FIXTURE)
	UNBOUNDOS_STORAGE_SMOKE=1 $(MAKE) image IMAGE=$(STORAGE_SMOKE_IMAGE)
	./scripts/qemu.sh --headless --image $(STORAGE_SMOKE_IMAGE) --storage-image $(STORAGE_FIXTURE) --assert-storage-marker

.PHONY: qemu-fault-de
qemu-fault-de:
	UNBOUNDOS_FORCE_FAULT=divide_error $(MAKE) image IMAGE=$(FORCE_IMAGE)
	./scripts/qemu.sh --headless --image $(FORCE_IMAGE) --assert-ssod divide_error

.PHONY: qemu-fault-ud
qemu-fault-ud:
	UNBOUNDOS_FORCE_FAULT=invalid_opcode $(MAKE) image IMAGE=$(FORCE_IMAGE)
	./scripts/qemu.sh --headless --image $(FORCE_IMAGE) --assert-ssod invalid_opcode

.PHONY: qemu-fault-pf
qemu-fault-pf:
	UNBOUNDOS_FORCE_FAULT=page_fault $(MAKE) image IMAGE=$(FORCE_IMAGE)
	./scripts/qemu.sh --headless --image $(FORCE_IMAGE) --assert-ssod page_fault

.PHONY: test
test:
	$(CARGO) test --workspace --exclude kernel

.PHONY: fidelity
fidelity:
	./scripts/fidelity_check.sh

.PHONY: ui-smoke
ui-smoke:
	python3 scripts/check_ui_smoke.py

.PHONY: tokenizer-smoke
tokenizer-smoke:
	python3 scripts/check_tokenizer_smoke.py

.PHONY: toy-transformer-smoke
toy-transformer-smoke:
	python3 scripts/check_toy_transformer_smoke.py

.PHONY: umdl-smoke
umdl-smoke:
	python3 scripts/check_umdl_smoke.py

.PHONY: quantized-smoke
quantized-smoke:
	python3 scripts/check_quantized_smoke.py

.PHONY: assistant-smoke
assistant-smoke:
	python3 scripts/check_assistant_smoke.py

.PHONY: retrieval-smoke
retrieval-smoke:
	python3 scripts/check_retrieval_smoke.py

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
