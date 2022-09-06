use std::{
    convert::Infallible,
    mem,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use http::HeaderMap;
use hyper::body::{HttpBody, SizeHint};

#[derive(Clone, Default)]
pub struct BodyBytes(Bytes);

impl BodyBytes {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

impl HttpBody for BodyBytes {
    type Data = Bytes;
    type Error = Infallible;

    #[inline]
    fn poll_data(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        if !self.is_empty() {
            let bytes = mem::take(&mut self.0);

            Poll::Ready(Some(Ok(bytes)))
        } else {
            Poll::Ready(None)
        }
    }

    #[inline]
    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        self.is_empty()
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(self.len() as u64)
    }
}

impl From<Vec<u8>> for BodyBytes {
    #[inline]
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes.into())
    }
}
