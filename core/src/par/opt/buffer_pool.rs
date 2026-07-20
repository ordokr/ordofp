//! Buffer reuse and allocation pooling.
//!
//! This module implements buffer allocation reuse to reduce memory footprint
//! and enable more parallelism.
//!
//! **Status: not wired into the GPU runtime.** The live GPU execution path
//! uses `KernelCache`'s own buffer pool (`backend/wgpu/kernel.rs`); the
//! lifetime/interference analysis here is currently exercised only by tests
//! and benches, and is kept as the basis for a future smarter allocator.

#![cfg(all(feature = "par", feature = "gpu-wgpu", feature = "gpu-buffer-pool"))]

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Buffer lifetime for interference graph construction.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BufferLifetime {
    /// Allocation site identifier.
    pub alloc_id: usize,
    /// Start of lifetime.
    pub start: usize,
    /// End of lifetime.
    pub end: usize,
}

/// Check if two buffer lifetimes overlap.
pub fn lifetimes_overlap(a: &BufferLifetime, b: &BufferLifetime) -> bool {
    // Two lifetimes overlap if: a.start < b.end && b.start < a.end
    a.start < b.end && b.start < a.end
}

/// Interference graph for buffer lifetimes.
///
/// Two buffers interfere if their lifetimes overlap.
pub struct InterferenceGraph {
    /// Map from buffer ID to set of interfering buffer IDs.
    edges: BTreeMap<usize, alloc::collections::BTreeSet<usize>>,
    /// Store buffer lifetimes for interference detection.
    lifetimes: Vec<BufferLifetime>,
}

impl InterferenceGraph {
    /// Create new interference graph.
    #[inline]
    fn new() -> Self {
        Self {
            edges: BTreeMap::new(),
            lifetimes: Vec::with_capacity(16),
        }
    }

    /// Add interference between two buffers.
    #[inline]
    fn add_interference(&mut self, a: usize, b: usize) {
        self.edges.entry(a).or_default().insert(b);
        self.edges.entry(b).or_default().insert(a);
    }

    /// Get interfering buffers for a given buffer.
    #[inline]
    fn get_interferences(&self, buffer_id: usize) -> Option<&alloc::collections::BTreeSet<usize>> {
        self.edges.get(&buffer_id)
    }

    /// Add a buffer lifetime.
    #[inline]
    fn add_lifetime(&mut self, lifetime: BufferLifetime) {
        self.lifetimes.push(lifetime);
    }

    /// Get all lifetimes.
    #[inline]
    fn get_all_lifetimes(&self) -> &[BufferLifetime] {
        &self.lifetimes
    }
}

/// Greedy graph coloring for buffer reuse.
///
/// Colors the interference graph to determine which buffers can share memory.
pub struct BufferPool {
    /// Interference graph.
    graph: InterferenceGraph,
    /// Color assignment (buffer ID -> color ID).
    colors: BTreeMap<usize, usize>,
}

impl BufferPool {
    /// Create new buffer pool.
    #[inline]
    pub fn new() -> Self {
        Self {
            graph: InterferenceGraph::new(),
            colors: BTreeMap::new(),
        }
    }

    /// Add buffer lifetime and detect interference.
    #[inline]
    pub fn add_buffer(&mut self, alloc_id: usize, start: usize, end: usize) {
        let lifetime = BufferLifetime {
            alloc_id,
            start,
            end,
        };

        // Scan existing lifetimes for overlaps BEFORE pushing the new one so we
        // can take a shared borrow of the slice without a heap clone.
        let mut interfering: Vec<usize> = Vec::new();
        for other in self.graph.get_all_lifetimes() {
            if other.alloc_id != alloc_id && lifetimes_overlap(&lifetime, other) {
                interfering.push(other.alloc_id);
            }
        }

        // Now add the lifetime (takes &mut self.graph).
        self.graph.add_lifetime(lifetime);

        // Register interference edges.
        for other_id in interfering {
            self.graph.add_interference(alloc_id, other_id);
        }
    }

    /// Compute coloring (greedy algorithm).
    ///
    /// Assigns each buffer the smallest color not used by its neighbors in the
    /// interference graph. This determines which buffers can share memory.
    pub fn compute_coloring(&mut self) {
        // Greedy coloring: assign each buffer the smallest color not used by neighbors
        // Get all buffer IDs from both edges and lifetimes to ensure all buffers are colored
        let mut buffer_ids: alloc::collections::BTreeSet<usize> =
            self.graph.edges.keys().copied().collect();
        for lifetime in self.graph.get_all_lifetimes() {
            buffer_ids.insert(lifetime.alloc_id);
        }
        let buffer_ids: Vec<usize> = buffer_ids.into_iter().collect();

        // Sort by degree (number of neighbors) for better coloring
        // Buffers with more neighbors are harder to color, so color them first
        let mut sorted_ids = buffer_ids;
        sorted_ids.sort_by_key(|&id| {
            // Sort by negative degree (higher degree first)
            let degree = self
                .graph
                .get_interferences(id)
                .map_or(0, std::collections::BTreeSet::len);
            core::cmp::Reverse(degree)
        });

        // Assign colors greedily
        for buffer_id in sorted_ids {
            let interferences = self.graph.get_interferences(buffer_id);

            // Find the smallest color not used by any neighbor
            let mut color = 0;
            loop {
                let color_in_use = interferences
                    .into_iter()
                    .flatten()
                    .any(|&neighbor_id| self.colors.get(&neighbor_id).is_some_and(|&c| c == color));

                if !color_in_use {
                    // Found available color
                    self.colors.insert(buffer_id, color);
                    break;
                }

                color += 1;
            }
        }
    }

    /// Get color for buffer (which pool it belongs to).
    #[inline]
    pub fn get_color(&self, buffer_id: usize) -> Option<usize> {
        self.colors.get(&buffer_id).copied()
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new()
    }
}
