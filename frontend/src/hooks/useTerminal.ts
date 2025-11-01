import { useState, useEffect, useCallback } from 'react';
import { useWebSocket } from './useWebSocket';

export type SessionType = 'Normal' | 'Config';

export interface TerminalLine {
  id: string;
  content: string;
  timestamp: Date;
  type: 'output' | 'input' | 'error' | 'system';
  sessionType?: SessionType;
}

const WS_URL = 'ws://localhost:8080/ws';

export const useTerminal = () => {
  const [lines, setLines] = useState<TerminalLine[]>([]);
  const [currentSession, setCurrentSession] = useState<SessionType>('Normal');
  const [isConnectedToSerial, setIsConnectedToSerial] = useState(false);
  const [connectedPort, setConnectedPort] = useState<string | null>(null);
  const [prompt, setPrompt] = useState('Miniverse>');
  
  const { isConnected: isWebSocketConnected, messages, sendMessage } = useWebSocket(WS_URL);

  const addLine = useCallback((content: string, type: TerminalLine['type'], sessionType?: SessionType) => {
    setLines(prev => [...prev, {
      id: `${Date.now()}-${Math.random()}`,
      content,
      timestamp: new Date(),
      type,
      sessionType
    }]);
  }, []);

  const sendCommand = useCallback((command: string) => {
    addLine(`${prompt} ${command}`, 'input', currentSession);
    
    sendMessage({
      type: 'Command',
      data: { command }
    });
  }, [prompt, currentSession, sendMessage, addLine]);

  const clearTerminal = useCallback(() => {
    setLines([]);
  }, []);

  useEffect(() => {
    messages.forEach(message => {
      switch (message.type) {
        case 'Output':
          addLine(message.data.content, 'output', message.data.session_type);
          break;
        case 'Error':
          addLine(message.data.message, 'error');
          break;
        case 'ModeChanged':
          setCurrentSession(message.data.mode);
          setPrompt(message.data.mode === 'Config' ? 'Miniverse(config)#' : 'Miniverse>');
          addLine(`Mode changed to ${message.data.mode}`, 'system');
          break;
        case 'Connected':
          setIsConnectedToSerial(true);
          setConnectedPort(message.data.port);
          break;
        case 'Disconnected':
          setIsConnectedToSerial(false);
          setConnectedPort(null);
          break;
      }
    });
  }, [messages, addLine]);

  return {
    lines,
    currentSession,
    isConnectedToSerial,
    connectedPort,
    prompt,
    isWebSocketConnected,
    sendCommand,
    clearTerminal
  };
};
