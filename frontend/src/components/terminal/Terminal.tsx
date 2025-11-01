import React, { useState, useEffect, useRef } from 'react';
import { useTerminal } from '../../hooks/useTerminal';
import { cn } from '../../lib/utils';

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
  const [command, setCommand] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (command.trim()) {
      sendCommand(command.trim());
      setCommand('');
    }
  };

  useEffect(() => {
    if (terminalRef.current) {
      terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
    }
  }, [lines]);

  return (
    <div className="relative w-full max-w-6xl mx-auto">
      {/* Scanline effect */}
      <div className="fixed inset-0 pointer-events-none">
        <div className="absolute inset-0 bg-gradient-to-b from-transparent via-cyber-green/5 to-transparent animate-scan-line"></div>
      </div>

      <div className={cn(
        "bg-terminal-surface border-2 border-terminal-border rounded-lg overflow-hidden",
        "shadow-2xl shadow-cyber-green/20"
      )}>
        {/* Header */}
        <div className="flex items-center justify-between p-4 bg-terminal-surface border-b border-terminal-border">
          <div className="flex items-center gap-3">
            <div className="flex gap-1.5">
              <div className="w-3 h-3 rounded-full bg-red-500"></div>
              <div className="w-3 h-3 rounded-full bg-yellow-500"></div>
              <div className="w-3 h-3 rounded-full bg-green-500"></div>
            </div>
            
            <div className="flex items-center gap-2 ml-4">
              <span className="text-cyber-green font-mono text-sm animate-pulse">
                MINIVERSE TERMINAL
              </span>
              <span className={cn(
                "px-2 py-1 rounded text-xs font-mono",
                currentSession === 'Config' 
                  ? "bg-cyber-purple/20 text-cyber-purple border border-cyber-purple/30"
                  : "bg-cyber-green/20 text-cyber-green border border-cyber-green/30"
              )}>
                {currentSession.toUpperCase()}
              </span>
            </div>
          </div>

          <button 
            onClick={clearTerminal}
            className="px-3 py-1 text-xs bg-terminal-border/50 hover:bg-terminal-border rounded transition-colors"
            title="Clear terminal"
          >
            Clear
          </button>
        </div>

        {/* Terminal display */}
        <div 
          ref={terminalRef}
          className="h-96 bg-terminal-bg p-4 overflow-y-auto font-mono text-sm"
          style={{ background: 'linear-gradient(180deg, #0a0a0a 0%, #111111 100%)' }}
        >
          {lines.length === 0 ? (
            <div className="flex items-center justify-center h-full text-terminal-muted">
              <div className="text-center">
                {/* <div className="text-2xl mb-2">🌌</div> */}
                <div>Waiting for connection...</div>
              </div>
            </div>
          ) : (
            <div className="space-y-1">
              {lines.map((line) => (
                <div key={line.id} className={cn(
                  "whitespace-pre-wrap",
                  line.type === 'error' && "text-terminal-error",
                  line.type === 'input' && "text-terminal-prompt",
                  line.type === 'system' && "text-cyber-blue",
                  line.type === 'output' && "text-terminal-text"
                )}>
                  {line.content}
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Input area */}
        <div className="border-t border-terminal-border bg-terminal-surface/50 p-4">
          <form onSubmit={handleSubmit} className="flex items-center gap-2 font-mono">
            <span className="text-terminal-prompt select-none">
              {prompt}
            </span>
            <input
              type="text"
              value={command}
              onChange={(e) => setCommand(e.target.value)}
              disabled={!isWebSocketConnected}
              className={cn(
                "flex-1 bg-transparent border-none outline-none text-terminal-text",
                "placeholder-terminal-muted caret-cyber-green",
                !isWebSocketConnected && "opacity-50 cursor-not-allowed"
              )}
              placeholder={!isWebSocketConnected ? "Not connected..." : "Type a command..."}
              autoComplete="off"
              spellCheck="false"
            />
          </form>
        </div>

        {/* Status bar */}
        <div className="flex items-center justify-between px-4 py-2 bg-terminal-bg border-t border-terminal-border text-xs">
          <div className="flex items-center gap-4">
            <span className="flex items-center gap-1">
              <div className={cn(
                "w-2 h-2 rounded-full",
                isWebSocketConnected ? "bg-cyber-green animate-pulse" : "bg-terminal-muted"
              )}></div>
              WebSocket: {isWebSocketConnected ? 'Connected' : 'Disconnected'}
            </span>
            <span className="flex items-center gap-1">
              <div className={cn(
                "w-2 h-2 rounded-full",
                isConnectedToSerial ? "bg-cyber-blue animate-pulse" : "bg-terminal-muted"
              )}></div>
              Serial: {connectedPort || 'Not connected'}
            </span>
          </div>
          <div className="text-terminal-muted">
            Session: {currentSession}
          </div>
        </div>
      </div>
    </div>
  );
};
