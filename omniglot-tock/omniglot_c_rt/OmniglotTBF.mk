include $(OG_TOCK_BASEDIR)/omniglot_c_rt/Configuration.mk

SRCDIR    ?= .

# From: https://stackoverflow.com/a/18258352
rwildcard = $(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))

CSRC      ?= $(call rwildcard,$(SRCDIR),*.c)
COBJ      := $(addprefix $(BUILDDIR)/, $(addsuffix .o, $(subst $(SRCDIR)/,,$(CSRC))))
# TODO: adapt the above rwildcard approach for ASSRC too
ASSRC     := $(foreach x, $(SRCDIR), $(wildcard $(addprefix $(x)/*,.S))) $(INIT_S)
ASOBJ     := $(addprefix $(BUILDDIR)/, $(addsuffix .S.o, $(notdir $(basename $(ASSRC)))))

.PHONY: all
all: $(BUILDDIR)/$(OG_TARGET)_$(OG_BIN_NAME).tab

.PHONY: clean
clean:
	rm -rf build

$(BUILDDIR)/%.c.o: $(SRCDIR)/%.c*
	mkdir -p $(shell dirname "$@")
	$(CC) $(CFLAGS) -o $@ -g -O -c $<

$(BUILDDIR)/sys.o: $(OG_TOCK_BASEDIR)/omniglot_c_rt/sys.c | $(BUILDDIR)
	mkdir -p $(BUILDDIR)
	$(CC) $(CFLAGS) -o $@ -g -O -c $<

$(BUILDDIR)/init_riscv32.S.o: $(INIT_RV32I_S)
	mkdir -p $(BUILDDIR)
	$(AS) $(ASFLAGS) -o $@ -g -c $<

$(BUILDDIR)/%.S.o: %.S*
	mkdir -p $(BUILDDIR)
	$(AS) $(ASFLAGS) -o $@ -g -c $<

$(BUILDDIR)/$(OG_TARGET)_$(OG_BIN_NAME).elf: \
    $(COBJ) $(ASOBJ) $(BUILDDIR)/sys.o \
    $(OG_SYSTEM_LIBS) \
    $(OG_LAYOUT_LD) \
    $(OG_TOCK_BASEDIR)/omniglot_c_rt/omniglot_layout.ld \
    $(OG_LINK_OBJ)
	mkdir -p $(BUILDDIR)
	OG_TOCK_BASEDIR=$(OG_TOCK_BASEDIR) envsubst '$$OG_TOCK_BASEDIR' \
	  < $(OG_LAYOUT_LD) > $(BUILDDIR)/omniglot_layout.ld
	$(LD) --no-relax -o $@ $(COBJ) $(ASOBJ) $(BUILDDIR)/sys.o $(OG_LINK_OBJ) $(OG_SYSTEM_LIBS) -T$(BUILDDIR)/omniglot_layout.ld $(LDFLAGS)

$(BUILDDIR)/$(OG_TARGET)_$(OG_BIN_NAME).tab: $(BUILDDIR)/$(OG_TARGET)_$(OG_BIN_NAME).elf
	mkdir -p $(BUILDDIR)
	elf2tab --verbose --disable -o $@ -n $(OG_BIN_NAME) $<,$(ARCH)
