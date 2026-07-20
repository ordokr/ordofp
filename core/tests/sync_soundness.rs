fn assert_sync<T: Sync>() {}

#[test]
fn test_sync() {
    assert_sync::<ordofp_core::tracing::CollectorMemoriae>();
}
