# Miniverse Project - Final Structure

## Architecture Overview

```
Miniverse/
├── arduino_sketch/          # Arduino firmware (ESP8266)
│   └── miniverse_mqtt.ino   # Serial + MQTT command handler
├── backend/                 # Rust WebSocket + Serial bridge
│   ├── src/
│   │   ├── main.rs          # Entry point, MQTT listener
│   │   ├── config.rs        # Configuration structs
│   │   ├── events.rs        # Event types (SystemEvent, ClientCommand)
│   │   ├── state.rs         # Shared AppState with broadcast channel
│   │   ├── handlers/        # HTTP/WebSocket handlers
│   │   ├── mqtt/            # MQTT client manager
│   │   ├── serial/          # Serial bridge and commands
│   │   │   ├── bridge.rs    # SerialPort wrapper with board detection
│   │   │   └── commands.rs  # Serial command router
│   │   └── websocket/       # WebSocket actors
│   └── Cargo.toml
├── frontend/                # Astro + xterm.js UI
│   ├── src/
│   │   ├── client/
│   │   │   └── terminal.ts  # TerminalManager (xterm integration)
│   │   ├── components/
│   │   │   └── Terminal.astro  # Status bar + terminal container
│   │   ├── lib/
│   │   │   └── websocket.ts # WebSocketClient
│   │   └── pages/
│   │       └── index.astro  # Main page
│   ├── package.json
│   └── tsconfig.json
└── mqtt_broker/
    └── config/
        └── mosquitto.conf   # MQTT broker config
```

## Component Responsibilities

### Backend (Rust)
- **WebSocket Server** (`localhost:8080/ws`)
  - Real-time bidirectional communication
  - Broadcasts events to all connected clients
  
- **MQTT Client**
  - Connects to `localhost:1883`
  - Subscribes to `miniverse/#`
  - Publishes sensor data and status
  
- **Serial Bridge**
  - Auto-detects Arduino boards (VID:PID mapping)
  - Sends commands with `\n` line ending
  - Reads responses with 5s timeout
  - Command mapping:
    - `temp` → `READ_TEMP`
    - `light on/off/toggle` → `LED_ON/OFF/TOGGLE`
    - `/info` → `INFO`
    - `/help`, `/version`, `/about` → Backend responses
    - Others → Forwarded directly to Arduino

### Frontend (Astro + TypeScript)
- **Terminal UI** (xterm.js)
  - Two modes: Normal / Config
  - ASCII-only rendering (no Unicode stretch)
  - Keyboard shortcuts (Ctrl+L, Ctrl+C, Tab, Arrows)
  
- **Status Bar**
  - WebSocket connection indicator
  - Serial connection indicator
  - Mode pill (Normal/Config)
  - Clear button
  
- **WebSocket Client**
  - Auto-reconnect on disconnect
  - Event-driven message handling
  - Command sender

### Arduino (ESP8266)
- **Serial Commands**
  - `LED_ON`, `LED_OFF`, `LED_TOGGLE`
  - `READ_TEMP`, `READ_HUM`, `READ_ALL`
  - `INFO` → Returns sensor list
  
- **MQTT Integration**
  - Publishes sensor readings
  - Subscribes to commands
  - Status updates

## Data Flow

```
User Input → Terminal UI → WebSocket Client → Backend WebSocket Handler
                                                      ↓
                                               Serial Bridge
                                                      ↓
                                               Arduino (Serial)
                                                      ↓
                                               Response (Serial)
                                                      ↓
                                            Backend Broadcast
                                                      ↓
                                         All WebSocket Clients
                                                      ↓
                                              Terminal Output
```

## Configuration

### Backend (src/config.rs)
- MQTT: `localhost:1883`
- HTTP Server: `localhost:8080`
- Serial: Auto-detect, default 115200 baud

### Frontend (package.json)
- Dev server: `localhost:4321`
- Dependencies: @xterm/xterm, @xterm/addon-fit, astro

### Arduino (miniverse_mqtt.ino)
- WiFi SSID/Password (update before upload)
- MQTT Server IP (update with your computer's IP)
- Sensors: DHT22 on pin 2, LED on pin 13

## Development

### Start All Services
```bash
# Terminal 1: MQTT Broker
mosquitto -c mqtt_broker/config/mosquitto.conf

# Terminal 2: Backend
cd backend && cargo run --release

# Terminal 3: Frontend
cd frontend && npm run dev
```

### Build for Production
```bash
# Backend
cd backend && cargo build --release

# Frontend
cd frontend && npm run build
```

## Command Reference

### Config Mode
- `ports` - List available serial ports
- `connect <index> [baud]` - Connect to serial port
- `disconnect` - Disconnect serial
- `status` - Show connection status
- `normal` - Switch to Normal mode

### Normal Mode
- `temp` - Read temperature
- `light on/off/toggle` - Control LED
- `/help` - Show help (backend)
- `/version` - Show version (backend)
- `/about` - Show about (backend)
- `/info` - Get sensor info (Arduino)
- `config` - Switch to Config mode

### System Commands (both modes)
- `help` - Show help
- `clear` - Clear screen
- `Ctrl+L` - Clear screen
- `Ctrl+C` - Cancel input
- `Tab` - Autocomplete
- `Arrow Up/Down` - History

## Troubleshooting

### WebSocket won't connect
- Verify backend is running on port 8080
- Check browser console for errors

### Serial timeout errors
- Ensure Arduino sketch is uploaded
- Verify baud rate matches (default: 115200)
- Check Arduino is responding to commands

### Prompt doesn't appear after mode switch
- Should be fixed in latest version
- Force refresh browser if persists

### Unicode characters stretched
- Should be fixed with ASCII-only banner
- Check terminal font is monospace

### Dependencies missing
```bash
# Frontend
cd frontend && npm install

# Backend
cd backend && cargo build
```

## Project Status
- ✅ Backend: Rust WebSocket + Serial + MQTT
- ✅ Frontend: Astro + xterm.js terminal
- ✅ Arduino: ESP8266 firmware with sensors
- ✅ UI: ASCII banner, status bar, two modes
- ✅ Commands: Mapped user-friendly → firmware
- ✅ Bug fixes: Prompt flow, Unicode stretch, timeouts
- ✅ Testing: Comprehensive checklist (TESTING.md)

**Ready for submission** 🚀
