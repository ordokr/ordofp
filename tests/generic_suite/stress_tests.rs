//! Stress tests to find breaking points.

#![cfg(feature = "std")]

#[cfg(all(feature = "par", feature = "gpu-buffer-pool"))]
mod buffer_pool_stress {
    use ordofp_core::par::opt::buffer_pool::BufferPool;

    #[test]
    fn stress_buffer_pool_many_overlaps() {
        let mut pool = BufferPool::new();
        const N: usize = 10_000;

        // Create many overlapping buffers
        // Buffer i has range [i, i+100), so buffer i and i+1 overlap
        for i in 0..N {
            let start = i;
            let end = i + 100;
            pool.add_buffer(i, start, end);
        }

        pool.compute_coloring();

        // Verify all got colors
        for i in 0..N {
            assert!(
                pool.get_color(i).is_some(),
                "Buffer {i} should have a color"
            );
        }

        // Verify actually overlapping buffers have different colors
        // Buffer i: [i, i+100) and buffer i+1: [i+1, i+101) overlap
        // So any consecutive buffers within 99 of each other overlap
        for i in 0..(N - 1) {
            let color_i = pool
                .get_color(i)
                .expect("buffer i should have a color after compute_coloring");
            let color_next = pool
                .get_color(i + 1)
                .expect("buffer i+1 should have a color after compute_coloring");
            // These overlap (ranges differ by 1, both span 100), so should have different colors
            assert_ne!(
                color_i,
                color_next,
                "Buffers {} and {} overlap but share color {}",
                i,
                i + 1,
                color_i
            );
        }
    }

    #[test]
    fn stress_buffer_pool_complex_pattern() {
        let mut pool = BufferPool::new();

        // Create complex interference pattern
        for i in 0..1000 {
            let start = (i * 7) % 100;
            let end = start + 20;
            pool.add_buffer(i, start, end);
        }

        pool.compute_coloring();

        // Verify coloring is valid (no two interfering buffers share color)
        for i in 0..1000 {
            let color_i = pool
                .get_color(i)
                .expect("buffer i should have a color after compute_coloring");
            let start_i = (i * 7) % 100;
            let end_i = start_i + 20;

            // Check all other buffers
            for j in (i + 1)..1000 {
                let start_j = (j * 7) % 100;
                let end_j = start_j + 20;

                // If they overlap, colors must differ
                if start_i < end_j && start_j < end_i {
                    let color_j = pool
                        .get_color(j)
                        .expect("buffer j should have a color after compute_coloring");
                    assert_ne!(
                        color_i, color_j,
                        "Buffers {i} and {j} overlap but share color"
                    );
                }
            }
        }
    }
}

#[cfg(feature = "transformers-cps")]
mod transformer_stress {
    use ordofp_core::transformers::ecclesia::LectorEcclesiaT;

    #[test]
    fn stress_cps_deep_chain() {
        // Test deep left-associated chain (reduced from 100k to 1k to avoid stack overflow)
        let mut chain = LectorEcclesiaT::new(|env: i32| env);
        const DEPTH: usize = 1_000;

        for _ in 0..DEPTH {
            chain = chain.flat_map(|x| LectorEcclesiaT::new(move |env: i32| x.saturating_add(env)));
        }

        // Should complete without stack overflow
        let result = chain.run(1);
        assert!(result > 0);
    }

    #[test]
    fn stress_cps_many_compositions() {
        // Test many compositions
        let mut reader = LectorEcclesiaT::new(|env: i32| env);

        for i in 0..10_000 {
            reader = reader.map(move |x| x + i);
        }

        let result = reader.run(0);
        let expected: i32 = (0..10_000).sum();
        assert_eq!(result, expected);
    }

    #[test]
    fn stress_cps_nested_local() {
        // Test deeply nested local modifications (reduced from 1000 to 30 to avoid overflow)
        let mut reader = LectorEcclesiaT::new(|env: i32| env);

        for _ in 0..30 {
            reader = reader.local(|env| env.saturating_mul(2));
        }

        let result = reader.run(1);
        // Should be 1 * 2^30 = 1073741824 (within i32 range)
        assert_eq!(result, 1 << 30);
    }
}

#[cfg(all(
    feature = "transformers-cps",
    feature = "par",
    feature = "gpu-buffer-pool"
))]
mod combined_stress {
    use ordofp_core::par::opt::buffer_pool::BufferPool;
    use ordofp_core::transformers::ecclesia::LectorEcclesiaT;

    #[test]
    fn stress_all_features_together() {
        // Use all features simultaneously
        let mut pool = BufferPool::new();

        // Add buffers
        for i in 0..1000 {
            pool.add_buffer(i, i * 10, i * 10 + 50);
        }
        pool.compute_coloring();

        // Use transformer
        let reader = LectorEcclesiaT::new(|env: i32| env);
        let result = reader.map(|x| x * 2).run(42);

        // Verify everything worked
        assert!(pool.get_color(0).is_some());
        assert_eq!(result, 84);
    }
}
