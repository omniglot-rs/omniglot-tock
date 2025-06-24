# OG_TOCK_BASEDIR is set the absolute path of the omniglot-tock crate:
OG_TOCK_BASEDIR ?= $(shell readlink -f "$(shell pwd)/../../../omniglot-tock")

OG_ARCH         := rv32imac
OG_LAYOUT_LD    := $(OG_TOCK_BASEDIR)/../examples/ubench-boards/earlgrey-cw310/og_layout.ld
OG_TARGET       := earlgrey-cw310

.PHONY: all
all: combined-image

include ../../ubench/Makefile

.PHONY: $(OG_TOCK_BASEDIR)/../examples/ubench/build/earlgrey-cw310_og_ubench.tab
$(OG_TOCK_BASEDIR)/../examples/ubench/build/earlgrey-cw310_og_ubench.tab:
	make -C ../../ubench/ OG_LAYOUT_LD=$(OG_LAYOUT_LD) OG_TARGET=$(OG_TARGET) OG_ARCH=$(OG_ARCH)

# Alias, to only build the foreign library:
foreign-libs: $(OG_TOCK_BASEDIR)/../examples/ubench/build/earlgrey-cw310_og_ubench.tab

.PHONY: $(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-ubench
$(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-ubench:
	OG_BINDGEN_CFLAGS="$(CFLAGS)" cargo build --release --features="$(FEATURES)"

$(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-ubench.bin: $(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-ubench
	riscv32-none-elf-objcopy --output-target=binary $< $@
	sha256sum $@

$(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-ubench-combined.bin: $(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-ubench.bin $(OG_TOCK_BASEDIR)/../examples/ubench/build/earlgrey-cw310_og_ubench.tab
	rm -vf $@
	tockloader flash --board opentitan_earlgrey --flash-file $@ --address 0x20000000 $(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-ubench.bin
	tockloader install --board opentitan_earlgrey --flash-file $@ --app-address 0x20030000 $(OG_TOCK_BASEDIR)/../examples/ubench/build/earlgrey-cw310_og_ubench.tab

.PHONY: combined-image
combined-image: $(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-ubench-combined.bin

.PHONY: clean
clean:
	cargo clean
	make -C $(OG_TOCK_BASEDIR)/../examples/ubench/ OG_LAYOUT_LD=$(OG_LAYOUT_LD) OG_TARGET=$(OG_TARGET) OG_ARCH=$(OG_ARCH) clean
