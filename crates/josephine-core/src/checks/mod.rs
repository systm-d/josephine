mod cpu;
mod disk;
mod memory;
mod systemd;
mod temperature;

pub use cpu::CpuCheck;
pub use disk::DiskCheck;
pub use memory::MemoryCheck;
pub use systemd::SystemdCheck;
pub use temperature::TemperatureCheck;

use crate::check::Check;
use crate::config::ChecksConfig;

pub fn build_checks(config: &ChecksConfig) -> Vec<Box<dyn Check>> {
    let mut checks: Vec<Box<dyn Check>> = Vec::new();

    if config.cpu.enabled {
        checks.push(Box::new(CpuCheck::new(config.cpu.clone())));
    }
    if config.memory.enabled {
        checks.push(Box::new(MemoryCheck::new(config.memory.clone())));
    }
    if config.disk.enabled {
        checks.push(Box::new(DiskCheck::new(config.disk.clone())));
    }
    if config.temperature.enabled {
        checks.push(Box::new(TemperatureCheck::new(
            config.temperature.clone(),
        )));
    }
    if config.systemd.enabled {
        checks.push(Box::new(SystemdCheck::new(config.systemd.clone())));
    }

    checks
}

pub fn interval_for_check(name: &str, config: &ChecksConfig) -> u64 {
    match name {
        "cpu" => config.cpu.interval_secs,
        "memory" => config.memory.interval_secs,
        "disk" => config.disk.interval_secs,
        "temperature" => config.temperature.interval_secs,
        "systemd" => config.systemd.interval_secs,
        _ => 60,
    }
}
