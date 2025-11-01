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
  const isIntentionalCloseRef = useRef(false);

  const connect = useCallback(() => {
    if (!shouldReconnectRef.current || isIntentionalCloseRef.current) return;

    try {
      console.log('Connecting to WebSocket...');
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

      ws.onclose = (event) => {
        console.log('WebSocket disconnected', event.code, event.reason);
        setIsConnected(false);
        setSocket(null);
        
        // Don't reconnect if:
        // 1. Intentional close
        // 2. Max attempts reached
        // 3. Should not reconnect flag is false
        if (isIntentionalCloseRef.current || !shouldReconnectRef.current) {
          console.log('Not reconnecting (intentional close)');
          return;
        }
        
        if (connectionAttemptRef.current >= 5) {
          console.log('Max reconnection attempts reached');
          return;
        }
        
        connectionAttemptRef.current += 1;
        const delay = Math.min(1000 * Math.pow(2, connectionAttemptRef.current), 10000);
        console.log(`Reconnecting in ${delay}ms... (attempt ${connectionAttemptRef.current}/5)`);
        reconnectTimeoutRef.current = window.setTimeout(connect, delay);
      };

      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };

    } catch (error) {
      console.error('Failed to create WebSocket:', error);
    }
  }, [url]);

  const sendMessage = useCallback((message: WebSocketMessage) => {
    if (socket && isConnected && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify(message));
    }
  }, [socket, isConnected]);

  const disconnect = useCallback(() => {
    console.log('Disconnecting WebSocket...');
    isIntentionalCloseRef.current = true;
    shouldReconnectRef.current = false;
    
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
    }
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.close(1000, 'Intentional disconnect');
    }
  }, [socket]);

  useEffect(() => {
    // Reset flags on mount
    shouldReconnectRef.current = true;
    isIntentionalCloseRef.current = false;
    
    connect();
    
    // Cleanup on unmount
    return () => {
      console.log('Component unmounting, cleaning up WebSocket...');
      isIntentionalCloseRef.current = true;
      shouldReconnectRef.current = false;
      
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (socket) {
        socket.close(1000, 'Component unmount');
      }
    };
  }, [url]); // Only reconnect when URL changes

  return {
    isConnected,
    messages,
    sendMessage,
    disconnect,
    reconnect: connect
  };
};
