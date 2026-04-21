# Architecture Patterns: BLE Proximity Unlock

**Domain:** Proximity locking/unlocking for Linux
**Researched:** 2025-02-12

## Recommended Architecture

A background daemon running as a **systemd user service**. It consists of three primary loops:
1.  **BLE Scanner Loop:** Continuously (or periodically) scans for the registered MAC address and collects RSSI data.
2.  **D-Bus Listener Loop:** Watches for `Lock`/`Unlock` and `IdleHint` signals from `systemd-logind`.
3.  **State Controller:** A state machine that reconciles BLE signal data with session state to decide when to trigger a lock/unlock.

### Component Boundaries

| Component | Responsibility | Communicates With |
| :--- | :--- | :--- |
| **BLE Monitor** (`bluer`) | Scans for devices, filters RSSI. | Bluetooth Controller (HCI) |
| **Session Monitor** (`zbus`) | Tracks `LockedHint` and session signals. | `systemd-logind` |
| **Decision Engine** | Applies hysteresis and threshold logic. | BLE Monitor, Session Monitor |
| **Action Runner** (`zbus`) | Executes `Lock` / `Unlock` methods. | `systemd-logind` |

### Data Flow

1.  **Scanner** emits `RssiEvent(mac, -80dBm)`.
2.  **Decision Engine** receives event, updates internal buffer (e.g., last 5 readings).
3.  **Decision Engine** determines "Out of Range" (if average RSSI < threshold).
4.  **Session Monitor** confirms "Session is Currently Unlocked".
5.  **Action Runner** calls `org.freedesktop.login1.Session.Lock()`.

## Patterns to Follow

### Pattern 1: RSSI Hysteresis (Exponential Moving Average)
Instead of a simple "if RSSI < -80", use an EMA to smooth out jitter.
```rust
fn update_rssi(current: f32, new: f32) -> f32 {
    let alpha = 0.2; // Smoothing factor
    (1.0 - alpha) * current + alpha * new
}
```

### Pattern 2: D-Bus Proxy Generation
Use `zbus` to generate proxies for the `logind` session.
```rust
#[proxy(interface = "org.freedesktop.login1.Session")]
trait Session {
    fn lock(&self) -> zbus::Result<()>;
    fn unlock(&self) -> zbus::Result<()>;
    #[zbus(property)]
    fn locked_hint(&self) -> zbus::Result<bool>;
}
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Blocking the Async Loop
Performing heavy computation or synchronous shell calls within the `tokio` loop.
**Instead:** Use `zbus` asynchronous methods and `tokio::spawn` for independent monitors.

### Anti-Pattern 2: Command Polling
Using `system("loginctl lock-session")` repeatedly.
**Instead:** Monitor D-Bus signals and only send commands when the state *must* change.

## Scalability Considerations

| Concern | 1 User / 1 Device | 1 User / 10 Devices | Multi-User Laptop |
| :--- | :--- | :--- | :--- |
| **CPU/Battery** | Negligible. | Increased scanning. | Managed by individual user services. |
| **Configuration** | simple `config.toml`. | Need device mapping. | `~/.config` per user. |

## Sources

- [zbus signal handling examples](https://docs.rs/zbus/latest/zbus/struct.Connection.html#signals)
- [BlueZ D-Bus API specification](https://github.com/bluez/bluez/blob/master/doc/device-api.txt)
