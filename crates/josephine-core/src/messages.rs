//! Notification messages — Joséphine's guardian-angel voice, in English (default)
//! and French. Warm, direct, a touch of celestial humour. Never alarmist.

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
                "Between us, {other} is signalling ({:.1} {}). \
                 Nothing serious… for now. `josephine doctor`?",
                metric.value, metric.unit
            ),
            Lang::Fr => format!(
                "Entre nous, {other} me fait un signe ({:.1} {}). \
                 Rien de grave… pour l'instant. `josephine doctor` ?",
                metric.value, metric.unit
            ),
        },
    }
}

pub fn recovery_message(check_name: &str, metric: &Metric, lang: Lang) -> String {
    match check_name {
        "cpu" => match lang {
            Lang::En => "Phew! Your processor is breathing again. \
                 I'll fold a wing back — all good."
                .into(),
            Lang::Fr => "Ouf ! Votre processeur respire à nouveau. \
                 Je retire une aile du ventilateur — c'est bon."
                .into(),
        },
        "memory" if metric.name == "swap_percent" => match lang {
            Lang::En => format!(
                "Swap is settling ({:.0} %). Your machine stops leaning on its \
                 reserves — thanks on its behalf.",
                metric.value
            ),
            Lang::Fr => format!(
                "Le swap se calme ({:.0} %). Votre machine arrête de compter \
                 sur ses réserves — merci pour elle.",
                metric.value
            ),
        },
        "memory" => match lang {
            Lang::En => format!(
                "Your memory is relaxing ({:.0} %). Everyone can breathe out, \
                 myself included.",
                metric.value
            ),
            Lang::Fr => format!(
                "Votre mémoire se détend ({:.0} %). \
                 Tout le monde peut souffler, moi y compris.",
                metric.value
            ),
        },
        "disk" => match lang {
            Lang::En => format!(
                "Your disk has room again ({:.0} %). Even angels appreciate a \
                 little free space.",
                metric.value
            ),
            Lang::Fr => format!(
                "Votre disque a de l'air ({:.0} %). \
                 Même les anges apprécient un peu d'espace libre.",
                metric.value
            ),
        },
        "temperature" => match lang {
            Lang::En => format!(
                "The temperature is coming down ({:.0} °C). No more furnace — \
                 your machine thanks me.",
                metric.value
            ),
            Lang::Fr => format!(
                "La température redescend ({:.0} °C). \
                 Fini la fournaise — votre machine me remercie.",
                metric.value
            ),
        },
        "systemd" if metric.name == "failed_units" => match lang {
            Lang::En => "All your services are back on their feet. \
                 I never doubted — well, almost."
                .into(),
            Lang::Fr => "Tous vos services sont remis sur pied. \
                 Moi, je n'ai jamais douté — enfin, presque."
                .into(),
        },
        "systemd" => match lang {
            Lang::En => format!(
                "The restarts have gone quiet ({:.0}). Stability is back on duty.",
                metric.value
            ),
            Lang::Fr => format!(
                "Les redémarrages se taisent ({:.0}). \
                 La stabilité est revenue au poste.",
                metric.value
            ),
        },
        "updates" => match lang {
            Lang::En => "Everything's up to date — your machine is shining like new. \
                 Well done, we can be proud."
                .into(),
            Lang::Fr => "Tout est à jour — votre machine brille comme un sou neuf. \
                 Beau travail, on peut être fières."
                .into(),
        },
        "network" => match lang {
            Lang::En => "The network is back, smooth and steady. \
                 I'll put my feathers away — everything's talking again."
                .into(),
            Lang::Fr => "Le réseau est revenu, fluide et vaillant. \
                 Je range mes plumes — tout communique de nouveau."
                .into(),
        },
        "battery" => match lang {
            Lang::En => "Your battery is back to strength (or you're plugged in). \
                 Phew — I breathe easier too."
                .into(),
            Lang::Fr => "Votre batterie a repris des forces (ou vous voilà branché). \
                 Ouf — je respire mieux, moi aussi."
                .into(),
        },
        "inode" => match lang {
            Lang::En => "The inodes have room again — the disk breathes, its files \
                 neatly filed."
                .into(),
            Lang::Fr => "Les inodes ont repris de l'air — le disque respire à nouveau, \
                 ses fichiers bien rangés."
                .into(),
        },
        "smart" => match lang {
            Lang::En => "Your disks look healthy again on the SMART front. \
                 I'll take a breather — you too, I hope."
                .into(),
            Lang::Fr => "Vos disques affichent de nouveau une mine saine côté SMART. \
                 Je souffle un peu — vous aussi, j'espère."
                .into(),
        },
        "kernel" => match lang {
            Lang::En => "The kernel has calmed — no more incidents on the horizon. \
                 Clear skies after the storm."
                .into(),
            Lang::Fr => "Le noyau s'est calmé — plus d'incident à l'horizon. \
                 Le beau temps après l'orage."
                .into(),
        },
        other => match lang {
            Lang::En => format!(
                "All's back to normal for {other} ({:.1} {}). \
                 I return to my quiet watch.",
                metric.value, metric.unit
            ),
            Lang::Fr => format!(
                "Tout est rentré dans l'ordre pour {other} ({:.1} {}). \
                 Je reprends ma veille discrète.",
                metric.value, metric.unit
            ),
        },
    }
}

// --- Self-update (`josephine update`) ---------------------------------------

pub fn update_up_to_date(version: &str, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "You're already on the latest version ({version}). Everything's fresh — \
             nothing for me to do, and I'm quite happy with that."
        ),
        Lang::Fr => format!(
            "Vous avez déjà la dernière version ({version}). \
             Tout est neuf, je n'ai rien à faire — et ça me va très bien."
        ),
    }
}

pub fn update_ahead(current: &str, latest: &str, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "Your version ({current}) is ahead of the latest published one ({latest}). \
             You've got a head start — I like the daring."
        ),
        Lang::Fr => format!(
            "Votre version ({current}) devance la dernière publiée ({latest}). \
             Vous avez une longueur d'avance — j'aime cette audace."
        ),
    }
}

pub fn update_available(version: &str, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "A new version is waiting for you: {version}. A little refresh and your \
             angel will wear her finest feathers."
        ),
        Lang::Fr => format!(
            "Une nouvelle version vous attend : {version}. \
             Un petit coup de neuf et votre ange portera ses plus belles plumes."
        ),
    }
}

pub fn update_done(version: &str, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "There we go, Joséphine is now on {version}. Thank you for your trust — \
             I'm back on watch, fresh and chipper."
        ),
        Lang::Fr => format!(
            "Voilà, Joséphine est passée en {version}. \
             Merci de votre confiance — je reprends ma veille, fraîche et pimpante."
        ),
    }
}

fn cpu_alert(value: f64, state: AlertState, thresholds: &CheckThresholds, lang: Lang) -> String {
    match (state, lang) {
        (AlertState::Critical, Lang::En) => format!(
            "Goodness… {value:.0}% CPU. Your machine is running faster than I can \
             flap my wings — and that's no compliment.\n\n\
             `josephine doctor`, quick."
        ),
        (AlertState::Critical, Lang::Fr) => format!(
            "Mon cher… {value:.0} % de CPU. \
             Votre machine court plus vite que moi avec mes ailes — \
             et ce n'est pas un compliment.\n\n\
             `josephine doctor`, vite."
        ),
        (AlertState::Warning, Lang::En) => format!(
            "Well now, {value:.0}% CPU (threshold: {:.0}%). \
             Something's stirring under the hood.\n\n\
             A quick `josephine doctor`?",
            thresholds.warning
        ),
        (AlertState::Warning, Lang::Fr) => format!(
            "Alors là, {value:.0} % de CPU (seuil : {:.0} %). \
             Quelque chose s'agite sous le capot.\n\n\
             Un petit `josephine doctor` ?",
            thresholds.warning
        ),
        (AlertState::Normal, _) => unreachable!(),
    }
}

fn memory_alert(value: f64, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "Your memory is at {value:.0}% — nearly full. Something's nibbling at \
             your resources, and it isn't me during my nap.\n\n\
             `josephine doctor` to see who?"
        ),
        Lang::Fr => format!(
            "Votre mémoire est à {value:.0} % — presque pleine. \
             Quelque chose grignote vos ressources, \
             et ce n'est pas moi pendant ma sieste.\n\n\
             `josephine doctor` pour voir qui ?"
        ),
    }
}

fn swap_alert(value: f64, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "Swap is racing ({value:.0}%). Your machine is cramming its thoughts \
             into a tight corner — not ideal for thinking.\n\n\
             `josephine doctor` can clear things up."
        ),
        Lang::Fr => format!(
            "Le swap s'emballe ({value:.0} %). \
             Votre machine compresse ses idées dans un coin étroit — \
             pas idéal pour réfléchir.\n\n\
             `josephine doctor` peut éclaircir tout ça."
        ),
    }
}

fn disk_alert(value: f64, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "Your disk is at {value:.0}% — it's coughing a little. Even in heaven, \
             storage isn't unlimited.\n\n\
             I can help you see what's piling up: `josephine doctor`."
        ),
        Lang::Fr => format!(
            "Votre disque est à {value:.0} % — il tousse un peu. \
             Même au paradis, on n'a pas de stockage illimité.\n\n\
             Je peux vous aider à voir ce qui encombre : `josephine doctor`."
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
            "{value:.0}°C! Your machine is hotter than an oven in midsummer. \
             My wings aren't enough to cool it.\n\n\
             Take a look: `josephine doctor`."
        ),
        (AlertState::Critical, Lang::Fr) => format!(
            "{value:.0} °C ! Votre machine chauffe plus qu'un four en plein été. \
             Mes ailes ne suffisent pas à la refroidir.\n\n\
             Un coup d'œil : `josephine doctor`."
        ),
        (AlertState::Warning, Lang::En) => format!(
            "It's getting warm in here ({value:.0}°C, threshold {limit:.0}°C). \
             Your fans deserve some encouragement.\n\n\
             `josephine doctor`?"
        ),
        (AlertState::Warning, Lang::Fr) => format!(
            "Il commence à faire chaud ici ({value:.0} °C, seuil {limit:.0} °C). \
             Vos ventilateurs méritent un encouragement.\n\n\
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
                "1 service threw in the towel".to_string()
            } else {
                format!("{n} services threw in the towel")
            };
            format!(
                "Between us, {services}. Nobody's perfect — except me, perhaps.\n\n\
                 The list: `josephine doctor`."
            )
        }
        Lang::Fr => {
            let services = if n <= 1 {
                "1 service a jeté l'éponge".to_string()
            } else {
                format!("{n} services ont jeté l'éponge")
            };
            format!(
                "Entre nous, {services}. \
                 Personne n'est parfait — sauf moi, peut-être.\n\n\
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
                "{n} {subject} {verb}. A little refresh and your machine will be \
                 dressed like an angel.\n\n\
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
                "{n} {subject} {verb}. \
                 Un petit coup de neuf et votre machine sera parée comme un ange.\n\n\
                 La liste : `josephine doctor`."
            )
        }
    }
}

fn network_alert(latency_ms: f64, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "Your network link is coughing a little ({latency_ms:.0} ms to the \
             gateway, or no answer at all). A glance at the Wi-Fi or the cable?\n\n\
             The details: `josephine doctor`."
        ),
        Lang::Fr => format!(
            "Votre lien réseau tousse un peu ({latency_ms:.0} ms vers la passerelle, \
             voire plus de réponse du tout). Un coup d'œil au Wi-Fi ou au câble ?\n\n\
             Les détails : `josephine doctor`."
        ),
    }
}

fn battery_alert(depletion_percent: f64, lang: Lang) -> String {
    let charge = 100.0 - depletion_percent;
    match lang {
        Lang::En => format!(
            "Your battery is down to {charge:.0}%. A quick plug-in and everyone \
             breathes again — mind the charger.\n\n\
             The full picture: `josephine doctor`."
        ),
        Lang::Fr => format!(
            "Votre batterie descend à {charge:.0} %. \
             Un petit branchement et tout le monde respire — pensez au chargeur.\n\n\
             L'état complet : `josephine doctor`."
        ),
    }
}

fn inode_alert(percent: f64, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "Inodes are filling up ({percent:.0}%). A swarm of tiny files is \
             smothering the disk — even with free space.\n\n\
             `josephine doctor` to spot the partition."
        ),
        Lang::Fr => format!(
            "Les inodes se remplissent ({percent:.0} %). \
             Une nuée de petits fichiers étouffe le disque — même avec de l'espace libre.\n\n\
             `josephine doctor` pour repérer la partition."
        ),
    }
}

fn smart_alert(failing: f64, lang: Lang) -> String {
    let n = failing as u64;
    match lang {
        Lang::En => format!(
            "{n} disk(s) are reporting a SMART weakness. Back up without delay — \
             forewarned is forearmed.\n\n\
             The details: `josephine doctor`."
        ),
        Lang::Fr => format!(
            "{n} disque(s) signalent une faiblesse SMART. \
             Sauvegardez sans tarder — un disque prévenu en vaut deux.\n\n\
             Le détail : `josephine doctor`."
        ),
    }
}

fn kernel_alert(count: f64, lang: Lang) -> String {
    let n = count as u64;
    match lang {
        Lang::En => format!(
            "The kernel stumbled {n} times this hour (OOM, oops…). Something's \
             rattling your machine under the hood.\n\n\
             `josephine doctor` to see more clearly."
        ),
        Lang::Fr => format!(
            "Le noyau a bronché {n} fois cette heure (OOM, oops…). \
             Quelque chose secoue votre machine sous le capot.\n\n\
             `josephine doctor` pour y voir plus clair."
        ),
    }
}

fn systemd_restarts_alert(count: f64, lang: Lang) -> String {
    match lang {
        Lang::En => format!(
            "A service has restarted {count:.0} times — it's struggling to find its \
             place, not even in the sky.\n\n\
             `josephine doctor` to understand."
        ),
        Lang::Fr => format!(
            "Un service a redémarré {count:.0} fois — \
             il peine à trouver sa place, même pas au ciel.\n\n\
             `josephine doctor` pour comprendre."
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
