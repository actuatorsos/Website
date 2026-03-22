#!/bin/bash
# ══════════════════════════════════════════════════════════════
# Actuators Platform — Digital Ocean Droplet Deploy Script
# Usage: ssh root@your-droplet 'bash -s' < deploy.sh
# ══════════════════════════════════════════════════════════════

set -e

APP_DIR="/opt/actuators"
REPO="https://github.com/actuatorsos/Website.git"
BRANCH="platform-v2-erp-complete"

echo "═══════════════════════════════════════════"
echo "  Actuators Platform — Deploy to Droplet"
echo "═══════════════════════════════════════════"

# ── 1. Install Docker if not present ──
if ! command -v docker &> /dev/null; then
    echo ">> Installing Docker..."
    curl -fsSL https://get.docker.com | sh
    systemctl enable docker
    systemctl start docker
fi

if ! command -v docker compose &> /dev/null; then
    echo ">> Installing Docker Compose plugin..."
    apt-get update && apt-get install -y docker-compose-plugin
fi

# ── 2. Clone or update repo ──
if [ -d "$APP_DIR" ]; then
    echo ">> Updating repository..."
    cd "$APP_DIR"
    git fetch origin
    git checkout "$BRANCH"
    git pull origin "$BRANCH"
else
    echo ">> Cloning repository..."
    git clone -b "$BRANCH" "$REPO" "$APP_DIR"
    cd "$APP_DIR"
fi

# ── 3. Create .env if not exists ──
if [ ! -f .env ]; then
    echo ">> Creating .env from template..."
    cp .env.example .env

    # Generate JWT secret
    JWT=$(openssl rand -hex 32)
    sed -i "s/CHANGE_ME_IN_PRODUCTION_generate_a_random_64_char_secret_here_now/$JWT/" .env

    echo ""
    echo "╔══════════════════════════════════════════╗"
    echo "║  IMPORTANT: Edit .env before continuing  ║"
    echo "║  nano $APP_DIR/.env                      ║"
    echo "║  Set: ADMIN_PASS, SURREAL_PASS           ║"
    echo "╚══════════════════════════════════════════╝"
    echo ""
    read -p "Press Enter after editing .env..."
fi

# ── 4. Build and start ──
echo ">> Building and starting containers..."
docker compose down 2>/dev/null || true
docker compose up -d --build

# ── 5. Wait for health check ──
echo ">> Waiting for application to start..."
for i in {1..30}; do
    if curl -sf http://localhost:8080/api/health > /dev/null 2>&1; then
        echo ""
        echo "═══════════════════════════════════════════"
        echo "  ✅ Actuators Platform is LIVE!"
        echo "  URL: http://$(curl -s ifconfig.me):8080"
        echo "═══════════════════════════════════════════"
        exit 0
    fi
    printf "."
    sleep 2
done

echo ""
echo "❌ Health check failed. Check logs:"
echo "   docker compose logs app"
exit 1
