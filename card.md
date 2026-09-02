# Strikeface

- [[Linux]], [[Security]], [[Systems_Administration]], [[UX]]

Modular authentication, session lifecycle, and lock manager designed for multi-stage boot and vault unlock architectures. Decouples the authentication daemon and session orchestrator from user interfaces via an IPC protocol, while supporting standalone direct execution. Enforces active brute-force rate limiting, integrates directly with Linux PAM, and unifies three operational lifecycles: Stage 1 boot login, persistent kiosk supervisor (headless session attach/detach), and active Wayland desktop session locking (`ext-session-lock-v1`) with a native Ratatui TUI frontend.
