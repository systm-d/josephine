//! Parsing for the `--since` window shared by `history` and `report`.
//!
//! One spelling of the flag across the CLI: a positive number followed by `d`
//! or `h`, case-insensitive, surrounding space tolerated.

use anyhow::{Context, Result, bail};
use josephine_core::i18n;

/// Parse a `--since` window like `7d` or `24h` into a count of hours.
pub fn parse_since(spec: &str) -> Result<i64> {
    let spec = spec.trim();
    let (value, per) = if let Some(days) = spec.strip_suffix(['d', 'D']) {
        (days, 24)
    } else if let Some(hours) = spec.strip_suffix(['h', 'H']) {
        (hours, 1)
    } else {
        bail!(i18n::t(
            "--since expects a window like `7d` or `24h`.",
            "--since attend une fenêtre comme `7d` ou `24h`.",
        ));
    };
    let n: i64 = value
        .trim()
        .parse()
        .ok()
        .filter(|&n| n > 0)
        .with_context(|| {
            i18n::t(
                "--since must be a positive number followed by `d` or `h`.",
                "--since doit être un nombre positif suivi de `d` ou `h`.",
            )
        })?;
    Ok(n * per)
}

/// Render a window in hours back into the shape a person typed — `6 h`,
/// `7 d` — for headings and JSON.
///
/// 24 h stays "24 h" rather than becoming "1 d": it is the default window and
/// what `history` has always called itself.
pub fn label(hours: i64) -> String {
    if hours != 24 && hours % 24 == 0 {
        format!("{} d", hours / 24)
    } else {
        format!("{hours} h")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_days_and_hours() {
        assert_eq!(parse_since("7d").unwrap(), 168);
        assert_eq!(parse_since("24h").unwrap(), 24);
        assert_eq!(parse_since(" 3D ").unwrap(), 72);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_since("7").is_err());
        assert!(parse_since("d").is_err());
        assert!(parse_since("0d").is_err());
        assert!(parse_since("-2h").is_err());
        assert!(parse_since("week").is_err());
    }

    #[test]
    fn labels_read_the_way_they_were_typed() {
        assert_eq!(label(24), "24 h", "the default window keeps its old name");
        assert_eq!(label(48), "2 d");
        assert_eq!(label(168), "7 d");
        assert_eq!(label(6), "6 h");
    }
}
