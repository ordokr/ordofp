//! Checkpoint Effect - Serializable Computation State
//!
//! This module provides **checkpoint and resume** capabilities for effectful
//! computations. Checkpoints capture the current state of a computation,
//! allowing it to be serialized, stored, and later resumed.
//!
//! # Key Concepts
//!
//! - **Checkpoint**: A snapshot of computation state at a specific point
//! - **Resume**: Continue a computation from a previously saved checkpoint
//! - **Serializable State**: State that can be persisted and restored
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::nexus::effects::checkpoint::*;
//!
//! // Build a checkpoint computation: save two checkpoints and chain them.
//! let comp = checkpoint("first", 10i32)
//!     .and_then(|first_id| checkpoint("second", 20i32).map(move |second_id| (first_id, second_id)));
//!
//! let mut ctx = CheckpointContext::new();
//! let (first_id, second_id) = comp.run(&mut ctx);
//!
//! assert_eq!(ctx.restore::<i32>(&first_id), Some(10));
//! assert_eq!(ctx.restore::<i32>(&second_id), Some(20));
//!
//! // Later: resume from the latest checkpoint with a given name.
//! let mut ctx2 = CheckpointContext::new();
//! ctx2.checkpoint("progress", &42i32);
//! let restored: Option<i32> = ctx2.restore_latest("progress");
//! assert_eq!(restored, Some(42));
//! ```
//!
//! # Use Cases
//!
//! - **Long-running computations**: Save progress periodically
//! - **Fault tolerance**: Resume after crashes
//! - **Debugging**: Capture state at specific points
//! - **Testing**: Replay from known states
//!
//! # Verification Tier
//!
//! **Tier 1**: Tested via unit tests verifying checkpoint/resume semantics.
//!
//! # Limitations
//!
//! - **No network transport**: This provides serialization, not distribution
//! - **Closure capture limits**: Captured closures must be serializable
//! - **No automatic checkpointing**: Explicit checkpoint calls required
//! - **State must be Clone**: Checkpoint requires cloning current state
//!
//! # Design Philosophy
//!
//! This provides the **serialization foundation** for distributed effects.
//! Actual network transport and coordination are left to users or future
//! modules. We focus on correct state capture and restoration.

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

use crate::nexus::effect::EffectMarker;
use crate::nexus::row::Row;

// =============================================================================
// Effect Marker
// =============================================================================

/// Bit flag for Checkpoint effect.
pub const CHECKPOINT_BIT: u128 = 1 << 35;

/// The Checkpoint effect marker type.
#[derive(Copy, Clone, Debug)]
pub struct CheckpointEffect;

impl EffectMarker for CheckpointEffect {
    const BIT: u128 = CHECKPOINT_BIT;
    const NAME: &'static str = "Checkpoint";
}

/// Type alias for a row containing only Checkpoint.
pub type CheckpointRow = Row<CHECKPOINT_BIT>;

// =============================================================================
// Checkpoint Identifier
// =============================================================================

/// Unique identifier for a checkpoint.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CheckpointId {
    /// The name/label of this checkpoint.
    name: String,
    /// Sequence number for ordering.
    sequence: u64,
}

impl CheckpointId {
    /// Create a new checkpoint ID.
    pub fn new(name: impl Into<String>, sequence: u64) -> Self {
        CheckpointId {
            name: name.into(),
            sequence,
        }
    }

    /// Get the checkpoint name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the sequence number.
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
}

impl fmt::Display for CheckpointId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.sequence)
    }
}

// =============================================================================
// Serializable State
// =============================================================================

/// A value that can be checkpointed.
///
/// This trait marks types that can be serialized and restored.
/// Implementations may delegate to serde; the byte-level contract keeps
/// this module dependency-free.
pub trait Checkpointable: Clone + 'static {
    /// Convert to a serializable representation.
    fn to_bytes(&self) -> Vec<u8>;

    /// Restore from a serializable representation.
    fn from_bytes(bytes: &[u8]) -> Option<Self>;
}

/// Implements `Checkpointable` for fixed-width integers as little-endian
/// bytes; `from_bytes` accepts any buffer holding at least the type's width.
macro_rules! impl_checkpointable_int {
    ($($ty:ty),+ $(,)?) => {
        $(impl Checkpointable for $ty {
            fn to_bytes(&self) -> Vec<u8> {
                self.to_le_bytes().to_vec()
            }

            fn from_bytes(bytes: &[u8]) -> Option<Self> {
                const N: usize = core::mem::size_of::<$ty>();
                let arr: [u8; N] = bytes.get(..N)?.try_into().ok()?;
                Some(<$ty>::from_le_bytes(arr))
            }
        })+
    };
}

impl_checkpointable_int!(i32, i64, u64);

impl Checkpointable for bool {
    fn to_bytes(&self) -> Vec<u8> {
        vec![u8::from(*self)]
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        bytes.first().map(|&b| b != 0)
    }
}

impl Checkpointable for String {
    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        String::from_utf8(bytes.to_vec()).ok()
    }
}

impl<T: Checkpointable> Checkpointable for Vec<T> {
    fn to_bytes(&self) -> Vec<u8> {
        // 8 bytes for the length header, then 8 bytes per element for the
        // per-item length prefix, plus the element bytes themselves. Using
        // `8 + self.len() * 16` as a conservative lower bound avoids the
        // first few reallocations for typical small-to-medium vectors.
        let mut result = Vec::with_capacity(8 + self.len() * 16);
        // Store length as u64
        result.extend_from_slice(&(self.len() as u64).to_le_bytes());
        // Store each element with its length prefix
        for item in self {
            let bytes = item.to_bytes();
            result.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
            result.extend_from_slice(&bytes);
        }
        result
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 8 {
            return None;
        }
        let len = usize::try_from(u64::from_le_bytes(bytes[..8].try_into().ok()?)).ok()?;
        let mut result = Vec::with_capacity(len);
        let mut offset = 8;
        for _ in 0..len {
            if offset + 8 > bytes.len() {
                return None;
            }
            let item_len = usize::try_from(u64::from_le_bytes(
                bytes[offset..offset + 8].try_into().ok()?,
            ))
            .ok()?;
            offset += 8;
            if offset + item_len > bytes.len() {
                return None;
            }
            let item = T::from_bytes(&bytes[offset..offset + item_len])?;
            result.push(item);
            offset += item_len;
        }
        Some(result)
    }
}

impl Checkpointable for () {
    fn to_bytes(&self) -> Vec<u8> {
        Vec::new()
    }

    fn from_bytes(_bytes: &[u8]) -> Option<Self> {
        Some(())
    }
}

// Tuple implementations
impl<A: Checkpointable, B: Checkpointable> Checkpointable for (A, B) {
    fn to_bytes(&self) -> Vec<u8> {
        let a_bytes = self.0.to_bytes();
        let b_bytes = self.1.to_bytes();
        // Capacity: 8-byte length prefix for A, A bytes, B bytes.
        let mut result = Vec::with_capacity(8 + a_bytes.len() + b_bytes.len());
        result.extend_from_slice(&(a_bytes.len() as u64).to_le_bytes());
        result.extend_from_slice(&a_bytes);
        result.extend_from_slice(&b_bytes);
        result
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 8 {
            return None;
        }
        let a_len = usize::try_from(u64::from_le_bytes(bytes[..8].try_into().ok()?)).ok()?;
        if bytes.len() < 8 + a_len {
            return None;
        }
        let a = A::from_bytes(&bytes[8..8 + a_len])?;
        let b = B::from_bytes(&bytes[8 + a_len..])?;
        Some((a, b))
    }
}

// =============================================================================
// Checkpoint Data
// =============================================================================

/// A saved checkpoint containing computation state.
#[derive(Clone, Debug)]
pub struct Checkpoint {
    /// Unique identifier for this checkpoint.
    pub id: CheckpointId,
    /// Serialized state data.
    pub state: Vec<u8>,
    /// Additional metadata (key-value pairs).
    pub metadata: BTreeMap<String, String>,
    /// Timestamp when checkpoint was created (as string for `no_std`).
    pub timestamp: String,
}

impl Checkpoint {
    /// Create a new checkpoint.
    pub fn new<T: Checkpointable>(id: CheckpointId, value: &T) -> Self {
        Checkpoint {
            id,
            state: value.to_bytes(),
            metadata: BTreeMap::new(),
            timestamp: String::new(), // Would use actual timestamp with std
        }
    }

    /// Create a checkpoint with metadata.
    pub fn with_metadata<T: Checkpointable>(
        id: CheckpointId,
        value: &T,
        metadata: BTreeMap<String, String>,
    ) -> Self {
        Checkpoint {
            id,
            state: value.to_bytes(),
            metadata,
            timestamp: String::new(),
        }
    }

    /// Restore the checkpointed value.
    pub fn restore<T: Checkpointable>(&self) -> Option<T> {
        T::from_bytes(&self.state)
    }

    /// Add metadata to this checkpoint.
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Get metadata value.
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Get the size of the serialized state in bytes.
    pub fn state_size(&self) -> usize {
        self.state.len()
    }
}

// =============================================================================
// Checkpoint Store
// =============================================================================

/// Storage for checkpoints.
///
/// This is an in-memory store. For production use, implement
/// persistence to disk, database, or distributed storage.
#[derive(Clone, Debug, Default)]
pub struct CheckpointStore {
    /// Stored checkpoints by ID.
    checkpoints: BTreeMap<CheckpointId, Checkpoint>,
    /// Next sequence number.
    next_sequence: u64,
    /// Maximum checkpoints to retain (0 = unlimited).
    max_checkpoints: usize,
}

impl CheckpointStore {
    /// Create a new checkpoint store.
    pub fn new() -> Self {
        CheckpointStore {
            checkpoints: BTreeMap::new(),
            next_sequence: 0,
            max_checkpoints: 0,
        }
    }

    /// Create a store with a maximum number of checkpoints.
    pub fn with_max_checkpoints(max: usize) -> Self {
        CheckpointStore {
            checkpoints: BTreeMap::new(),
            next_sequence: 0,
            max_checkpoints: max,
        }
    }

    /// Save a checkpoint.
    pub fn save<T: Checkpointable>(&mut self, name: impl Into<String>, value: &T) -> CheckpointId {
        self.store(name, |id| Checkpoint::new(id, value))
    }

    /// Save a checkpoint with metadata.
    pub fn save_with_metadata<T: Checkpointable>(
        &mut self,
        name: impl Into<String>,
        value: &T,
        metadata: BTreeMap<String, String>,
    ) -> CheckpointId {
        self.store(name, |id| Checkpoint::with_metadata(id, value, metadata))
    }

    /// Allocate the next id, insert the checkpoint built by `make`, and
    /// enforce the retention limit.
    fn store(
        &mut self,
        name: impl Into<String>,
        make: impl FnOnce(CheckpointId) -> Checkpoint,
    ) -> CheckpointId {
        let id = CheckpointId::new(name, self.next_sequence);
        self.next_sequence += 1;

        self.checkpoints.insert(id.clone(), make(id.clone()));
        self.evict_oldest_over_limit();

        id
    }

    /// Evict the oldest checkpoint once the configured limit is exceeded.
    ///
    /// The oldest checkpoint is the one with the lowest global sequence
    /// number, not the lexicographically-first key (`CheckpointId` Ord is
    /// (name, sequence)).
    fn evict_oldest_over_limit(&mut self) {
        if self.max_checkpoints > 0
            && self.checkpoints.len() > self.max_checkpoints
            && let Some(oldest_id) = self
                .checkpoints
                .keys()
                .min_by_key(|id| id.sequence())
                .cloned()
        {
            self.checkpoints.remove(&oldest_id);
        }
    }

    /// Load a checkpoint by ID.
    pub fn load(&self, id: &CheckpointId) -> Option<&Checkpoint> {
        self.checkpoints.get(id)
    }

    /// Load the latest checkpoint with a given name.
    pub fn load_latest(&self, name: &str) -> Option<&Checkpoint> {
        self.checkpoints
            .iter()
            .rev()
            .find(|(id, _)| id.name() == name)
            .map(|(_, cp)| cp)
    }

    /// Delete a checkpoint.
    pub fn delete(&mut self, id: &CheckpointId) -> bool {
        self.checkpoints.remove(id).is_some()
    }

    /// Delete all checkpoints with a given name.
    pub fn delete_by_name(&mut self, name: &str) -> usize {
        let to_remove: Vec<_> = self
            .checkpoints
            .keys()
            .filter(|id| id.name() == name)
            .cloned()
            .collect();
        let count = to_remove.len();
        for id in to_remove {
            self.checkpoints.remove(&id);
        }
        count
    }

    /// Clear all checkpoints.
    pub fn clear(&mut self) {
        self.checkpoints.clear();
    }

    /// Get the number of stored checkpoints.
    pub fn len(&self) -> usize {
        self.checkpoints.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.checkpoints.is_empty()
    }

    /// List all checkpoint IDs.
    pub fn list_ids(&self) -> Vec<&CheckpointId> {
        self.checkpoints.keys().collect()
    }

    /// Get total size of all checkpoints in bytes.
    pub fn total_size(&self) -> usize {
        self.checkpoints.values().map(Checkpoint::state_size).sum()
    }
}

// =============================================================================
// Checkpoint Computation
// =============================================================================

/// A computation that can create and use checkpoints.
pub struct CheckpointComputation<A> {
    /// The computation function.
    run_fn: Box<dyn FnOnce(&mut CheckpointContext) -> A>,
}

impl<A: 'static> CheckpointComputation<A> {
    /// Create a new checkpoint computation.
    pub fn new<F: FnOnce(&mut CheckpointContext) -> A + 'static>(f: F) -> Self {
        CheckpointComputation {
            run_fn: Box::new(f),
        }
    }

    /// Run the computation with a context.
    pub fn run(self, ctx: &mut CheckpointContext) -> A {
        (self.run_fn)(ctx)
    }

    /// Pure value (no checkpointing).
    pub fn pure(value: A) -> Self
    where
        A: Clone,
    {
        CheckpointComputation::new(move |_| value)
    }

    /// Map over the result.
    pub fn map<B: 'static, F: FnOnce(A) -> B + 'static>(self, f: F) -> CheckpointComputation<B> {
        CheckpointComputation::new(move |ctx| {
            let a = (self.run_fn)(ctx);
            f(a)
        })
    }

    /// Chain checkpoint computations.
    pub fn and_then<B: 'static, F: FnOnce(A) -> CheckpointComputation<B> + 'static>(
        self,
        f: F,
    ) -> CheckpointComputation<B> {
        CheckpointComputation::new(move |ctx| {
            let a = (self.run_fn)(ctx);
            f(a).run(ctx)
        })
    }
}

// =============================================================================
// Checkpoint Context
// =============================================================================

/// Context for checkpoint computations.
pub struct CheckpointContext {
    /// The checkpoint store.
    pub store: CheckpointStore,
    /// Whether checkpointing is enabled.
    pub enabled: bool,
    /// Statistics.
    pub stats: CheckpointStats,
}

/// Statistics for checkpoint operations.
#[derive(Clone, Debug, Default)]
pub struct CheckpointStats {
    /// Number of checkpoints created.
    pub checkpoints_created: usize,
    /// Number of checkpoints restored.
    pub checkpoints_restored: usize,
    /// Total bytes checkpointed.
    pub bytes_checkpointed: usize,
}

impl CheckpointContext {
    /// Create a new checkpoint context.
    pub fn new() -> Self {
        CheckpointContext {
            store: CheckpointStore::new(),
            enabled: true,
            stats: CheckpointStats::default(),
        }
    }

    /// Create a context with a custom store.
    pub fn with_store(store: CheckpointStore) -> Self {
        CheckpointContext {
            store,
            enabled: true,
            stats: CheckpointStats::default(),
        }
    }

    /// Create a checkpoint of the current state.
    pub fn checkpoint<T: Checkpointable>(
        &mut self,
        name: impl Into<String>,
        value: &T,
    ) -> CheckpointId {
        if !self.enabled {
            // Return a dummy ID when disabled
            return CheckpointId::new("disabled", 0);
        }

        let id = self.store.save(name, value);
        self.stats.checkpoints_created += 1;
        // `save` already serialized `value` into the stored checkpoint; read the
        // byte length back via an O(1) borrow instead of re-serializing the whole
        // value (a full duplicate `to_bytes()` allocation) just to measure it.
        self.stats.bytes_checkpointed += self.store.load(&id).map_or(0, Checkpoint::state_size);
        id
    }

    /// Restore from a checkpoint.
    pub fn restore<T: Checkpointable>(&mut self, id: &CheckpointId) -> Option<T> {
        let checkpoint = self.store.load(id)?;
        let value = checkpoint.restore()?;
        self.stats.checkpoints_restored += 1;
        Some(value)
    }

    /// Restore from the latest checkpoint with a given name.
    pub fn restore_latest<T: Checkpointable>(&mut self, name: &str) -> Option<T> {
        let checkpoint = self.store.load_latest(name)?;
        let value = checkpoint.restore()?;
        self.stats.checkpoints_restored += 1;
        Some(value)
    }

    /// Enable checkpointing.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable checkpointing (for performance).
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if checkpointing is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get statistics.
    pub fn stats(&self) -> &CheckpointStats {
        &self.stats
    }

    /// Reset statistics.
    pub fn reset_stats(&mut self) {
        self.stats = CheckpointStats::default();
    }
}

impl Default for CheckpointContext {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Convenience Functions
// =============================================================================

/// Create a checkpoint (returns a computation).
pub fn checkpoint<T: Checkpointable + 'static>(
    name: impl Into<String> + 'static,
    value: T,
) -> CheckpointComputation<CheckpointId> {
    let name = name.into();
    CheckpointComputation::new(move |ctx| ctx.checkpoint(&name, &value))
}

/// Restore from a checkpoint (returns a computation).
pub fn restore<T: Checkpointable + 'static>(id: CheckpointId) -> CheckpointComputation<Option<T>> {
    CheckpointComputation::new(move |ctx| ctx.restore(&id))
}

/// Restore from the latest checkpoint with a name.
pub fn restore_latest<T: Checkpointable + 'static>(
    name: impl Into<String> + 'static,
) -> CheckpointComputation<Option<T>> {
    let name = name.into();
    CheckpointComputation::new(move |ctx| ctx.restore_latest(&name))
}

/// Pure value in checkpoint context.
pub fn pure<A: Clone + 'static>(value: A) -> CheckpointComputation<A> {
    CheckpointComputation::pure(value)
}

// =============================================================================
// Resumable Computation
// =============================================================================

/// Step function of a [`ResumableComputation`]: maps the current state to
/// the next [`StepResult`].
type StepFn<S, A> = Box<dyn Fn(&S) -> StepResult<S, A>>;

/// A computation that can be suspended and resumed.
///
/// This wraps a stateful computation with checkpoint support.
pub struct ResumableComputation<S: Checkpointable, A> {
    /// Current state.
    state: S,
    /// The step function.
    step: StepFn<S, A>,
    /// Checkpoint name prefix.
    checkpoint_prefix: String,
}

/// Result of a computation step.
#[derive(Clone, Debug)]
pub enum StepResult<S, A> {
    /// Computation continues with new state.
    Continue(S),
    /// Computation is complete with result.
    Done(A),
    /// Computation should checkpoint and continue.
    Checkpoint(S),
}

impl<S: Checkpointable, A: Clone + 'static> ResumableComputation<S, A> {
    /// Create a new resumable computation.
    pub fn new<F: Fn(&S) -> StepResult<S, A> + 'static>(
        initial_state: S,
        step: F,
        checkpoint_prefix: impl Into<String>,
    ) -> Self {
        ResumableComputation {
            state: initial_state,
            step: Box::new(step),
            checkpoint_prefix: checkpoint_prefix.into(),
        }
    }

    /// Run until completion, checkpointing as requested.
    pub fn run(mut self, ctx: &mut CheckpointContext) -> A {
        let mut step_count = 0u64;
        loop {
            match (self.step)(&self.state) {
                StepResult::Continue(new_state) => {
                    self.state = new_state;
                    step_count += 1;
                }
                StepResult::Done(result) => {
                    return result;
                }
                StepResult::Checkpoint(new_state) => {
                    self.state = new_state.clone();
                    let name = alloc::format!("{}_{}", self.checkpoint_prefix, step_count);
                    ctx.checkpoint(&name, &new_state);
                    step_count += 1;
                }
            }
        }
    }

    /// Resume from a checkpoint.
    pub fn resume(ctx: &mut CheckpointContext, checkpoint_prefix: &str) -> Option<S> {
        ctx.restore_latest(checkpoint_prefix)
    }

    /// Get the current state.
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Set the state (for resuming).
    pub fn set_state(&mut self, state: S) {
        self.state = state;
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_checkpoint_id() {
        let id = CheckpointId::new("test", 42);
        assert_eq!(id.name(), "test");
        assert_eq!(id.sequence(), 42);
        assert_eq!(id.to_string(), "test@42");
    }

    #[test]
    fn test_checkpointable_i32() {
        let value: i32 = 12345;
        let bytes = value.to_bytes();
        let restored = i32::from_bytes(&bytes);
        assert_eq!(restored, Some(12345));
    }

    #[test]
    fn test_checkpointable_i64() {
        let value: i64 = 1_234_567_890_123;
        let bytes = value.to_bytes();
        let restored = i64::from_bytes(&bytes);
        assert_eq!(restored, Some(1_234_567_890_123));
    }

    #[test]
    fn test_checkpointable_string() {
        let value = String::from("hello world");
        let bytes = value.to_bytes();
        let restored = String::from_bytes(&bytes);
        assert_eq!(restored, Some(String::from("hello world")));
    }

    #[test]
    fn test_checkpointable_vec() {
        let value: Vec<i32> = vec![1, 2, 3, 4, 5];
        let bytes = value.to_bytes();
        let restored = Vec::<i32>::from_bytes(&bytes);
        assert_eq!(restored, Some(vec![1, 2, 3, 4, 5]));
    }

    #[test]
    fn test_checkpointable_tuple() {
        let value: (i32, String) = (42, String::from("test"));
        let bytes = value.to_bytes();
        let restored = <(i32, String)>::from_bytes(&bytes);
        assert_eq!(restored, Some((42, String::from("test"))));
    }

    #[test]
    fn test_checkpoint_store_basic() {
        let mut store = CheckpointStore::new();

        let id1 = store.save("state_a", &100i32);
        let id2 = store.save("state_b", &200i32);

        assert_eq!(store.len(), 2);

        let cp1 = store
            .load(&id1)
            .expect("checkpoint id1 should exist after save");
        let cp2 = store
            .load(&id2)
            .expect("checkpoint id2 should exist after save");

        assert_eq!(cp1.restore::<i32>(), Some(100));
        assert_eq!(cp2.restore::<i32>(), Some(200));
    }

    #[test]
    fn test_checkpoint_store_latest() {
        let mut store = CheckpointStore::new();

        store.save("counter", &10i32);
        store.save("counter", &20i32);
        store.save("counter", &30i32);

        let latest = store
            .load_latest("counter")
            .expect("latest checkpoint for 'counter' should exist after three saves");
        assert_eq!(latest.restore::<i32>(), Some(30));
    }

    #[test]
    fn test_checkpoint_store_delete() {
        let mut store = CheckpointStore::new();

        let id = store.save("temp", &42i32);
        assert_eq!(store.len(), 1);

        assert!(store.delete(&id));
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_checkpoint_store_max_checkpoints() {
        let mut store = CheckpointStore::with_max_checkpoints(3);

        store.save("a", &1i32);
        store.save("b", &2i32);
        store.save("c", &3i32);
        assert_eq!(store.len(), 3);

        store.save("d", &4i32);
        assert_eq!(store.len(), 3); // Still 3, oldest evicted
    }

    #[test]
    fn test_checkpoint_store_evicts_oldest_by_sequence() {
        // Regression: eviction must remove the OLDEST checkpoint (lowest
        // global sequence), not the lexicographically-first CheckpointId.
        let mut store = CheckpointStore::with_max_checkpoints(2);

        let id_z_first = store.save("z", &1i32); // sequence 0 — oldest
        let id_a = store.save("a", &2i32); // sequence 1
        let id_z_second = store.save("z", &3i32); // sequence 2 — triggers eviction

        assert_eq!(store.len(), 2);
        assert!(
            store.load(&id_z_first).is_none(),
            "oldest (z, 0) must be evicted"
        );
        assert!(store.load(&id_a).is_some(), "(a, 1) must survive");
        assert!(store.load(&id_z_second).is_some(), "(z, 2) must survive");
    }

    #[test]
    fn test_checkpoint_store_save_with_metadata_evicts_oldest_by_sequence() {
        // Same regression for the save_with_metadata eviction path.
        let mut store = CheckpointStore::with_max_checkpoints(2);

        let id_z_first = store.save_with_metadata("z", &1i32, BTreeMap::new());
        let id_a = store.save_with_metadata("a", &2i32, BTreeMap::new());
        let id_z_second = store.save_with_metadata("z", &3i32, BTreeMap::new());

        assert_eq!(store.len(), 2);
        assert!(
            store.load(&id_z_first).is_none(),
            "oldest (z, 0) must be evicted"
        );
        assert!(store.load(&id_a).is_some(), "(a, 1) must survive");
        assert!(store.load(&id_z_second).is_some(), "(z, 2) must survive");
    }

    #[test]
    fn test_checkpoint_context() {
        let mut ctx = CheckpointContext::new();

        let id = ctx.checkpoint("progress", &50i32);
        let restored: Option<i32> = ctx.restore(&id);

        assert_eq!(restored, Some(50));
        assert_eq!(ctx.stats().checkpoints_created, 1);
        assert_eq!(ctx.stats().checkpoints_restored, 1);
    }

    #[test]
    fn test_checkpoint_context_disabled() {
        let mut ctx = CheckpointContext::new();
        ctx.disable();

        let id = ctx.checkpoint("progress", &50i32);
        assert_eq!(id.name(), "disabled");
    }

    #[test]
    fn test_checkpoint_computation() {
        let comp = checkpoint("state", 42i32);
        let mut ctx = CheckpointContext::new();
        let id = comp.run(&mut ctx);

        let restore_comp = restore::<i32>(id);
        let value = restore_comp.run(&mut ctx);

        assert_eq!(value, Some(42));
    }

    #[test]
    fn test_checkpoint_computation_chain() {
        let comp = checkpoint("first", 10i32)
            .and_then(|id1| checkpoint("second", 20i32).map(move |id2| (id1, id2)));

        let mut ctx = CheckpointContext::new();
        let (id1, id2) = comp.run(&mut ctx);

        assert_eq!(ctx.restore::<i32>(&id1), Some(10));
        assert_eq!(ctx.restore::<i32>(&id2), Some(20));
    }

    #[test]
    fn test_resumable_computation() {
        // A simple computation that sums numbers up to a limit
        let step = |state: &(i32, i32)| {
            let (current, target) = *state;
            if current >= target {
                StepResult::Done(current)
            } else if current % 10 == 0 && current > 0 {
                // Checkpoint every 10 steps
                StepResult::Checkpoint((current + 1, target))
            } else {
                StepResult::Continue((current + 1, target))
            }
        };

        let comp = ResumableComputation::new((0i32, 25i32), step, "counter");
        let mut ctx = CheckpointContext::new();
        let result = comp.run(&mut ctx);

        assert_eq!(result, 25);
        assert!(ctx.stats().checkpoints_created >= 2); // At 10 and 20
    }

    #[test]
    fn test_resumable_computation_resume() {
        let mut ctx = CheckpointContext::new();

        // First, run partially and checkpoint
        ctx.checkpoint("progress", &(15i32, 30i32));

        // Now resume
        let state: Option<(i32, i32)> =
            ResumableComputation::<(i32, i32), i32>::resume(&mut ctx, "progress");

        assert_eq!(state, Some((15, 30)));
    }

    #[test]
    fn test_checkpoint_metadata() {
        let mut store = CheckpointStore::new();
        let mut metadata = BTreeMap::new();
        metadata.insert(String::from("version"), String::from("1.0"));
        metadata.insert(String::from("author"), String::from("test"));

        let id = store.save_with_metadata("state", &42i32, metadata);

        let cp = store
            .load(&id)
            .expect("checkpoint should exist after save_with_metadata");
        assert_eq!(cp.get_metadata("version"), Some(&String::from("1.0")));
        assert_eq!(cp.get_metadata("author"), Some(&String::from("test")));
    }

    #[test]
    fn test_total_size() {
        let mut store = CheckpointStore::new();

        store.save("a", &1i32); // 4 bytes
        store.save("b", &2i64); // 8 bytes
        store.save("c", &String::from("hello")); // 5 bytes

        assert_eq!(store.total_size(), 4 + 8 + 5);
    }

    /// Test demonstrating a realistic checkpoint workflow.
    #[test]
    fn test_realistic_checkpoint_workflow() {
        let mut ctx = CheckpointContext::new();

        // Simulate a long computation with periodic checkpoints
        let mut state = 0i32;
        let mut checkpoint_count = 0;

        for i in 0..100 {
            state += i;

            // Checkpoint every 25 iterations
            if i % 25 == 0 && i > 0 {
                ctx.checkpoint(alloc::format!("iteration_{i}"), &state);
                checkpoint_count += 1;
            }
        }

        assert_eq!(checkpoint_count, 3); // At 25, 50, 75
        assert_eq!(ctx.stats().checkpoints_created, 3);

        // Restore from a middle checkpoint
        let restored: Option<i32> = ctx.restore_latest("iteration_50");
        assert!(restored.is_some());

        // The restored value should be the sum at iteration 50
        // Sum from 0 to 50 = 50 * 51 / 2 = 1275
        assert_eq!(restored, Some(1275));
    }
}
