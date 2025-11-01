# Miniverse - Physical Computing Project 2025 - IT KMITL

This project is a part of 06016409 PHYSICAL COMPUTING Semester 1/2568 at School of Information Technology, KMITL

## สมาชิกกลุ่ม
|รหัสนักศึกษา|ชื่อ|นามสกุล|
|--|--|--|
| 67070012 | กิรณา | ศรีเพชรพูล |
| 67070065 | ธนบดี | อังกุลดี |
| 67070081 | ธีธัช | สุขวิมลไพศาล |
| 67070277 | วรภา | พุ่มฉัตร |

---

# Miniverse Discovery Terminal

Advanced Arduino Serial Communication Interface for Physical Computing projects.

## Architecture Overview
┌─────────────────────────────────────────────────────────────┐
│ Browser │
│ ┌──────────────────────────────────────────────────────┐ │
│ │ Frontend (Astro + React) │ │
│ │ - Terminal Component │ │
│ │ - WebSocket Hook │ │
│ │ - Terminal Logic Hook │ │
│ └──────────────────────────────────────────────────────┘ │
└───────────────────────────┬─────────────────────────────────┘
│ WebSocket
│ ws://localhost:8080/ws
┌───────────────────────────▼─────────────────────────────────┐
│ Backend (Rust + Actix) │
│ ┌──────────────────────────────────────────────────────┐ │
│ │ WebSocket Handler │ │
│ │ - Receives commands │ │
│ │ - Sends responses │ │
│ └───────────────────┬──────────────────────────────────┘ │
│ │ │
│ ┌───────────────────▼──────────────────────────────────┐ │
│ │ CLI Processor │ │
│ │ - Command parsing │ │
│ │ - Session management (Normal/Config) │ │
│ └───────────────────┬──────────────────────────────────┘ │
│ │ │
│ ┌───────────────────▼──────────────────────────────────┐ │
│ │ Serial Manager │ │
│ │ - Port scanning │ │
│ │ - Connection management │ │
│ │ - Data transmission │ │
│ └───────────────────┬──────────────────────────────────┘ │
└────────────────────────┼────────────────────────────────────┘
│ tokio-serial
│
┌────────────────────────▼────────────────────────────────────┐
│ Arduino Board │
│ - Receives serial commands │
│ - Sends sensor data │
│ - Executes control commands │
└─────────────────────────────────────────────────────────────┘

Output example:

Step 2: Connect to Arduino
Or with custom baud rate:

Step 3: Check Connection
Output:

Step 4: Send Commands to Arduino
After connecting, any command sent will be transmitted to Arduino via serial:

Step 5: Receive Data from Arduino
Arduino responses are automatically displayed in the terminal. The Serial Manager listens for incoming data and displays it as output.

Step 6: Disconnect
Arduino Code Example
Here's a simple Arduino sketch to test communication:

WebSocket Message Protocol
Client to Server
Server to Client
Output Message:

Error Message:

Mode Changed:

Connected:

Disconnected:

Port List:

Installation
Prerequisites
Rust 1.70+
Node.js 18+
npm or yarn
Install Dependencies
Development
Start Both Servers
This starts:

Backend on http://localhost:8080
Frontend on http://localhost:4321
Start Separately
Build for Production
Built files are in frontend/dist/

Project Structure
Troubleshooting
Backend Issues
Port 8080 already in use:

Rust compilation errors:

Frontend Issues
Port 4321 already in use:

Module not found errors:

Serial Port Issues
Permission denied (Linux):

Permission denied (macOS):

Grant terminal app full disk access in System Preferences > Security & Privacy
Port not found:

Check if Arduino is connected: ls /dev/tty* (macOS/Linux)
Check Arduino IDE's port selection
Try different USB cable or port
WebSocket Issues
Connection refused:

Ensure backend is running
Check if firewall is blocking port 8080
Verify WebSocket URL in useTerminal.ts
Constant reconnection:

Check backend logs for errors
Verify network connectivity
Check browser console for errors
Performance Considerations
Backend
Uses async/await for non-blocking I/O
WebSocket heartbeat every 5 seconds
Serial read timeout: 1 second
Graceful shutdown handling
Frontend
React component memoization
Efficient re-rendering with keys
Auto-scroll optimization
Command history limited to last 100 commands
Security Notes
WebSocket runs on localhost only by default
No authentication implemented (add if needed)
CORS configured for local development
Serial commands sent directly to device (sanitize if needed)
License
MIT

Support
For issues and questions, create an issue in the project repository.
