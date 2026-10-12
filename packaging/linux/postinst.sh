#!/bin/sh
# Aplica a regra udev instalada pelo pacote, para o mouse funcionar sem root.
if command -v udevadm >/dev/null 2>&1; then
    udevadm control --reload-rules || true
    udevadm trigger --subsystem-match=hidraw || true
fi
exit 0
