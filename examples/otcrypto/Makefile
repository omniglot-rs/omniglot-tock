# Makefile for EF demo binary

# OG_TOCK_BASEDIR is set the absolute path of the omniglot-tock crate:
OG_TOCK_BASEDIR ?= $(shell readlink -f "$(shell pwd)/../../omniglot-tock")

OG_BIN_NAME     := ogotcrypto
OG_CFLAGS       := -I$(OG_TOCK_BASEDIR)/../third-party/opentitan-cryptolib/include
OG_LINK_OBJ     := $(OG_TOCK_BASEDIR)/../third-party/opentitan-cryptolib/libotcrypto.a

SRCDIR          := $(OG_TOCK_BASEDIR)/../examples/otcrypto/c_src/
BUILDDIR        := $(OG_TOCK_BASEDIR)/../examples/otcrypto/build/

include $(OG_TOCK_BASEDIR)/omniglot_c_rt/OmniglotTBF.mk

$(OG_TOCK_BASEDIR)/../third-party/opentitan-cryptolib.tar.xz:
	wget -O"$(OG_TOCK_BASEDIR)/../third-party/opentitan-cryptolib.tar.xz" \
	  https://alpha.mirror.svc.schuermann.io/files/omniglot/2024-12-03_opentitan_cryptolib_earlgrey_es_sival_bbbd9b0d00_patched.tar.xz
	cd "$(OG_TOCK_BASEDIR)/../third-party"; \
	  echo '95a955f9037850489d77b80afd97bf4338225bd7197e12f47dbf29632eafbf97  opentitan-cryptolib.tar.xz' | sha256sum -c

$(OG_TOCK_BASEDIR)/../third-party/opentitan-cryptolib/libotcrypto.a: $(OG_TOCK_BASEDIR)/../third-party/opentitan-cryptolib.tar.xz
	rm -rvf "$(OG_TOCK_BASEDIR)/../third-party/opentitan-cryptolib"
	mkdir -p "$(OG_TOCK_BASEDIR)/../third-party/opentitan-cryptolib"
	tar -xvf "$(OG_TOCK_BASEDIR)/../third-party/opentitan-cryptolib.tar.xz" \
	  -C "$(OG_TOCK_BASEDIR)/../third-party/opentitan-cryptolib" \
	  --strip-components=1 crypto/
