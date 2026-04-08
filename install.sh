#!/bin/bash
# install.sh — build and install the Rust Network Monitor Plasma applet
set -e

APPLET_ID="org.kde.plasma.rustyapplet"
PLUGIN_URI="org/kde/plasma/rustyapplet"
QML_USER_DIR="$HOME/.local/lib/qt6/qml"
PLUGIN_DIR="$QML_USER_DIR/$PLUGIN_URI"
APPLET_DIR="$HOME/.local/share/plasma/plasmoids/$APPLET_ID"
ENV_DIR="$HOME/.config/plasma-workspace/env"

# Parse flags
RELEASE=0
RESTART=0
for arg in "$@"; do
    case $arg in
        --release) RELEASE=1 ;;
        --restart) RESTART=1 ;;
    esac
done

# Build
if [ "$RELEASE" -eq 1 ]; then
    echo "Building (release)..."
    cargo build --release
    SO="target/release/libplasma_applet_rs.so"
else
    echo "Building (debug)..."
    cargo build
    SO="target/debug/libplasma_applet_rs.so"
fi

# Install QML plugin
echo "Installing QML plugin → $PLUGIN_DIR"
mkdir -p "$PLUGIN_DIR"
cp "$SO" "$PLUGIN_DIR/libplasma_applet_rs.so"

cat > "$PLUGIN_DIR/qmldir" <<EOF
module org.kde.plasma.rustyapplet
plugin plasma_applet_rs
EOF

# Ensure plasmashell finds the plugin by adding QML_IMPORT_PATH to its env
mkdir -p "$ENV_DIR"
ENV_FILE="$ENV_DIR/qml_import_path.sh"
if ! grep -q "$QML_USER_DIR" "$ENV_FILE" 2>/dev/null; then
    echo "Adding $QML_USER_DIR to plasmashell QML path → $ENV_FILE"
    echo "export QML_IMPORT_PATH=\"$QML_USER_DIR:\$QML_IMPORT_PATH\"" >> "$ENV_FILE"
    echo "export QML2_IMPORT_PATH=\"$QML_USER_DIR:\$QML2_IMPORT_PATH\"" >> "$ENV_FILE"
fi

# Install Plasma package
echo "Installing Plasma package → $APPLET_DIR"
if command -v kpackagetool6 &>/dev/null; then
    kpackagetool6 -t Plasma/Applet --install package/ 2>/dev/null \
        || kpackagetool6 -t Plasma/Applet --upgrade package/
else
    mkdir -p "$APPLET_DIR"
    cp -r package/. "$APPLET_DIR/"
fi

echo ""
echo "Done. To activate:"
echo "  1. Log out and back in (so plasmashell picks up the new QML path)"
echo "     — or restart plasmashell: kquitapp6 plasmashell && kstart6 plasmashell"
echo "  2. Right-click the panel → Add Widgets → search 'Network Monitor'"
echo "  3. Drag it to the panel"
echo ""
echo "For testing without restarting:"
echo "  QML2_IMPORT_PATH=$QML_USER_DIR plasmoidviewer -a $APPLET_ID"

if [ "$RESTART" -eq 1 ]; then
    echo ""
    echo "Restarting plasmashell..."
    # Run restart in a subshell detached from this terminal, so killing
    # plasmashell doesn't take the terminal with it.
    nohup bash -c 'kquitapp6 plasmashell; sleep 2; kstart plasmashell' \
        > /tmp/plasmashell-restart.log 2>&1 &
    echo "Plasmashell restarting in background (log: /tmp/plasmashell-restart.log)"
fi
