# Codex Plan: Strikeface as greetd Greeter

## Context

On Codex, greetd handles the session lifecycle. Strikeface needs to speak greetd's IPC protocol instead of doing PAM + session launch directly.

## Flow

```
systemd → greetd (root, owns VT)
  → starts default session: sway --config /etc/greetd/sway-config
    → sway runs as greeter user
    → sway launches foot + strikeface --greetd
      → strikeface renders TUI, collects password
      → strikeface sends credentials to greetd over $GREETD_SOCK
      → greetd authenticates via PAM
      → greetd kills sway (and foot, and strikeface)
      → greetd starts niri as tau on the physical seat
```

## What to Build

1. **`--greetd` mode**: instead of calling PAM directly, strikeface connects to `$GREETD_SOCK` and speaks the greetd IPC protocol.

2. **greetd IPC protocol** (simple JSON over unix socket, length-prefixed):
   - `create_session { username }` → response: `auth_message { auth_message_type, auth_message }`
   - `post_auth_message_response { response: password }` → response: `success`
   - `start_session { cmd: ["niri"], env: [...] }` → greetd takes over
   - See `greetd-docs/index.md` and `greetd/greetd_ipc/` for protocol details.

3. **PAM service**: in greetd mode, strikeface does NOT call PAM at all. greetd does the PAM dance. Strikeface just forwards credentials.

4. **No session.rs**: in greetd mode, `session::launch` is never called. greetd handles fork+setuid+exec.

5. **150ms feedback trick**: still works. Send password to greetd, wait 150ms, check for success response. If no response yet → go red. greetd's PAM penalty provides the same timing window.

## What Stays the Same

- All TUI rendering (clock, password field, red flash)
- `--user` flag
- The visual experience is identical between Tower and Codex

## Not Needed for Codex

- `--loop` (greetd restarts the greeter after session ends)
- `--session` (greetd handles session start)
- setuid installation (strikeface runs unprivileged as greeter user)
