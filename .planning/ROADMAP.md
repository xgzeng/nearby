# ROADMAP

## Phases

- [ ] **Phase 1: Foundation** - Config serialization and TUI framework setup
- [ ] **Phase 2: Interaction** - Device discovery and calibration flow
- [ ] **Phase 3: Integration** - Mode-switching and systemd service setup
- [ ] **Phase 4: Security** - Polkit rules and security enhancements

## Phase Details

### Phase 1: Foundation
**Goal**: The TUI can load/save configuration and has basic navigation.
**Depends on**: Nothing
**Requirements**: TOML, SCHEMA, WIZARD, TOOLING, PERM, STACK, FILE-PERM
**Success Criteria**:
  1. User can start the TUI setup command and see a welcome screen.
  2. The TUI can read existing `config.toml` (if any).
  3. The TUI can write a valid (but empty of devices) `config.toml` to `~/.config/nearby/`.
  4. Application checks for Bluetooth permissions on startup.
**Plans**:
- [ ] 01-01-PLAN.md — Foundation - Configuration & Persistence
- [ ] 01-02-PLAN.md — Foundation - TUI Entry & CLI
**UI hint**: yes

### Phase 2: Interaction
**Goal**: Users can discover BLE devices and set thresholds with live feedback.
**Depends on**: Phase 1
**Requirements**: SCAN, LIST, FILTER, SELECT, CALIBRATE, THRESHOLD, ACTION, METADATA, EMPTY, VALID
**Success Criteria**:
  1. User can see a live list of nearby BLE devices with names and RSSI.
  2. User can select a device from the list.
  3. User can see a live RSSI meter for calibration.
  4. User can set "Away" and "Nearby" thresholds with validation.
**Plans**: TBD
**UI hint**: yes

### Phase 3: Integration
**Goal**: The setup can enable/start the background service and notify the daemon.
**Depends on**: Phase 2
**Requirements**: RELOAD, SYSTEMD
**Success Criteria**:
  1. User can choose to enable/start `nearby.service` (user unit) from the TUI.
  2. The TUI detects if the daemon is running and can send a reload signal.
  3. User can exit the setup and have a working configuration for the daemon.
**Plans**: TBD
**UI hint**: yes

### Phase 4: Security
**Goal**: The setup ensures security and proper permissions for system integration.
**Depends on**: Phase 3
**Requirements**: BACKUP, WARNING
**Success Criteria**:
  1. User sees a warning before enabling "unlock" action.
  2. User can choose to backup existing configuration before overwriting.
  3. (Optional) Polkit rules are suggested or verified for locking/unlocking.
**Plans**: TBD
**UI hint**: yes

## Progress Table

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 0/2 | Not started | - |
| 2. Interaction | 0/1 | Not started | - |
| 3. Integration | 0/1 | Not started | - |
| 4. Security | 0/1 | Not started | - |
<!-- generated-by: gsd-doc-writer -->
