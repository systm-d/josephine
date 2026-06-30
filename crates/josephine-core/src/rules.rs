use std::collections::HashMap;

use crate::check::{Metric, Severity, metric_severity};
use crate::config::CheckThresholds;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertState {
    Normal,
    Warning,
    Critical,
}

impl AlertState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Warning => "WARNING",
            Self::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Clone)]
pub struct StateTransition {
    pub check_name: String,
    pub metric_name: String,
    pub from: AlertState,
    pub to: AlertState,
    pub value: f64,
    pub message: String,
    pub notify: bool,
    pub recovered: bool,
}

pub struct RulesEngine {
    states: HashMap<(String, String), AlertState>,
}

impl Default for RulesEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RulesEngine {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    pub fn evaluate_check(
        &mut self,
        check_name: &str,
        metrics: &[Metric],
        thresholds: &CheckThresholds,
    ) -> Vec<StateTransition> {
        metrics
            .iter()
            .filter(|m| m.threshold_warning.is_some())
            .filter_map(|metric| self.evaluate_metric(check_name, metric, thresholds))
            .collect()
    }

    fn evaluate_metric(
        &mut self,
        check_name: &str,
        metric: &Metric,
        thresholds: &CheckThresholds,
    ) -> Option<StateTransition> {
        let key = (check_name.to_string(), metric.name.clone());
        let previous = self.states.get(&key).copied().unwrap_or(AlertState::Normal);
        let current = severity_to_alert(metric_severity(metric));

        if previous == current {
            return None;
        }

        self.states.insert(key.clone(), current);

        let (from, to, notify, recovered) = match (previous, current) {
            (AlertState::Normal, AlertState::Warning) => (previous, current, true, false),
            (AlertState::Normal, AlertState::Critical) => (previous, current, true, false),
            (AlertState::Warning, AlertState::Critical) => (previous, current, true, false),
            (AlertState::Warning, AlertState::Normal)
            | (AlertState::Critical, AlertState::Normal) => {
                (previous, AlertState::Normal, true, true)
            }
            _ => (previous, current, false, false),
        };

        if !notify {
            return None;
        }

        let message = if recovered {
            crate::messages::recovery_message(check_name, metric)
        } else {
            crate::messages::alert_message(check_name, metric, thresholds, to)
        };

        Some(StateTransition {
            check_name: check_name.to_string(),
            metric_name: metric.name.clone(),
            from,
            to,
            value: metric.value,
            message,
            notify: true,
            recovered,
        })
    }
}

fn severity_to_alert(severity: Severity) -> AlertState {
    match severity {
        Severity::Info => AlertState::Normal,
        Severity::Attention => AlertState::Warning,
        Severity::Critique => AlertState::Critical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::Metric;

    fn metric(value: f64, warning: f64, critical: f64) -> Metric {
        Metric {
            name: "usage_percent".into(),
            value,
            unit: "%".into(),
            threshold_warning: Some(warning),
            threshold_critical: Some(critical),
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
    fn no_notification_when_state_unchanged() {
        let mut engine = RulesEngine::new();
        let t = thresholds();
        let m = metric(90.0, 85.0, 95.0);

        let first = engine.evaluate_check("cpu", std::slice::from_ref(&m), &t);
        assert_eq!(first.len(), 1);

        let second = engine.evaluate_check("cpu", &[m], &t);
        assert!(second.is_empty());
    }

    #[test]
    fn recovery_notifies_once() {
        let mut engine = RulesEngine::new();
        let t = thresholds();

        engine.evaluate_check("cpu", &[metric(90.0, 85.0, 95.0)], &t);
        let recovery = engine.evaluate_check("cpu", &[metric(50.0, 85.0, 95.0)], &t);

        assert_eq!(recovery.len(), 1);
        assert!(recovery[0].recovered);
    }

    #[test]
    fn escalation_warning_to_critical() {
        let mut engine = RulesEngine::new();
        let t = thresholds();

        engine.evaluate_check("cpu", &[metric(86.0, 85.0, 95.0)], &t);
        let critical = engine.evaluate_check("cpu", &[metric(96.0, 85.0, 95.0)], &t);

        assert_eq!(critical.len(), 1);
        assert_eq!(critical[0].to, AlertState::Critical);
    }
}
