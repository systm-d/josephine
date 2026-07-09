pub const BAR_WIDTH: usize = 16;

pub fn bar_plain(value: f64, scale: f64, width: usize) -> String {
    let ratio = (value / scale.max(1.0)).clamp(0.0, 1.0);
    let filled = (ratio * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}
