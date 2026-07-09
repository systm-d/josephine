//! Notification messages, in English (default) and French. Direct, calm,
//! reassuring. Never alarmist.

use crate::check::Metric;
use crate::config::CheckThresholds;
use crate::i18n::Lang;
use crate::rules::AlertState;

pub fn alert_message(
    check_name: &str,
    metric: &Metric,
    thresholds: &CheckThresholds,
    state: AlertState,
    lang: Lang,
) -> String {
    match check_name {
        "cpu" => cpu_alert(metric.value, state, thresholds, lang),
        "memory" if metric.name == "swap_percent" => swap_alert(metric.value, lang),
        "memory" => memory_alert(metric.value, lang),
        "disk" => disk_alert(metric.value, lang),
        "temperature" => temperature_alert(metric.value, state, metric, thresholds, lang),
        "systemd" if metric.name == "failed_units" => systemd_failed_alert(metric.value, lang),
        "systemd" => systemd_restarts_alert(metric.value, lang),
        "updates" => updates_alert(metric.value, lang),
        "network" => network_alert(metric.value, lang),
        "battery" => battery_alert(metric.value, lang),
        "inode" => inode_alert(metric.value, lang),
        "smart" => smart_alert(metric.value, lang),
        "kernel" => kernel_alert(metric.value, lang),
        other => match lang {
            Lang::En => format!(
                "{other} is out of range ({:.1} {}). \
                 Nothing serious yet — `josephine doctor`?",
                metric.value, metric.unit
            ),
            Lang::Fr => format!(
                "{other} sort de sa plage ({:.1} {}). \
                 Rien de grave pour l'instant — `josephine doctor` ?",
                metric.value, metric.unit
            ),
        },
    }
}

pub fn recovery_message(check_name: &str, metric: &Metric, lang: Lang) -> String {
    match check_name {
        "cpu" => match lang {
            Lang::En => "Your processor is breathing again — CPU load is back to normal.".into(),
            Lang::Fr => {
                "Votre processeur respire à nouveau — la charge CPU est revenue à la normale."
                    .into()
            }
        },
        "memory" if metric.name == "swap_percent" => match lang {
            Lang::En => format!("Swap is back to normal ({:.0} %).", metric.value),
            Lang::Fr => format!("Le swap est revenu à la normale ({:.0} %).", metric.value),
        },
        "memory" => match lang {
            Lang::En => format!("Memory usage is back to normal ({:.0} %).", metric.value),
            Lang::Fr => format!(
                "L'utilisation mémoire est revenue à la normale ({:.0} %).",
                metric.value
            ),
        },
        "disk" => match lang {
            Lang::En => format!("Your disk has room again ({:.0} %).", metric.value),
            Lang::Fr => format!(
                "Votre disque a de nouveau de la place ({:.0} %).",
                metric.value
            ),
        },
        "temperature" => match lang {
            Lang::En => format!("The temperature is coming down ({:.0} °C).", metric.value),
            Lang::Fr => format!("La température redescend ({:.0} °C).", metric.value),
        },
        "systemd" if metric.name == "failed_units" => match lang {
            Lang::En => "All your services are back up.".into(),
            Lang::Fr => "Tous vos services sont de nouveau opérationnels.".into(),
        },
        "systemd" => match lang {
            Lang::En => format!(
                "Restarts have stopped ({:.0}). Stability is back.",
                metric.value
            ),
            Lang::Fr => format!(
                "Les redémarrages se sont arrêtés ({:.0}). La stabilité est revenue.",
                metric.value
            ),
        },
        "updates" => match lang {
            Lang::En => "Everything is up to date.".into(),
            Lang::Fr => "Tout est à jour.".into(),
        },
        "network" => match lang {
            Lang::En => "The network is back, stable and steady.".into(),
            Lang::Fr => "Le réseau est revenu, stable et fluide.".into(),
        },
        "battery" => match lang {
            Lang::En => "Your battery is back to a healthy level (or you're plugged in).".into(),
            Lang::Fr => {
                "Votre batterie est revenue à un niveau correct (ou vous êtes branché).".into()
            }
        },
        "inode" => match lang {
            Lang::En => "Inodes have room again on the disk.".into(),
            Lang::Fr => "Les inodes ont de nouveau de la place sur le disque.".into(),
        },
        "smart" => match lang {
            Lang::En => "Your disks look healthy again on the SMART front.".into(),
            Lang::Fr => "Vos disques sont de nouveau sains côté SMART.".into(),
        },
        "kernel" => match lang {
            Lang::En => "No more kernel incidents in the last hour.".into(),
            Lang::Fr => "Plus aucun incident noyau sur la dernière heure.".into(),
        },
        other => match lang {
            Lang::En => format!(
                "{other} is back to normal ({:.1} {}).",
                metric.value, metric.unit
            ),
            Lang::Fr => format!(
                "{other} est revenu à la normale ({:.1} {}).",
                metric.value, metric.unit
            ),
        },
    }
}

// --- Self-update (`josephine update`) ---------------------------------------

pub fn update_up_to_date(version: &str, lang: Lang) -> String {
    match lang {
        Lang::En => format!("You're already on the latest version ({version})."),
        Lang::Fr => format!("Vous avez déjà la dernière version ({version})."),
    }
}

pub fn update_ahead(current: &str, latest: &str, lang: Lang) -> String {
    match lang {
        Lang::En => {
            format!("Your version ({current}) is ahead of the latest published one ({latest}).")
        }
        Lang::Fr => format!("Votre version ({current}) devance la dernière publiée ({latest})."),
    }
}

pub fn update_available(version: &str, lang: Lang) -> String {
    match lang {
        Lang::En => format!("A new version is available: {version}."),
        Lang::Fr => format!("Une nouvelle version est disponible : {version}."),
    }
}

pub fn update_done(version: &str, lang: Lang) -> String {
    match lang {
        Lang::En => format!("Update complete — Joséphine is now on {version}."),
        Lang::Fr => {
            format!("Mise à jour terminée — Joséphine est maintenant en version {version}.")
        }
    }
}

fn cpu_alert(value: f64, state: AlertState, thresholds: &CheckThresholds, lang: Lang) -> String {
    match (state, lang) {
        (AlertState::Critical, Lang::En) => format!(
            "CPU usage is at {value:.0}% — critical. Your machine is under heavy load.\n\n\
             `josephine doctor`, now."
        ),
        (AlertState::Critical, Lang::Fr) => format!(
            "Le CPU est à {value:.0} % — critique. \
             Votre machine est sous forte charge.\n\n\
             `josephine doctor`, maintenant."
        ),
        (AlertState::Warning, Lang::En) => format!(
            "CPU usage is at {value:.0}% (threshold: {:.0}%). Worth a look.\n\n\
             `josephine doctor`?",
            thresholds.warning
        ),
        (AlertState::Warning, Lang::Fr) => format!(
            "Le CPU est à {value:.0} % (seuil : {:.0} %). Ça mérite un coup d'œil.\n\n\
             `josephine doctor` ?",
            thresholds.warning
        ),
        (AlertState::Normal, _) => unreachable!(),
    }
}

fn memory_alert(value: f64, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "Memory is at {value:.0}% — nearly full.\n\n\
             `josephine doctor` to see what's using it."
        ),
        Lang::Fr => format!(
            "La mémoire est à {value:.0} % — presque pleine.\n\n\
             `josephine doctor` pour voir ce qui l'utilise."
        ),
    }
}

fn swap_alert(value: f64, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "Swap usage is high ({value:.0}%).\n\n\
             `josephine doctor` for more detail."
        ),
        Lang::Fr => format!(
            "L'utilisation du swap est élevée ({value:.0} %).\n\n\
             `josephine doctor` pour en savoir plus."
        ),
    }
}

fn disk_alert(value: f64, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "Disk usage is at {value:.0}% — space is running low.\n\n\
             See what's taking up room: `josephine doctor`."
        ),
        Lang::Fr => format!(
            "Le disque est à {value:.0} % — l'espace se raréfie.\n\n\
             Voyez ce qui prend de la place : `josephine doctor`."
        ),
    }
}

fn temperature_alert(
    value: f64,
    state: AlertState,
    metric: &Metric,
    thresholds: &CheckThresholds,
    lang: Lang,
) -> String {
    let limit = metric
        .threshold_critical
        .or(metric.threshold_warning)
        .unwrap_or(thresholds.critical);

    match (state, lang) {
        (AlertState::Critical, Lang::En) => format!(
            "{value:.0}°C — critical. Your machine is running very hot.\n\n\
             Take a look: `josephine doctor`."
        ),
        (AlertState::Critical, Lang::Fr) => format!(
            "{value:.0} °C — critique. Votre machine chauffe fortement.\n\n\
             Un coup d'œil : `josephine doctor`."
        ),
        (AlertState::Warning, Lang::En) => format!(
            "Temperature is rising ({value:.0}°C, threshold {limit:.0}°C).\n\n\
             `josephine doctor`?"
        ),
        (AlertState::Warning, Lang::Fr) => format!(
            "La température monte ({value:.0} °C, seuil {limit:.0} °C).\n\n\
             `josephine doctor` ?"
        ),
        (AlertState::Normal, _) => unreachable!(),
    }
}

fn systemd_failed_alert(count: f64, lang: Lang) -> String {
    let n = count as u64;
    match lang {
        Lang::En => {
            let services = if n <= 1 {
                "1 service has stopped".to_string()
            } else {
                format!("{n} services have stopped")
            };
            format!(
                "{services}.\n\n\
                 The list: `josephine doctor`."
            )
        }
        Lang::Fr => {
            let services = if n <= 1 {
                "1 service s'est arrêté".to_string()
            } else {
                format!("{n} services se sont arrêtés")
            };
            format!(
                "{services}.\n\n\
                 La liste : `josephine doctor`."
            )
        }
    }
}

fn updates_alert(count: f64, lang: Lang) -> String {
    let n = count as u64;
    match lang {
        Lang::En => {
            let (subject, verb) = if n <= 1 {
                ("update", "is waiting for you")
            } else {
                ("updates", "are waiting for you")
            };
            format!(
                "{n} {subject} {verb}.\n\n\
                 The list: `josephine doctor`."
            )
        }
        Lang::Fr => {
            let (subject, verb) = if n <= 1 {
                ("mise à jour", "vous attend")
            } else {
                ("mises à jour", "vous attendent")
            };
            format!(
                "{n} {subject} {verb}.\n\n\
                 La liste : `josephine doctor`."
            )
        }
    }
}

fn network_alert(latency_ms: f64, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "Your network link is unstable ({latency_ms:.0} ms to the gateway, \
             or no response at all).\n\n\
             Check the Wi-Fi or the cable: `josephine doctor`."
        ),
        Lang::Fr => format!(
            "Le lien réseau est instable ({latency_ms:.0} ms vers la passerelle, \
             voire aucune réponse).\n\n\
             Vérifiez le Wi-Fi ou le câble : `josephine doctor`."
        ),
    }
}

fn battery_alert(depletion_percent: f64, lang: Lang) -> String {
    let charge = 100.0 - depletion_percent;
    match lang {
        Lang::En => format!(
            "Your battery is down to {charge:.0}%.\n\n\
             Plug in when you can: `josephine doctor` for the full picture."
        ),
        Lang::Fr => format!(
            "Votre batterie descend à {charge:.0} %.\n\n\
             Branchez-la quand vous pouvez : `josephine doctor` pour le détail."
        ),
    }
}

fn inode_alert(percent: f64, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "Inodes are filling up ({percent:.0}%) — many small files, even with \
             free space left.\n\n\
             `josephine doctor` to spot the partition."
        ),
        Lang::Fr => format!(
            "Les inodes se remplissent ({percent:.0} %) — de nombreux petits fichiers, \
             même avec de l'espace libre.\n\n\
             `josephine doctor` pour repérer la partition."
        ),
    }
}

fn smart_alert(failing: f64, lang: Lang) -> String {
    let n = failing as u64;
    match lang {
        Lang::En => format!(
            "{n} disk(s) are reporting a SMART weakness.\n\n\
             Back up without delay: `josephine doctor` for the details."
        ),
        Lang::Fr => format!(
            "{n} disque(s) signalent une faiblesse SMART.\n\n\
             Sauvegardez sans tarder : `josephine doctor` pour le détail."
        ),
    }
}

fn kernel_alert(count: f64, lang: Lang) -> String {
    let n = count as u64;
    match lang {
        Lang::En => format!(
            "{n} kernel incident(s) this hour (OOM, oops…).\n\n\
             `josephine doctor` for more detail."
        ),
        Lang::Fr => format!(
            "{n} incident(s) noyau cette heure (OOM, oops…).\n\n\
             `josephine doctor` pour en savoir plus."
        ),
    }
}

fn systemd_restarts_alert(count: f64, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "A service has restarted {count:.0} times.\n\n\
             `josephine doctor` to see why."
        ),
        Lang::Fr => format!(
            "Un service a redémarré {count:.0} fois.\n\n\
             `josephine doctor` pour comprendre pourquoi."
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::Metric;

    fn sample_metric(value: f64) -> Metric {
        Metric {
            name: "usage_percent".into(),
            value,
            unit: "%".into(),
            threshold_warning: Some(85.0),
            threshold_critical: Some(95.0),
        }
    }

    fn thresholds() -> CheckThresholds {
        CheckThresholds {
            enabled: true,
            interval_secs: 30,
            warning: 85.0,
            critical: 95.0,
        }
    }

    #[test]
    fn messages_never_use_alarmist_vocabulary() {
        let forbidden = ["ERROR", "FATAL", "PANIC", "CRASH", "ÉCHEC"];
        for lang in [Lang::En, Lang::Fr] {
            let msg = cpu_alert(96.0, AlertState::Critical, &thresholds(), lang);
            for word in forbidden {
                assert!(
                    !msg.to_uppercase().contains(word),
                    "{lang:?} message contains {word}: {msg}"
                );
            }
        }
    }

    #[test]
    fn recovery_messages_are_warm() {
        assert!(recovery_message("cpu", &sample_metric(40.0), Lang::En).contains("breathing"));
        assert!(recovery_message("cpu", &sample_metric(40.0), Lang::Fr).contains("respire"));
    }

    #[test]
    fn alerts_mention_doctor_in_both_languages() {
        for lang in [Lang::En, Lang::Fr] {
            assert!(disk_alert(92.0, lang).contains("josephine doctor"));
        }
    }

    #[test]
    fn update_messages_stay_warm() {
        let forbidden = ["ERROR", "FATAL", "PANIC", "CRASH", "ÉCHEC"];
        for lang in [Lang::En, Lang::Fr] {
            let messages = [
                update_up_to_date("0.5.0", lang),
                update_ahead("0.6.0", "0.5.0", lang),
                update_available("0.6.0", lang),
                update_done("0.6.0", lang),
            ];
            for msg in messages {
                for word in forbidden {
                    assert!(!msg.to_uppercase().contains(word), "alarmist: {msg}");
                }
            }
        }
    }
}
