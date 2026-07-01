use anyhow::Result;

pub enum StubCommand {
    Clean { dry_run: bool },
    Fix,
    Report,
}

pub fn run(cmd: StubCommand) -> Result<()> {
    let (teaser, hint) = match cmd {
        StubCommand::Clean { dry_run: true } => (
            "Bientôt, je ferai le ménage sous vos yeux — sans rien déranger, promis sur mes ailes.",
            "`josephine clean --dry-run` — nettoyage avec aperçu",
        ),
        StubCommand::Clean { dry_run: false } => (
            "Bientôt, je retrousserai mes ailes pour un grand ménage de printemps.",
            "`josephine clean` — nettoyage guidé",
        ),
        StubCommand::Fix => (
            "Bientôt, je soignerai les petits bobos de votre machine — un miracle à la fois.",
            "`josephine fix` — corrections accompagnées",
        ),
        StubCommand::Report => (
            "Bientôt, je vous rédigerai un rapport digne d'un carnet de bord céleste.",
            "`josephine report` — rapport complet exportable",
        ),
    };

    println!("✨ {teaser}");
    println!("   D'ici là : {hint}");
    Ok(())
}
