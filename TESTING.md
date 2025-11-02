# Miniverse Testing Checklist

## Pre-flight Check
- [x] Backend builds without errors (`cargo build --release`)
- [x] Frontend builds without errors (`npm run build`)
- [x] All dependencies installed correctly
- [x] MQTT broker running (`mosquitto` on port 1883)
- [x] Backend running (`localhost:8080`)
- [x] Frontend running (`localhost:4321`)

## UI Rendering Tests
- [ ] Welcome banner displays correctly (ASCII art, no stretched characters)
- [ ] Terminal font is monospace and readable
- [ ] Status bar shows correct indicators:
  - [ ] Mode pill (Normal/Config)
  - [ ] WebSocket dot (green when connected)
  - [ ] Serial dot (gray/green based on connection)
  - [ ] Clear button works

## Prompt Behavior Tests
- [ ] Prompt appears immediately on page load: `Miniverse(Normal)> `
- [ ] After typing command and pressing Enter, new prompt appears
- [ ] Switching modes shows immediate prompt:
  - [ ] Type `config` → prompt changes to `Miniverse(Config)#> `
  - [ ] Type `normal` → prompt changes back to `Miniverse(Normal)> `
- [ ] **No spacebar needed** to trigger prompt display
- [ ] Empty Enter (just pressing Enter) shows prompt without error

## Keyboard Shortcuts
- [ ] `Ctrl+L` clears screen and shows welcome banner
- [ ] `Ctrl+C` cancels current line and shows new prompt
- [ ] `Tab` autocompletes from command history
- [ ] `Arrow Up` cycles through command history (backward)
- [ ] `Arrow Down` cycles through command history (forward)

## Config Mode Commands
- [ ] `help` - Shows help table
- [ ] `clear` - Clears terminal
- [ ] `config` - Switches to Config mode
- [ ] `ports` - Lists available serial ports with proper table formatting
  - [ ] Port column shows full path (no wrapping)
  - [ ] Device column shows board name (no wrapping)
  - [ ] Table has proper borders and alignment

## Serial Connection Tests
- [ ] `connect 0` - Connects to first available port
- [ ] `connect 0 9600` - Connects with custom baud rate
- [ ] `status` - Shows current connection status
- [ ] `disconnect` - Disconnects serial
- [ ] Status bar updates when serial connects/disconnects

## Arduino Command Tests (Normal Mode)
After connecting to Arduino with `connect <port>`:

### Firmware Info Commands
- [ ] `/help` - Returns backend help text (instant, no timeout)
- [ ] `/version` - Returns "Miniverse Firmware: v1.0.0" (instant)
- [ ] `/about` - Returns firmware description (instant)
- [ ] `/info` - Sends `INFO` to Arduino, receives sensor list

### Sensor Commands
- [ ] `temp` - Reads temperature from DHT22
  - Expected: `Temperature: XX.X°C`
  - [ ] Response arrives within 5 seconds (no timeout)
  
- [ ] `light on` - Turns LED on
  - Expected: `Light on`
  - [ ] Response arrives within 5 seconds
  
- [ ] `light off` - Turns LED off
  - Expected: `Light off`
  
- [ ] `light toggle` - Toggles LED state
  - Expected: `Light on` or `Light off`

### Legacy Commands (if Arduino supports)
- [ ] `read temp` - Reads temperature
- [ ] `read all` - Reads all sensors
- [ ] `led on` - LED on
- [ ] `led off` - LED off
- [ ] `led toggle` - LED toggle

## Error Handling Tests
- [ ] Typing Arduino command in Normal mode without serial connection shows:
  - `[ERR] Serial not connected. Use "config" -> "ports" -> "connect <n> [baud]".`
- [ ] Invalid command shows appropriate error
- [ ] Serial timeout (if Arduino doesn't respond) shows: `[ERR] ERROR [serial]: Timeout`
- [ ] Connection to invalid port index shows: `[ERR] Invalid port index`

## Performance Tests
- [ ] Commands respond within 5 seconds
- [ ] Terminal scrolling is smooth with many lines
- [ ] WebSocket reconnects automatically if backend restarts
- [ ] No memory leaks after extended use

## Known Issues Resolved
- [x] ~~Unicode characters stretching terminal~~ → Fixed with ASCII-only banner
- [x] ~~Prompt missing after mode switch~~ → Fixed with immediate `prompt()` call
- [x] ~~Space+Enter creating duplicate prompts~~ → Fixed with empty command check
- [x] ~~Ports table text wrapping~~ → Fixed with wider columns (40ch/35ch)
- [x] ~~Serial timeout errors~~ → Fixed with 5000ms timeout + command mapping

## Integration Test Flow
Complete this sequence without errors:

1. Open http://localhost:4321
2. Welcome banner displays (ASCII art)
3. Type `help` → see help table
4. Type `config` → mode changes to Config, prompt appears
5. Type `ports` → see serial port table
6. Type `connect 0` → serial connects (if Arduino is connected)
7. Type `normal` → mode changes to Normal, prompt appears
8. Type `/help` → see help (instant, no Arduino needed)
9. Type `/version` → see version (instant)
10. Type `temp` → see temperature reading (from Arduino)
11. Type `light on` → LED turns on
12. Type `light off` → LED turns off
13. Type `Ctrl+L` → screen clears, welcome banner shows
14. Type `disconnect` (in config mode) → serial disconnects

## Sign-off
- [ ] All tests pass ✅
- [ ] No console errors in browser DevTools
- [ ] No errors in backend logs
- [ ] Ready for submission 🚀
