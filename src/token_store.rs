
use super::mio::Token;
use super::uuid::{UUIDFactory,UUID};

/// Binary Heap of UUID's
///
/// UUID's are `u64` they can be freely converted to and from [`Tokens`](https://docs.rs/mio/0.6.1/mio/struct.Token.html)
/// which MIO uses to track connections.
pub struct TokenStore {
    data: Vec<UUID>
}
impl TokenStore {

    /// Build's a new TokenStore presized based on the maximum number of connections.
    pub fn new(max_connections: usize, factory: &mut UUIDFactory) -> TokenStore {
        let mut data = Vec::with_capacity(max_connections + 2);
        for _ in 0..(max_connections+1) {
            data.push( factory.get_uuid() );
        }
        //differ sorting until last which is more efficient
        data.sort();
        TokenStore {
            data: data
        }
    }

    /// Return a Token to the heap
    pub fn return_token(&mut self, t: Token ) {
        let uuid = UUID::from(t);
        self.return_uuid(uuid);
    }

    /// Return a UUID to the heap
    #[inline(always)]
    pub fn return_uuid(&mut self, u: UUID) {
        self.data.push(u);
        self.data.sort();
        self.data.dedup();
    }

    /// Get a UUID
    ///
    /// Return's none if there are no new items to fetch
    pub fn get_uuid(&mut self) -> Option<UUID> {
        let item = self.data.pop();
        self.data.sort();
        item
    }
}
