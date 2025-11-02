# 🌌 MINIVERSE - Complete Usage Guide v3.0

## 🎯 Two-Mode Terminal System

### **Normal Mode** (Default)
- Read-only monitoring
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
   Miniverse(Config)#> ./info
   ```

7. **Control devices:**
   ```
   Miniverse(Config)#> led on
   Miniverse(Config)#> read temp
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

### Arduino Control (Config Mode, Must be connected)
| Command | Description |
|---------|-------------|
| `led on` | Turn LED on |
| `led off` | Turn LED off |
| `led toggle` | Toggle LED state |
| `read temp` | Read temperature |
| `read humidity` | Read humidity |
| `read all` | Read all sensors |

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

Miniverse(Config)#> ./info
Connected Sensors:
  [1] DHT22 (Pin 2)
  [2] LED (Pin 13)

Board: Arduino Uno WiFi Rev2
Firmware: v1.0.0

Miniverse(Config)#> led on
LED on

Miniverse(Config)#> read temp
Temperature: 25.3°C

Miniverse(Config)#> exit
✓ Mode: normal

Miniverse(Normal)> 
```

---

## 🐛 Troubleshooting

### Can't enter Config mode?
- You can always enter Config mode from Normal mode
- Type: `config`

### Can't connect to Arduino?
1. List ports: `ports`
2. Check if Arduino is connected via USB
3. Try different baud rates: `connect 0 9600`

### Commands not working?
- Make sure you're in Config mode (prompt shows `#>`)
- Check if Arduino is connected: `status`
- Verify Arduino code is uploaded

---

**Built with ❤️ for Physical Computing**
