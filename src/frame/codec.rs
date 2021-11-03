use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use super::{Error, RawPayload};

//
// ├───          Header (40)         ───┤
// ┌────────────┬───────────────────────┬──────────────────┐
// │  Type (8)  │  Payload Length (32)  │    Payload ...   │
// └────────────┴───────────────────────┴──────────────────┘
//

#[derive(Debug)]
pub struct Codec {
    state: State,
}

#[derive(Debug, Clone, Copy)]
struct Header {
    type_code: u8,
    payload_length: usize,
}

impl Header {
    const TYPE_FIELD_LEN: usize = 1;
    const LENGTH_FIELD_LEN: usize = 4;
    const LEN: usize = Self::TYPE_FIELD_LEN + Self::LENGTH_FIELD_LEN;
}

#[derive(Debug, Clone, Copy)]
enum State {
    Header,
    Payload(Header),
}

impl Codec {
    pub fn new() -> Self {
        Self {
            state: State::Header,
        }
    }
}

impl Default for Codec {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for Codec {
    type Item = RawPayload;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let payload = self
            .decode_header(src)
            .and_then(|header| self.decode_payload(header, src));
        Ok(payload)
    }
}

impl Codec {
    fn decode_header(&mut self, src: &mut BytesMut) -> Option<Header> {
        if let State::Payload(header) = self.state {
            return Some(header);
        }

        if src.len() < Header::LEN {
            return None;
        }

        // Gets type & length from bytes in network (big) endian byte order.
        let type_code = src.get_uint(Header::TYPE_FIELD_LEN) as u8;
        let payload_length = src.get_uint(Header::LENGTH_FIELD_LEN) as usize;
        let header = Header {
            type_code,
            payload_length,
        };

        src.reserve(header.payload_length);
        self.state = State::Payload(header);

        Some(header)
    }

    fn decode_payload(&mut self, header: Header, src: &mut BytesMut) -> Option<RawPayload> {
        if src.len() < header.payload_length {
            return None;
        }
        let bytes = src.split_to(header.payload_length);

        src.reserve(Header::LEN);
        self.state = State::Header;

        Some(RawPayload::new(header.type_code, bytes.freeze()))
    }
}

impl Encoder<RawPayload> for Codec {
    type Error = Error;

    fn encode(&mut self, item: RawPayload, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let payload_length = item.content.len();
        dst.reserve(Header::LEN + payload_length);

        // Writes type & length to bytes in network (big) endian byte order.
        dst.put_uint(item.type_code as u64, Header::TYPE_FIELD_LEN);
        dst.put_uint(payload_length as u64, Header::LENGTH_FIELD_LEN);
        dst.put_slice(&item.content);

        Ok(())
    }
}
