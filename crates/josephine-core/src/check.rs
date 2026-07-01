use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Attention,
    Critique,
}

#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub threshold_warning: Option<f64>,
    pub threshold_critical: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub check_name: String,
    pub metrics: Vec<Metric>,
    pub details: Vec<String>,
    pub top_processes: Vec<String>,
    /// Ready-to-display value for the `status` one-liner (e.g. `73% (28G / 38G)`).
    /// Falls back to the primary metric when `None`.
    pub status_value: Option<String>,
}

impl CheckResult {
    pub fn worst_severity(&self) -> Severity {
        let mut worst = Severity::Info;
        for metric in &self.metrics {
            let sev = metric_severity(metric);
            if sev > worst {
                worst = sev;
            }
        }
        worst
    }
}

/// Format a byte count as a compact gibibyte string (`28G`, `4.9G`, `512M`).
pub fn human_size(bytes: f64) -> String {
    let gib = bytes / 1_073_741_824.0;
    if gib >= 1.0 {
        if gib >= 10.0 {
            format!("{gib:.0}G")
        } else {
            format!("{gib:.1}G")
        }
    } else {
        format!("{:.0}M", bytes / 1_048_576.0)
    }
}

pub fn metric_severity(metric: &Metric) -> Severity {
    if let Some(critical) = metric.threshold_critical
        && metric.value >= critical
    {
        return Severity::Critique;
    }
    if let Some(warning) = metric.threshold_warning
        && metric.value >= warning
    {
        return Severity::Attention;
    }
    Severity::Info
}

pub trait Check: Send {
    fn name(&self) -> &str;
    fn run(&mut self) -> Result<CheckResult>;
}
