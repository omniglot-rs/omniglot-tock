# Makefile for EF ubench TBF

# OG_TOCK_BASEDIR is set the absolute path of the omniglot-tock crate:
OG_TOCK_BASEDIR ?= $(shell readlink -f "$(shell pwd)/../../omniglot-tock")

OG_BIN_NAME     := og_ubench

SRCDIR          := $(OG_TOCK_BASEDIR)/../examples/ubench/c_src/
BUILDDIR        := $(OG_TOCK_BASEDIR)/../examples/ubench/build/

include $(OG_TOCK_BASEDIR)/omniglot_c_rt/OmniglotTBF.mk
