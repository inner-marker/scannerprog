#!/usr/bin/env bash
# Post-process an AppImage built by `dx bundle` so it uses the host's
# WebKitGTK / JavaScriptCore / Wayland libraries instead of bundled copies.
# Usage: ./fix-appimage.sh path/to/YourApp.AppImage   (file is replaced in place)
set -euo pipefail

# Run AppImages (the input and appimagetool) without FUSE; CI runners usually lack libfuse2
export APPIMAGE_EXTRACT_AND_RUN=1

# Absolute path to the AppImage, so it still resolves after we cd elsewhere
APPIMAGE="$(realpath "${1:?usage: fix-appimage.sh path/to/App.AppImage}")"

# Cache appimagetool here so it is only downloaded once
TOOL="$HOME/.cache/appimagetool-x86_64.AppImage"
if [[ ! -x "$TOOL" ]]; then
    mkdir -p "$(dirname "$TOOL")"
    wget -q -O "$TOOL" \
        https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
    chmod +x "$TOOL"
fi

# Work in a throwaway directory; delete it however the script exits
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
cd "$WORK"

# Unpack the AppImage into ./squashfs-root
cp "$APPIMAGE" in.AppImage
chmod +x in.AppImage
./in.AppImage --appimage-extract >/dev/null

# Drop the libs that must match the host:
#  - WebKit/JSC must match the host's /usr/libexec/webkit2gtk-4.1/WebKitWebProcess
#  - libwayland-* must match the host compositor/GPU stack
#  - GTK/GLib/Cairo/Pango/etc.: the host's WebKit is now used, and it must load
#    against the host's own copies, not older ones from the build machine (e.g. Ubuntu CI)
LIB=squashfs-root/usr/lib
rm -fv "$LIB"/libwebkit2gtk-4.1.so* "$LIB"/libjavascriptcoregtk-4.1.so* \
       "$LIB"/libwayland-*.so* \
       "$LIB"/libgtk-3.so* "$LIB"/libgdk-3.so* "$LIB"/libgdk_pixbuf-2.0.so* \
       "$LIB"/libglib-2.0.so* "$LIB"/libgobject-2.0.so* "$LIB"/libgio-2.0.so* \
       "$LIB"/libgmodule-2.0.so* "$LIB"/libgthread-2.0.so* \
       "$LIB"/libcairo*.so* "$LIB"/libpango*.so* "$LIB"/libharfbuzz*.so* \
       "$LIB"/libatk*.so* "$LIB"/libepoxy.so* "$LIB"/libsoup-3.0.so* \
       "$LIB"/libgst*.so*

# Repack over the original file
ARCH=x86_64 "$TOOL" squashfs-root "$APPIMAGE"
echo "Fixed: $APPIMAGE (now requires webkit2gtk4.1 on the host)"
