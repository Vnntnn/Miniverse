#!/bin/bash

echo "🔍 Miniverse System Health Check"
echo "=================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check MQTT
echo -n "📡 MQTT Broker (port 1883): "
if lsof -i :1883 > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Running${NC}"
else
    echo -e "${RED}✗ Not running${NC}"
    echo "   Start with: mosquitto -c mqtt_broker/config/mosquitto.conf"
fi

# Check Backend
echo -n "🦀 Rust Backend (port 8080): "
if lsof -i :8080 > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Running${NC}"
else
    echo -e "${RED}✗ Not running${NC}"
    echo "   Start with: cd backend && cargo run --release"
fi

# Check Frontend
echo -n "🌐 Frontend Dev Server (port 4321): "
if lsof -i :4321 > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Running${NC}"
else
    echo -e "${RED}✗ Not running${NC}"
    echo "   Start with: cd frontend && npm run dev"
fi

echo ""
echo "📦 Dependencies Check"
echo "--------------------"

# Backend deps
echo -n "Backend build: "
if [ -f "backend/Cargo.toml" ]; then
    cd backend
    if cargo check --quiet 2>/dev/null; then
        echo -e "${GREEN}✓ OK${NC}"
    else
        echo -e "${YELLOW}⚠ Needs rebuild${NC}"
    fi
    cd ..
else
    echo -e "${RED}✗ Missing${NC}"
fi

# Frontend deps
echo -n "Frontend deps: "
if [ -f "frontend/package.json" ]; then
    cd frontend
    if [ -d "node_modules" ]; then
        echo -e "${GREEN}✓ Installed${NC}"
    else
        echo -e "${YELLOW}⚠ Run: npm install${NC}"
    fi
    cd ..
else
    echo -e "${RED}✗ Missing${NC}"
fi

echo ""
echo "📋 Project Files"
echo "----------------"

# Check key files
files=(
    "backend/src/main.rs"
    "backend/src/serial/commands.rs"
    "backend/src/serial/bridge.rs"
    "frontend/src/client/terminal.ts"
    "frontend/src/components/Terminal.astro"
    "frontend/src/lib/websocket.ts"
    "arduino_sketch/miniverse_mqtt.ino"
    "TESTING.md"
    "PROJECT_STRUCTURE.md"
    "SUMMARY_TH.md"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo -e "${GREEN}✓${NC} $file"
    else
        echo -e "${RED}✗${NC} $file"
    fi
done

echo ""
echo "🌐 Access Points"
echo "---------------"
echo "   Frontend:  http://localhost:4321"
echo "   Backend:   http://localhost:8080"
echo "   WebSocket: ws://localhost:8080/ws"
echo "   MQTT:      localhost:1883"
echo ""

# Overall status
if lsof -i :1883 > /dev/null 2>&1 && \
   lsof -i :8080 > /dev/null 2>&1 && \
   lsof -i :4321 > /dev/null 2>&1; then
    echo -e "${GREEN}✅ All services running! System ready.${NC}"
    echo "   Open http://localhost:4321 in your browser"
else
    echo -e "${YELLOW}⚠ Some services not running${NC}"
    echo "   See instructions above to start missing services"
fi

echo ""
