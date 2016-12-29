
use super::mio::{Poll, Token,Ready,PollOpt};
use super::mio::tcp::{TcpStream, TcpListener};
use super::uuid::UUID;
use super::token_store::TokenStore;
use super::httparse::{Request,Status,Header,EMPTY_HEADER};
use super::httparse;
use super::packet::ReqPacket;

use std::io::prelude::*;
use std::io::{Error,ErrorKind};
use std::time::{Instant,Duration};


/// Connection Structure
///
/// Represents a incoming TCP Connection
pub struct Conn {
    stream: TcpStream,
    id: UUID,
    last: Instant,
    buffer: Vec<u8>
}
impl PartialEq for Conn {
    #[inline(always)]
    fn eq(&self, other: &Conn) -> bool {
        self.id == other.id
    }
}
impl Eq for Conn { }
impl PartialOrd for Conn {
    fn partial_cmp(&self, other: &Conn) -> Option<::std::cmp::Ordering> {
        self.id.partial_cmp( &other.id)
    }
}
impl Ord for Conn {
    #[inline(always)]
    fn cmp(&self, other: &Conn) -> ::std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

enum RA {
    Succ(usize),
    WB
}

///Attempt to parse a request
enum RequestParsed {
    Incomplete,
    Err(httparse::Error),
    Finished(usize,usize)
}

impl Conn {

    /// Get a new connection, and register it with the event loop
    ///
    /// #None
    ///
    /// This function returns `Option::None`
    ///
    /// -No UUID's are avaliable (maximum connections reached)
    /// -Accept returned a `would block` error
    pub fn new(s: &TcpListener, ts: &mut TokenStore, p: &Poll) -> Result<Conn,Option<Error>> {
        let u = match ts.get_uuid() {
            Option::None => return Err(None),
            Option::Some(x) => x,
        };
        let t: Token = u.clone().into();
        let stream = match s.accept() {
            Ok((x,_)) => x,
            Err(e) => return match e.kind() {
                ErrorKind::WouldBlock => {
                    ts.return_uuid(u);
                    Err(None)
                },
                _ => Err(Some(e))
            }
        };
        match p.register(&stream,t,Ready::readable(), PollOpt::edge()) {
            Ok(_) => { },
            Err(e) => return Err(Some(e))
        };
        Ok(Conn {
            stream: stream,
            id: u,
            last: Instant::now(),
            buffer: Vec::with_capacity(4096)
        })
    }

    /// Attempt to read data
    ///
    /// Return's true if new data is read.
    pub fn read_data(&mut self) -> Result<bool,Error> {
        let start_len = self.buffer.len();
        let flag = match self.stream.read_to_end(&mut self.buffer) {
            Ok(x) => RA::Succ(x),
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => RA::WB,
                _ => return Err(e)
            }
        };
        match flag {
            RA::Succ(val) => Ok(val > 0),
            RA::WB => {
                let now_len = self.buffer.len();
                Ok(now_len > start_len)
            }
        }
    }

    /// Check if connection has timed out
    pub fn is_timeout(&self, timeout: &Duration) -> bool {
        let age = self.last.elapsed();
        age.ge(timeout)
    }

    #[inline(always)]
    fn borrow_slice<'a>(&'a self) -> &'a [u8] {
        self.buffer.as_slice()
    }

    ///Borrow data and try to return parsed HTTPRequest
    fn to_request<'a>(&'a self, headers: &mut [Header<'a>]) -> RequestParsed {
        let mut req = Request::new(headers);
        let size = match req.parse(self.borrow_slice()) {
            Ok(Status::Partial) => return RequestParsed::Incomplete,
            Ok(Status::Complete(x)) => x,
            Err(e) => return RequestParsed::Err(e)
        };
        //header count
        let num_headers: usize = req.headers.iter().filter(|x|x.name.len()>0).count();
        RequestParsed::Finished(size,num_headers)
    }

    /// Act on contents of request parse attempt
    fn drop_packet(&mut self, req: RequestParsed) -> Option<ReqPacket> {
        use std::mem::swap;
        match req {
            RequestParsed::Finished(len,headers) => {
                let mut item = self.buffer.split_off(len);
                swap(&mut item, &mut self.buffer);
                let capac = self.buffer.capacity();
                let add = 4096-capac;
                self.buffer.reserve(add);
                Some(ReqPacket::new(item, headers, self.id.clone()))
            },
            _ => None
        }
    }
}
