use anon_sdk::algorithms::{AnonymizationAlgorithm, k_anonymity::KAnonymity};
use anon_sdk::{DataType, Dataset};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn create_test_dataset(rows: usize) -> Dataset {
    let mut dataset = Dataset::new();

    let ages: Vec<String> = (0..rows).map(|i| (20 + i % 60).to_string()).collect();
    let genders: Vec<String> = (0..rows)
        .map(|i| if i % 2 == 0 { "M" } else { "F" }.to_string())
        .collect();
    let zipcodes: Vec<String> = (0..rows)
        .map(|i| format!("{:05}", 10000 + i % 90000))
        .collect();

    dataset.add_column("age".to_string(), ages, DataType::Numeric);
    dataset.add_column("gender".to_string(), genders, DataType::Categorical);
    dataset.add_column("zipcode".to_string(), zipcodes, DataType::Categorical);

    dataset
}

fn benchmark_k_anonymity(c: &mut Criterion) {
    let dataset = create_test_dataset(1000);
    let k_anon = KAnonymity::new(
        3,
        vec![
            "age".to_string(),
            "gender".to_string(),
            "zipcode".to_string(),
        ],
    )
    .unwrap();

    c.bench_function("k_anonymity_1000_rows", |b| {
        b.iter(|| k_anon.anonymize(black_box(&dataset)))
    });
}

criterion_group!(benches, benchmark_k_anonymity);
criterion_main!(benches);
