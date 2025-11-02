# 🔧 Arduino Setup Guide

## Hardware Requirements

### Core Components:
- **Arduino Board:** Uno, Nano, or Mega
- **ESP8266 or ESP32:** For WiFi connectivity
- **Power Supply:** USB or external 5V

### Optional Sensors:
- **DHT22:** Temperature & Humidity
- **LDR:** Light sensor
- **PIR:** Motion sensor
- **Ultrasonic:** Distance sensor

---

## Wiring Diagrams

### 1. Arduino + ESP8266 (Basic)

```
┌─────────────┐         ┌──────────────┐
│  Arduino    │         │   ESP8266    │
│             │         │  (NodeMCU)   │
│         TX  ├────────►│ RX           │
│         RX  │◄────────┤ TX           │
│        GND  ├────────►│ GND          │
│         5V  ├────────►│ VIN          │
└─────────────┘         └──────────────┘
```

### 2. DHT22 Temperature Sensor

```
┌──────────┐
│  DHT22   │
│          │
│   VCC    ├────► Arduino 5V
│   GND    ├────► Arduino GND
│   DATA   ├────► Arduino Pin 2
└──────────┘
```

### 3. LED Control

```
        ┌─────┐
  Pin13 ├─────┤►├─── GND
        │     └─────┘
        │     LED + Resistor (220Ω)
```

---

## Software Setup

### Install Libraries (Arduino IDE):

**Tools → Manage Libraries:**

1. **DHT sensor library** by Adafruit
2. **Adafruit Unified Sensor**
3. **PubSubClient** (MQTT)
4. **ESP8266WiFi** (for ESP8266)
   - or **WiFi** (for ESP32)

---

## Code Configuration

### 1. Update WiFi Credentials

```cpp
const char* ssid = "YOUR_WIFI_SSID";
const char* password = "YOUR_WIFI_PASSWORD";
```

### 2. Update MQTT Broker IP

Find your computer's IP:

**macOS/Linux:**
```bash
ifconfig | grep "inet "
```

**Windows:**
```bash
ipconfig
```

Then update:
```cpp
const char* mqtt_server = "192.168.1.100"; // Your IP here
```

### 3. Configure Pins

```cpp
#define DHTPIN 2        // DHT22 data pin
#define LED_PIN 13      // Built-in LED
```

---

## MQTT Topics

### Subscribe (Arduino listens to):
- `miniverse/command` - Commands from computer

### Publish (Arduino sends to):
- `miniverse/sensor/temperature` - Temperature readings
- `miniverse/sensor/humidity` - Humidity readings
- `miniverse/status` - Status messages

---

## Command Protocol

### Format:
```
<command> <parameter>
```

### Examples:

**LED Control:**
```
led on
led off
led toggle
```

**Sensor Reading:**
```
read temp
read humidity
read all
```

**Custom:**
```
ping
status
reset
```

---

## Testing

### 1. Upload Code
- Connect Arduino via USB
- Select correct board and port
- Upload sketch

### 2. Open Serial Monitor
- Set baud rate: **115200**
- Should see:
  ```
  Connecting to WiFi...
  WiFi connected
  IP address: 192.168.1.150
  Connected to MQTT broker
  ```

### 3. Test from Computer

**Terminal 1 - Subscribe:**
```bash
mosquitto_sub -h localhost -t "miniverse/#" -v
```

**Terminal 2 - Publish:**
```bash
mosquitto_pub -h localhost -t "miniverse/command" -m "led on"
```

---

## Common Issues

### ESP8266 won't connect to WiFi:
- Check SSID/password
- Ensure 2.4GHz WiFi (not 5GHz)
- Check signal strength

### MQTT connection fails:
- Verify broker IP is correct
- Check firewall settings
- Ensure broker is running

### Sensor not reading:
- Check wiring
- Verify pin numbers
- Test with simple sketch first

---

## Example Arduino Code

```cpp
#include <ESP8266WiFi.h>
#include <PubSubClient.h>
#include <DHT.h>

// WiFi
const char* ssid = "YOUR_SSID";
const char* password = "YOUR_PASSWORD";

// MQTT
const char* mqtt_server = "192.168.1.100";
WiFiClient espClient;
PubSubClient client(espClient);

// DHT22
#define DHTPIN 2
#define DHTTYPE DHT22
DHT dht(DHTPIN, DHTTYPE);

// LED
#define LED_PIN 13

void setup() {
  Serial.begin(115200);
  pinMode(LED_PIN, OUTPUT);
  
  dht.begin();
  setupWiFi();
  
  client.setServer(mqtt_server, 1883);
  client.setCallback(callback);
}

void setupWiFi() {
  Serial.print("Connecting to WiFi");
  WiFi.begin(ssid, password);
  
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  
  Serial.println("\nWiFi connected!");
  Serial.print("IP: ");
  Serial.println(WiFi.localIP());
}

void callback(char* topic, byte* payload, unsigned int length) {
  String message = "";
  for (int i = 0; i < length; i++) {
    message += (char)payload[i];
  }
  
  Serial.print("Message: ");
  Serial.println(message);
  
  // Handle commands
  if (message == "led on") {
    digitalWrite(LED_PIN, HIGH);
    client.publish("miniverse/status", "LED on");
  }
  else if (message == "led off") {
    digitalWrite(LED_PIN, LOW);
    client.publish("miniverse/status", "LED off");
  }
  else if (message == "read temp") {
    float temp = dht.readTemperature();
    char buf[10];
    dtostrf(temp, 4, 1, buf);
    client.publish("miniverse/sensor/temperature", buf);
  }
}

void reconnect() {
  while (!client.connected()) {
    Serial.print("Connecting to MQTT...");
    
    if (client.connect("ArduinoClient")) {
      Serial.println("connected");
      client.subscribe("miniverse/command");
    } else {
      Serial.print("failed, rc=");
      Serial.println(client.state());
      delay(5000);
    }
  }
}

void loop() {
  if (!client.connected()) {
    reconnect();
  }
  client.loop();
}
```

---

## 🎯 Next Steps

1. ✅ Upload code to Arduino
2. ✅ Test with serial monitor
3. ✅ Verify MQTT messages
4. ✅ Connect to Miniverse terminal
5. ✅ Try commands!

Good luck! 🚀
