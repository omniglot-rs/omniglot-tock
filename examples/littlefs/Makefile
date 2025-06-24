# Makefile for EF demo binary

# OG_TOCK_BASEDIR is set the absolute path of the omniglot-tock crate:
OG_TOCK_BASEDIR ?= $(shell readlink -f "$(shell pwd)/../../omniglot-tock")

OG_BIN_NAME     := og_littlefs
OG_CFLAGS       := -I$(shell readlink -f $(OG_TOCK_BASEDIR)/../third-party/littlefs)
OG_LINK_OBJ     := $(OG_TOCK_BASEDIR)/../third-party/littlefs/build-riscv32/liblfs.a
SRCDIR          := $(OG_TOCK_BASEDIR)/../examples/littlefs/c_src/
BUILDDIR        := $(OG_TOCK_BASEDIR)/../examples/littlefs/build/

include $(OG_TOCK_BASEDIR)/omniglot_c_rt/OmniglotTBF.mk

.PHONY: $(OG_TOCK_BASEDIR)/../third-party/littlefs/build-riscv32/liblfs.a
$(OG_TOCK_BASEDIR)/../third-party/littlefs/build-riscv32/liblfs.a:
	mkdir -p $(OG_TOCK_BASEDIR)/../third-party/littlefs/build-riscv32/
	make -C $(OG_TOCK_BASEDIR)/../third-party/littlefs/

