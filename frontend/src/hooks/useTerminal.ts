import { useState, useCallback, useRef, useEffect } from 'react';
import { useWebSocket } from './useWebSocket';

export type SessionType = 'Normal' | 'Config';

export interface TerminalLine {
  id: string;
  content: string;
  timestamp: Date;
  type: 'output' | 'input' | 'error' | 'system';
  sessionType?: SessionType;
}

export const useTerminal = () => {
  const [lines, setLines] = useState<TerminalLine[]>([]);
  const [currentSession, setCurrentSession] = useState<SessionType>('Normal');
  const [isConnectedToSerial, setIsConnectedToSerial] = useState(false);
  const [connectedPort, setConnectedPort] = useState<string | null>(null);
  const [prompt, setPrompt] = useState('Miniverse>');
  const lineIdCounter = useRef(0);

  const { isConnected, messages, sendMessage } = useWebSocket('ws://localhost:8080/ws');

  const addLine = useCallback((content: string, type: TerminalLine['type'] = 'output', sessionType?: SessionType) => {
    const line: TerminalLine = {
      id: (++lineIdCounter.current).toString(),
      content,
      timestamp: new Date(),
      type,
      sessionType: sessionType || currentSession
    };
    setLines(prev => [...prev, line]);
  }, [currentSession]);

  const sendCommand = useCallback((command: string) => {
    addLine(`${prompt} ${command}`, 'input');
    sendMessage({
      type: 'Command',
      data: { command }
    });
  }, [prompt, addLine, sendMessage]);

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
          addLine(`Error: ${message.data.message}`, 'error');
          break;
        case 'Connected':
          setIsConnectedToSerial(true);
          setConnectedPort(message.data.port);
          break;
        case 'Disconnected':
          setIsConnectedToSerial(false);
          setConnectedPort(null);
          break;
        case 'ModeChanged':
          const newSession = message.data.mode as SessionType;
          setCurrentSession(newSession);
          setPrompt(newSession === 'Normal' ? 'Miniverse(-Normal-)>' : 'Miniverse(-config-)#');
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
    isWebSocketConnected: isConnected,
    sendCommand,
    clearTerminal,
    addLine
  };
};
