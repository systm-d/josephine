//! Starter configurations for the kind of machine Joséphine is watching.
//!
//! The defaults suit a laptop, because that is where she started. They are a
//! poor fit elsewhere: a desktop has no battery to report on, and a server
//! wants to hear about a failed unit or a filling disk sooner than a personal
//! machine does — while never being woken about a battery it does not have.
//!
//! A profile is a *starting point*, not a mode: `config init --profile server`
//! writes a plain config file the user then owns and edits. Nothing reads the
//! profile name afterwards.

use std::fmt;
use std::str::FromStr;

use crate::config::Config;
use crate::i18n;

/// The kind of machine a starter config is written for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// A personal portable machine: the defaults, battery included.
    Laptop,
    /// A personal machine on mains power: no battery to watch.
    Desktop,
    /// A machine that serves other people: services and storage matter more,
    /// and nobody is sitting in front of it.
    Server,
}

impl Profile {
    pub const ALL: [Profile; 3] = [Profile::Laptop, Profile::Desktop, Profile::Server];

    pub fn name(self) -> &'static str {
        match self {
            Profile::Laptop => "laptop",
            Profile::Desktop => "desktop",
            Profile::Server => "server",
        }
    }

    /// One line, in the user's language, for `--help` and the written file.
    pub fn description(self) -> &'static str {
        match self {
            Profile::Laptop => i18n::t(
                "A portable machine: battery and temperature watched, the defaults.",
                "Une machine portable : batterie et température surveillées, les réglages par défaut.",
            ),
            Profile::Desktop => i18n::t(
                "A machine on mains power: no battery, everything else as usual.",
                "Une machine sur secteur : pas de batterie, le reste comme d'habitude.",
            ),
            Profile::Server => i18n::t(
                "A machine that serves others: services and storage watched closely, no battery, no desktop notification.",
                "Une machine qui sert les autres : services et stockage surveillés de près, pas de batterie, pas de notification bureau.",
            ),
        }
    }

    /// The starter configuration for this kind of machine.
    pub fn config(self) -> Config {
        let mut config = Config::default();
        match self {
            // The defaults were written for a laptop; nothing to change.
            Profile::Laptop => {}

            Profile::Desktop => {
                // There is no battery to report on, and a check that always
                // says "no battery" is a line of noise on every run.
                config.checks.battery.enabled = false;
            }

            Profile::Server => {
                config.checks.battery.enabled = false;

                // Nobody is sitting in front of it, so libnotify has no one to
                // talk to; the journal is where a server's alerts belong.
                config.notifications.desktop = false;
                config.notifications.terminal = true;

                // A failed unit is the thing a server is for. Warn on the
                // first one, and treat two as critical rather than three.
                config.checks.systemd.failed_critical = 2.0;
                config.checks.systemd.interval_secs = 60;

                // Storage fills quietly and takes the service down with it, so
                // look sooner and react earlier than on a personal machine.
                config.checks.disk.warning = 80.0;
                config.checks.disk.critical = 90.0;
                config.checks.disk.interval_secs = 60;
                config.checks.inode.warning = 80.0;
                config.checks.inode.critical = 90.0;
                config.checks.inode.interval_secs = 120;

                // Failed authentications matter more on a reachable machine.
                config.checks.security.interval_secs = 300;

                // Keep longer history: a server's slow trends are the point,
                // and the forecast has more to fit a line to.
                config.history.retention_days = 90;
            }
        }
        config
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for Profile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "laptop" => Ok(Profile::Laptop),
            "desktop" => Ok(Profile::Desktop),
            "server" => Ok(Profile::Server),
            other => Err(match i18n::lang() {
                i18n::Lang::En => {
                    format!("unknown profile `{other}` — I know laptop, desktop and server")
                }
                i18n::Lang::Fr => {
                    format!("profil inconnu « {other} » — je connais laptop, desktop et server")
                }
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preset_is_valid() {
        // The guard that matters: a preset that fails `validate` would write a
        // config file Joséphine then refuses to start with.
        for profile in Profile::ALL {
            profile
                .config()
                .validate()
                .unwrap_or_else(|e| panic!("preset `{profile}` is invalid: {e}"));
        }
    }

    #[test]
    fn laptop_is_the_defaults() {
        assert_eq!(Profile::Laptop.config(), Config::default());
    }

    #[test]
    fn machines_on_mains_power_do_not_watch_a_battery() {
        assert!(!Profile::Desktop.config().checks.battery.enabled);
        assert!(!Profile::Server.config().checks.battery.enabled);
        assert!(Profile::Laptop.config().checks.battery.enabled);
    }

    #[test]
    fn a_server_watches_services_and_storage_more_closely() {
        let server = Profile::Server.config();
        let default = Config::default();

        assert!(server.checks.disk.warning < default.checks.disk.warning);
        assert!(server.checks.inode.warning < default.checks.inode.warning);
        assert!(server.checks.systemd.failed_critical < default.checks.systemd.failed_critical);
        assert!(server.checks.systemd.interval_secs < default.checks.systemd.interval_secs);
    }

    #[test]
    fn a_server_speaks_to_the_journal_not_a_desktop() {
        let server = Profile::Server.config();
        assert!(!server.notifications.desktop);
        assert!(server.notifications.terminal);
    }

    #[test]
    fn names_round_trip() {
        for profile in Profile::ALL {
            assert_eq!(Profile::from_str(profile.name()).unwrap(), profile);
        }
        assert_eq!(Profile::from_str("  SERVER ").unwrap(), Profile::Server);
        assert!(Profile::from_str("toaster").is_err());
    }
}
