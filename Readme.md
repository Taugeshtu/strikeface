## why strikeface

I wanted to run a cursed kiosk-ed setup on one of my machines, and also homogenize the UX between login and session lock.

## how strikeface

- `+greetd -daemon parts +ratatui +lock mode`
- picks PAM config (`/etc/pam.d/strikeface` vs `/etc/pam.d/login`) based on whether `--session` is passed

## config

Needs a PAM service configured in `/etc/pam.d/strikeface`.

NixOS:
```nix
security.pam.services.strikeface = {};
```

## install

Build & install with cargo:
```bash
cargo install --git https://github.com/Taugeshtu/strikeface.git --root ~/.local
```

_Alternatively:_
```sh
git clone git@github.com:Taugeshtu/strikeface.git
cd strikeface
cargo install --path . --root ~/.local
```

## license

GPL-3.0-or-later. PAM handling adapted from Kenny Levinsen's `greetd`.
