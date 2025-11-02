export type SystemEvent = 
  | { type: 'mqtt_message'; topic: string; payload: string }
  | { type: 'serial_status'; connected: boolean; port: string | null; baud_rate: number | null; board_name: string | null }
  | { type: 'sensor_info'; sensors: SensorDetail[]; board: string; firmware: string }
  | { type: 'output'; content: string }
  | { type: 'error'; source: string; message: string }
  | { type: 'connected' }
  | { type: 'mode_changed'; mode: string }
  | { type: 'transport_changed'; transport: string; publish_topic: string; subscribe_topics: string[] };

export interface SensorDetail {
  id: number;
  name: string;
  pin: string;
}

export type ClientCommand =
  | { type: 'command'; command: string }
  | { type: 'mode'; mode: string }
  | { type: 'subscribe'; topic: string }
  | { type: 'publish'; topic: string; payload: string };

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private url: string;
  private reconnectTimer: number | null = null;
  private handlers = new Map<string, Set<(e: SystemEvent) => void>>();

  constructor(url = 'ws://localhost:8080/ws') {
    this.url = url;
  }

  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.ws = new WebSocket(this.url);

      this.ws.onopen = () => {
        console.log('✓ WebSocket connected');
        resolve();
      };

      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data) as SystemEvent;
          this.emit(data);
        } catch (err) {
          console.error('Parse error:', err);
        }
      };

      this.ws.onerror = reject;
      this.ws.onclose = () => this.scheduleReconnect();
    });
  }

  private scheduleReconnect() {
    if (this.reconnectTimer) return;
    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null;
      this.connect().catch(console.error);
    }, 3000);
  }

  send(cmd: ClientCommand) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(cmd));
    }
  }

  sendCommand(command: string) {
    this.send({ type: 'command', command });
  }

  changeMode(mode: string) {
    this.send({ type: 'mode', mode });
  }

  on(type: string, handler: (e: SystemEvent) => void) {
    if (!this.handlers.has(type)) {
      this.handlers.set(type, new Set());
    }
    this.handlers.get(type)!.add(handler);
  }

  private emit(event: SystemEvent) {
    this.handlers.get(event.type)?.forEach(h => h(event));
    this.handlers.get('*')?.forEach(h => h(event));
  }

  disconnect() {
    if (this.reconnectTimer) clearTimeout(this.reconnectTimer);
    this.ws?.close();
  }

  isConnected() {
    return this.ws?.readyState === WebSocket.OPEN;
  }
}

export const wsClient = new WebSocketClient();
