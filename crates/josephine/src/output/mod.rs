mod bars;
mod doctor;
mod runner;
mod status;
mod style;

pub use doctor::print_doctor;
pub use runner::run_checks_with_progress;
pub use status::print_status_table;
// `accent`, `severity_paint`, `sober_header`, `status_glyph` and `HEADER_WIDTH`
// are Task 1's new primitives; Tasks 2-4 (status/doctor/history rewrites)
// consume them, so `main`/`commands` don't reach them yet.
#[allow(unused_imports)]
pub use style::{
    HEADER_WIDTH, accent, check_label, confirm, format_metric_value, is_tty, primary_metric,
    print_banner, severity_paint, sober_header, sparkline, status_glyph,
};
