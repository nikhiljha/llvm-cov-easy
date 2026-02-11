use criterion::{Criterion, criterion_group, criterion_main};

fn bench_analyze_json(c: &mut Criterion) {
    let json = include_str!("../tests/fixtures/show-missing-lines.json");
    c.bench_function("analyze_json", |b| {
        b.iter(|| llvm_cov_easy::analyze_json(json).unwrap());
    });
}

fn bench_analyze_and_format(c: &mut Criterion) {
    let json = include_str!("../tests/fixtures/show-missing-lines.json");
    c.bench_function("analyze_and_format", |b| {
        b.iter(|| llvm_cov_easy::analyze_and_format(json).unwrap());
    });
}

criterion_group!(benches, bench_analyze_json, bench_analyze_and_format);
criterion_main!(benches);
