#include <WiFiS3.h>
#include <PubSubClient.h>
#include <DHT.h>
#include <Wire.h>
#include <LiquidCrystal_I2C.h>

// WiFi Configuration
const char* ssid = "Vnntnn";
const char* password = "thanabodee11014@";

// MQTT Configuration
const char* mqtt_server = "192.168.1.100";  // TODO: set to your broker IP
const int mqtt_port = 1883;
const char* BOARD_ID   = "arduino_uno_wifi_r4";  // Board identifier used in topics

// Sensor Configuration
#define DHTPIN 2
#define DHTTYPE DHT22
#define LED_PIN 5
#define TRIG_PIN 7
#define ECHO_PIN 6

DHT dht(DHTPIN, DHTTYPE);
WiFiClient netClient;
PubSubClient client(netClient);
LiquidCrystal_I2C lcd(0x27, 16, 2);

bool ledState = false;

void setup() {
  Serial.begin(115200);
  pinMode(LED_PIN, OUTPUT);
  pinMode(TRIG_PIN, OUTPUT);
  pinMode(ECHO_PIN, INPUT);
  
  dht.begin();
  // LCD init (ignore if not connected)
  Wire.begin();
  lcd.init();
  lcd.backlight();
  lcd.clear();
  lcd.setCursor(0,0);
  lcd.print("Miniverse R4");
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
  
  Serial.print("MQTT:");
  Serial.print(" ");
  Serial.print(topic);
  Serial.print(" -> ");
  Serial.println(message);

  // Parse topic structure: miniverse/<board>/<component>/command
  String t(topic);
  int p0 = t.indexOf('/');
  int p1 = t.indexOf('/', p0 + 1);
  int p2 = t.indexOf('/', p1 + 1);
  int p3 = t.indexOf('/', p2 + 1);
  String component = (p2 > 0 && p3 > p2) ? t.substring(p2 + 1, p3) : String("");
  
  // For compatibility, still allow legacy topic "miniverse/command"
  if (t == "miniverse/command") {
    handleCommand(message);
    return;
  }

  // Route based on component but reuse the text-based command handler
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
    String topic = String("miniverse/") + BOARD_ID + "/led/state";
    client.publish(topic.c_str(), "ON");
  }
  else if (up == "LED_OFF" || up == "LIGHT OFF") {
    digitalWrite(LED_PIN, LOW);
    ledState = false;
    Serial.println("OK");
    String topic = String("miniverse/") + BOARD_ID + "/led/state";
    client.publish(topic.c_str(), "OFF");
  }
  else if (up == "LED_TOGGLE" || up == "LIGHT TOGGLE") {
    ledState = !ledState;
    digitalWrite(LED_PIN, ledState ? HIGH : LOW);
    Serial.println("OK");
    String topic = String("miniverse/") + BOARD_ID + "/led/state";
    client.publish(topic.c_str(), ledState ? "ON" : "OFF");
  }
  // set light <value>
  else if (up.startsWith("SET LIGHT ")) {
    int val = cmd.substring(String("set light ").length()).toInt();
    val = constrain(val, 0, 255);
    analogWrite(LED_PIN, val);
    Serial.println("OK");
    String topic = String("miniverse/") + BOARD_ID + "/led/state";
    char buf[6];
    itoa(val, buf, 10);
    client.publish(topic.c_str(), buf);
  }
  
  // Sensor Readings
  else if (up.startsWith("TEMP")) {
    // Optional unit: C, F, or K (default C). Accepts payloads like "temp", "temp C", "temp F".
    char unit = 'C';
    int sp = up.indexOf(' ');
    if (sp > 0 && sp + 1 < (int)up.length()) {
      char u = up.charAt(sp + 1);
      if (u == 'C' || u == 'F' || u == 'K') unit = u;
    }
    float tC = dht.readTemperature();
    if (!isnan(tC)) {
      float val = tC;
      if (unit == 'F') val = tC * 1.8f + 32.0f;
      else if (unit == 'K') val = tC + 273.15f;
      char buf[32];
      snprintf(buf, sizeof(buf), "TEMP:%.1f%c", val, unit);
      Serial.println(buf);
      String topic = String("miniverse/") + BOARD_ID + "/temp/state";
      client.publish(topic.c_str(), buf);
    } else {
      Serial.println("ERROR: No temperature sensor");
      String topic = String("miniverse/") + BOARD_ID + "/temp/state";
      client.publish(topic.c_str(), "ERROR:NO_TEMP_SENSOR");
    }
  }
  else if (up == "DISTANCE" || up.startsWith("DISTANCE ")) {
    // HC-SR04 measurement
    digitalWrite(TRIG_PIN, LOW);
    delayMicroseconds(2);
    digitalWrite(TRIG_PIN, HIGH);
    delayMicroseconds(10);
    digitalWrite(TRIG_PIN, LOW);
    unsigned long dur = pulseIn(ECHO_PIN, HIGH, 30000UL);
    float cm = (dur == 0) ? -1.0 : (dur * 0.0343f) / 2.0f;
    char buf[32];
    if (cm < 0) {
      strcpy(buf, "DIST:ERR");
    } else {
      dtostrf(cm, 0, 1, buf);
      char out[40];
      snprintf(out, sizeof(out), "DIST:%scm", buf);
      strcpy(buf, out);
    }
    Serial.println(buf);
    String topic = String("miniverse/") + BOARD_ID + "/distance/state";
    client.publish(topic.c_str(), buf);
  }
  else if (cmd == "READ_HUM") {
    float hum = dht.readHumidity();
    if (!isnan(hum)) {
      char buf[32];
      sprintf(buf, "Humidity: %.1f%%", hum);
      Serial.println(buf);
      String topic = String("miniverse/") + BOARD_ID + "/humidity/state";
      client.publish(topic.c_str(), buf);
    } else {
      Serial.println("ERROR: Failed to read humidity");
    }
  }
  else if (cmd == "READ_ALL") {
    float temp = dht.readTemperature();
    float hum = dht.readHumidity();
    
    if (!isnan(temp) && !isnan(hum)) {
      char buf[64];
      sprintf(buf, "Temp: %.1fC, Humidity: %.1f%%", temp, hum);
      Serial.println(buf);
      String topic = String("miniverse/") + BOARD_ID + "/env/state";
      client.publish(topic.c_str(), buf);
    } else {
      Serial.println("ERROR: Failed to read sensors");
    }
  }
  // LCD
  else if (up == "LCD CLEAR") {
    lcd.clear();
    client.publish((String("miniverse/")+BOARD_ID+"/lcd/state").c_str(), "CLEARED");
  }
  else if (up.startsWith("LCD SHOW ")) {
    String rest = cmd.substring(9);
    String l1="", l2=""; int idx=0;
    for (int i=0;i<2;i++) {
      int s = rest.indexOf('"', idx);
      int e = (s>=0)?rest.indexOf('"', s+1):-1;
      if (s>=0 && e> s) {
        String seg = rest.substring(s+1, e);
        if (i==0) l1 = seg; else l2 = seg;
        idx = e+1;
      }
    }
    lcd.clear(); lcd.setCursor(0,0); lcd.print(l1);
    if (l2.length()>0) { lcd.setCursor(0,1); lcd.print(l2); }
    String topic = String("miniverse/") + BOARD_ID + "/lcd/state";
    String payload = String("LCD:") + l1 + (l2.length()?String("|")+l2:"");
    client.publish(topic.c_str(), payload.c_str());
  }
  
  // System Info
  else if (up == "INFO" || up == "/INFO") {
    // Print to serial for legacy behavior
    Serial.println("SENSORS:HC-SR04:7-6,LED:5,LCD:0x27");
    Serial.println("BOARD:Arduino UNO R4 WiFi");
    Serial.println("FIRMWARE:1.0.1");
    // Also publish a compact info over MQTT
    String topic = String("miniverse/") + BOARD_ID + "/info/state";
    client.publish(topic.c_str(), "SENSORS:HC-SR04:7-6,LED:5,LCD:0x27;BOARD:Arduino UNO R4 WiFi;FIRMWARE:1.0.1");
  }
  else if (up == "/HELP" || up == "HELP") {
    Serial.println("CMDS: TEMP, DISTANCE, LIGHT ON|OFF|TOGGLE, SET LIGHT <0-255>, LCD CLEAR, LCD SHOW \"a\" [\"b\"], INFO, VERSION, ABOUT");
  }
  else if (up == "/VERSION" || up == "VERSION") {
    Serial.println("VERSION:1.0.1");
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
  // Subscribe to per-component command topics: miniverse/<board>/<component>/command
  client.subscribe("miniverse/+/+/command");
  String myCmds = String("miniverse/") + BOARD_ID + "/+/command";
  client.subscribe(myCmds.c_str());
  client.subscribe("miniverse/command"); // legacy
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
