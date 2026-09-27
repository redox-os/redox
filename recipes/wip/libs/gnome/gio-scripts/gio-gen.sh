#!/usr/bin/env bash

# these are commands that extracted from gio meson scripts

# glib
g-ir-scanner \
--output=gir/GLib-2.0.gir \
--no-libtool \
--quiet \
--reparse-validate \
--identifier-prefix=G \
--symbol-prefix=g \
--symbol-prefix=glib \
--c-include=glib.h \
--namespace=GLib \
--nsversion=2.0 \
--library=glib-2.0 \
--library=gobject-2.0 \
--external-library \
--pkg=glib-2.0 \
--cflags-begin \
-I/usr/include/glib-2.0 \
-I/usr/lib/glib-2.0/include \
-DGLIB_COMPILATION \
-DGOBJECT_COMPILATION \
-D__G_I18N_LIB_H__ \
-DGETTEXT_PACKAGE=Dummy \
--cflags-end \
/usr/include/glib-2.0/gobject/gobject-visibility.h \
/usr/include/glib-2.0/gobject/glib-types.h \
/usr/lib/glib-2.0/include/glibconfig.h \
/usr/include/glib-2.0/glib-unix.h \
/usr/include/glib-2.0/glib/*
