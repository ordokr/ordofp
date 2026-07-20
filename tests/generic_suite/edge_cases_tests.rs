//! Edge case tests to find bugs.

#![cfg(feature = "std")]

#[cfg(all(feature = "par", feature = "gpu-buffer-pool"))]
mod buffer_pool_edges {
    use ordofp_core::par::opt::buffer_pool::BufferPool;

    #[test]
    fn edge_buffer_pool_empty() {
        let mut pool = BufferPool::new();
        pool.compute_coloring();

        assert_eq!(pool.get_color(0), None);
    }

    #[test]
    fn edge_buffer_pool_single() {
        let mut pool = BufferPool::new();
        pool.add_buffer(0, 0, 10);
        pool.compute_coloring();

        assert_eq!(pool.get_color(0), Some(0));
    }

    #[test]
    fn edge_buffer_pool_adjacent_no_overlap() {
        let mut pool = BufferPool::new();
        pool.add_buffer(0, 0, 10);
        pool.add_buffer(1, 10, 20); // Adjacent, no overlap
        pool.compute_coloring();

        // Adjacent buffers (end == start) don't overlap, so can share color
        let color0 = pool.get_color(0);
        let color1 = pool.get_color(1);
        assert!(color0.is_some());
        assert!(color1.is_some());
    }

    #[test]
    fn edge_buffer_pool_exact_overlap() {
        let mut pool = BufferPool::new();
        pool.add_buffer(0, 0, 10);
        pool.add_buffer(1, 0, 10); // Exact same lifetime
        pool.compute_coloring();

        let color0 = pool
            .get_color(0)
            .expect("buffer 0 should be colored after compute_coloring");
        let color1 = pool
            .get_color(1)
            .expect("buffer 1 should be colored after compute_coloring");
        assert_ne!(color0, color1); // Should have different colors
    }

    #[test]
    fn edge_buffer_pool_zero_length() {
        let mut pool = BufferPool::new();
        pool.add_buffer(0, 0, 0); // Zero-length buffer
        pool.add_buffer(1, 0, 10);
        pool.compute_coloring();

        // Zero-length buffer (start == end) doesn't overlap with anything
        let color0 = pool.get_color(0);
        let color1 = pool.get_color(1);
        assert!(color0.is_some());
        assert!(color1.is_some());
    }
}

#[cfg(feature = "transformers-cps")]
mod transformer_edges {
    use ordofp_core::transformers::ecclesia::*;

    #[test]
    fn edge_transformer_empty_composition() {
        let reader = LectorEcclesiaT::new(|env: i32| env);
        let result = reader.run(42);
        assert_eq!(result, 42);
    }

    #[test]
    fn edge_transformer_identity_map() {
        let reader = LectorEcclesiaT::new(|env: i32| env);
        let mapped = reader.map(|x| x); // Identity
        let result = mapped.run(42);
        assert_eq!(result, 42);
    }

    #[test]
    fn edge_transformer_zero_env() {
        let reader = LectorEcclesiaT::new(|env: i32| env);
        let result = reader.run(0);
        assert_eq!(result, 0);
    }

    #[test]
    fn edge_transformer_negative_env() {
        let reader = LectorEcclesiaT::new(|env: i32| env);
        let result = reader.run(-42);
        assert_eq!(result, -42);
    }

    #[test]
    fn edge_transformer_local_identity() {
        let reader = LectorEcclesiaT::new(|env: i32| env);
        let local = reader.local(|env| env); // Identity modification
        let result = local.run(42);
        assert_eq!(result, 42);
    }
}
