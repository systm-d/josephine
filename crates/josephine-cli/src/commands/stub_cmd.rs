use anyhow::Result;

pub enum StubCommand {
    Clean { dry_run: bool },
    Fix,
    Report,
}

pub fn run(cmd: StubCommand) -> Result<()> {
    let hint = match cmd {
        StubCommand::Clean { dry_run: true } => {
            "`josephine clean --dry-run` — nettoyage avec aperçu"
        }
        StubCommand::Clean { dry_run: false } => "`josephine clean` — nettoyage guidé",
        StubCommand::Fix => "`josephine fix` — corrections accompagnées",
        StubCommand::Report => "`josephine report` — rapport complet exportable",
    };

    println!("✨ Cette fonctionnalité arrive bientôt avec Joséphine.");
    println!("   {hint}");
    Ok(())
}
