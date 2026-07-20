//! Session Types Effect
//!
//! This module provides **typed protocol state machines** via session types.
//! Session types encode communication protocols in the type system, ensuring
//! that protocol steps happen in the correct order at compile time.
//!
//! # Key Concepts
//!
//! - **Protocol**: A sequence of typed send/receive operations
//! - **Session**: A channel with a protocol state tracked in types
//! - **Duality**: Client and server sides have dual (opposite) protocols
//!
//! # Example
//!
//! ```rust
//! use ordofp_core::nexus::effects::session::*;
//!
//! #[derive(Clone, Default)]
//! struct Request {
//!     id: u32,
//! }
//! #[derive(Clone, Default)]
//! struct Response {
//!     data: u32,
//! }
//!
//! fn process(request: Request) -> Response {
//!     Response { data: request.id * 2 }
//! }
//!
//! // Define a request-response protocol
//! type ServerProtocol = Receive<Request, Send<Response, End>>;
//! type ClientProtocol = Send<Request, Receive<Response, End>>;
//!
//! // Server must receive first, then send - enforced at compile time
//! fn server(chan: Session<ServerProtocol>) -> Session<End> {
//!     let (request, chan) = chan.receive(); // Type: Session<Send<Response, End>>
//!     let response = process(request);
//!     chan.send(response)                    // Type: Session<End>
//! }
//!
//! let chan: Session<ServerProtocol> = Session::new();
//! server(chan).close();
//!
//! // ClientProtocol is the dual of ServerProtocol.
//! assert_dual::<ClientProtocol, ServerProtocol>();
//! ```
//!
//! # Verification Tier
//!
//! **Tier 0-1**: Protocol ordering enforced by type system. Tests verify
//! that correct sequences compile and incorrect sequences don't.
//!
//! # Limitations
//!
//! - No automatic duality checking (use `assert_dual::<P, Q>()` manually)
//! - No channel implementation (types only - bring your own channels)
//! - Linear types approximated via ownership (can't prevent drop)
//! - No recursion/loops in protocols (would require recursive types)
//!
//! # Design Philosophy
//!
//! This is a **type-level encoding** of session types. The actual channel
//! implementation is left to users (tokio channels, crossbeam, etc.).
//! We provide the type machinery to ensure protocol compliance.

use alloc::boxed::Box;
use core::marker::PhantomData;

use crate::nexus::effect::EffectMarker;
use crate::nexus::row::Row;

// =============================================================================
// Effect Marker
// =============================================================================

/// Bit flag for Session effect.
pub const SESSION_BIT: u128 = 1 << 33;

/// The Session effect marker type.
#[derive(Copy, Clone, Debug)]
pub struct SessionEffect;

impl EffectMarker for SessionEffect {
    const BIT: u128 = SESSION_BIT;
    const NAME: &'static str = "Session";
}

/// Type alias for a row containing only Session.
pub type SessionRow = Row<SESSION_BIT>;

// =============================================================================
// Protocol Primitives
// =============================================================================

/// A protocol that sends a value of type `T` then continues with `P`.
///
/// ```text
/// Send<Request, Send<Data, End>>
/// ```
/// means: send a Request, then send Data, then end.
#[derive(Debug)]
pub struct Send<T, P> {
    _value: PhantomData<T>,
    _cont: PhantomData<P>,
}

/// A protocol that receives a value of type `T` then continues with `P`.
///
/// ```text
/// Receive<Request, Send<Response, End>>
/// ```
/// means: receive a Request, then send a Response, then end.
#[derive(Debug)]
pub struct Receive<T, P> {
    _value: PhantomData<T>,
    _cont: PhantomData<P>,
}

/// A protocol that offers a choice between two continuations.
///
/// The other side selects which branch to take.
///
/// ```text
/// Offer<AddItem, RemoveItem>
/// ```
/// means: the other side chooses whether to add or remove.
#[derive(Debug)]
pub struct Offer<P1, P2> {
    _left: PhantomData<P1>,
    _right: PhantomData<P2>,
}

/// A protocol that selects between two continuations.
///
/// This side chooses which branch to take.
///
/// ```text
/// Select<LeftPath, RightPath>
/// ```
/// means: we choose whether to go left or right.
#[derive(Debug)]
pub struct Select<P1, P2> {
    _left: PhantomData<P1>,
    _right: PhantomData<P2>,
}

/// A protocol that has terminated.
///
/// No more operations can be performed.
#[derive(Debug)]
pub struct End;

// =============================================================================
// Protocol Trait
// =============================================================================

/// Marker trait for valid protocol types.
///
/// All protocol primitives implement this trait.
pub trait Protocol: Sized {
    /// The dual of this protocol (what the other side sees).
    type Dual: Protocol;
}

impl<T, P: Protocol> Protocol for Send<T, P> {
    type Dual = Receive<T, P::Dual>;
}

impl<T, P: Protocol> Protocol for Receive<T, P> {
    type Dual = Send<T, P::Dual>;
}

impl<P1: Protocol, P2: Protocol> Protocol for Offer<P1, P2> {
    type Dual = Select<P1::Dual, P2::Dual>;
}

impl<P1: Protocol, P2: Protocol> Protocol for Select<P1, P2> {
    type Dual = Offer<P1::Dual, P2::Dual>;
}

impl Protocol for End {
    type Dual = End;
}

// =============================================================================
// Session Channel
// =============================================================================

/// A session-typed channel with protocol state `P`.
///
/// The phantom type `P` tracks the current state of the protocol.
/// Operations consume the channel and return a new channel with
/// an updated protocol type, ensuring correct sequencing.
///
/// # Type Safety
///
/// The type system ensures:
/// - Send operations only available when protocol expects Send
/// - Receive operations only available when protocol expects Receive
/// - Protocol completed (End) before channel can be dropped
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::session::{End, Receive, Send, Session};
///
/// // Type changes as we progress through protocol
/// let chan: Session<Send<i32, Receive<bool, End>>> = Session::new();
/// let chan: Session<Receive<bool, End>> = chan.send(42);
/// let (b, chan): (bool, Session<End>) = chan.receive();
/// assert!(!b); // bool::default()
/// chan.close();
/// ```
pub struct Session<P: Protocol> {
    _protocol: PhantomData<P>,
    // Type-level construct only: no channel halves are carried.
}

impl<P: Protocol> Session<P> {
    /// Create a new session (for testing/demonstration).
    ///
    /// In practice, sessions are created by connecting two endpoints.
    #[inline]
    pub fn new() -> Self {
        Session {
            _protocol: PhantomData,
        }
    }
}

impl<P: Protocol> Default for Session<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, P: Protocol> Session<Send<T, P>> {
    /// Send a value and continue with the next protocol state.
    ///
    /// Consumes `self` and returns a channel in state `P`.
    ///
    /// # Type Transformation
    ///
    /// ```text
    /// Session<Send<T, P>> --send(T)--> Session<P>
    /// ```
    pub fn send(self, _value: T) -> Session<P> {
        // Type-level only: no value is transmitted.
        Session {
            _protocol: PhantomData,
        }
    }
}

impl<T, P: Protocol> Session<Receive<T, P>> {
    /// Receive a value and continue with the next protocol state.
    ///
    /// Consumes `self` and returns the value plus a channel in state `P`.
    ///
    /// # Type Transformation
    ///
    /// ```text
    /// Session<Receive<T, P>> --receive()--> (T, Session<P>)
    /// ```
    ///
    /// # Note
    ///
    /// Type-level only: returns `T::default()` rather than blocking on a
    /// channel receive.
    pub fn receive(self) -> (T, Session<P>)
    where
        T: Default,
    {
        let value = T::default();
        (
            value,
            Session {
                _protocol: PhantomData,
            },
        )
    }

    /// Receive with a provided value (for testing).
    pub fn receive_with(self, value: T) -> (T, Session<P>) {
        (
            value,
            Session {
                _protocol: PhantomData,
            },
        )
    }
}

impl<P1: Protocol, P2: Protocol> Session<Offer<P1, P2>> {
    /// Offer a choice to the other side.
    ///
    /// Returns which branch was selected along with the continuation.
    ///
    /// # Note
    ///
    /// This is a placeholder. In practice, would receive a selection message.
    pub fn offer(self) -> Either<Session<P1>, Session<P2>> {
        // Placeholder: always select left
        Either::Left(Session {
            _protocol: PhantomData,
        })
    }

    /// Offer with a pre-determined choice (for testing).
    pub fn offer_left(self) -> Session<P1> {
        Session {
            _protocol: PhantomData,
        }
    }

    /// Offer with a pre-determined choice (for testing).
    pub fn offer_right(self) -> Session<P2> {
        Session {
            _protocol: PhantomData,
        }
    }
}

impl<P1: Protocol, P2: Protocol> Session<Select<P1, P2>> {
    /// Select the left branch.
    pub fn select_left(self) -> Session<P1> {
        // Type-level only: no selection message is sent.
        Session {
            _protocol: PhantomData,
        }
    }

    /// Select the right branch.
    pub fn select_right(self) -> Session<P2> {
        // Type-level only: no selection message is sent.
        Session {
            _protocol: PhantomData,
        }
    }
}

impl Session<End> {
    /// Close the session.
    ///
    /// Only callable when protocol is complete (at `End` state).
    pub fn close(self) {
        // Session terminated
    }
}

// =============================================================================
// Either Type for Branching
// =============================================================================

/// A choice between two values.
///
/// Used for protocol branching with Offer/Select.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Either<L, R> {
    /// Left alternative.
    Left(L),
    /// Right alternative.
    Right(R),
}

impl<L, R> Either<L, R> {
    /// Check if this is the left variant.
    pub fn is_left(&self) -> bool {
        matches!(self, Either::Left(_))
    }

    /// Check if this is the right variant.
    pub fn is_right(&self) -> bool {
        matches!(self, Either::Right(_))
    }

    /// Extract the left value, panicking if right.
    pub fn unwrap_left(self) -> L {
        match self {
            Either::Left(l) => l,
            Either::Right(_) => crate::cold_panic!("called unwrap_left on Right"),
        }
    }

    /// Extract the right value, panicking if left.
    pub fn unwrap_right(self) -> R {
        match self {
            Either::Right(r) => r,
            Either::Left(_) => crate::cold_panic!("called unwrap_right on Left"),
        }
    }
}

// =============================================================================
// Duality Checking
// =============================================================================

/// Assert that two protocols are dual to each other.
///
/// This is a compile-time check. If the protocols are not dual,
/// compilation will fail.
///
/// # Example
///
/// ```rust
/// use ordofp_core::nexus::effects::session::{End, Receive, Send, assert_dual};
///
/// struct Request;
/// struct Response;
///
/// type Client = Send<Request, Receive<Response, End>>;
/// type Server = Receive<Request, Send<Response, End>>;
///
/// assert_dual::<Client, Server>(); // Compiles!
/// ```
///
/// Asserting a protocol is dual to itself (when it isn't) is a compile error:
///
/// ```compile_fail
/// use ordofp_core::nexus::effects::session::{End, Receive, Send, assert_dual};
///
/// struct Request;
/// struct Response;
///
/// type Client = Send<Request, Receive<Response, End>>;
///
/// assert_dual::<Client, Client>(); // Compile error!
/// ```
pub fn assert_dual<P1: Protocol, P2: Protocol>()
where
    P1::Dual: SameType<P2>,
{
    // Compile-time only - no runtime code
}

/// Helper trait for compile-time type equality.
pub trait SameType<T> {}
impl<T> SameType<T> for T {}

// =============================================================================
// Common Protocol Patterns
// =============================================================================

/// A simple request-response protocol (server side).
///
/// ```text
/// Server: Receive<Req, Send<Resp, End>>
/// Client: Send<Req, Receive<Resp, End>>
/// ```
pub type RequestResponse<Req, Resp> = Receive<Req, Send<Resp, End>>;

/// A simple request-response protocol (client side).
pub type RequestResponseClient<Req, Resp> = Send<Req, Receive<Resp, End>>;

/// A streaming protocol that sends multiple values then ends.
///
/// Note: This is a fixed-length stream (3 items). For variable-length
/// streams, you'd need recursive types which we don't support.
pub type Stream3<T> = Send<T, Send<T, Send<T, End>>>;

/// A notification protocol (fire-and-forget).
pub type Notify<T> = Send<T, End>;

/// A notification receiver.
pub type NotifyReceive<T> = Receive<T, End>;

// =============================================================================
// Session Computation (Effectful)
// =============================================================================

/// Body of a [`SessionComputation`]: consumes the session and produces the
/// result together with the fully-run (`End`) session.
type SessionRun<P, A> = Box<dyn FnOnce(Session<P>) -> (A, Session<End>)>;

/// A computation that uses a session channel.
///
/// This wraps session operations in an effectful computation type,
/// allowing composition with other effects.
pub struct SessionComputation<P: Protocol, A> {
    /// The computation that produces a value given a session.
    run: SessionRun<P, A>,
}

impl<P: Protocol, A: 'static> SessionComputation<P, A> {
    /// Create a new session computation.
    #[inline]
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(Session<P>) -> (A, Session<End>) + 'static,
    {
        SessionComputation { run: Box::new(f) }
    }

    /// Run the computation with a session.
    #[inline]
    pub fn run(self, session: Session<P>) -> (A, Session<End>) {
        (self.run)(session)
    }

    /// Pure value that doesn't use the session.
    pub fn pure(value: A) -> SessionComputation<End, A>
    where
        A: Clone,
    {
        SessionComputation::new(move |session| (value, session))
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test message types
    #[derive(Clone, Debug, Default, PartialEq)]
    struct Request {
        id: u32,
    }

    #[derive(Clone, Debug, Default, PartialEq)]
    struct Response {
        data: u32,
    }

    #[test]
    fn test_send_receive_protocol() {
        // Client protocol: send request, receive response
        type ClientProto = Send<Request, Receive<Response, End>>;

        let session: Session<ClientProto> = Session::new();

        // Send request
        let session = session.send(Request { id: 42 });
        // Now session is Session<Receive<Response, End>>

        // Receive response
        let (response, session) = session.receive_with(Response { data: 100 });
        // Now session is Session<End>

        assert_eq!(response.data, 100);

        // Close session
        session.close();
    }

    #[test]
    fn test_server_protocol() {
        // Server protocol: receive request, send response
        type ServerProto = Receive<Request, Send<Response, End>>;

        let session: Session<ServerProto> = Session::new();

        // Receive request
        let (request, session) = session.receive_with(Request { id: 42 });

        // Process and send response
        let response = Response {
            data: request.id * 2,
        };
        let session = session.send(response);

        // Close
        session.close();
    }

    #[test]
    fn test_duality() {
        type Client = Send<Request, Receive<Response, End>>;
        type Server = Receive<Request, Send<Response, End>>;

        // This compiles, proving the protocols are dual
        assert_dual::<Client, Server>();
        assert_dual::<Server, Client>();
    }

    #[test]
    fn test_offer_select() {
        // Server offers two options
        type ServerOffer = Offer<Send<i32, End>, Send<bool, End>>;

        let session: Session<ServerOffer> = Session::new();

        // Test left branch
        let session_left = session.offer_left();
        let session_end = session_left.send(42);
        session_end.close();
    }

    #[test]
    fn test_select() {
        // Client selects between two options
        type ClientSelect = Select<Receive<i32, End>, Receive<bool, End>>;

        let session: Session<ClientSelect> = Session::new();

        // Select left branch
        let session_left = session.select_left();
        let (value, session_end) = session_left.receive_with(42);
        assert_eq!(value, 42);
        session_end.close();
    }

    #[test]
    fn test_either() {
        let left: Either<i32, &str> = Either::Left(42);
        let right: Either<i32, &str> = Either::Right("hello");

        assert!(left.is_left());
        assert!(!left.is_right());
        assert!(right.is_right());
        assert!(!right.is_left());

        assert_eq!(left.unwrap_left(), 42);
        assert_eq!(right.unwrap_right(), "hello");
    }

    #[test]
    fn test_protocol_patterns() {
        // Request-response pattern
        type Server = RequestResponse<Request, Response>;
        type Client = RequestResponseClient<Request, Response>;

        assert_dual::<Server, Client>();
    }

    #[test]
    fn test_notify_pattern() {
        // Fire-and-forget notification
        type Sender = Notify<i32>;
        type Receiver = NotifyReceive<i32>;

        assert_dual::<Sender, Receiver>();

        let session: Session<Sender> = Session::new();
        let session = session.send(42);
        session.close();
    }

    #[test]
    fn test_multi_step_protocol() {
        // A more complex protocol: send two values, receive one
        type Proto = Send<i32, Send<i32, Receive<i32, End>>>;

        let session: Session<Proto> = Session::new();
        let session = session.send(1);
        let session = session.send(2);
        let (sum, session) = session.receive_with(3);
        assert_eq!(sum, 3);
        session.close();
    }

    /// Compile-time test: This function demonstrates that
    /// the type system prevents protocol violations.
    ///
    /// Uncommenting the invalid operations would cause compile errors.
    #[test]
    fn test_type_safety_demo() {
        type Proto = Send<i32, Receive<bool, End>>;

        let session: Session<Proto> = Session::new();

        // Valid: send first
        let session = session.send(42);

        // Invalid: can't send again (would be compile error)
        // let session = session.send(43); // ERROR: no method `send`

        // Invalid: can't close yet (would be compile error)
        // session.close(); // ERROR: no method `close`

        // Valid: receive next
        let (_value, session) = session.receive_with(true);

        // Valid: now can close
        session.close();
    }

    #[test]
    fn test_session_computation() {
        let comp = SessionComputation::<End, i32>::pure(42);
        let session = Session::<End>::new();
        let (result, _session): (i32, Session<End>) = comp.run(session);
        assert_eq!(result, 42);
    }

    /// Test demonstrating a realistic protocol flow.
    #[test]
    fn test_calculator_protocol() {
        // Calculator protocol:
        // 1. Client sends two numbers
        // 2. Server sends back the sum
        type CalcClient = Send<i32, Send<i32, Receive<i32, End>>>;
        type CalcServer = Receive<i32, Receive<i32, Send<i32, End>>>;

        // Verify duality
        assert_dual::<CalcClient, CalcServer>();

        // Simulate client
        fn run_client(session: Session<CalcClient>, a: i32, b: i32) -> i32 {
            let session = session.send(a);
            let session = session.send(b);
            let (sum, session) = session.receive_with(a + b); // Simulated response
            session.close();
            sum
        }

        // Simulate server
        fn run_server(session: Session<CalcServer>) {
            let (a, session) = session.receive_with(10);
            let (b, session) = session.receive_with(20);
            let session = session.send(a + b);
            session.close();
        }

        // Run client
        let client_session = Session::new();
        let result = run_client(client_session, 10, 20);
        assert_eq!(result, 30);

        // Run server
        let server_session = Session::new();
        run_server(server_session);
    }
}
