//! Tests for buffer reuse and interference graph.

#![cfg(all(feature = "par", feature = "gpu-buffer-pool"))]

use ordofp_core::par::opt::buffer_pool::{BufferLifetime, BufferPool, lifetimes_overlap};

#[test]
fn test_buffer_pool_basic() {
    let mut pool = BufferPool::new();

    // Add non-overlapping buffers
    pool.add_buffer(0, 0, 10);
    pool.add_buffer(1, 20, 30);

    pool.compute_coloring();

    // Non-overlapping buffers should get same color (can share memory)
    let color0 = pool.get_color(0);
    let color1 = pool.get_color(1);

    assert!(color0.is_some());
    assert!(color1.is_some());
    // They might get same color (0) since they don't interfere
    assert_eq!(color0, color1);
}

#[test]
fn test_buffer_pool_interference() {
    let mut pool = BufferPool::new();

    // Add overlapping buffers
    pool.add_buffer(0, 0, 20); // Lives from 0 to 20
    pool.add_buffer(1, 10, 30); // Lives from 10 to 30 (overlaps with 0)

    pool.compute_coloring();

    // Overlapping buffers should get different colors
    let color0 = pool.get_color(0);
    let color1 = pool.get_color(1);

    assert!(color0.is_some());
    assert!(color1.is_some());
    assert_ne!(
        color0, color1,
        "Overlapping buffers should have different colors"
    );
}

#[test]
fn test_buffer_pool_chain() {
    let mut pool = BufferPool::new();

    // Chain of overlapping buffers
    pool.add_buffer(0, 0, 10);
    pool.add_buffer(1, 5, 15); // Overlaps with 0
    pool.add_buffer(2, 10, 20); // Overlaps with 1

    pool.compute_coloring();

    let color0 = pool
        .get_color(0)
        .expect("buffer 0 should have a color after compute_coloring");
    let color1 = pool
        .get_color(1)
        .expect("buffer 1 should have a color after compute_coloring");
    let color2 = pool
        .get_color(2)
        .expect("buffer 2 should have a color after compute_coloring");

    // Adjacent buffers should have different colors
    assert_ne!(color0, color1);
    assert_ne!(color1, color2);
    // But 0 and 2 might share (they don't overlap)
    // This depends on greedy coloring order
}

#[test]
fn test_buffer_pool_complex() {
    let mut pool = BufferPool::new();

    // Complex interference pattern
    pool.add_buffer(0, 0, 20);
    pool.add_buffer(1, 10, 30);
    pool.add_buffer(2, 25, 35);
    pool.add_buffer(3, 40, 50); // No interference

    pool.compute_coloring();

    // Buffer 3 should be able to share with 0 or 2 (no interference)
    let color0 = pool
        .get_color(0)
        .expect("buffer 0 should have a color after compute_coloring");
    let color1 = pool
        .get_color(1)
        .expect("buffer 1 should have a color after compute_coloring");
    let color2 = pool
        .get_color(2)
        .expect("buffer 2 should have a color after compute_coloring");
    let _color3 = pool
        .get_color(3)
        .expect("buffer 3 should have a color after compute_coloring");

    assert_ne!(color0, color1); // Overlap
    assert_ne!(color1, color2); // Overlap
    // color3 might share with color0 (no interference)
}

#[test]
fn test_buffer_pool_empty() {
    let mut pool = BufferPool::new();

    pool.compute_coloring();

    // No buffers, so no colors
    assert_eq!(pool.get_color(0), None);
}

#[test]
fn test_buffer_pool_single() {
    let mut pool = BufferPool::new();

    pool.add_buffer(0, 0, 10);
    pool.compute_coloring();

    // Single buffer should get color 0
    assert_eq!(pool.get_color(0), Some(0));
}

#[test]
fn test_buffer_pool_many_buffers() {
    let mut pool = BufferPool::new();

    // Add many buffers with various overlaps
    for i in 0..100 {
        let start = i * 5;
        let end = start + 10;
        pool.add_buffer(i, start, end);
    }

    pool.compute_coloring();

    // Verify all buffers got colors
    for i in 0..100 {
        assert!(
            pool.get_color(i).is_some(),
            "Buffer {i} should have a color"
        );
    }

    // Verify overlapping buffers have different colors
    for i in 0..99 {
        let color_i = pool
            .get_color(i)
            .expect("buffer i should have an assigned color");
        let color_next = pool
            .get_color(i + 1)
            .expect("buffer i+1 should have an assigned color");

        // Buffers i and i+1 overlap (i ends at i*5+10, i+1 starts at (i+1)*5 = i*5+5)
        // So they should have different colors
        if (i * 5 + 10) > ((i + 1) * 5) {
            assert_ne!(
                color_i,
                color_next,
                "Overlapping buffers {} and {} should have different colors",
                i,
                i + 1
            );
        }
    }
}

#[test]
fn test_buffer_lifetime_overlap() {
    // Test the lifetime overlap detection logic
    let a = BufferLifetime {
        alloc_id: 0,
        start: 0,
        end: 10,
    };
    let b = BufferLifetime {
        alloc_id: 1,
        start: 5,
        end: 15,
    };
    let c = BufferLifetime {
        alloc_id: 2,
        start: 20,
        end: 30,
    };

    // a and b overlap (0 < 15 && 5 < 10)
    // a and c don't overlap (0 < 30 && 20 < 10 is false)
    // b and c don't overlap (5 < 30 && 20 < 15 is false)

    // This is tested indirectly through BufferPool, but we can verify the logic
    assert!(lifetimes_overlap(&a, &b)); // a and b overlap
    assert!(!lifetimes_overlap(&a, &c)); // a and c don't overlap
    assert!(!lifetimes_overlap(&b, &c)); // b and c don't overlap
}
