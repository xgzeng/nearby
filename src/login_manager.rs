use std::time::Duration;
use std::time::UNIX_EPOCH;
use zbus_systemd::login1::{ManagerProxy, SeatProxy, SessionProxy};

enum SessionFilter {
    Seat(String),
    All,
}

pub struct LoginManager {
    manager: ManagerProxy<'static>,
    conn: zbus::Connection,
    session_type: SessionFilter,
}

pub async fn list_seat_names() -> zbus::Result<Vec<String>> {
    let conn = zbus::Connection::system().await?;
    let manager = ManagerProxy::new(&conn).await?;
    let seats = manager.list_seats().await?;
    let seat_names = seats
        .into_iter()
        .map(|(seat, _)| seat)
        .collect::<Vec<String>>();
    Ok(seat_names)
}

impl LoginManager {
    pub async fn new_for_all() -> zbus::Result<Self> {
        let conn = zbus::Connection::system().await?;
        let manager = ManagerProxy::new(&conn).await?;
        Ok(Self {
            manager,
            conn,
            session_type: SessionFilter::All,
        })
    }

    pub async fn new_for_seat(seat: &str) -> zbus::Result<Self> {
        let conn = zbus::Connection::system().await?;
        let manager = ManagerProxy::new(&conn).await?;
        Ok(Self {
            manager,
            conn,
            session_type: SessionFilter::Seat(seat.to_string()),
        })
    }

    pub async fn get_idle_hint(&self) -> zbus::Result<(bool, u64)> {
        match &self.session_type {
            SessionFilter::Seat(seat) => {
                let path = self.manager.get_seat(seat.to_string()).await?;
                let seat_proxy = SeatProxy::builder(&self.conn).path(path)?.build().await?;
                let idle_hint = seat_proxy.idle_hint().await?;
                let idle_hint_time = seat_proxy.idle_since_hint().await?;
                Ok((idle_hint, idle_hint_time))
            }
            SessionFilter::All => {
                let idle_hint = self.manager.idle_hint().await?;
                let idle_hint_time = self.manager.idle_since_hint().await?;
                Ok((idle_hint, idle_hint_time))
            }
        }
    }

    pub async fn get_idle_hint_info(&self) -> zbus::Result<(bool, Duration)> {
        let (idle, idle_since_micros) = self.get_idle_hint().await?;
        let idle_since = UNIX_EPOCH + Duration::from_micros(idle_since_micros);
        let idle_for = std::time::SystemTime::now()
            .duration_since(idle_since)
            .unwrap_or(Duration::from_secs(0));
        Ok((idle, idle_for))
    }

    async fn lock_all(&self) -> zbus::Result<()> {
        self.manager.lock_sessions().await?;
        Ok(())
    }

    async fn unlock_all(&self) -> zbus::Result<()> {
        self.manager.unlock_sessions().await?;
        Ok(())
    }

    // Helper function to find the session proxy for a given seat
    async fn seat_session(&self, seat: &str) -> zbus::Result<Option<SessionProxy<'_>>> {
        let sessions = self.manager.list_sessions().await?;
        for (_, _, _, session_seat, path) in sessions {
            if session_seat == seat {
                let session = SessionProxy::builder(&self.conn)
                    .path(path)?
                    .build()
                    .await?;
                return Ok(Some(session));
            }
        }

        Ok(None)
    }

    /// Helper to lock either a specific seat (if configured) or all sessions
    pub async fn lock(&self) -> zbus::Result<()> {
        match &self.session_type {
            SessionFilter::Seat(s) => {
                if let Some(session) = self.seat_session(s).await? {
                    if let Err(e) = session.lock().await {
                        log::error!("Failed to lock session for seat {}: {}", s, e);
                    }
                } else {
                    log::warn!("No session found for seat {}", s);
                }
            }
            SessionFilter::All => self.lock_all().await?,
        }
        Ok(())
    }

    /// Helper to unlock either a specific seat (if configured) or all sessions
    pub async fn unlock(&self) -> zbus::Result<()> {
        match &self.session_type {
            SessionFilter::Seat(s) => {
                if let Some(session) = self.seat_session(s).await? {
                    if let Err(e) = session.unlock().await {
                        log::error!("Failed to unlock session for seat {}: {}", s, e);
                    }
                } else {
                    log::warn!("No session found for seat {}", s);
                }
            }
            SessionFilter::All => self.unlock_all().await?,
        }
        Ok(())
    }
}
