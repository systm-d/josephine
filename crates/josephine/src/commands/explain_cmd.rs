//! `josephine explain` — what each check watches, why it matters, and how to act.

use anyhow::Result;
use josephine_core::i18n::{self, Lang};

use crate::output::{check_label, sober_header};

struct CheckExplanation {
    name: &'static str,
    what: (&'static str, &'static str),
    why: (&'static str, &'static str),
    remedy: (&'static str, &'static str),
}

const CHECKS: &[CheckExplanation] = &[
    CheckExplanation {
        name: "cpu",
        what: (
            "Processor load and the busiest processes.",
            "Charge processeur et processus les plus actifs.",
        ),
        why: (
            "Sustained high CPU can slow everything down or point to a runaway process.",
            "Une charge CPU élevée ralentit tout ou signale un processus incontrôlé.",
        ),
        remedy: (
            "`josephine doctor` lists the top consumers; investigate or stop the culprit.",
            "`josephine doctor` liste les plus gourmands ; identifiez ou arrêtez le coupable.",
        ),
    },
    CheckExplanation {
        name: "memory",
        what: (
            "RAM and swap usage, plus the hungriest processes.",
            "Utilisation RAM et swap, et processus les plus gourmands.",
        ),
        why: (
            "Low free memory triggers swapping and OOM kills — work slows or apps vanish.",
            "Peu de mémoire libre provoque du swap et des OOM — tout ralentit ou des apps disparaissent.",
        ),
        remedy: (
            "`josephine doctor` shows who uses the RAM; close heavy apps or add memory.",
            "`josephine doctor` montre qui consomme la RAM ; fermez les apps lourdes ou ajoutez de la mémoire.",
        ),
    },
    CheckExplanation {
        name: "disk",
        what: (
            "Free space on each mounted partition.",
            "Espace libre sur chaque partition montée.",
        ),
        why: (
            "A full disk stops writes, breaks updates and can corrupt databases.",
            "Un disque plein bloque les écritures, casse les mises à jour et peut corrompre des bases.",
        ),
        remedy: (
            "`josephine clean` previews reclaimable space; `josephine doctor` shows which partition.",
            "`josephine clean` estime l'espace récupérable ; `josephine doctor` indique la partition.",
        ),
    },
    CheckExplanation {
        name: "temperature",
        what: (
            "CPU and NVMe sensor temperatures.",
            "Températures des capteurs CPU et NVMe.",
        ),
        why: (
            "Overheating throttles performance and shortens hardware life.",
            "La surchauffe limite les performances et réduit la durée de vie du matériel.",
        ),
        remedy: (
            "Check airflow and dust; `sensors` for detail. Laptop on a soft surface? Move it.",
            "Vérifiez ventilation et poussière ; `sensors` pour le détail. PC portable sur textile ? Déplacez-le.",
        ),
    },
    CheckExplanation {
        name: "systemd",
        what: (
            "Failed units and services that restart too often.",
            "Unités en échec et services qui redémarrent trop souvent.",
        ),
        why: (
            "A failed service means something you rely on may be down; crash loops hide root causes.",
            "Un service en échec signifie qu'un composant est peut-être arrêté ; les boucles de redémarrage masquent la cause.",
        ),
        remedy: (
            "`josephine fix` suggests restarts; `systemctl status <unit>` for the full story.",
            "`josephine fix` propose des redémarrages ; `systemctl status <unité>` pour le détail.",
        ),
    },
    CheckExplanation {
        name: "updates",
        what: (
            "Pending package updates (apt, dnf or pacman).",
            "Mises à jour de paquets en attente (apt, dnf ou pacman).",
        ),
        why: (
            "Unpatched packages leave known vulnerabilities and bugs on the system.",
            "Des paquets non mis à jour laissent des failles et bugs connus sur le système.",
        ),
        remedy: (
            "Apply updates when convenient: `sudo apt upgrade`, `sudo dnf upgrade`, etc.",
            "Appliquez les mises à jour quand vous pouvez : `sudo apt upgrade`, `sudo dnf upgrade`, etc.",
        ),
    },
    CheckExplanation {
        name: "network",
        what: (
            "Round-trip latency to the default gateway.",
            "Latence aller-retour vers la passerelle par défaut.",
        ),
        why: (
            "High latency or packet loss means local network trouble before the wider internet.",
            "Une latence élevée ou des pertes signalent un souci réseau local avant Internet.",
        ),
        remedy: (
            "Check Wi-Fi signal, cables and router; `ping` the gateway for a quick read.",
            "Vérifiez signal Wi-Fi, câbles et routeur ; `ping` la passerelle pour un test rapide.",
        ),
    },
    CheckExplanation {
        name: "battery",
        what: (
            "Charge level and depletion rate on battery power.",
            "Niveau de charge et vitesse de décharge sur batterie.",
        ),
        why: (
            "A battery draining fast or stuck low means you may lose work mid-session.",
            "Une batterie qui se vide vite ou reste basse peut couper votre travail en cours.",
        ),
        remedy: (
            "Plug in if you can; check power settings and apps keeping the GPU awake.",
            "Branchez si possible ; vérifiez l'alimentation et les apps qui réveillent le GPU.",
        ),
    },
    CheckExplanation {
        name: "inode",
        what: (
            "Inode usage on writable filesystems.",
            "Utilisation des inodes sur les systèmes de fichiers accessibles en écriture.",
        ),
        why: (
            "A disk can be \"full\" on inodes while still showing free space — many tiny files.",
            "Un disque peut être « plein » en inodes tout en affichant de l'espace libre — beaucoup de petits fichiers.",
        ),
        remedy: (
            "`josephine doctor` shows the worst partition; find and prune caches or temp trees.",
            "`josephine doctor` montre la pire partition ; trouvez et purgez caches ou fichiers temporaires.",
        ),
    },
    CheckExplanation {
        name: "smart",
        what: (
            "SMART health status of disks (opt-in, often needs root).",
            "État de santé SMART des disques (opt-in, souvent root requis).",
        ),
        why: (
            "SMART warnings often precede hard drive failure by days or weeks.",
            "Les alertes SMART précèdent souvent une panne disque de quelques jours ou semaines.",
        ),
        remedy: (
            "Back up immediately; enable the check in config if you have `smartctl` rights.",
            "Sauvegardez immédiatement ; activez le check dans la config si vous avez les droits `smartctl`.",
        ),
    },
    CheckExplanation {
        name: "kernel",
        what: (
            "Kernel incidents in the last hour (OOM kills, oops, panics).",
            "Incidents noyau sur la dernière heure (OOM, oops, panics).",
        ),
        why: (
            "Kernel faults destabilise the whole machine — not just one app.",
            "Les fautes noyau déstabilisent toute la machine — pas seulement une app.",
        ),
        remedy: (
            "Check `dmesg` and `journalctl -k`; `josephine doctor` for the count.",
            "Vérifiez `dmesg` et `journalctl -k` ; `josephine doctor` pour le décompte.",
        ),
    },
    CheckExplanation {
        name: "filesystem",
        what: (
            "Writable filesystems unexpectedly mounted read-only.",
            "Systèmes de fichiers habituellement accessibles en écriture montés en lecture seule.",
        ),
        why: (
            "A silent read-only remount often means disk errors or corruption — data loss risk.",
            "Un remontage silencieux en lecture seule signale souvent erreurs disque ou corruption — risque de perte.",
        ),
        remedy: (
            "Back up what matters, check `dmesg`; investigate the flagged mount in `josephine doctor`.",
            "Sauvegardez ce qui compte, vérifiez `dmesg` ; inspectez le montage signalé dans `josephine doctor`.",
        ),
    },
    CheckExplanation {
        name: "timesync",
        what: (
            "Whether the system clock is synchronised via NTP.",
            "Si l'horloge système est synchronisée via NTP.",
        ),
        why: (
            "Clock drift breaks log ordering, TLS validation and scheduled jobs.",
            "Une horloge qui dérive casse l'ordre des journaux, la validation TLS et les tâches planifiées.",
        ),
        remedy: (
            "`timedatectl set-ntp true` usually fixes it; check `timedatectl status`.",
            "`timedatectl set-ntp true` règle souvent le problème ; vérifiez `timedatectl status`.",
        ),
    },
    CheckExplanation {
        name: "security",
        what: (
            "Failed login and authentication attempts in the last hour.",
            "Tentatives de connexion et d'authentification échouées sur la dernière heure.",
        ),
        why: (
            "Bursts of failed logins may mean someone is probing your machine.",
            "Des rafales de connexions échouées peuvent signifier qu'on sonde votre machine.",
        ),
        remedy: (
            "Review `journalctl -u sshd`; tighten SSH (keys only, fail2ban) if it wasn't you.",
            "Consultez `journalctl -u sshd` ; renforcez SSH (clés seules, fail2ban) si ce n'était pas vous.",
        ),
    },
];

pub fn run(check: Option<&str>) -> Result<()> {
    sober_header(Some(i18n::t("explain", "explain")), None);

    match check {
        None => print_list(),
        Some(name) => {
            if let Some(entry) = CHECKS.iter().find(|c| c.name == name) {
                print_detail(entry);
            } else {
                print_unknown(name);
            }
        }
    }

    Ok(())
}

fn print_list() {
    println!(
        "{}",
        i18n::t(
            "What Joséphine watches — one line each. Detail: `josephine explain <check>`.",
            "Ce que Joséphine surveille — une ligne chacun. Détail : `josephine explain <check>`.",
        )
    );
    println!();
    for entry in CHECKS {
        let label = check_label(entry.name);
        let what = i18n::t(entry.what.0, entry.what.1);
        println!("  {label} ({}) — {what}", entry.name);
    }
}

fn print_detail(entry: &CheckExplanation) {
    let label = check_label(entry.name);
    println!("{label} ({})", entry.name);
    println!();
    println!(
        "{} {}",
        i18n::t("What:", "Quoi :"),
        i18n::t(entry.what.0, entry.what.1)
    );
    println!(
        "{} {}",
        i18n::t("Why:", "Pourquoi :"),
        i18n::t(entry.why.0, entry.why.1)
    );
    println!(
        "{} {}",
        i18n::t("Remedy:", "Remède :"),
        i18n::t(entry.remedy.0, entry.remedy.1)
    );
}

fn print_unknown(name: &str) {
    let names: Vec<&str> = CHECKS.iter().map(|c| c.name).collect();
    match i18n::lang() {
        Lang::En => {
            println!("Unknown check \"{name}\". Known checks:");
            for n in &names {
                println!("  {n}");
            }
        }
        Lang::Fr => {
            println!("Check inconnu « {name} ». Checks connus :");
            for n in &names {
                println!("  {n}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_fourteen_checks_are_listed() {
        assert_eq!(CHECKS.len(), 14);
    }
}
