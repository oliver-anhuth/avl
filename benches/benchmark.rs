use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{rngs::StdRng, Rng, SeedableRng};

use avl::AvlTreeMap;

const N: usize = 100_000;

pub fn benchmarks(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(0);
    let values: Vec<i32> = (1..=N).map(|_| rng.gen()).collect();

    c.bench_function("map_insert", |b| {
        let mut map = AvlTreeMap::new();
        b.iter(|| {
            for value in &values {
                map.insert(*value, *value);
            }
        })
    });

    let mut map = AvlTreeMap::new();
    for value in &values {
        map.insert(*value, *value);
    }

    c.bench_function("map_get", |b| {
        b.iter(|| {
            for value in &values {
                black_box(map.get(value));
            }
        })
    });

    c.bench_function("map_iter", |b| {
        b.iter(|| {
            for (k, v) in &map {
                black_box((k, v));
            }
        })
    });

    c.bench_function("map_remove", |b| {
        let mut map = map.clone();
        b.iter(|| {
            for value in &values {
                map.remove(value);
            }
        })
    });
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
