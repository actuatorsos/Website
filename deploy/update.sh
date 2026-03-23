#!/bin/bash
# ══════════════════════════════════════════════════════════════
# Actuators Platform — Quick Update Script
# Run after pushing new code to GitHub
# Usage: bash /opt/actuators/repo/deploy/update.sh
# ══════════════════════════════════════════════════════════════
set -e
source "$HOME/.cargo/env" 2>/dev/null || true

APP_DIR="/opt/actuators"

echo "▸ Pulling latest code..."
cd $APP_DIR/repo
git pull origin platform-v2-erp-complete

echo "▸ Building..."
cargo build --release --bin Actuators 2>&1 | tail -3

echo "▸ Updating files..."
cp target/release/Actuators /usr/local/bin/Actuators
cp -r static/* $APP_DIR/static/ 2>/dev/null || true
cp -r templates/* $APP_DIR/templates/
cp -r locales/* $APP_DIR/locales/
cp src/db/schema.surql $APP_DIR/src/db/schema.surql

echo "▸ Restarting..."
systemctl restart actuators
sleep 3

if curl -sf http://127.0.0.1:3000/api/health > /dev/null 2>&1; then
    echo "✓ Update complete! App is running."
else
    echo "✗ App not responding. Check: journalctl -u actuators -f"
fi
