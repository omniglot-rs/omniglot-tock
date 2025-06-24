# OG_TOCK_BASEDIR is set the absolute path of the omniglot-tock crate:
OG_TOCK_BASEDIR ?= $(shell readlink -f "$(shell pwd)/../../../omniglot-tock")

OG_ARCH         := rv32imac
OG_LAYOUT_LD    := $(OG_TOCK_BASEDIR)/../examples/otcrypto-boards/earlgrey-cw310/og_layout.ld
OG_TARGET       := earlgrey-cw310

.PHONY: all
all: combined-image

include ../../otcrypto/Makefile

.PHONY: $(OG_TOCK_BASEDIR)/../examples/otcrypto/build/earlgrey-cw310_ogotcrypto.tab
$(OG_TOCK_BASEDIR)/../examples/otcrypto/build/earlgrey-cw310_ogotcrypto.tab:
	make -C ../../otcrypto/ OG_LAYOUT_LD=$(OG_LAYOUT_LD) OG_TARGET=$(OG_TARGET) OG_ARCH=$(OG_ARCH)

# Alias, to only build the foreign library:
foreign-libs: $(OG_TOCK_BASEDIR)/../examples/otcrypto/build/earlgrey-cw310_ogotcrypto.tab

.PHONY: $(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310
$(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310: $(OG_TOCK_BASEDIR)/../third-party/opentitan-cryptolib/libotcrypto.a
	OG_BINDGEN_CFLAGS="$(CFLAGS)" cargo build --release --features "$(FEATURES)"

$(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310.bin: $(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310
	riscv32-none-elf-objcopy --output-target=binary $< $@
	sha256sum $@

$(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-combined.bin: $(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310.bin $(OG_TOCK_BASEDIR)/../examples/otcrypto/build/earlgrey-cw310_ogotcrypto.tab
	rm -vf $@
	tockloader flash --board opentitan_earlgrey --flash-file $@ --address 0x20000000 $(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310.bin
	tockloader install --board opentitan_earlgrey --flash-file $@ --app-address 0x20030000 $(OG_TOCK_BASEDIR)/../examples/otcrypto/build/earlgrey-cw310_ogotcrypto.tab

.PHONY: combined-image
combined-image: $(OG_TOCK_BASEDIR)/../target/riscv32imc-unknown-none-elf/release/omniglot-earlgrey-cw310-combined.bin

.PHONY: clean
clean:
	cargo clean
	make -C $(OG_TOCK_BASEDIR)/../examples/otcrypto/ OG_LAYOUT_LD=$(OG_LAYOUT_LD) OG_TARGET=$(OG_TARGET) OG_ARCH=$(OG_ARCH) clean
