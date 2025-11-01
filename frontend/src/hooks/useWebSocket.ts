import { useState, useEffect, useCallback, useRef } from 'react';

export interface WebSocketMessage {
  type: string;
  data?: any;
}

export const useWebSocket = (url: string) => {
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [messages, setMessages] = useState<WebSocketMessage[]>([]);
  const reconnectTimeoutRef = useRef<number>();
  const shouldReconnectRef = useRef(true);
  const connectionAttemptRef = useRef(0);

  const connect = useCallback(() => {
    if (!shouldReconnectRef.current) return;

    try {
      const ws = new WebSocket(url);
      
      ws.onopen = () => {
        console.log('WebSocket connected');
        setIsConnected(true);
        setSocket(ws);
        connectionAttemptRef.current = 0;
      };

      ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);
          setMessages(prev => [...prev, message]);
        } catch (error) {
          console.error('Failed to parse message:', error);
        }
      };

      ws.onclose = () => {
        console.log('WebSocket disconnected');
        setIsConnected(false);
        setSocket(null);
        
        if (shouldReconnectRef.current && connectionAttemptRef.current < 5) {
          connectionAttemptRef.current += 1;
          const delay = Math.min(1000 * Math.pow(2, connectionAttemptRef.current), 10000);
          console.log(`Reconnecting in ${delay}ms...`);
          reconnectTimeoutRef.current = window.setTimeout(connect, delay);
        }
      };

      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };

    } catch (error) {
      console.error('Failed to create WebSocket:', error);
    }
  }, [url]);

  const sendMessage = useCallback((message: WebSocketMessage) => {
    if (socket && isConnected) {
      socket.send(JSON.stringify(message));
    }
  }, [socket, isConnected]);

  const disconnect = useCallback(() => {
    shouldReconnectRef.current = false;
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }
    if (socket) {
      socket.close();
    }
  }, [socket]);

  useEffect(() => {
    shouldReconnectRef.current = true;
    connect();
    
    return () => {
      shouldReconnectRef.current = false;
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (socket) {
        socket.close();
      }
    };
  }, []);

  return {
    isConnected,
    messages,
    sendMessage,
    disconnect,
    reconnect: connect
  };
};
