use zbus_systemd::login1::ManagerProxy;

pub async fn get_idle_hint() -> zbus::Result<(bool, u64)> {
    let manager = login_manager().await?;
    let idle_hint = manager.idle_hint().await?;
    let idle_hint_time = manager.idle_since_hint().await?;
    Ok((idle_hint, idle_hint_time))
}

pub async fn login_manager() -> zbus::Result<ManagerProxy<'static>> {
    let conn = zbus::Connection::system().await?;
    let manager = ManagerProxy::new(&conn).await?;
    Ok(manager)
}
