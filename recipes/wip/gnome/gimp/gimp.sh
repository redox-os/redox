#!/usr/bin/env bash

export GTK_USE_PORTAL=0

export GDK_PIXBUF_MODULE_FILE="$HOME/.cache/gimp-gdk-pixbuf-loaders.cache"

if [ ! -f "$GDK_PIXBUF_MODULE_FILE" ]; then
    echo "Generating GdkPixbuf loaders cache..."
    mkdir -p "$HOME/.cache"
    gdk-pixbuf-query-loaders > "$GDK_PIXBUF_MODULE_FILE"
fi

exec /usr/bin/gimp-3.2 "$@"
