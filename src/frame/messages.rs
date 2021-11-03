use crate::message::{
    ClientMessage, Handshake, HandshakeReply, MessageReply, Ping, Pong, ServerMessage,
};

use super::{ReceivableJSONPayload, SendableJSONPayload};

//
//       Code     │   From Client   │  From Server
//  ──────────────┼─────────────────┼────────────────
//       0x00     │   Handshake     │ HandshakeReply
//                │                 │
//       0x01     │  ClientMessage  │  ServerMessage
//                │                 │
//       0x02     │      N/A        │  MessageReply
//                │                 │
//       0xFF     │      Ping       │     Pong
//
//  0x03 ~ 0xFE: reserved
//

macro_rules! impl_payload {
    (sendable: $msg_ident:ident > $type_code:expr) => {
        impl SendableJSONPayload for $msg_ident {
            fn type_code(&self) -> u8 {
                $type_code
            }
        }
    };
    (receivable: $msg_ident:ident > $type_code:expr) => {
        impl ReceivableJSONPayload for $msg_ident {
            fn type_code() -> u8 {
                $type_code
            }
        }
    };
}

impl_payload!(receivable: Handshake > 0x00);
impl_payload!(sendable: HandshakeReply > 0x00);

impl_payload!(receivable: ClientMessage > 0x01);
impl_payload!(sendable: ServerMessage > 0x01);

impl_payload!(sendable: MessageReply > 0x02);

impl_payload!(receivable: Ping > 0xFF);
impl_payload!(sendable: Pong > 0xFF);

pub mod client {
    use super::{ReceivableJSONPayload, SendableJSONPayload};
    use crate::message::{
        ClientMessage, Handshake, HandshakeReply, MessageReply, Ping, Pong, ServerMessage,
    };

    impl_payload!(sendable: Handshake > 0x00);
    impl_payload!(receivable: HandshakeReply > 0x00);

    impl_payload!(sendable: ClientMessage > 0x01);
    impl_payload!(receivable: ServerMessage > 0x01);

    impl_payload!(receivable: MessageReply > 0x02);

    impl_payload!(sendable: Ping > 0xFF);
    impl_payload!(receivable: Pong > 0xFF);
}
