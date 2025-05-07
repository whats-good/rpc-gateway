use bytes::Bytes;
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use ouroboros::self_referencing;
use serde::{Deserialize, Serialize};

fn generate_payload(id: i64) -> Bytes {
    let p = format!(
        r#"{{"jsonrpc":"2.0","id":{},"method":"eth_getBlockByNumber","params":["latest", false]}}"#,
        id
    );
    Bytes::from(p)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Id {
    String(String),
    Number(i64),
    Null,
}

#[derive(Debug, Deserialize)]
struct SerdeValueRpcCall {
    id: Id,
    method: String,
    jsonrpc: Option<String>,
    params: Option<serde_json::value::Value>,
}

#[derive(Debug, Deserialize)]
struct SimdJsonOwnedRpcCall {
    id: Id,
    method: String,
    jsonrpc: Option<String>,
    params: Option<simd_json::value::OwnedValue>,
}

#[derive(Debug, Deserialize)]
struct SerdeJsonRawValueRpcCall<'a> {
    id: Id,
    method: String,
    jsonrpc: Option<String>,

    #[serde(borrow)]
    params: Option<&'a serde_json::value::RawValue>,
}

pub fn benchmark_processing(c: &mut Criterion) {
    c.bench_function("serde_value", |b| {
        b.iter_batched(
            || generate_payload(1), // setup (not timed)
            |payload| serde_json::from_slice::<SerdeValueRpcCall>(&payload).unwrap(),
            BatchSize::SmallInput, // or BatchSize::PerIteration
        );
    });

    c.bench_function("simd_json_owned", |b| {
        b.iter_batched(
            || generate_payload(1), // setup (not timed)
            |payload| {
                let mut bytes = payload.to_vec();
                simd_json::from_slice::<SimdJsonOwnedRpcCall>(&mut bytes).unwrap()
            },
            BatchSize::SmallInput, // or BatchSize::PerIteration
        );
    });

    c.bench_function("serde_json_raw_value", |b| {
        b.iter_batched(
            || generate_payload(1), // setup (not timed)
            |payload| {
                let call = serde_json::from_slice::<SerdeJsonRawValueRpcCall>(&payload).unwrap();
            },
            BatchSize::SmallInput, // or BatchSize::PerIteration
        );
    });
}

criterion_group!(benches, benchmark_processing);
criterion_main!(benches);
