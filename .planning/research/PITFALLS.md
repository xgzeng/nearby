# Domain Pitfalls: BLE Proximity Unlock

**Domain:** Proximity locking/unlocking for Linux
**Researched:** 2025-02-12

## Critical Pitfalls

Mistakes that cause rewrites or major security issues.

### Pitfall 1: MAC Spoofing & Security Blindness
**What goes wrong:** User assumes the tool is "secure" because it requires proximity.
**Why it happens:** BLE proximity (without LE Secure Connections or FIDO2) can be spoofed using an ESP32 for under $10.
**Consequences:** Unauthorized unlock of the machine.
**Prevention:** 1. Default to **Auto-Lock only**. 2. Require pairing via **LE Secure Connections**. 3. Clearly warn the user.

### Pitfall 2: RSSI "Flapping" (Ghost Locks)
**What goes wrong:** PC locks while the user is sitting right there.
**Why it happens:** Signal strength (RSSI) fluctuates wildly based on radio interference (WIFI, microwaves) or physical obstacles (human body).
**Consequences:** Frustrated user, broken workflow, eventually disabling the tool.
**Prevention:** Implement a buffer/average for RSSI and require multiple "out of range" readings before locking.

## Moderate Pitfalls

### Pitfall 1: Battery Drain
**What goes wrong:** Laptop battery life drops significantly.
**Why it happens:** Constant Bluetooth scanning prevents the CPU from reaching deep sleep (C-states).
**Prevention:** Adjust scanning intervals based on whether the device was recently seen. If the device is far away, scan less often.

### Pitfall 2: logind / D-Bus Deadlocks
**What goes wrong:** The tool crashes or hangs when the system suspends or resumes.
**Why it happens:** D-Bus connections can be severed during sleep/wake.
**Prevention:** Handle `systemd` sleep/wake signals and reconnect the D-Bus bus if necessary.

## Minor Pitfalls

### Pitfall 1: Conflicting Screen Lockers
**What goes wrong:** The tool calls `lock-session`, but the user has a custom locker that doesn't listen to `logind`.
**Prevention:** Provide a fallback command configuration for the user.

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
| :--- | :--- | :--- |
| **BLE Scanner** | Bluetooth adapter busy. | Handle `InProgress` or `AlreadyStarted` errors gracefully. |
| **Logic/Locking** | Locking while user is active. | Incorporate `IdleHint` from `logind` to prevent false locks. |
| **Auto-Unlock** | Zero-Click pairing vulnerabilities. | Ensure the adapter is not "Discoverable" while waiting for the device. |

## Sources

- [CVE-2023-45866 - Bluetooth pairing vulnerability](https://nvd.nist.gov/vuln/detail/CVE-2023-45866)
- [Arch Wiki: Power management/Bluetooth](https://wiki.archlinux.org/title/Bluetooth#Power_management)
