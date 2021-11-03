use std::fmt::Display;
use std::io::Error as IoError;

mod codec;
use tokio::io::{AsyncRead, AsyncWrite};

pub use self::codec::Codec;

mod payload;
pub use self::payload::{
    RawPayload, ReceivableJSONPayload, ReceivablePayload, SendableJSONPayload, SendablePayload,
};

mod reader_writer;
pub use self::reader_writer::{Reader, Writer};

pub mod messages;

pub type FramedRead<T> = tokio_util::codec::FramedRead<T, Codec>;
pub type FramedWrite<T> = tokio_util::codec::FramedWrite<T, Codec>;

pub fn new_framed_read<T>(reader: T) -> FramedRead<T>
where
    T: AsyncRead,
{
    FramedRead::new(reader, Codec::new())
}

pub fn new_framed_write<T>(writer: T) -> FramedWrite<T>
where
    T: AsyncWrite,
{
    FramedWrite::new(writer, Codec::new())
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    TypeMismatch(u8),
    Coding(serde_json::Error),
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Coding(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Io(err) => format!("IO error: {}", err),
            Self::TypeMismatch(type_code) => format!("Type mismatch: {}", type_code),
            Self::Coding(err) => format!("Coding error: {}", err),
        };
        f.write_str(&str)
    }
}

impl std::error::Error for Error {}
