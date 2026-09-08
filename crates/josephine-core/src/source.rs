//! Where a check reads the machine from.
//!
//! Checks read two kinds of thing: files the kernel exposes (`/proc`,
//! `/sys`, `/etc`) and the output of a handful of system commands (`df`,
//! `ping`, `journalctl`). In production both come from the running machine.
//!
//! Both are behind a seam here so a test can point a check at a fixture tree
//! and canned command output, and exercise read → parse → `CheckResult`
//! without depending on the host having a thermal sensor, a battery, or a
//! reachable gateway. Production behaviour is unchanged: `new()` on every
//! check still reads the real machine.

use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

/// The filesystem a check reads system files from.
///
/// [`Sysfs::system`] is the running machine. [`Sysfs::rooted`] re-bases every
/// absolute lookup under a directory, so `/sys/class/thermal` becomes
/// `<root>/sys/class/thermal`.
#[derive(Debug, Clone, Default)]
pub struct Sysfs {
    /// `None` is the real filesystem; `Some(dir)` a fixture tree.
    root: Option<PathBuf>,
}

impl Sysfs {
    /// The running machine.
    pub fn system() -> Self {
        Self { root: None }
    }

    /// A fixture tree standing in for the machine.
    pub fn rooted(root: impl Into<PathBuf>) -> Self {
        Self {
            root: Some(root.into()),
        }
    }

    /// Resolve an absolute system path under this root.
    pub fn path(&self, absolute: &str) -> PathBuf {
        match &self.root {
            None => PathBuf::from(absolute),
            Some(root) => root.join(absolute.trim_start_matches('/')),
        }
    }

    /// Read an absolute system path, or `None` when it is absent or unreadable
    /// — which, for a check, is simply "this machine does not expose that".
    pub fn read(&self, absolute: &str) -> Option<String> {
        std::fs::read_to_string(self.path(absolute)).ok()
    }
}

/// The system commands a check shells out to.
///
/// One method, because every caller wants the same thing: the standard output
/// of a command that succeeded. A command that is missing, fails, or exits
/// non-zero yields `None`, and every check already treats that as "cannot
/// tell" rather than an error.
pub trait Commands: Send + Sync {
    fn output(&self, program: &str, args: &[&str]) -> Option<String>;
}

/// Runs the real thing.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemCommands;

impl Commands for SystemCommands {
    fn output(&self, program: &str, args: &[&str]) -> Option<String> {
        let output = Command::new(program).args(args).output().ok()?;
        if !output.status.success() {
            return None;
        }
        Some(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

/// The commands source a check is built with by default.
pub fn system_commands() -> Arc<dyn Commands> {
    Arc::new(SystemCommands)
}

/// Canned command output, keyed by program name — the test counterpart of
/// [`SystemCommands`].
///
/// It lives here rather than in the tests so that both the in-crate unit tests
/// and the integration tests under `tests/` can reach it.
#[derive(Debug, Clone, Default)]
pub struct StubCommands {
    responses: Vec<(String, String)>,
}

impl StubCommands {
    pub fn new() -> Self {
        Self::default()
    }

    /// Answer `program` with `stdout`. A program with no answer yields `None`,
    /// standing in for a command that is not installed.
    pub fn with(mut self, program: &str, stdout: &str) -> Self {
        self.responses
            .push((program.to_string(), stdout.to_string()));
        self
    }

    pub fn arc(self) -> Arc<dyn Commands> {
        Arc::new(self)
    }
}

impl Commands for StubCommands {
    fn output(&self, program: &str, _args: &[&str]) -> Option<String> {
        self.responses
            .iter()
            .find(|(name, _)| name == program)
            .map(|(_, stdout)| stdout.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_root_leaves_absolute_paths_alone() {
        let sysfs = Sysfs::system();
        assert_eq!(
            sysfs.path("/sys/class/thermal"),
            PathBuf::from("/sys/class/thermal")
        );
    }

    #[test]
    fn rooted_paths_land_under_the_fixture() {
        let sysfs = Sysfs::rooted("/tmp/fixture");
        assert_eq!(
            sysfs.path("/sys/class/thermal"),
            PathBuf::from("/tmp/fixture/sys/class/thermal")
        );
    }

    #[test]
    fn a_stub_answers_only_the_programs_it_knows() {
        let commands = StubCommands::new().with("df", "canned");
        assert_eq!(commands.output("df", &["-iPT"]), Some("canned".into()));
        assert_eq!(commands.output("journalctl", &["-k"]), None);
    }
}
