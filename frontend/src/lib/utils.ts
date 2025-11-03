export function formatTimestamp(date: Date): string {
  return date.toLocaleTimeString();
}

export function parseCommand(input: string): { command: string; args: string[] } {
  const parts = input.trim().split(/\s+/);
  return {
    command: parts[0] || '',
    args: parts.slice(1)
  };
}
