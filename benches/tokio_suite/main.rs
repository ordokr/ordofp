//! Tokio-feature bench suite (one binary; modules were formerly standalone
//! criterion benches — grouped to cut the gate's link count).
mod async_transformers;
mod zio_bind;

criterion::criterion_main!(async_transformers::benches, zio_bind::benches);
