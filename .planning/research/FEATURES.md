# Feature Landscape: BLE Proximity Unlock

**Domain:** Proximity locking/unlocking for Linux
**Researched:** 2025-02-12

## Table Stakes

Features users expect. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
| :--- | :--- | :--- | :--- |
| **Auto-Lock** | Core value: locks PC when user walks away. | Low | Use `RSSI` threshold + `loginctl lock-session`. |
| **Configurable Thresholds** | Every environment (office vs home) has different radio ranges. | Low | Need a way to set `near` and `far` RSSI values. |
| **Session Status Check** | Prevents redundant lock commands if already locked. | Medium | Use D-Bus to check `LockedHint` or `Lock` signals. |
| **Paired Device Selection** | Users have multiple devices; need to select which one triggers the lock. | Medium | Discovery phase to list and pick a MAC address. |

## Differentiators

Features that set product apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
| :--- | :--- | :--- | :--- |
| **Hysteresis Filtering** | Prevents "ghost locks" when signal momentarily drops. | Medium | Require $N$ consecutive "out of range" readings. |
| **Smart Unlock** | Automatically unlocks when user approaches. | High | **Security risk!** Requires pairing and potentially secure handshake. |
| **Multi-Device Support** | "Lock if *all* devices away, unlock if *any* approach." | Medium | Logical AND/OR for multiple MAC addresses. |
| **Idle Timeout Override** | Disable proximity locking while media is playing or user is active. | Medium | Use `IdleHint` from `logind`. |
| **Battery Saver Mode** | Dynamic scanning interval based on proximity. | High | Slow down scans when far away; speed up when close. |

## Anti-Features

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
| :--- | :--- | :--- |
| **Password Injection** | Simulating keyboard to type password is insecure. | Use `loginctl unlock-session` or PAM integration. |
| **WIFI-based Proximity** | RSSI in WIFI is too unreliable for distance. | Stick to BLE RSSI. |
| **Cloud Sync** | No reason to store device MACs in the cloud. | Keep configuration local. |

## Feature Dependencies

```mermaid
Device Pairing → Presence Detection → RSSI Monitoring → Lock Trigger
Session State Tracking → Lock/Unlock Decisions
```

## MVP Recommendation

Prioritize:
1. **Presence Detection:** Basic scan for a specific MAC address.
2. **Auto-Lock:** Trigger lock when MAC address is not found for $X$ seconds.
3. **RSSI Thresholds:** Fine-tune distance for locking.
4. **Session Status Check:** Avoid redundant D-Bus calls.

Defer: **Auto-Unlock** (high security risk, implement as optional/experimental later).

## Sources

- [BLEUnlock GitHub](https://github.com/the-maldridge/bleunlock) (Similar project for reference)
- [BlueProximity Arch Wiki](https://wiki.archlinux.org/title/BlueProximity) (Legacy features list)
