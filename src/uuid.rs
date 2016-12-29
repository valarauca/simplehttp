//! UUID
//!
//! These are not cryptographically strong UUID's. They are just used to create some level of
//! unique-ness to connections. In reality this is just a wrapper type around `u64`. As a single
//! system is normally limited to <1mil connections at any time this gives a _strong enough_
//! unique guarantee.
//!
//! These values aren't used for anything security related. Just to handle connections in internal
//! data structures.
//!
//! The internal structue uses the [`rand`](https://crates.io/crates/rand) crate. The factory is
//! a [`ChaChaRng`](https://doc.rust-lang.org/rand/rand/chacha/struct.ChaChaRng.html) which is
//! initially seed by using [`OsRng`](https://doc.rust-lang.org/rand/rand/os/struct.OsRng.html).
//! So in practice this is likely cryptographically secure but it hasn't been _proven_ so.

use super::rand::Rng;
use super::rand::os::OsRng;
use super::rand::chacha::ChaChaRng;
use super::mio::Token;
use std::io;


// The internal structue uses the [`rand`](https://crates.io/crates/rand) crate. The factory is
// a [`ChaChaRng`](https://doc.rust-lang.org/rand/rand/chacha/struct.ChaChaRng.html) which is
// initially seed by using [`OsRng`](https://doc.rust-lang.org/rand/rand/os/struct.OsRng.html).
// So in practice this is likely cryptographically secure but it hasn't been _proven_ so.
pub struct UUIDFactory {
    rand: ChaChaRng
}
impl UUIDFactory {

    /// Build a new UUIDFactory.
    ///
    /// Can return an `io::Error` on OS Error when acquireing OsRand
    pub fn new() -> io::Result<UUIDFactory> {
        let mut os_rand = OsRng::new()?;
        let mut chacha = ChaChaRng::new_unseeded();
        let seed0 = os_rand.next_u64();
        let seed1 = os_rand.next_u64();
        chacha.set_counter(seed0, seed1);
        Ok(UUIDFactory { rand: chacha } )
    }

    /// Self explainitory really
    #[inline(always)]
    pub fn get_uuid(&mut self) -> UUID {
        let val = self.rand.next_u64();
        UUID(val)
    }
}

/// Rename of `u64` type
///
/// The real goal is to abstract away thinking of this as a `u64`. Also to make managing MIO's
/// tokens easier.
#[derive(Debug,PartialEq,Eq,PartialOrd,Ord,Copy,Clone)]
pub struct UUID( u64 );
impl From<Token> for UUID {
    #[inline(always)]
    fn from(t: Token) -> UUID {
        UUID( t.0 as u64)
    }
}
impl Into<Token> for UUID {
    #[inline(always)]
    fn into(self) -> Token {
        Token( self.0 as usize)
    }
}
