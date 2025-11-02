import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { wsClient, type SystemEvent, type SensorDetail } from '../lib/websocket';

type Mode = 'normal' | 'config';

export class TerminalManager {
  private term: Terminal;
  private fitAddon: FitAddon;
  private history: string[] = [];
  private historyIndex = -1;
  private currentLine = '';
  private mode: Mode = 'normal';
  private serialConnected = false;
  private serialPort = '';
  private boardName = '';
  private sensors: SensorDetail[] = [];
  private pendingPrompt = false;

  constructor(container: HTMLElement) {
    this.term = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'Menlo, Monaco, "Courier New", monospace',
      letterSpacing: 0,
      lineHeight: 1,
      theme: {
        background: '#0D0F14',
        foreground: '#C8D1E1',
        cursor: '#C8D1E1',
        selection: '#283042',
      },
      rows: 30,
      cols: 120, // Increased from 100 to prevent wrapping
    });

    this.fitAddon = new FitAddon();
    this.term.loadAddon(this.fitAddon);
    this.term.open(container);
    this.fitAddon.fit();

  this.setupListeners();
  this.showWelcome();
  }

  private setupListeners() {
    this.term.onData((data) => {
      const code = data.charCodeAt(0);

      // Enter
      if (code === 13) {
        this.handleCommand(this.currentLine);
        this.currentLine = '';
      // Backspace
      } else if (code === 127) {
        if (this.currentLine.length > 0) {
          this.currentLine = this.currentLine.slice(0, -1);
          this.term.write('\b \b');
        }
        // user interacted with the current line -> prompt is now "in use"
        this.pendingPrompt = false;
      // Ctrl+L => clear
      } else if (code === 12) {
        this.clear(true);
      // Ctrl+C => cancel current line
      } else if (code === 3) {
        this.term.write('^C');
        this.currentLine = '';
        this.term.writeln('');
        this.prompt();
      // Tab => autocomplete from history (latest match)
      } else if (code === 9) {
        const suggestion = this.autocompleteFromHistory(this.currentLine);
        if (suggestion && suggestion !== this.currentLine) {
          this.replaceCurrentLine(suggestion);
        }
      // Arrow keys (escape sequences)
      } else if (code === 27) {
        const seq = data.slice(1);
        if (seq === '[A' && this.historyIndex > 0) {
          this.historyIndex--;
          this.replaceCurrentLine(this.history[this.historyIndex]);
        } else if (seq === '[B' && this.historyIndex < this.history.length - 1) {
          this.historyIndex++;
          this.replaceCurrentLine(this.history[this.historyIndex]);
        }
      // Printable characters
      } else if (code >= 32 && code <= 126) {
        this.currentLine += data;
        this.term.write(data);
        this.pendingPrompt = false;
      }
    });

    wsClient.on('*', (e: SystemEvent) => this.handleEvent(e));
    window.addEventListener('resize', () => this.fitAddon.fit());
  }

  private handleEvent(e: SystemEvent) {
    switch (e.type) {
      case 'connected':
        this.writeln('\n[OK] Connected to Miniverse Backend');
        break;
        
      case 'serial_status':
        this.serialConnected = e.connected;
        if (e.connected && e.port && e.board_name) {
          this.serialPort = e.port;
          this.boardName = e.board_name;
          this.writeln(`\n[OK] Serial: ${e.port} - ${e.board_name} @ ${e.baud_rate} baud`);
        } else {
          this.serialPort = '';
          this.boardName = '';
          this.writeln('\n[ERR] Serial disconnected');
        }
        break;
        
      case 'sensor_info':
        this.sensors = e.sensors;
        this.writeln('\nConnected Sensors:');
        e.sensors.forEach(s => {
          this.writeln(`  [${s.id}] ${s.name} (${s.pin})`);
        });
        this.writeln(`\nBoard: ${e.board}`);
        this.writeln(`Firmware: ${e.firmware}`);
        break;
        
      case 'output':
        this.writeln(`\n${e.content}`);
        break;
        
      case 'error':
        this.writeln(`\n[ERR] ERROR [${e.source}]: ${e.message}`);
        break;
        
      case 'mode_changed':
        this.mode = e.mode as Mode;
        this.writeln(`\n[OK] Mode: ${e.mode}`);
        // Immediately refresh prompt on mode change
        this.pendingPrompt = false;
        this.prompt();
        break;
        
      case 'mqtt_message':
        this.writeln(`\n[MQTT] ${e.topic}: ${e.payload}`);
        break;
    }
  }

  private handleCommand(cmd: string) {
    const trimmed = cmd.trim();
    if (!trimmed) {
      // Ignore empty commands (space+enter) to prevent duplicate prompts
      this.prompt();
      return;
    }

    this.term.writeln('');
    this.history.push(trimmed);
    this.historyIndex = this.history.length;

    // Local commands
    if (trimmed === 'clear') {
      this.term.clear();
      this.pendingPrompt = false;
      this.prompt();
      return;
    }

    if (trimmed === 'help') {
      this.showHelp();
      this.prompt();
      return;
    }

    if (trimmed === 'config') {
      this.mode = 'config';
      wsClient.changeMode('config');
      this.pendingPrompt = false;
      this.prompt();
      return;
    }

    if (trimmed === 'normal' || trimmed === 'exit') {
      this.mode = 'normal';
      wsClient.changeMode('normal');
      this.pendingPrompt = false;
      this.prompt();
      return;
    }

    // Provide guidance if user tries Arduino commands without a serial connection
    const looksArduino = ['temp','distance','date','time','season','light','set','lcd','/info','/help','/version','/about']
      .some(k => trimmed.startsWith(k));
    if (looksArduino && !this.serialConnected) {
      this.writeln('\n[ERR] Serial not connected. Use "config" -> "ports" -> "connect <n> [baud]".');
      this.prompt();
      return;
    }

    // Send to backend
    this.pendingPrompt = false;
    wsClient.sendCommand(trimmed);
    this.prompt();
  }

  private replaceCurrentLine(text: string) {
    const promptText = this.getPromptText();
    this.term.write(`\r\x1b[K${promptText}`);
    this.currentLine = text;
    this.term.write(text);
  }

  private autocompleteFromHistory(prefix: string): string | null {
    if (!prefix) return null;
    for (let i = this.history.length - 1; i >= 0; i--) {
      const h = this.history[i];
      if (h.startsWith(prefix)) return h;
    }
    return null;
  }

  private getPromptText(): string {
    if (this.mode === 'config') {
      return 'Miniverse(Config)#> ';
    }
    return 'Miniverse(Normal)> ';
  }

  private prompt() {
    if (this.pendingPrompt) return;
    this.term.write(`\n${this.getPromptText()}`);
    this.pendingPrompt = true;
  }

  private writeln(text: string) {
    this.term.writeln(text);
  }

  private showWelcome() {
    // Ultra-simple banner - no special characters that break rendering
    const banner = [
      '',
      '  ========================================',
      '  |                                      |',
      '  |       M I N I V E R S E              |',
      '  |                                      |',
      '  |    Discovery Terminal v1.0           |',
      '  |                                      |',
      '  ========================================',
      '',
      '  Physical Computing & IoT Control System',
      '  Arduino Interface Terminal',
      '',
      '  Commands: help | config | normal',
      ''
    ].join('\n');

    this.writeln(banner);
    this.prompt();
  }

  private showHelp() {
    this.writeln('');
    this.writeln('MINIVERSE COMMANDS');
    this.writeln('==================');
    this.writeln('');
    this.writeln('System:');
    this.writeln('  help       - Show this help');
    this.writeln('  clear      - Clear screen');
    this.writeln('  config     - Enter config mode');
    this.writeln('  normal     - Enter normal mode');
    this.writeln('');
    this.writeln('Config Mode:');
    this.writeln('  ports              - List serial ports');
    this.writeln('  connect <n> [baud] - Connect to port');
    this.writeln('  disconnect         - Disconnect serial');
    this.writeln('  status             - Show status');
    this.writeln('');
    this.writeln('Normal Mode:');
    this.writeln('  temp           - Read temperature');
    this.writeln('  light on/off   - Control LED');
    this.writeln('  /info          - Device info');
    this.writeln('  /help          - Firmware help');
    this.writeln('  /version       - Firmware version');
    this.writeln('');
  }

  // Public API for external UI controls (e.g., status bar Clear button)
  public clear(showWelcome: boolean = true) {
    this.term.clear();
    this.pendingPrompt = false;
    if (showWelcome) {
      this.showWelcome();
    }
    this.prompt();
  }

  destroy() {
    this.term.dispose();
    wsClient.disconnect();
  }
}
