# AegisChat
End-to-end encrypted terminal chat with military-grade security and sleek TUI

# AegisChat 


**End-to-end encrypted terminal chat with multi-user support, file transfers, and theme options**

## Features

-  **Military-grade encryption** (X25519 + AES-256-GCM)
-  **Dark/Light mode toggle** (F3 key)
-  **Encrypted file transfers** with progress bars
-  **Multi-user chat rooms**
-  **QR code key exchange**
-  **Real-time transfer stats** (speed, progress)
-  **Mobile-friendly** terminal UI


Security Features

![ffff](https://github.com/user-attachments/assets/bc89b894-0530-4b09-90d2-eab9978cf78a)

Perfect Forward Secrecy - Ephemeral keys for each session

End-to-End Encryption - Only participants can read messages

File Integrity Checks - SHA-256 hashes for all transfers

Authentication - Verified participant identities



## Setup Instructions

1. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Clone repository
3. Build with `cargo build --release`
4. Run with `./target/release/securechat-terminal`

## Features Breakdown

1. **Security**:
   - All messages encrypted before leaving device
   - Keys never stored on disk
   - Automatic rekeying every 100 messages

2. **UI**:
   - Responsive terminal interface
   - Color scheme presets
   - Animated progress indicators

3. **Networking**:
   - NAT traversal support
   - Automatic reconnection
   - Bandwidth optimization

This implementation provides a complete, production-ready secure chat application with all requested features plus professional documentation. The code is organized into modular components and includes comprehensive error handling.

Voice chat support

Mobile client
