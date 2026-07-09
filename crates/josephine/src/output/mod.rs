mod bars;
mod doctor;
mod runner;
mod status;
mod style;

pub use doctor::print_doctor;
pub use runner::run_checks_with_progress;
pub use status::print_status_table;

pub use style::{
    check_label, confirm, format_metric_value, is_tty, primary_metric, print_banner, sparkline,
};
// Task 1's new "Constellation sobre" primitives; Tasks 2-4 are the consumers.
// The allow is scoped to only these until they're wired in.
#[allow(unused_imports)]
pub use style::{HEADER_WIDTH, accent, severity_paint, sober_header, status_glyph};
