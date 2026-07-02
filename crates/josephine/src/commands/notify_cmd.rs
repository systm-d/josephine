//! `josephine notify test` — send a desktop notification to confirm libnotify
//! is wired up on this machine.

use anyhow::Result;
use clap::Subcommand;
use josephine_core::i18n;
use josephine_core::notify;

#[derive(Subcommand)]
pub enum NotifyAction {
    /// Send a test desktop notification
    Test,
}

pub fn run(action: NotifyAction) -> Result<()> {
    match action {
        NotifyAction::Test => {
            notify::send_josephine(i18n::t(
                "Hello! If you're reading this, my wings reach your desktop just fine. ✨",
                "Coucou ! Si vous lisez ceci, mes ailes touchent bien votre bureau. ✨",
            ))?;
            println!(
                "{}",
                i18n::t(
                    "✨ Notification sent — take a look at your notifications.",
                    "✨ Notification envoyée — jetez un œil du côté de vos notifications.",
                )
            );
            println!(
                "{}",
                i18n::t(
                    "   Nothing? Check that libnotify / your notification centre is running.",
                    "   Rien vu ? Vérifiez que libnotify / votre centre de notifications tourne.",
                )
            );
        }
    }
    Ok(())
}
