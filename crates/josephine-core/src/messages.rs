//! Messages de notification — ton Joséphine, ange gardienne.
//!
//! Bienveillant, direct, un brin d'humour céleste. Jamais alarmiste.

use crate::check::Metric;
use crate::config::CheckThresholds;
use crate::rules::AlertState;

pub fn alert_message(
    check_name: &str,
    metric: &Metric,
    thresholds: &CheckThresholds,
    state: AlertState,
) -> String {
    match check_name {
        "cpu" => cpu_alert(metric.value, state, thresholds),
        "memory" if metric.name == "swap_percent" => swap_alert(metric.value),
        "memory" => memory_alert(metric.value),
        "disk" => disk_alert(metric.value),
        "temperature" => temperature_alert(metric.value, state, metric, thresholds),
        "systemd" if metric.name == "failed_units" => systemd_failed_alert(metric.value),
        "systemd" => systemd_restarts_alert(metric.value),
        "updates" => updates_alert(metric.value),
        "network" => network_alert(metric.value),
        "battery" => battery_alert(metric.value),
        other => format!(
            "Entre nous, {other} me fait un signe ({:.1} {}). \
             Rien de grave… pour l'instant. `josephine doctor` ?",
            metric.value, metric.unit
        ),
    }
}

pub fn recovery_message(check_name: &str, metric: &Metric) -> String {
    match check_name {
        "cpu" => "Ouf ! Votre processeur respire à nouveau. \
                  Je retire une aile du ventilateur — c'est bon."
            .into(),
        "memory" if metric.name == "swap_percent" => format!(
            "Le swap se calme ({:.0} %). Votre machine arrête de compter \
             sur ses réserves — merci pour elle.",
            metric.value
        ),
        "memory" => format!(
            "Votre mémoire se détend ({:.0} %). \
             Tout le monde peut souffler, moi y compris.",
            metric.value
        ),
        "disk" => format!(
            "Votre disque a de l'air ({:.0} %). \
             Même les anges apprécient un peu d'espace libre.",
            metric.value
        ),
        "temperature" => format!(
            "La température redescend ({:.0} °C). \
             Fini la fournaise — votre machine me remercie.",
            metric.value
        ),
        "systemd" if metric.name == "failed_units" => "Tous vos services sont remis sur pied. \
             Moi, je n'ai jamais douté — enfin, presque."
            .into(),
        "systemd" => format!(
            "Les redémarrages se taisent ({:.0}). \
             La stabilité est revenue au poste.",
            metric.value
        ),
        "updates" => "Tout est à jour — votre machine brille comme un sou neuf. \
             Beau travail, on peut être fières."
            .into(),
        "network" => "Le réseau est revenu, fluide et vaillant. \
             Je range mes plumes — tout communique de nouveau."
            .into(),
        "battery" => "Votre batterie a repris des forces (ou vous voilà branché). \
             Ouf — je respire mieux, moi aussi."
            .into(),
        other => format!(
            "Tout est rentré dans l'ordre pour {other} ({:.1} {}). \
             Je reprends ma veille discrète.",
            metric.value, metric.unit
        ),
    }
}

// --- Self-update (`josephine update`) ---------------------------------------

/// Already on the newest published version.
pub fn update_up_to_date(version: &str) -> String {
    format!(
        "Vous avez déjà la dernière version ({version}). \
         Tout est neuf, je n'ai rien à faire — et ça me va très bien."
    )
}

/// Local build is ahead of anything published (a dev build).
pub fn update_ahead(current: &str, latest: &str) -> String {
    format!(
        "Votre version ({current}) devance la dernière publiée ({latest}). \
         Vous avez une longueur d'avance — j'aime cette audace."
    )
}

/// A newer version is available to install.
pub fn update_available(version: &str) -> String {
    format!(
        "Une nouvelle version vous attend : {version}. \
         Un petit coup de neuf et votre ange portera ses plus belles plumes."
    )
}

/// The update finished installing.
pub fn update_done(version: &str) -> String {
    format!(
        "Voilà, Joséphine est passée en {version}. \
         Merci de votre confiance — je reprends ma veille, fraîche et pimpante."
    )
}

fn cpu_alert(value: f64, state: AlertState, thresholds: &CheckThresholds) -> String {
    match state {
        AlertState::Critical => format!(
            "Mon cher… {:.0} % de CPU. \
             Votre machine court plus vite que moi avec mes ailes — \
             et ce n'est pas un compliment.\n\n\
             `josephine doctor`, vite.",
            value
        ),
        AlertState::Warning => format!(
            "Alors là, {:.0} % de CPU (seuil : {:.0} %). \
             Quelque chose s'agite sous le capot.\n\n\
             Un petit `josephine doctor` ?",
            value, thresholds.warning
        ),
        AlertState::Normal => unreachable!(),
    }
}

fn memory_alert(value: f64) -> String {
    format!(
        "Votre mémoire est à {:.0} % — presque pleine. \
         Quelque chose grignote vos ressources, \
         et ce n'est pas moi pendant ma sieste.\n\n\
         `josephine doctor` pour voir qui ?",
        value
    )
}

fn swap_alert(value: f64) -> String {
    format!(
        "Le swap s'emballe ({:.0} %). \
         Votre machine compresse ses idées dans un coin étroit — \
         pas idéal pour réfléchir.\n\n\
         `josephine doctor` peut éclaircir tout ça.",
        value
    )
}

fn disk_alert(value: f64) -> String {
    format!(
        "Votre disque est à {:.0} % — il tousse un peu. \
         Même au paradis, on n'a pas de stockage illimité.\n\n\
         Je peux vous aider à voir ce qui encombre : `josephine doctor`.",
        value
    )
}

fn temperature_alert(
    value: f64,
    state: AlertState,
    metric: &Metric,
    thresholds: &CheckThresholds,
) -> String {
    let limit = metric
        .threshold_critical
        .or(metric.threshold_warning)
        .unwrap_or(thresholds.critical);

    match state {
        AlertState::Critical => format!(
            "{:.0} °C ! Votre machine chauffe plus qu'un four en plein été. \
             Mes ailes ne suffisent pas à la refroidir.\n\n\
             Un coup d'œil : `josephine doctor`.",
            value
        ),
        AlertState::Warning => format!(
            "Il commence à faire chaud ici ({:.0} °C, seuil {:.0} °C). \
             Vos ventilateurs méritent un encouragement.\n\n\
             `josephine doctor` ?",
            value, limit
        ),
        AlertState::Normal => unreachable!(),
    }
}

fn systemd_failed_alert(count: f64) -> String {
    let n = count as u64;
    let services = if n <= 1 {
        "1 service a jeté l'éponge".to_string()
    } else {
        format!("{n} services ont jeté l'éponge")
    };
    format!(
        "Entre nous, {services}. \
         Personne n'est parfait — sauf moi, peut-être.\n\n\
         La liste : `josephine doctor`.",
    )
}

fn updates_alert(count: f64) -> String {
    let n = count as u64;
    let (subject, verb) = if n <= 1 {
        ("mise à jour", "vous attend")
    } else {
        ("mises à jour", "vous attendent")
    };
    format!(
        "{n} {subject} {verb}. \
         Un petit coup de neuf et votre machine sera parée comme un ange.\n\n\
         La liste : `josephine doctor`.",
    )
}

fn network_alert(latency_ms: f64) -> String {
    format!(
        "Votre lien réseau tousse un peu ({latency_ms:.0} ms vers la passerelle, \
         voire plus de réponse du tout). Un coup d'œil au Wi-Fi ou au câble ?\n\n\
         Les détails : `josephine doctor`.",
    )
}

fn battery_alert(depletion_percent: f64) -> String {
    let charge = 100.0 - depletion_percent;
    format!(
        "Votre batterie descend à {charge:.0} %. \
         Un petit branchement et tout le monde respire — pensez au chargeur.\n\n\
         L'état complet : `josephine doctor`.",
    )
}

fn systemd_restarts_alert(count: f64) -> String {
    format!(
        "Un service a redémarré {:.0} fois — \
         il peine à trouver sa place, même pas au ciel.\n\n\
         `josephine doctor` pour comprendre.",
        count
    )
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
        let msg = cpu_alert(96.0, AlertState::Critical, &thresholds());
        for word in forbidden {
            assert!(
                !msg.to_uppercase().contains(word),
                "message contient {word}: {msg}"
            );
        }
    }

    #[test]
    fn recovery_messages_are_warm() {
        let msg = recovery_message("cpu", &sample_metric(40.0));
        assert!(msg.contains("Ouf") || msg.contains("respire"));
    }

    #[test]
    fn alerts_mention_doctor() {
        let msg = disk_alert(92.0);
        assert!(msg.contains("josephine doctor"));
    }

    #[test]
    fn update_messages_stay_warm() {
        let forbidden = ["ERROR", "FATAL", "PANIC", "CRASH", "ÉCHEC"];
        let messages = [
            update_up_to_date("0.2.1"),
            update_ahead("0.3.0", "0.2.1"),
            update_available("0.3.0"),
            update_done("0.3.0"),
        ];
        for msg in messages {
            for word in forbidden {
                assert!(
                    !msg.to_uppercase().contains(word),
                    "message contient {word}: {msg}"
                );
            }
        }
    }
}
