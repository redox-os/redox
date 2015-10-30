# Component settings
COMPONENT := parserutils
COMPONENT_VERSION := 0.2.1
# Default to a static library
COMPONENT_TYPE ?= lib-static

# Setup the tooling
PREFIX ?= /opt/netsurf
NSSHARED ?= $(PREFIX)/share/netsurf-buildsystem
include $(NSSHARED)/makefiles/Makefile.tools

TESTRUNNER := $(PERL) $(NSTESTTOOLS)/testrunner.pl

# Toolchain flags
WARNFLAGS := -Wall -W -Wundef -Wpointer-arith -Wcast-align \
	-Wwrite-strings -Wstrict-prototypes -Wmissing-prototypes \
	-Wmissing-declarations -Wnested-externs -pedantic
# BeOS/Haiku standard library headers create warnings.
#ifneq ($(BUILD),i586-pc-haiku)
#  WARNFLAGS := $(WARNFLAGS) -Werror
#endif

CFLAGS := -D_BSD_SOURCE -I$(CURDIR)/include/ \
	-I$(CURDIR)/src $(WARNFLAGS) $(CFLAGS)
ifneq ($(GCCVER),2)
  CFLAGS := $(CFLAGS) -std=c99
else
  # __inline__ is a GCCism
  CFLAGS := $(CFLAGS) -Dinline="__inline__"
endif

include $(NSBUILD)/Makefile.top

# Extra installation rules
Is := include/parserutils
I := /include/parserutils
INSTALL_ITEMS := $(INSTALL_ITEMS) $(I):$(Is)/errors.h;$(Is)/functypes.h;$(Is)/parserutils.h;$(Is)/types.h

Is := include/parserutils/charset
I := /include/parserutils/charset
INSTALL_ITEMS := $(INSTALL_ITEMS) $(I):$(Is)/codec.h;$(Is)/mibenum.h;$(Is)/utf16.h;$(Is)/utf8.h

Is := include/parserutils/input
I := /include/parserutils/input
INSTALL_ITEMS := $(INSTALL_ITEMS) $(I):$(Is)/inputstream.h

Is := include/parserutils/utils
I := /include/parserutils/utils
INSTALL_ITEMS := $(INSTALL_ITEMS) $(I):$(Is)/buffer.h;$(Is)/stack.h;$(Is)/vector.h

INSTALL_ITEMS := $(INSTALL_ITEMS) /$(LIBDIR)/pkgconfig:lib$(COMPONENT).pc.in
INSTALL_ITEMS := $(INSTALL_ITEMS) /$(LIBDIR):$(OUTPUT)
