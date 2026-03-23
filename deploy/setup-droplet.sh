#!/bin/bash
# ══════════════════════════════════════════════════════════════
# Actuators Platform — Droplet Setup Script
# Run as root on Ubuntu 18.04+ Droplet
# Usage: bash setup-droplet.sh
# ══════════════════════════════════════════════════════════════
set -e

DOMAIN="actuators.me"
APP_USER="actuators"
APP_DIR="/opt/actuators"
DB_USER="Actuators"
DB_PASS="Actuators123"
DB_NS="Actuators"
DB_NAME="erp"
ADMIN_EMAIL="actuators.os@gmail.com"
ADMIN_PASS="Actuators_2026"
JWT_SECRET="k8m2p5v9x3q7w1j6n0r4t8y2u5a9c3f7h1l4o8s2d6g0i3e7b1n5q9w3x7z0m4p"

echo "══════════════════════════════════════════"
echo "  Actuators Platform — Server Setup"
echo "══════════════════════════════════════════"

# ── 1. System Update ──
echo ""
echo "▸ Updating system packages..."
apt-get update -qq
apt-get upgrade -y -qq

# ── 2. Install Dependencies ──
echo "▸ Installing dependencies..."
apt-get install -y -qq curl wget git build-essential pkg-config libssl-dev nginx certbot python3-certbot-nginx

# ── 3. Install Rust ──
if ! command -v rustc &> /dev/null; then
    echo "▸ Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "▸ Rust already installed: $(rustc --version)"
    source "$HOME/.cargo/env" 2>/dev/null || true
fi

# ── 4. Install SurrealDB v2.4.0 ──
if ! command -v surreal &> /dev/null; then
    echo "▸ Installing SurrealDB v2.4.0..."
    curl -sSf https://install.surrealdb.com | sh -s -- --version v2.4.0
else
    echo "▸ SurrealDB already installed: $(surreal version)"
fi

# ── 5. Create App User & Directory ──
echo "▸ Setting up application directory..."
mkdir -p $APP_DIR
mkdir -p /var/lib/surrealdb

# ── 6. Clone & Build ──
echo "▸ Cloning repository..."
if [ -d "$APP_DIR/repo" ]; then
    cd $APP_DIR/repo
    git fetch origin
    git checkout platform-v2-erp-complete
    git pull origin platform-v2-erp-complete
else
    git clone -b platform-v2-erp-complete https://github.com/actuatorsos/Website.git $APP_DIR/repo
    cd $APP_DIR/repo
fi

echo "▸ Building Actuators (this may take 5-10 minutes)..."
cargo build --release --bin Actuators 2>&1 | tail -5

# Copy binary and assets
cp target/release/Actuators /usr/local/bin/Actuators
mkdir -p $APP_DIR/{static,templates,locales,data,src/db}
cp -r static/* $APP_DIR/static/ 2>/dev/null || true
cp -r templates/* $APP_DIR/templates/
cp -r locales/* $APP_DIR/locales/
cp src/db/schema.surql $APP_DIR/src/db/schema.surql
cp -r data/* $APP_DIR/data/ 2>/dev/null || true
mkdir -p $APP_DIR/static/uploads/{videos,courses}

echo "▸ Binary installed: $(Actuators --version 2>/dev/null || echo 'OK')"

# ── 7. Create Environment File ──
cat > $APP_DIR/.env << ENVEOF
SURREAL_URL=ws://127.0.0.1:8000
SURREAL_USER=$DB_USER
SURREAL_PASS=$DB_PASS
SURREAL_NS=$DB_NS
SURREAL_DB=$DB_NAME
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
ADMIN_USER=$ADMIN_EMAIL
ADMIN_PASS=$ADMIN_PASS
JWT_SECRET=$JWT_SECRET
JWT_EXPIRY_HOURS=24
RUST_LOG=info
OPENROUTER_API_KEY=${OPENROUTER_API_KEY:-}
ENVEOF
chmod 600 $APP_DIR/.env

# ── 8. SurrealDB Systemd Service ──
cat > /etc/systemd/system/surrealdb.service << 'SVCEOF'
[Unit]
Description=SurrealDB Database
After=network.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/root/.surrealdb/surreal start --user Actuators --pass Actuators123 --bind 127.0.0.1:8000 file:/var/lib/surrealdb
Restart=always
RestartSec=5
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
SVCEOF

# ── 9. Actuators Systemd Service ──
cat > /etc/systemd/system/actuators.service << SVCEOF
[Unit]
Description=Actuators Platform
After=surrealdb.service
Requires=surrealdb.service

[Service]
Type=simple
WorkingDirectory=$APP_DIR
EnvironmentFile=$APP_DIR/.env
ExecStart=/usr/local/bin/Actuators
Restart=always
RestartSec=5
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
SVCEOF

# ── 10. Nginx Configuration ──
cat > /etc/nginx/sites-available/actuators << NGXEOF
server {
    listen 80;
    server_name $DOMAIN www.$DOMAIN;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_cache_bypass \$http_upgrade;
        proxy_read_timeout 300s;
        proxy_send_timeout 300s;
        client_max_body_size 50M;
    }

    location /static/ {
        alias $APP_DIR/static/;
        expires 30d;
        add_header Cache-Control "public, immutable";
    }
}
NGXEOF

ln -sf /etc/nginx/sites-available/actuators /etc/nginx/sites-enabled/actuators
rm -f /etc/nginx/sites-enabled/default 2>/dev/null || true
nginx -t

# ── 11. Start Services ──
echo "▸ Starting services..."
systemctl daemon-reload
systemctl enable surrealdb actuators
systemctl restart surrealdb
sleep 5
systemctl restart actuators
sleep 5
systemctl restart nginx

# ── 12. Verify ──
echo ""
echo "══════════════════════════════════════════"
echo "  Checking Services"
echo "══════════════════════════════════════════"
echo "SurrealDB: $(systemctl is-active surrealdb)"
echo "Actuators: $(systemctl is-active actuators)"
echo "Nginx:     $(systemctl is-active nginx)"
echo ""

# Health check
sleep 3
if curl -sf http://127.0.0.1:3000/api/health > /dev/null 2>&1; then
    echo "✓ Actuators is running!"
else
    echo "✗ Actuators not responding yet. Check: journalctl -u actuators -f"
fi

echo ""
echo "══════════════════════════════════════════"
echo "  Setup Complete!"
echo "══════════════════════════════════════════"
echo ""
echo "  Next steps:"
echo "  1. Verify DNS: dig +short $DOMAIN A"
echo "  2. SSL cert:   certbot --nginx -d $DOMAIN -d www.$DOMAIN"
echo "  3. Test:       curl http://$DOMAIN/api/health"
echo "  4. Login:      https://$DOMAIN/admin/login?lang=ar"
echo ""
echo "  Credentials:"
echo "  Email:    $ADMIN_EMAIL"
echo "  Password: $ADMIN_PASS"
echo ""
echo "  Useful commands:"
echo "  journalctl -u actuators -f    # App logs"
echo "  journalctl -u surrealdb -f    # DB logs"
echo "  systemctl restart actuators   # Restart app"
echo ""
