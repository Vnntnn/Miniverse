#!/bin/bash

echo "=========================================="
echo "  Starting Miniverse MQTT System"
echo "=========================================="

pkill -f mosquitto 2>/dev/null || true
pkill -f miniverse-backend 2>/dev/null || true
pkill -f "astro dev" 2>/dev/null || true

sleep 1

# Auto-detect mosquitto location
MOSQUITTO=$(find /usr/local/Cellar/mosquitto -name mosquitto -type f 2>/dev/null | head -n 1)

if [ -z "$MOSQUITTO" ]; then
    MOSQUITTO=$(find /opt/homebrew -name mosquitto -type f 2>/dev/null | head -n 1)
fi

if [ -z "$MOSQUITTO" ]; then
    echo "Error: Cannot find mosquitto"
    echo "Install: brew install mosquitto"
    exit 1
fi

echo "[1/3] Starting MQTT Broker..."
echo "Using: $MOSQUITTO"
$MOSQUITTO -c mqtt_broker/config/mosquitto.conf &
MQTT_PID=$!
sleep 2

echo "[2/3] Starting Backend..."
cd backend
cargo run --release 2>&1 | grep -v "Compiling\|Finished" &
BACKEND_PID=$!
cd ..
sleep 3

echo "[3/3] Starting Frontend..."
cd frontend
npm install --silent
npm run dev &
FRONTEND_PID=$!
cd ..

echo ""
echo "=========================================="
echo "  Services Running"
echo "=========================================="
echo "MQTT:     localhost:1883"
echo "Backend:  http://localhost:8080"
echo "Frontend: http://localhost:4321"
echo ""
echo "Press Ctrl+C to stop"
echo "=========================================="

trap "echo 'Stopping...'; kill $MQTT_PID $BACKEND_PID $FRONTEND_PID 2>/dev/null; exit" INT TERM
wait
