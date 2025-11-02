# 🌌 MINIVERSE - Usage Guide (UNO R4 WiFi)

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
   Miniverse(Config)#> info
   ```

7. **Control devices (Serial):**
   ```
   Miniverse(Config)#> light on
   Miniverse(Config)#> temp
   Miniverse(Config)#> set light 128
   ```

8. **Or control via MQTT (per-component topics):**
   ```
   Miniverse(Normal)> mqtt subs
   Miniverse(Normal)> mqtt sub miniverse/#
   # Example publish (will be sent by backend normally):
   Miniverse(Normal)> mqtt pub miniverse/arduino_uno_wifi_r4/led/command "set light 128"
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

### Arduino Control (Normal Mode)
| Command | Description |
|---------|-------------|
| `temp` | Read temperature (unit chosen by firmware) |
| `light on` | Turn LED on |
| `light off` | Turn LED off |
| `light toggle` | Toggle LED state |
| `set light <0-255>` | Set LED brightness (PWM) |
| `lcd clear` | Clear LCD |
| `lcd show "a" ["b"]` | Show text on LCD (16x2) |
| `distance` | Read distance from HC‑SR04 |
| `info` | Display sensors/board/firmware |
| `version`, `about` | Firmware meta |

### MQTT Commands (Both Modes)
| Command | Description |
|---------|-------------|
| `mqtt sub <topic>` | Subscribe to topic |
| `mqtt unsub <topic>` | Unsubscribe from topic |
| `mqtt subs` | List current subscriptions |
| `mqtt pub <topic> <payload>` | Publish a payload |

---

## 🔌 Hardware Setup

### Wiring (Arduino UNO R4 WiFi)

```
UNO R4 WiFi
┌──────────────────────────────┐
│                              │
│  Pin 5   ─────► LED (PWM)
│  Pin 7   ─────► HC‑SR04 TRIG
│  Pin 6   ─────► HC‑SR04 ECHO
│  I2C 0x27 ───► 16x2 LCD (SDA/SCL)
│  5V/GND  ───► Sensors Power/GND
│                              │
└──────────────────────────────┘
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
   [1] HC-SR04 (Pin 7-6)
   [2] LED (Pin 5)
   [3] LCD (0x27)

Board: Arduino UNO R4 WiFi
Firmware: v1.0.1

Miniverse(Config)#> light on
OK

Miniverse(Config)#> distance
DIST:42.1cm

Miniverse(Config)#> set light 128
OK

Miniverse(Config)#> lcd show "Hello" "World"
LCD:Hello|World

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
