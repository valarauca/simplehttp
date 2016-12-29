
use super::uuid::UUID;
use super::httparse::Request;


pub struct ReqPacket {
    data: Vec<u8>,
    headers: usize,
    id: UUID
}
impl ReqPacket {

    #[inline(always)]
    pub fn new(buffer: Vec<u8>, headers: usize, id: UUID) -> ReqPacket {
        ReqPacket {
            data: buffer,
            headers: headers,
            id: id
        }
    }
}
