use amt::ec_algebra::UniformRand;
use ark_poly::{EvaluationDomain, Radix2EvaluationDomain};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::{rngs::StdRng, SeedableRng};
use zg_encoder::constants::{G1Curve, Scalar, BLOB_ROW_ENCODED};

fn ec_fft(v2_coeffs: &mut [G1Curve]) -> () {
    let fft_domain =
        Radix2EvaluationDomain::<Scalar>::new(v2_coeffs.len()).unwrap();
    fft_domain.fft(v2_coeffs);
}

fn criterion_benchmark(c: &mut Criterion) {
    let seed = 222u64;
    let mut rng = StdRng::seed_from_u64(seed);
    let mut v1_coeffs = (0..BLOB_ROW_ENCODED)
        .map(|_| G1Curve::rand(&mut rng))
        .collect::<Vec<_>>();

    let mut group = c.benchmark_group("ec-fft");
    group.bench_function(BLOB_ROW_ENCODED.to_string(), |b| {
        b.iter(|| ec_fft(black_box(&mut v1_coeffs)))
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
