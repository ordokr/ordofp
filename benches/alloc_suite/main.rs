//! Alloc-feature bench suite (one binary; modules were formerly standalone
//! criterion benches — grouped to cut the gate's link count).
//!
//! A manual `main` (the `criterion_main!` expansion) is used instead of the
//! macro because `fusion`'s extra groups are cfg-gated on optional features.
mod bayes_benchmark;
mod effect_benchmarks;
mod fusion;
mod generic;
mod grade_aggregation;
mod hlist;
mod labelled;
mod monoid;
mod nonempty_filter;
mod optics_get;
mod path;
mod pfds_bulk;
mod pfds_hot;
mod semigroup;
mod transfigure;
mod transformers;

fn main() {
    bayes_benchmark::benches();
    effect_benchmarks::benches();
    fusion::benches();
    #[cfg(all(feature = "async", feature = "fusion"))]
    fusion::flumen_fusion_benches();
    #[cfg(feature = "par")]
    fusion::par_flumen_benches();
    generic::benches();
    grade_aggregation::grade_aggregation();
    hlist::benches();
    labelled::benches();
    monoid::benches();
    nonempty_filter::benches();
    optics_get::optics();
    path::benches();
    pfds_bulk::pfds_benches();
    pfds_hot::pfds_hot();
    semigroup::benches();
    transfigure::benches();
    transformers::benches();
    criterion::Criterion::default()
        .configure_from_args()
        .final_summary();
}
