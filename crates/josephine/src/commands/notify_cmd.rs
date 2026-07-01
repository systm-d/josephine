//! `josephine notify test` — send a desktop notification to confirm libnotify
//! is wired up on this machine.

use anyhow::Result;
use clap::Subcommand;
use josephine_core::notify;

#[derive(Subcommand)]
pub enum NotifyAction {
    /// Envoie une notification de test sur le bureau
    Test,
}

pub fn run(action: NotifyAction) -> Result<()> {
    match action {
        NotifyAction::Test => {
            notify::send_josephine(
                "Coucou ! Si vous lisez ceci, mes ailes touchent bien votre bureau. ✨",
            )?;
            println!("✨ Notification envoyée — jetez un œil du côté de vos notifications.");
            println!("   Rien vu ? Vérifiez que libnotify / votre centre de notifications tourne.");
        }
    }
    Ok(())
}
