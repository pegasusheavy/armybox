#!/system/bin/sh
# This script runs during post-fs-data mode
# It runs before Zygote is started

MODDIR=${0%/*}

# Nothing needed here for armybox
# The symlinks are created during installation
