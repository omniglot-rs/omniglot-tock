# Makefile for LwIP library

# OG_TOCK_BASEDIR is set the absolute path of the omniglot-tock crate:
OG_TOCK_BASEDIR ?= $(shell readlink -f "$(shell pwd)/../../omniglot-tock")

OG_BIN_NAME     := og_lwip
OG_CFLAGS       := -I$(OG_TOCK_BASEDIR)/../third-party/lwip/include -I$(OG_TOCK_BASEDIR)/../examples/lwip/config

SRCDIR          := $(OG_TOCK_BASEDIR)/../examples/lwip/c_src/
BUILDDIR        := $(OG_TOCK_BASEDIR)/../examples/lwip/build/

LWIP_SRCDIR     := $(OG_TOCK_BASEDIR)/../third-party/lwip
LWIP_BUILDDIR   := $(OG_TOCK_BASEDIR)/../examples/lwip/lwip-build/

LWIP_CSRC       := \
    $(LWIP_SRCDIR)/core/def.c \
    $(LWIP_SRCDIR)/core/inet_chksum.c \
    $(LWIP_SRCDIR)/core/init.c \
    $(LWIP_SRCDIR)/core/ip.c \
    $(LWIP_SRCDIR)/core/ipv4/acd.c \
    $(LWIP_SRCDIR)/core/ipv4/autoip.c \
    $(LWIP_SRCDIR)/core/ipv4/dhcp.c \
    $(LWIP_SRCDIR)/core/ipv4/etharp.c \
    $(LWIP_SRCDIR)/core/ipv4/icmp.c \
    $(LWIP_SRCDIR)/core/ipv4/ip4.c \
    $(LWIP_SRCDIR)/core/ipv4/ip4_addr.c \
    $(LWIP_SRCDIR)/core/ipv4/ip4_frag.c \
    $(LWIP_SRCDIR)/core/ipv6/icmp6.c \
    $(LWIP_SRCDIR)/core/ipv6/ip6.c \
    $(LWIP_SRCDIR)/core/ipv6/ip6_addr.c \
    $(LWIP_SRCDIR)/core/ipv6/ip6_frag.c \
    $(LWIP_SRCDIR)/core/ipv6/mld6.c \
    $(LWIP_SRCDIR)/core/ipv6/nd6.c \
    $(LWIP_SRCDIR)/core/mem.c \
    $(LWIP_SRCDIR)/core/memp.c \
    $(LWIP_SRCDIR)/core/netif.c \
    $(LWIP_SRCDIR)/core/pbuf.c \
    $(LWIP_SRCDIR)/core/raw.c \
    $(LWIP_SRCDIR)/core/tcp.c \
    $(LWIP_SRCDIR)/core/tcp_in.c \
    $(LWIP_SRCDIR)/core/tcp_out.c \
    $(LWIP_SRCDIR)/core/timeouts.c \
    $(LWIP_SRCDIR)/core/udp.c \
    $(LWIP_SRCDIR)/netif/ethernet.c
LWIP_COBJ       := $(addprefix $(LWIP_BUILDDIR)/, $(addsuffix .o, $(subst $(LWIP_SRCDIR)/,,$(LWIP_CSRC))))

OG_LINK_OBJ     := $(LWIP_BUILDDIR)/lwip.a

include $(OG_TOCK_BASEDIR)/omniglot_c_rt/OmniglotTBF.mk

$(LWIP_BUILDDIR)/%.c.o: $(LWIP_SRCDIR)/%.c*
	mkdir -p $(shell dirname "$@")
	$(CC) $(CFLAGS) -o $@ -g -O -c $<

$(LWIP_BUILDDIR)/lwip.a: $(LWIP_COBJ)
	mkdir -p $(shell dirname "$@")
	ar cr $@ $^
