use criterion::{black_box, criterion_group, criterion_main, Criterion};

extern crate fixed_vec;

use fixed_vec::FixedVec;

extern crate type_name_value;

use type_name_value::name;

fn my_adding_func_a(index_a: usize, index_b: usize) -> Vec<u32> {
    let mut v = vec![0u32; 100];

    for _ in 0..10000 {
        v[index_a] += 5;
        v[index_b] += 10;
    }

    v
}

fn my_adding_func_b(index_a: usize, index_b: usize) -> Vec<u32> {
    let v = vec![0u32; 100];
    let v = name!(v);
    let mut v = FixedVec::fix(v);

    let index_a = v.check_index(index_a).unwrap();
    let index_b = v.check_index(index_b).unwrap();
    for _ in 0..10000 {
        *v.get_mut(index_a) += 5;
        *v.get_mut(index_b) += 10;
    }
    v.unfix()
}

fn comparison_benchmark(c: &mut Criterion) {
    c.bench_function("inc 2 indices 10000 times (no fixed_vec)", |b| {
        b.iter(|| {
            my_adding_func_a(black_box(10), black_box(15))
        })
    });

    c.bench_function("inc 2 indices 10000 times (with fixed_vec)", |b| {
        b.iter(|| {
            my_adding_func_b(black_box(10), black_box(15))
        })
    });

    c.bench_function("inc many indices 1000 times (no fixed_vec)", |b| {
        b.iter(|| {
            let mut v = black_box(vec![0u32; 100]);
            let range_a = black_box(10);
            let range_b = black_box(35);
            for _ in 0..1000 {
                for i in range_a..range_b {
                    v[i] += black_box(1);
                }
            }
            v
        });
    });

    c.bench_function("inc many indices 1000 times (with fixed_vec)", |b| {
        b.iter(|| {
            let v = black_box(vec![0u32; 100]);
            let v = name!(v);
            let mut v = FixedVec::fix(v);
            let range_a = black_box(10);
            let range_b = black_box(35);

            let range = range_a..range_b;
            let range = v.check_range(range).unwrap();

            for _ in 0..1000 {
                for i in range.clone() {
                    *v.get_mut(i) += black_box(1);
                }
            }
            v
        });
    });
}

criterion_group!(benches, comparison_benchmark);
criterion_main!(benches);
