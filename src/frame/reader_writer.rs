use futures::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::either::Either;

use super::{
    new_framed_read, new_framed_write, Error, FramedRead, FramedWrite, RawPayload,
    ReceivablePayload, Result, SendablePayload,
};

pub struct Reader<T>
where
    T: AsyncRead,
{
    inner: FramedRead<T>,
    peeked: Option<RawPayload>,
}

impl<T> Reader<T>
where
    T: AsyncRead + Send + Unpin,
{
    pub fn new(inner: T) -> Self {
        Self {
            inner: new_framed_read(inner),
            peeked: None,
        }
    }

    pub async fn read_either<L, R>(&mut self) -> Option<Result<Either<L, R>>>
    where
        L: ReceivablePayload,
        R: ReceivablePayload,
    {
        if let Some(left) = self
            .read_if_match_type::<L>()
            .await
            .map(|x| x.map(Either::Left))
        {
            return Some(left);
        }
        self.read::<R>().await.map(|x| x.map(Either::Right))
    }

    pub async fn read<P>(&mut self) -> Option<Result<P>>
    where
        P: ReceivablePayload,
    {
        self.read_raw()
            .await
            .map(|r| r.and_then(|x| x.into_payload()))
    }

    pub async fn read_if_match_type<P>(&mut self) -> Option<Result<P>>
    where
        P: ReceivablePayload,
    {
        match self.read_raw().await? {
            Ok(raw) => {
                let payload = raw.into_payload();
                if !matches!(payload, Err(Error::TypeMismatch(_))) {
                    Some(payload)
                } else {
                    self.peeked = Some(raw);
                    None
                }
            }
            Err(err) => Some(Err(err)),
        }
    }

    pub async fn read_raw(&mut self) -> Option<Result<RawPayload>> {
        if let Some(peeked) = self.peeked.take() {
            return Some(Ok(peeked));
        }
        self.inner.next().await
    }
}

pub struct Writer<T>(FramedWrite<T>);

impl<T> Writer<T>
where
    T: AsyncWrite + Unpin,
{
    pub fn new(inner: T) -> Self {
        Self(new_framed_write(inner))
    }

    pub async fn write<P>(&mut self, payload: P) -> Result<()>
    where
        P: SendablePayload,
    {
        let payload = payload.as_raw()?;
        self.0.send(payload).await
    }
}
