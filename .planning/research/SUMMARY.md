# Research Summary: Nearby (BLE Proximity Unlock & TUI Setup)

**Project:** nearby  
**Status:** Research Complete  
**Goal:** Provide a TUI for configuration setup and a background daemon for BLE-based proximity locking/unlocking on Linux systemd environments.

## Executive Summary

Nearby is a security-enhancing tool designed to manage Linux session locks based on the proximity of a Bluetooth Low Energy (BLE) device (e.g., a phone or smartwatch). The project consists of two main components: a **background daemon** that monitors device signal strength (RSSI) and interacts with `systemd-logind`, and a **TUI-based setup wizard** to simplify device discovery and configuration.

The recommended approach uses **Rust** for its safety and performance, leveraging **BlueR** for native Linux Bluetooth integration and **zbus** for D-Bus communication with `systemd-logind`. For the TUI setup experience, **Inquire** is the preferred choice for a linear, user-friendly configuration flow. The primary challenge identified is the inherent unreliability and spoofability of BLE RSSI, which necessitates robust signal filtering (Hysteresis) and a "Security First" philosophy (prioritizing Auto-Lock over Auto-Unlock).

## Key Findings

### Technology Stack (STACK.md)
- **Core:** Rust 1.70+ with `tokio` for async concurrency.
- **Bluetooth:** `bluer` (Official BlueZ Rust interface) is chosen over `btleplug` for better integration with Linux-native pairing/trusted status.
- **System:** `zbus` for high-level D-Bus interaction with `systemd-logind` and `Polkit` for session management authorization.
- **TUI:** `Inquire` for linear setup wizards; `Cursive` if a persistent settings menu is required.

### Feature Set (FEATURES.md)
- **Table Stakes:** Auto-Lock (RSSI-based), configurable thresholds, session status awareness, and a device discovery/selection tool.
- **Differentiators:** RSSI Hysteresis (to prevent "ghost locks"), multi-device support (AND/OR logic), and idle-timeout overrides.
- **Anti-Features:** Password injection (insecure) and WIFI-based proximity (unreliable).

### Architecture (ARCHITECTURE.md)
- **Component Pattern:** A background daemon running as a `systemd --user` service.
- **Processing Loops:** 
    1. **BLE Scanner:** Periodic scanning and RSSI collection.
    2. **D-Bus Listener:** Tracking session lock/unlock and idle state.
    3. **Decision Engine:** Applying Exponential Moving Averages (EMA) to RSSI data to trigger actions.
- **Security Pattern:** Use `org.freedesktop.login1` Session methods for locking rather than shell execution of `loginctl`.

### Pitfalls & Risks (PITFALLS.md)
- **Security:** BLE proximity is spoofable. Recommendation: Default to **Auto-Lock only** and warn users about Auto-Unlock risks.
- **Stability:** "RSSI Flapping" causes frequent unintended locks. Solution: Implement signal buffering and EMA.
- **Efficiency:** Constant scanning drains battery. Solution: Implement dynamic scanning intervals based on proximity state.

## Roadmap Implications

### Suggested Phase Structure

1.  **Phase 1: Foundation (Core Daemon):** 
    - Implement basic BLE scanning (`bluer`) and D-Bus session monitoring (`zbus`). 
    - *Rationale:* Core functionality must be validated before the UI can configure it.
2.  **Phase 2: Logic & Filtering:** 
    - Implement the EMA/Hysteresis logic and the state machine for locking.
    - *Avoid:* Pitfall of "ghost locks" by ensuring robust filtering is in place.
3.  **Phase 3: TUI Configuration (Goal-Specific):** 
    - Build the setup wizard using `Inquire`.
    - Features: Device discovery, RSSI calibration tool (live feedback), and `config.toml` generation.
4.  **Phase 4: System Integration:** 
    - Create systemd user service units and Polkit rules for seamless installation.

### Research Flags
- **Needs Research:** Polkit rule generation for non-root users to trigger locks (may vary across distros).
- **Standard Patterns:** BLE discovery and D-Bus proxy generation are well-documented.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | **HIGH** | BlueR and zbus are the standard for modern Linux-Rust integration. |
| Features | **HIGH** | Clear distinction between core value and "shiny" but risky features. |
| Architecture | **HIGH** | State-machine/Loop pattern is proven for this type of daemon. |
| Pitfalls | **MEDIUM** | Real-world RSSI behavior is notoriously "noisy"; calibration will be key. |

### Gaps to Address
- **Multi-user environments:** Research how the daemon behaves on shared machines with multiple concurrent sessions.
- **TUI Live Feedback:** Investigate if `Inquire` can handle a "live RSSI meter" or if `Ratatui` is needed for that specific calibration step.

## Sources
- BlueR Documentation & BlueZ D-Bus API Spec.
- Systemd-logind D-Bus documentation.
- Rust TUI library comparisons (Ratatui, Inquire, Cursive).
- CVE-2023-45866 regarding Bluetooth pairing security.
