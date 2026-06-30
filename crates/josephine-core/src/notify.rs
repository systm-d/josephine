use anyhow::{Context, Result};
use notify_rust::Notification;

pub fn send_desktop(title: &str, body: &str) -> Result<()> {
    Notification::new()
        .summary(title)
        .body(body)
        .appname("josephine")
        .show()
        .context("envoi de la notification desktop")?;
    Ok(())
}

pub fn send_josephine(message: &str) -> Result<()> {
    send_desktop("✨ Joséphine", message)
}
