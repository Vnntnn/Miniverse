# 🌌 MINIVERSE - Complete Usage Guide v3.1

## 🎯 Two-Mode Terminal System

### **Normal Mode** (Default)
- Monitoring
- View system status
- See MQTT messages
- Prompt: `Miniverse(Normal)>`

### **Config Mode** (Interactive)
- Connect to Arduino
- Control sensors
- Send commands
- Prompt: `Miniverse(Config)#>`

---

## 🚀 Quick Start

1. **Start everything:**
   ```bash
   ./start_all.sh
   ```

2. **Open browser:**
   ```
   http://localhost:4321
   ```

3. **Enter Config Mode:**
   ```
   Miniverse(Normal)> config
   ```

4. **List ports:**
   ```
   Miniverse(Config)#> ports
   ```

5. **Connect:**
   ```
   Miniverse(Config)#> connect 0 115200
   ```

6. **Check sensors:**
   ```
   Miniverse(Config)#> /info
   ```

7. **Control devices (Serial):**
   ```
   Miniverse(Config)#> light on
   Miniverse(Config)#> temp
   Miniverse(Config)#> set light 128
   ```

8. **Or control via MQTT:**
   ```
   Miniverse(Normal)> mqtt sub miniverse/#
   Miniverse(Normal)> mqtt pub miniverse/command "light on"
   ```

---

## 📋 Complete Command Reference

### System Commands (Both Modes)
| Command | Description |
|---------|-------------|
| `help` | Show help message |
| `clear` | Clear screen |
| `config` | Enter Config mode |
| `normal` or `exit` | Return to Normal mode |

### Serial Commands (Config Mode Only)
| Command | Description |
|---------|-------------|
| `ports` | List available serial ports |
| `connect <n> [baud]` | Connect to port index n (default: 115200) |
| `disconnect` | Disconnect from port |
| `status` | Show connection status |
| `./info` | Display connected sensors |

### Arduino Control (Config Mode, Serial)
| Command | Description |
|---------|-------------|
| `temp` | Read temperature |
| `light on` | Turn LED on |
| `light off` | Turn LED off |
| `light toggle` | Toggle LED state |
| `set light <0-255>` | Set LED brightness (PWM) |
| `/info` | Display sensors/board/firmware |
| `/help`, `/version`, `/about` | Firmware info |

### MQTT Commands (Both Modes)
| Command | Description |
|---------|-------------|
| `mqtt sub <topic>` | Subscribe to topic |
| `mqtt pub <topic> <payload>` | Publish a payload |

---

## 🔌 Hardware Setup

### Wiring Example:

```
Arduino Uno WiFi Rev2
┌─────────────────┐
│                 │
│  Pin 2  ─────► DHT22 (Data)
│  Pin 13 ─────► LED (Anode)
│  5V     ─────► DHT22 (VCC)
│  GND    ─────► DHT22 (GND) + LED (Cathode)
│                 │
└─────────────────┘
```

---

## 📊 Example Session

```
Miniverse(Normal)> help
[Shows help message]

Miniverse(Normal)> config
✓ Mode: config

Miniverse(Config)#> ports
Available Ports:
  [0] /dev/cu.usbserial-110 - Arduino Uno WiFi Rev2
  [1] /dev/cu.usbmodem14101 - Arduino Nano 33 IoT

Miniverse(Config)#> connect 0 115200
✓ Serial: /dev/cu.usbserial-110 - Arduino Uno WiFi Rev2 @ 115200 baud

Miniverse(Config)#> /info
Connected Sensors:
  [1] DHT22 (Pin 2)
  [2] LED (Pin 13)

Board: Arduino Uno WiFi Rev2
Firmware: v1.0.0

Miniverse(Config)#> light on
OK

Miniverse(Config)#> temp
TEMP:25.3C

Miniverse(Config)#> exit
✓ Mode: normal

Miniverse(Normal)> 
```

---

## 🐛 Troubleshooting

### Prompt doesn’t return after a command?
- Firmware must reply to every command (OK or DATA). Use the provided sketch.
- If using MQTT publish, you may not get an immediate payload back—that’s normal.

### Can't connect to Arduino?
1. List ports: `ports`
2. Check if Arduino is connected via USB
3. Try different baud rates: `connect 0 9600`

### Commands not working?
- For Serial: ensure Config mode and `status` shows connected.
- For MQTT: ensure broker is running and the device is subscribed to `miniverse/command`.
- Verify Arduino code is uploaded (see `arduino_sketch/`).

---

**Built with ❤️ for Physical Computing**
