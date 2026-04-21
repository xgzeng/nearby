# Technology Stack: Secure Proximity

**Project:** nearby
**Researched:** 2024-05-24

## Recommended Stack

### Core Framework
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| [Rust](https://www.rust-lang.org/) | 1.70+ | Core Logic | Memory safety and performance for background services. |
| [BlueR](https://crates.io/crates/bluer) | ^0.16 | Bluetooth Interface | Official Rust interface for BlueZ (Linux). Supports device properties like `Paired`, `Trusted`. |

### System Integration
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| [Polkit](https://www.freedesktop.org/wiki/Software/polkit/) | N/A | Authorization | Allows fine-grained permission control for `loginctl` without `sudo`. |
| [systemd-logind](https://www.freedesktop.org/software/systemd/man/latest/systemd-logind.service.html) | N/A | Session Management | Standard Linux way to manage locks/unlocks. |

### Supporting Libraries
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| [tokio](https://tokio.rs/) | ^1.0 | Async Runtime | Required for `bluer` and handling multiple devices concurrently. |
| [zbus](https://crates.io/crates/zbus) | ^3.0 | D-Bus Communication | If manual interaction with `logind` or `Polkit` is needed beyond shell commands. |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| BLE Library | BlueR | btleplug | BlueR has better integration with BlueZ's native features (Paired/Trusted/IRK) on Linux. |
| Auth | Polkit | sudo | `sudo` is too broad and harder to manage securely via scripts. |
| Locking | loginctl | gnome-screensaver | `loginctl` is desktop-agnostic. |

## Installation

### Dependencies (Linux)
```bash
# BlueZ development headers
sudo apt install libdbus-1-dev
```

### Polkit Rule Example
Create `/etc/polkit-1/rules.d/45-nearby-unlock.rules`:
```javascript
polkit.addRule(function(action, subject) {
    if (action.id == "org.freedesktop.login1.manage-sessions" &&
        subject.isInGroup("wheel")) {
        return polkit.Result.YES;
    }
});
```

## Sources

- [BlueR Documentation](https://docs.rs/bluer/latest/bluer/)
- [Polkit Manual](https://www.freedesktop.org/software/polkit/docs/latest/polkit.8.html)
