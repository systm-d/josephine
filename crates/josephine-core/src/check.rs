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
