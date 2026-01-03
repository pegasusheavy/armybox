#!/system/bin/sh
# Magisk Module Installer Script

SKIPUNZIP=1

# Extract module files
ui_print "- Extracting module files"
unzip -o "$ZIPFILE" -x 'META-INF/*' -d $MODPATH >&2

# Set permissions
ui_print "- Setting permissions"
set_perm_recursive $MODPATH 0 0 0755 0644
set_perm $MODPATH/system/bin/armybox 0 2000 0755

# Install symlinks
ui_print "- Creating applet symlinks"
APPLETS=$($MODPATH/system/bin/armybox --list 2>/dev/null)
for applet in $APPLETS; do
    # Skip if file already exists
    if [ ! -e "$MODPATH/system/bin/$applet" ]; then
        ln -sf armybox "$MODPATH/system/bin/$applet"
    fi
done

# Count installed
COUNT=$(echo "$APPLETS" | wc -w)
ui_print "- Installed $COUNT applet symlinks"

ui_print "- Done!"
