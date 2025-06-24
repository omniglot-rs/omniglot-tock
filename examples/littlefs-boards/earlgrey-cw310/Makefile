# OG_TOCK_BASEDIR is set the absolute path of the omniglot-tock crate:
OG_TOCK_BASEDIR ?= $(shell readlink -f "$(shell pwd)/../../../omniglot-tock")

OG_ARCH         := rv32imac
OG_LAYOUT_LD    := $(OG_TOCK_BASEDIR)/../examples/littlefs-boards/earlgrey-cw310/og_layout.ld
OG_TARGET       := earlgrey-cw310

.PHONY: all
all: combined-image

include ../../littlefs/Makefile

.PHONY: $(OG_TOCK_BASEDIR)/../examples/littlefs/build/earlgrey-cw310_og_littlefs.tab
$(OG_TOCK_BASEDIR)/../examples/littlefs/build/earlgrey-cw310_og_littlefs.tab:
	make -C ../../littlefs/ OG_LAYOUT_LD=$(OG_LAYOUT_LD) OG_TARGET=$(OG_TARGET) OG_ARCH=$(OG_ARCH)

# Alias, to only build the foreign library:
foreign-libs: $(OG_TOCK_BASEDIR)/../examples/littlefs/build/earlgrey-cw310_og_littlefs.tab

.PHONY: $(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-littlefs
$(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-littlefs:
	OG_BINDGEN_CFLAGS="$(CFLAGS)" cargo build --release --features="$(FEATURES)"

$(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-littlefs.bin: $(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-littlefs
	riscv32-none-elf-objcopy --output-target=binary $< $@
	sha256sum $@

$(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-littlefs-combined.bin: $(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-littlefs.bin $(OG_TOCK_BASEDIR)/../examples/littlefs/build/earlgrey-cw310_og_littlefs.tab
	rm -vf $@
	tockloader flash --board opentitan_earlgrey --flash-file $@ --address 0x20000000 $(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-littlefs.bin
	tockloader install --board opentitan_earlgrey --flash-file $@ --app-address 0x20030000 $(OG_TOCK_BASEDIR)/../examples/littlefs/build/earlgrey-cw310_og_littlefs.tab

.PHONY: combined-image
combined-image: $(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-littlefs-combined.bin

.PHONY: clean
clean:
	cargo clean
	make -C $(OG_TOCK_BASEDIR)/../examples/littlefs/ OG_LAYOUT_LD=$(OG_LAYOUT_LD) OG_TARGET=$(OG_TARGET) OG_ARCH=$(OG_ARCH) clean
