//! Distributed Execution
//!
//! > *"Distributio est virtus computationis"*
//! > — Distribution is the virtue of computation. (Latin)
//!
//! This module provides infrastructure for distributed execution of
//! UCIR computations across multiple nodes in a cluster.
//!
//! # Overview
//!
//! The distributed execution system enables:
//!
//! - **Serializable Computations**: UCIR nodes can be serialized and sent to remote nodes
//! - **Distributed Effects**: Effect handlers that span multiple nodes
//! - **Cluster Management**: Discovery, load balancing, and fault tolerance
//!
//! # Architecture
//!
//! ```text
//!     ┌─────────────────────────────────────────────────────┐
//!     │                  COORDINATOR NODE                    │
//!     │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
//!     │  │   UCIR IR   │──│ Serializer  │──│  Scheduler  │  │
//!     │  └─────────────┘  └─────────────┘  └─────────────┘  │
//!     └───────────────────────┬─────────────────────────────┘
//!                             │
//!           ┌─────────────────┼─────────────────┐
//!           ▼                 ▼                 ▼
//!     ┌──────────┐      ┌──────────┐      ┌──────────┐
//!     │  Node 1  │      │  Node 2  │      │  Node 3  │
//!     │ Executor │      │ Executor │      │ Executor │
//!     └──────────┘      └──────────┘      └──────────┘
//! ```
//!
//! # Scholastic Naming
//!
//! | English | Latin | Etymology |
//! |---------|-------|-----------|
//! | Node | Nodus | *nodus* = knot, node |
//! | Cluster | Grex | *grex* = flock, cluster |
//! | Remote | Remotus | *remotus* = distant, far |
//! | Affinity | Affinitas | *affinitas* = relationship |
//! | Protocol | Protocollum | *protocollum* = first sheet |

mod cluster;
mod effect;
mod node;
mod protocol;
mod serializable;

pub use cluster::*;
pub use effect::*;
pub use node::*;
pub use protocol::*;
pub use serializable::*;
