<!-- generated-by: gsd-doc-writer -->
# REQUIREMENTS.md - Nearby TUI Configuration Setup

This document outlines the requirements for the TUI-based configuration tool for the `nearby` project. The goal is to provide a user-friendly setup wizard to discover BLE devices and configure proximity-based locking/unlocking.

## 1. Overview
The TUI configuration tool will be a standalone command or a sub-command of the `nearby` binary. It simplifies the process of creating and maintaining the `config.toml` file, eliminating the need for manual MAC address entry and threshold guesswork.

## 2. Functional Requirements

### 2.1 Device Discovery and Selection
- **SCAN**: The TUI must perform a BLE scan using the `bluer` library.
- **LIST**: Display a list of discovered devices with:
  - Device Name (if available)
  - MAC Address
  - Current RSSI (signal strength)
- **FILTER**: (Optional) Filter out devices with extremely weak signals or non-connectable devices.
- **SELECT**: Allow the user to select a device from the list to configure.

### 2.2 RSSI Calibration and Configuration
- **CALIBRATE**: Provide a "calibration mode" where the user can move their device to the desired "Away" and "Nearby" positions while seeing a live RSSI/distance meter.
- **THRESHOLD**: 
  - Configure `away` threshold (in meters, calculated from RSSI).
  - Configure `nearby` threshold (in meters).
- **ACTION**:
  - Assign commands to thresholds: `lock`, `unlock`, or `keep-unlocked`.
  - Support multiple actions per device (e.g., Lock when Away, Unlock when Nearby).
- **METADATA**: Allow the user to provide a friendly `name` for the connection (e.g., "My Phone").

### 2.3 Configuration Persistence
- **TOML**: The TUI must generate or update the `~/.config/nearby/config.toml` file.
- **SCHEMA**: The generated TOML must match the `ConfigData` structure used by the daemon.
- **BACKUP**: (Optional) Create a backup of the existing `config.toml` before overwriting.

### 2.4 Daemon Integration
- **RELOAD**: If the `nearby` daemon is running, the TUI should ideally notify it to reload the configuration after saving.
- **SYSTEMD**: Provide an option to enable/start the `nearby.service` (user unit) after a successful configuration.

## 3. User Interface Requirements

### 3.1 Interaction Model
- **WIZARD**: Use a linear flow for the initial setup.
- **TOOLING**: Use `inquire` for prompt-based interactions. For live calibration, consider `ratatui` if `inquire` is insufficient.

### 3.2 Error Handling and Feedback
- **PERM**: Inform the user if Bluetooth permissions (e.g., `CAP_NET_ADMIN`) are missing.
- **EMPTY**: Provide a way to retry the scan or exit gracefully if no devices are discovered.
- **VALID**: Ensure thresholds are positive numbers and MAC addresses are valid.

## 4. Technical Constraints
- **STACK**: Rust 1.70+, `bluer`, `serde`, `toml`, Linux (systemd).

## 5. Security Considerations
- **WARNING**: Display a warning about security risks of BLE proximity unlocking.
- **FILE-PERM**: Ensure `config.toml` is created with user-only permissions (600).

## 6. Success Criteria
- [ ] User can run `nearby setup` and see a list of nearby BLE devices.
- [ ] User can select their device and set distance thresholds.
- [ ] A valid `config.toml` is written to the correct location.
- [ ] The `nearby` daemon successfully loads the generated configuration.
- [ ] The TUI handles common failure modes (Bluetooth off, no devices) without crashing.

## Traceability

| ID | Phase | Status |
|----|-------|--------|
| SCAN | Phase 2 | Pending |
| LIST | Phase 2 | Pending |
| FILTER | Phase 2 | Pending |
| SELECT | Phase 2 | Pending |
| CALIBRATE | Phase 2 | Pending |
| THRESHOLD | Phase 2 | Pending |
| ACTION | Phase 2 | Pending |
| METADATA | Phase 2 | Pending |
| TOML | Phase 1 | Pending |
| SCHEMA | Phase 1 | Pending |
| BACKUP | Phase 4 | Pending |
| RELOAD | Phase 3 | Pending |
| SYSTEMD | Phase 3 | Pending |
| WIZARD | Phase 1 | Pending |
| TOOLING | Phase 1 | Pending |
| PERM | Phase 1 | Pending |
| EMPTY | Phase 2 | Pending |
| VALID | Phase 2 | Pending |
| STACK | Phase 1 | Pending |
| WARNING | Phase 4 | Pending |
| FILE-PERM | Phase 1 | Pending |
