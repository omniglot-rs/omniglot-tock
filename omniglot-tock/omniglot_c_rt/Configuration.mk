# ---------- BASE SETUP --------------------------------------------------------

# Check that required variables are set:
ifeq (,$(OG_TOCK_BASEDIR))
  $(error Requires OG_TOCK_BASEDIR to be set)
endif

ifeq (,$(OG_BIN_NAME))
  $(error Requires OG_BIN_NAME to be set)
endif

ifeq (,$(OG_LAYOUT_LD))
  $(error Requires OG_LAYOUT_LD to be set)
endif

ifeq (,$(OG_TARGET))
  $(error Requires OG_TARGET to be set)
endif


# Set defaults:
BUILDDIR ?= build

# ---------- PRECOMPILED LIBTOCK-C LIBRARIES -----------------------------------

TOCK_USERLAND_BASE_DIR = $(OG_TOCK_BASEDIR)/../third-party/libtock-c
include $(TOCK_USERLAND_BASE_DIR)/Precompiled.mk

# ---------- TOOLCHAIN DISCOVERY -----------------------------------------------

# We don't want the environment to pollute our toolchain selection,
# this will almost always result in the wrong compiler being used:
undefine CC
undefine CXX
undefine AS
undefine LD

# RISC-V toolchains, irrespective of their name-tuple, can compile for
# essentially any target. Thus, try a few known names and choose the
# one for which a compiler is found.
ifneq (,$(shell which riscv64-none-elf-gcc 2>/dev/null))
  TOOLCHAIN_rv32i := riscv64-none-elf-
else ifneq (,$(shell which riscv32-none-elf-gcc 2>/dev/null))
  TOOLCHAIN_rv32i := riscv32-none-elf-
else ifneq (,$(shell which riscv64-elf-gcc 2>/dev/null))
  TOOLCHAIN_rv32i := riscv64-elf-
else ifneq (,$(shell which riscv64-unknown-elf-clang 2>/dev/null))
  TOOLCHAIN_rv32i := riscv64-unknown-elf-
else ifneq (,$(shell which riscv32-unknown-elf-clang 2>/dev/null))
  TOOLCHAIN_rv32i := riscv32-unknown-elf-
else ifneq (,$(shell which clang 2>/dev/null))
  # Assume that this clang build has support for RISC-V
  TOOLCHAIN_rv32i := llvm-
  CC              := clang -target riscv32
  CXX             := clang -target riscv32
  AS              := clang -target riscv32
  LD              := ld.lld
else
  $(warning Failed to find a suitable RISC-V toolchain.)
  # Fall back onto a non-existant binary, in case we build for ARM:
  TOOLCHAIN_rv32i := riscv32-unknown-elf-
endif

# For Cortex-M, we don't have too many options...
TOOLCHAIN_cortexm = arm-none-eabi-

# ---------- TARGET TOOLCHAIN SELECTION ----------------------------------------

# OG target "compression", for when compilers support multiple archs.
ifeq ($(OG_ARCH),rv32i)
  OG_ARCH_FAMILY := rv32i
  OG_RV32I_MARCH := rv32i
  NEWLIB_INC     := riscv/riscv64-unknown-elf/include
  NEWLIB_TARGET  := riscv/riscv64-unknown-elf/lib/rv32i/ilp32
  LIBCPP_INC     := riscv/riscv64-unknown-elf/include
  LIBCPP_TARGET  := riscv/riscv64-unknown-elf/lib/rv32i/ilp32
  LIBGCC_TARGET_PREFIX := riscv64-unknown-elf
  LIBGCC_TARGET_SUFFIX := rv32i/ilp32
else ifeq ($(OG_ARCH),rv32imc)
  OG_ARCH_FAMILY := rv32i
  OG_RV32I_MARCH := rv32imc
  NEWLIB_INC     := riscv/riscv64-unknown-elf/include
  # TODO: we don't have an imc version of this library?
  NEWLIB_TARGET  := riscv/riscv64-unknown-elf/lib/rv32im/ilp32
  LIBCPP_INC     := riscv/riscv64-unknown-elf/include
  LIBCPP_TARGET  := riscv/riscv64-unknown-elf/lib/rv32im/ilp32
  LIBGCC_TARGET_PREFIX := riscv64-unknown-elf
  LIBGCC_TARGET_SUFFIX := rv32im/ilp32
else ifeq ($(OG_ARCH),rv32imac)
  OG_ARCH_FAMILY := rv32i
  OG_RV32I_MARCH := rv32imac
  NEWLIB_INC     := riscv/riscv64-unknown-elf/include
  NEWLIB_TARGET  := riscv/riscv64-unknown-elf/lib/rv32imac/ilp32
  LIBCPP_INC     := riscv/riscv64-unknown-elf/include
  LIBCPP_TARGET  := riscv/riscv64-unknown-elf/lib/rv32imac/ilp32
  LIBGCC_TARGET_PREFIX := riscv64-unknown-elf
  LIBGCC_TARGET_SUFFIX := rv32imac/ilp32
else ifeq ($(OG_ARCH),cortexm4)
  # Nothing to set.
else
  $(error Unknown OG_ARCH)
endif

ifeq ($(OG_ARCH_FAMILY),rv32i)
  CC              ?= $(TOOLCHAIN_rv32i)gcc
  CXX             ?= $(TOOLCHAIN_rv32i)g++
  AS              ?= $(TOOLCHAIN_rv32i)as
  LD              ?= $(TOOLCHAIN_rv32i)ld

  # Determine the version of the RISC-V compiler. This is used to select the
  # version of the libgcc library that is compatible.
  CC_rv32_version := $(shell $(CC) -dumpfullversion)
  CC_rv32_version_major := $(shell echo $(CC_rv32_version) | cut -f1 -d.)

  # Match compiler version to support libtock-newlib versions.
  #
  # Keep in sync with the libtock-c submodule:
  ifeq ($(CC_rv32_version_major),10)
    NEWLIB_VERSION_rv32 := 4.2.0.20211231
  else ifeq ($(CC_rv32_version_major),11)
    NEWLIB_VERSION_rv32 := 4.2.0.20211231
  else ifeq ($(CC_rv32_version_major),12)
    NEWLIB_VERSION_rv32 := 4.3.0.20230120
  else ifeq ($(CC_rv32_version_major),13)
    NEWLIB_VERSION_rv32 := 4.3.0.20230120
  else ifeq ($(CC_rv32_version_major),14)
    NEWLIB_VERSION_rv32 := 4.4.0.20231231
  else
    NEWLIB_VERSION_rv32 := 4.4.0.20231231
  endif
  NEWLIB_BASE_DIR := $(TOCK_USERLAND_BASE_DIR)/lib/libtock-newlib-$(NEWLIB_VERSION_rv32)

  # Match compiler version to supported libtock-libc++ versions.
  #
  # Keep in sync with the libtock-c submodule:
  ifeq ($(CC_rv32_version_major),10)
    LIBCPP_VERSION_rv32 := 10.5.0
  else ifeq ($(CC_rv32_version_major),11)
    LIBCPP_VERSION_rv32 := 10.5.0
  else ifeq ($(CC_rv32_version_major),12)
    LIBCPP_VERSION_rv32 := 12.3.0
  else ifeq ($(CC_rv32_version_major),13)
    LIBCPP_VERSION_rv32 := 13.2.0
  else ifeq ($(CC_rv32_version_major),14)
    LIBCPP_VERSION_rv32 := 14.1.0
  else
    LIBCPP_VERSION_rv32 := 14.1.0
  endif
  LIBCPP_BASE_DIR := $(TOCK_USERLAND_BASE_DIR)/lib/libtock-libc++-$(LIBCPP_VERSION_rv32)

  ARCH            := rv32imc

  CFLAGS          := \
    -march=$(OG_RV32I_MARCH) -mabi=ilp32 -mcmodel=medlow \
    -std=c99 -nodefaultlibs -nostdlib -ffreestanding \
    -isystem=$(NEWLIB_BASE_DIR)/$(NEWLIB_INC) \
    -isystem $(LIBCPP_BASE_DIR)/$(LIBCPP_INC)/c++/$(LIBCPP_VERSION_rv32) \
    -isystem $(LIBCPP_BASE_DIR)/$(LIBCPP_INC)/c++/$(LIBCPP_VERSION_rv32)/riscv64-unknown-elf \
    -isystem $(LIBCPP_BASE_DIR)/riscv/riscv64-unknown-elf/sys-include \
    $(OG_CFLAGS)
  ASFLAGS         := -march=$(OG_RV32I_MARCH) -mabi=ilp32
  CXXFLAGS        := -nostdinc++ $(CFLAGS)
  LDFLAGS         := -melf32lriscv
  INIT_RV32I_S    := $(OG_TOCK_BASEDIR)/omniglot_c_rt/init_riscv32.S
  INIT_S          := $(INIT_RV32I_S)

  OG_SYSTEM_LIBS     := \
    $(NEWLIB_BASE_DIR)/$(NEWLIB_TARGET)/libc.a \
    $(NEWLIB_BASE_DIR)/$(NEWLIB_TARGET)/libm.a \
    $(LIBCPP_BASE_DIR)/$(LIBCPP_TARGET)/libstdc++.a \
    $(LIBCPP_BASE_DIR)/$(LIBCPP_TARGET)/libsupc++.a \
    $(LIBCPP_BASE_DIR)/riscv/lib/gcc/$(LIBGCC_TARGET_PREFIX)/$(LIBCPP_VERSION_rv32)/$(LIBGCC_TARGET_SUFFIX)/libgcc.a
else ifeq ($(OG_ARCH),cortexm4)
  CC              ?= $(TOOLCHAIN_cortexm)gcc
  CXX             ?= $(TOOLCHAIN_cortexm)g++
  AS              ?= $(TOOLCHAIN_cortexm)as
  LD              ?= $(TOOLCHAIN_cortexm)ld

  ARCH            := cortex-m4
  CFLAGS          := -std=gnu11
  ASFLAGS         := -mthumb
  CXXFLAGS        := $(CFLAGS)
  LDFLAGS         :=
  INIT_CORTEXM_S  := $(OG_TOCK_BASEDIR)/omniglot_c_rt/init_cortexm.S
  INIT_S          := $(INIT_CORTEXM_S)
endif
