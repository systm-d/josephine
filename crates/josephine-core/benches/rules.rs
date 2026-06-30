use criterion::{Criterion, black_box, criterion_group, criterion_main};
use josephine_core::Config;

fn config_default_validate(c: &mut Criterion) {
    c.bench_function("config_default_validate", |b| {
        b.iter(|| {
            let cfg = Config::default();
            black_box(cfg.validate()).ok();
        });
    });
}

criterion_group!(benches, config_default_validate);
criterion_main!(benches);
