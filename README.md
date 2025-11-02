# Miniverse - Physical Computing Project 2025 - IT KMITL

This project is a part of 06016409 PHYSICAL COMPUTING Semester 1/2568 at School of Information Technology, KMITL

## สมาชิกกลุ่ม
|รหัสนักศึกษา|ชื่อ|นามสกุล|
|--|--|--|
| 67070012 | กิรณา | ศรีเพชรพูล |
| 67070065 | ธนบดี | อังกุลดี |
| 67070081 | ธีธัช | สุขวิมลไพศาล |
| 67070277 | วรภา | พุ่มฉัตร |

---

## MQTT topic model

Miniverse uses per-component topics with the following schema:

```
miniverse/<board_id>/<component>/{command|state}
```

- `board_id` is derived from the Arduino board name (lowercase, spaces replaced by `_`). Example: `Arduino UNO WiFi R4 CMSIS_DAP` -> `arduino_uno_wifi_r4_cmsis_dap`.
- `component` examples: `temp`, `distance`, `led`, `lcd`, `info`.
- Commands are published to the `.../command` topic and the device responds on the corresponding `.../state` topic.

Examples:

```
# request current temperature in Celsius
miniverse/arduino_uno_wifi_r4_cmsis_dap/temp/command   payload: "temp C"

# set LED brightness
miniverse/arduino_uno_wifi_r4_cmsis_dap/led/command    payload: "set light 180"

# write to LCD (1 or 2 lines)
miniverse/arduino_uno_wifi_r4_cmsis_dap/lcd/command    payload: "lcd show \"Hello\" \"Miniverse\""

# ask for device info over MQTT
miniverse/arduino_uno_wifi_r4_cmsis_dap/info/command   payload: "info"
```

Firmware publishes telemetry and acknowledgements on `.../state`. The UI includes an inline MQTT Monitor to help observe these messages per topic in real time.

## Command set (Terminal)

Normal mode:

- `temp <C|F|K>` – read temperature
- `distance [id]` – read distance
- `light on | light off` – LED shortcut full/zero
- `set light <0-255> [color]` – set LED brightness/color
- `lcd clear` – clear LCD
- `lcd show "a" ["b"]` – show 1–2 lines on LCD (quotes required)
- `info | about | version`

Config mode:

- `ports` – list serial ports
- `connect <n> [baud]`, `disconnect`, `status`
- `transport serial|mqtt`
- `mqtt sub <topic>`, `mqtt unsub <topic>`, `mqtt subs` (list current subscriptions)

Tips:

- The terminal blocks unknown commands on the client side and the backend validates again.
- In MQTT transport, `info` publishes to `miniverse/<board>/info/command` and expects the firmware to reply on `.../info/state`.

