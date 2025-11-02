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
      // Let FitAddon determine columns based on container size
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
        this.writeln('');
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
        this.writeln('');
        this.writeln(`\x1b[38;2;0;200;0m[OK]\x1b[0m Connected to Miniverse Backend`);
        this.prompt();
        break;
        
      case 'serial_status':
        this.serialConnected = e.connected;
        if (e.connected && e.port && e.board_name) {
          this.serialPort = e.port;
          this.boardName = e.board_name;
          this.writeln('');
          this.writeln(`\x1b[38;2;0;200;0m[OK]\x1b[0m Serial: ${e.port} - ${e.board_name} @ ${e.baud_rate} baud`);
        } else {
          this.serialPort = '';
          this.boardName = '';
          this.writeln('');
          this.writeln(`\x1b[31m[ERR]\x1b[0m Serial disconnected`);
        }
        // Force a fresh prompt after async status lines may have printed below an older prompt
        this.pendingPrompt = false;
        this.prompt();
        break;
        
      case 'sensor_info':
  this.sensors = e.sensors;
  this.writeln('');
  this.writeln('Connected Sensors:');
        e.sensors.forEach(s => {
          this.writeln(`  [${s.id}] ${s.name} (${s.pin})`);
        });
  this.writeln(``);
  this.writeln(`Board: ${e.board}`);
        this.writeln(`Firmware: ${e.firmware}`);
        this.prompt();
        break;
        
      case 'output':
  this.writeln('');
  this.writeln(`${e.content}`);
        this.pendingPrompt = false;
        this.prompt();
        break;
        
      case 'error':
  this.writeln('');
  this.writeln(`\x1b[31m[ERR]\x1b[0m ERROR [${e.source}]: ${e.message}`);
        this.pendingPrompt = false;
        this.prompt();
        break;
        
      case 'mode_changed':
  this.mode = e.mode as Mode;
  this.writeln('');
  this.writeln(`\x1b[38;2;0;200;0m[OK]\x1b[0m Mode: ${e.mode}`);
        // Immediately refresh prompt on mode change
        this.pendingPrompt = false;
        this.prompt();
        break;
        
      case 'mqtt_message':
  this.writeln('');
  this.writeln(`\x1b[38;2;0;180;200m[MQTT]\x1b[0m ${e.topic}: ${e.payload}`);
        // Ensure the prompt is visible after async MQTT output arrives
        this.pendingPrompt = false;
        this.prompt();
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

    this.writeln('');
    this.history.push(trimmed);
    this.historyIndex = this.history.length;

    // Client-side validation: allow only known patterns
    if (!this.isAllowedCommand(trimmed)) {
      this.writeln('\x1b[31m[ERR]\x1b[0m Unknown command. Type `help` for available commands.');
      this.prompt();
      return;
    }

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

    // Guard: transport commands allowed only in config mode
    if (trimmed.toLowerCase().startsWith('transport ')
        && this.mode !== 'config') {
      this.writeln('');
      this.writeln('\x1b[31m[ERR]\x1b[0m transport is available only in CONFIG mode. Type `config` first.');
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
    const looksArduino = ['temp','distance','date','time','season','light','set','lcd','/info','/help','/version','/about','/INFO','/HELP','/VERSION','/ABOUT']
      .some(k => trimmed.startsWith(k));
    if (looksArduino && !this.serialConnected) {
      this.writeln('\n[ERR] Serial not connected. Use "config" -> "ports" -> "connect <n> [baud]".');
      this.prompt();
      return;
    }

    // Send to backend; prompt will be shown upon 'output' or 'error'
    this.pendingPrompt = false;
    wsClient.sendCommand(trimmed);
  }

  private isAllowedCommand(line: string): boolean {
    const m = this.mode;
    const low = line.toLowerCase();
    // always allowed
    if (['help','clear','config','normal','exit'].includes(low)) return true;
    // config-mode commands
    const isConfig = (
      low.startsWith('ports') ||
      low.startsWith('connect ') || low === 'connect' ||
      low.startsWith('disconnect') || low.startsWith('status') ||
      low.startsWith('transport serial') || low.startsWith('transport mqtt') ||
      low.startsWith('mqtt sub ') || low.startsWith('mqtt unsub ')
    );
    if (m === 'config' && isConfig) return true;

    // normal-mode commands
    const isNormal = (
      low === 'info' || low === 'about' || low === 'version' ||
      low === 'temp' || low.startsWith('temp ') ||
      low === 'distance' || low.startsWith('distance ') ||
      low.startsWith('set light ') ||
      low === 'light on' || low === 'light off' ||
      low === 'lcd clear' || low.startsWith('lcd show ')
    );
    if (m === 'normal' && isNormal) return true;
    return false;
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
    // Always move to a new line and reset column to avoid creeping indents
    this.term.write(`\r\n`);
    this.term.write(this.getPromptText());
    this.pendingPrompt = true;
    // ensure latest line is visible
    try { this.term.scrollToBottom(); } catch {}
  }

  private writeln(text: string) {
    // Normalize CR/LF and render each line distinctly
    const normalized = text.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
    normalized.split(/\n/).forEach(line => this.term.writeln(line));
    // keep view pinned to the latest output
    try { this.term.scrollToBottom(); } catch {}
  }

  private showWelcome() {
    // Dynamic ASCII banner sized to terminal columns (capped for readability)
    const cols = Math.max(20, Math.min(this.term.cols, 60));
    const border = '='.repeat(cols);
    const blank = '|' + ' '.repeat(cols - 2) + '|';
    const centerLine = (text: string) => {
      const pad = cols - 2 - text.length;
      const left = Math.floor(pad / 2);
      const right = pad - left;
      return '|' + ' '.repeat(left) + text + ' '.repeat(right) + '|';
    };

    const lines = [
      '',
      '  ' + border,
      '  ' + blank,
      '  ' + centerLine('MINIVERSE'),
      '  ' + blank,
      '  ' + centerLine('Discovery Terminal v1.0'),
      '  ' + blank,
      '  ' + border,
      '',
      '  Physical Computing & IoT Control System',
      '  Arduino Interface Terminal',
      '',
      '  Commands: help | config | normal',
      ''
    ];

    lines.forEach(l => this.term.writeln(l));
    this.prompt();
  }

  private showHelp() {
    this.writeln('');
    this.writeln('MINIVERSE COMMANDS');
    this.writeln('==================');
    this.writeln('');
    this.writeln('System:');
    this.writeln('  help           - Show this help');
    this.writeln('  clear          - Clear screen');
    this.writeln('  config         - Enter config mode');
    this.writeln('  normal         - Enter normal mode');
    this.writeln('');
    this.writeln('Config Mode:');
    this.writeln('  ports                  - List serial ports');
    this.writeln('  connect <n> [baud]     - Connect to port');
    this.writeln('  disconnect             - Disconnect serial');
    this.writeln('  status                 - Show status');
    this.writeln('  transport serial|mqtt  - Select routing (CONFIG only)');
    this.writeln('  mqtt sub <topic>       - Subscribe to topic');
    this.writeln('  mqtt unsub <topic>     - Unsubscribe from topic');
    this.writeln('');
    this.writeln('Normal Mode:');
    this.writeln('  temp <C|F|K>           - Read temperature');
    this.writeln('  distance [id]          - Read distance');
    this.writeln('  light on | light off   - LED shortcut full/zero');
    this.writeln('  set light <0-255> [color] - Set LED brightness');
    this.writeln('  lcd clear              - Clear LCD');
    this.writeln('  lcd show "a" ["b"]   - Show text on LCD');
    this.writeln('  info | about | version');
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
