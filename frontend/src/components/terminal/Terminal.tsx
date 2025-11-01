import React, { useState, useEffect, useRef } from 'react';
import { useTerminal } from '../../hooks/useTerminal';

export const Terminal: React.FC = () => {
  const {
    lines,
    currentSession,
    isConnectedToSerial,
    connectedPort,
    prompt,
    isWebSocketConnected,
    sendCommand,
    clearTerminal
  } = useTerminal();

  const terminalRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const [command, setCommand] = useState('');
  const [commandHistory, setCommandHistory] = useState<string[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (command.trim()) {
      sendCommand(command.trim());
      setCommandHistory(prev => [...prev, command.trim()]);
      setCommand('');
      setHistoryIndex(-1);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (commandHistory.length > 0) {
        const newIndex = historyIndex === -1 ? commandHistory.length - 1 : Math.max(0, historyIndex - 1);
        setHistoryIndex(newIndex);
        setCommand(commandHistory[newIndex]);
      }
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (historyIndex !== -1) {
        const newIndex = Math.min(commandHistory.length - 1, historyIndex + 1);
        setHistoryIndex(newIndex);
        setCommand(commandHistory[newIndex] || '');
      }
    }
  };

  useEffect(() => {
    if (terminalRef.current) {
      terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
    }
  }, [lines]);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  return (
    <div className="w-full h-screen bg-black text-gray-300 font-mono flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 bg-gray-900 border-b border-gray-800">
        <div className="flex items-center gap-4 text-sm">
          <span className="font-semibold">Miniverse Terminal</span>
          <span className={`px-2 py-0.5 rounded text-xs ${
            currentSession === 'Config' 
              ? 'bg-blue-900 text-blue-200 border border-blue-700'
              : 'bg-gray-800 text-gray-300 border border-gray-700'
          }`}>
            {currentSession}
          </span>
        </div>
        
        <div className="flex items-center gap-4 text-xs">
          <span className="flex items-center gap-1.5">
            <div className={`w-2 h-2 rounded-full ${isWebSocketConnected ? 'bg-green-500' : 'bg-red-500'}`}></div>
            WS: {isWebSocketConnected ? 'Connected' : 'Disconnected'}
          </span>
          <span className="flex items-center gap-1.5">
            <div className={`w-2 h-2 rounded-full ${isConnectedToSerial ? 'bg-green-500' : 'bg-gray-600'}`}></div>
            Serial: {connectedPort || 'Not connected'}
          </span>
          <button 
            onClick={clearTerminal}
            className="px-2 py-1 bg-gray-800 hover:bg-gray-700 rounded transition-colors"
          >
            Clear
          </button>
        </div>
      </div>

      {/* Terminal Output */}
      <div 
        ref={terminalRef}
        className="flex-1 overflow-y-auto p-4 space-y-1"
        onClick={() => inputRef.current?.focus()}
      >
        {lines.length === 0 ? (
          <div className="text-gray-600 text-sm">
            <div>Miniverse Discovery Terminal</div>
            <div>Type 'help' for available commands</div>
          </div>
        ) : (
          lines.map((line) => (
            <div key={line.id} className={`text-sm ${
              line.type === 'error' ? 'text-red-400' :
              line.type === 'input' ? 'text-white' :
              line.type === 'system' ? 'text-blue-400' :
              'text-gray-300'
            }`}>
              <pre className="whitespace-pre-wrap font-mono">{line.content}</pre>
            </div>
          ))
        )}
      </div>

      {/* Input */}
      <div className="border-t border-gray-800 bg-gray-900">
        <form onSubmit={handleSubmit} className="flex items-center p-4">
          <span className="text-green-400 mr-2 select-none">{prompt}</span>
          <input
            ref={inputRef}
            type="text"
            value={command}
            onChange={(e) => setCommand(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={!isWebSocketConnected}
            className="flex-1 bg-transparent border-none outline-none text-white placeholder-gray-600"
            placeholder={!isWebSocketConnected ? "Connecting..." : "Type a command..."}
            autoComplete="off"
            spellCheck="false"
          />
        </form>
      </div>
    </div>
  );
};
