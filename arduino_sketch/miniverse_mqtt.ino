#include <ESP8266WiFi.h>
#include <PubSubClient.h>
#include <DHT.h>

// WiFi Configuration
const char* ssid = "YOUR_WIFI_SSID";
const char* password = "YOUR_WIFI_PASSWORD";

// MQTT Configuration
const char* mqtt_server = "192.168.1.100";  // Update with your computer's IP
const int mqtt_port = 1883;

// Sensor Configuration
#define DHTPIN 2
#define DHTTYPE DHT22
#define LED_PIN 13

DHT dht(DHTPIN, DHTTYPE);
WiFiClient espClient;
PubSubClient client(espClient);

bool ledState = false;

void setup() {
  Serial.begin(115200);
  pinMode(LED_PIN, OUTPUT);
  
  dht.begin();
  setupWiFi();
  
  client.setServer(mqtt_server, mqtt_port);
  client.setCallback(mqttCallback);
  
  Serial.println("MINIVERSE Arduino Ready");
}

void setupWiFi() {
  Serial.print("Connecting to WiFi");
  WiFi.begin(ssid, password);
  
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }
  
  Serial.println("\nWiFi connected");
  Serial.print("IP: ");
  Serial.println(WiFi.localIP());
}

void mqttCallback(char* topic, byte* payload, unsigned int length) {
  String message = "";
  for (unsigned int i = 0; i < length; i++) {
    message += (char)payload[i];
  }
  
  Serial.print("MQTT: ");
  Serial.println(message);
  
  handleCommand(message);
}

void handleCommand(String cmd) {
  cmd.trim();
  
  // Normalize to uppercase for matching while keeping original for arguments
  String up = cmd; up.toUpperCase();

  // LED Controls
  if (up == "LED_ON" || up == "LIGHT ON") {
    digitalWrite(LED_PIN, HIGH);
    ledState = true;
    Serial.println("OK");
    client.publish("miniverse/status", "LED turned on");
  }
  else if (up == "LED_OFF" || up == "LIGHT OFF") {
    digitalWrite(LED_PIN, LOW);
    ledState = false;
    Serial.println("OK");
    client.publish("miniverse/status", "LED turned off");
  }
  else if (up == "LED_TOGGLE" || up == "LIGHT TOGGLE") {
    ledState = !ledState;
    digitalWrite(LED_PIN, ledState ? HIGH : LOW);
    Serial.println("OK");
    client.publish("miniverse/status", ledState ? "LED on" : "LED off");
  }
  // set light <value>
  else if (up.startsWith("SET LIGHT ")) {
    int val = cmd.substring(String("set light ").length()).toInt();
    val = constrain(val, 0, 255);
    analogWrite(LED_PIN, val);
    Serial.println("OK");
  }
  
  // Sensor Readings
  else if (up == "READ_TEMP" || up == "TEMP") {
    float temp = dht.readTemperature();
    if (!isnan(temp)) {
      char buf[32];
      sprintf(buf, "TEMP:%.1fC", temp);
      Serial.println(buf);
    } else {
      Serial.println("ERROR: Failed to read temperature");
    }
  }
  else if (cmd == "READ_HUM") {
    float hum = dht.readHumidity();
    if (!isnan(hum)) {
      char buf[32];
      sprintf(buf, "Humidity: %.1f%%", hum);
      Serial.println(buf);
    } else {
      Serial.println("ERROR: Failed to read humidity");
    }
  }
  else if (cmd == "READ_ALL") {
    float temp = dht.readTemperature();
    float hum = dht.readHumidity();
    
    if (!isnan(temp) && !isnan(hum)) {
      char buf[64];
      sprintf(buf, "Temp: %.1f°C, Humidity: %.1f%%", temp, hum);
      Serial.println(buf);
    } else {
      Serial.println("ERROR: Failed to read sensors");
    }
  }
  
  // System Info
  else if (up == "INFO" || up == "/INFO") {
    Serial.println("SENSORS:DHT22:2,LED:13");
    Serial.println("BOARD:Arduino UNO R4 WiFi");
    Serial.println("FIRMWARE:1.0.0");
  }
  else if (up == "/HELP" || up == "HELP") {
    Serial.println("CMDS: TEMP, LIGHT ON|OFF|TOGGLE, SET LIGHT <0-255>, /INFO, /VERSION, /ABOUT");
  }
  else if (up == "/VERSION" || up == "VERSION") {
    Serial.println("VERSION:1.0.0");
  }
  else if (up == "/ABOUT" || up == "ABOUT") {
    Serial.println("ABOUT:Miniverse Arduino Firmware");
  }
  
  // Unknown command
  else {
    Serial.print("ERROR: Unknown command: ");
    Serial.println(cmd);
  }
}

void reconnect() {
  while (!client.connected()) {
    Serial.print("Connecting to MQTT...");
    
    if (client.connect("MiniverseArduino")) {
      Serial.println("connected");
      client.subscribe("miniverse/command");
      client.publish("miniverse/status", "Arduino connected");
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
  
  // Handle serial commands
  if (Serial.available()) {
    String cmd = Serial.readStringUntil('\n');
    cmd.trim();
    handleCommand(cmd);
  }
}
