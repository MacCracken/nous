use std::path::Path;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use nous::{NousResolver, PackageSource, ResolutionStrategy};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn write_manifest(dir: &Path, name: &str, runtime: &str) {
    let pkg_dir = dir.join("installed").join(name);
    std::fs::create_dir_all(&pkg_dir).unwrap();
    let manifest = serde_json::json!({
        "agent": { "name": name, "version": "1.0.0", "description": format!("Bench package {name}") },
        "dependencies": {},
        "runtime": runtime,
    });
    std::fs::write(pkg_dir.join("manifest.json"), manifest.to_string()).unwrap();
}

fn setup_registry(count: usize) -> (tempfile::TempDir, tempfile::TempDir) {
    let market = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    for i in 0..count {
        write_manifest(market.path(), &format!("bench-pkg-{i}"), "native");
    }
    (market, cache)
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

fn bench_detect_source(c: &mut Criterion) {
    let names = [
        "acme/scanner",
        "com.example.app",
        "nginx",
        "myapp.flutter",
        "",
        "io.flutter.demo",
    ];
    c.bench_function("detect_source", |b| {
        b.iter(|| {
            for name in &names {
                black_box(NousResolver::detect_source(name));
            }
        });
    });
}

fn bench_resolve_marketplace(c: &mut Criterion) {
    let (market, cache) = setup_registry(50);
    let resolver = NousResolver::new(market.path(), cache.path()).unwrap();

    c.bench_function("resolve_marketplace_hit", |b| {
        b.iter(|| {
            black_box(resolver.resolve("bench-pkg-25").unwrap());
        });
    });

    c.bench_function("resolve_marketplace_miss", |b| {
        b.iter(|| {
            black_box(resolver.resolve("nonexistent-xyz").unwrap());
        });
    });
}

fn bench_search(c: &mut Criterion) {
    let (market, cache) = setup_registry(100);
    let resolver = NousResolver::new(market.path(), cache.path()).unwrap();

    c.bench_function("search_100_packages", |b| {
        b.iter(|| {
            black_box(resolver.search("bench").unwrap());
        });
    });
}

fn bench_list_installed(c: &mut Criterion) {
    let (market, cache) = setup_registry(100);
    let resolver = NousResolver::new(market.path(), cache.path()).unwrap();

    c.bench_function("list_installed_100", |b| {
        b.iter(|| {
            black_box(resolver.list_installed().unwrap());
        });
    });
}

fn bench_resolve_strategies(c: &mut Criterion) {
    let (market, cache) = setup_registry(50);

    let strategies = [
        ("marketplace_first", ResolutionStrategy::MarketplaceFirst),
        ("system_first", ResolutionStrategy::SystemFirst),
        (
            "only_marketplace",
            ResolutionStrategy::OnlySource(PackageSource::Marketplace),
        ),
        ("search_all", ResolutionStrategy::SearchAll),
    ];

    let mut group = c.benchmark_group("resolve_strategy");
    for (name, strategy) in &strategies {
        let resolver = NousResolver::new(market.path(), cache.path())
            .unwrap()
            .with_strategy(strategy.clone());
        group.bench_function(*name, |b| {
            b.iter(|| {
                black_box(resolver.resolve("bench-pkg-10").unwrap());
            });
        });
    }
    group.finish();
}

fn bench_serde_roundtrip(c: &mut Criterion) {
    let pkg = nous::ResolvedPackage {
        name: "bench-pkg".to_string(),
        version: "1.2.3".to_string(),
        source: PackageSource::Marketplace,
        description: "A benchmark package".to_string(),
        size_bytes: Some(4096),
        dependencies: vec!["dep-a".to_string(), "dep-b".to_string()],
        trusted: true,
    };

    c.bench_function("serde_roundtrip_resolved_package", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&pkg)).unwrap();
            let _: nous::ResolvedPackage = serde_json::from_str(&json).unwrap();
        });
    });
}

criterion_group!(
    benches,
    bench_detect_source,
    bench_resolve_marketplace,
    bench_search,
    bench_list_installed,
    bench_resolve_strategies,
    bench_serde_roundtrip,
);
criterion_main!(benches);
