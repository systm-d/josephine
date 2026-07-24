use anyhow::{Context, Result};
use notify_rust::Notification;

use crate::i18n;

pub fn send_desktop(title: &str, body: &str) -> Result<()> {
    Notification::new()
        .summary(title)
        .body(body)
        .appname("josephine")
        .show()
        .context(i18n::t(
            "sending the desktop notification",
            "envoi de la notification desktop",
        ))?;
    Ok(())
}

pub fn send_josephine(message: &str) -> Result<()> {
    send_desktop("✦ Joséphine", message)
}
