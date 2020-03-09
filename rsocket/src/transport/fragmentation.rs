use crate::frame::{self, Body, Frame};
use crate::payload::Payload;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::collections::LinkedList;

pub(crate) struct Joiner {
    inner: LinkedList<Frame>,
}

pub(crate) struct Splitter {
    mtu: usize,
}

impl Into<Payload> for Joiner {
    fn into(self) -> Payload {
        let mut bf = BytesMut::new();
        let mut bf2 = BytesMut::new();
        self.inner.into_iter().for_each(|it: Frame| {
            let (d, m) = match it.get_body() {
                Body::RequestResponse(body) => body.split(),
                Body::RequestStream(body) => body.split(),
                Body::RequestChannel(body) => body.split(),
                Body::RequestFNF(body) => body.split(),
                Body::Payload(body) => body.split(),
                _ => (None, None),
            };
            if let Some(raw) = d {
                bf.put(raw);
            }
            if let Some(raw) = m {
                bf2.put(raw);
            }
        });

        let data = if bf.len() > 0 {
            Some(bf.freeze())
        } else {
            None
        };
        let metadata = if bf2.len() > 0 {
            Some(bf2.freeze())
        } else {
            None
        };
        Payload::new(data, metadata)
    }
}

impl Joiner {
    pub(crate) fn new(first: Frame) -> Joiner {
        let mut inner = LinkedList::new();
        inner.push_back(first);
        Joiner { inner }
    }

    pub(crate) fn get_stream_id(&self) -> u32 {
        self.first().get_stream_id()
    }

    pub(crate) fn get_flag(&self) -> u16 {
        self.first().get_flag() & !frame::FLAG_FOLLOW
    }

    pub(crate) fn get_frame_type(&self) -> u16 {
        self.first().get_frame_type()
    }

    pub(crate) fn first(&self) -> &Frame {
        return self.inner.front().unwrap();
    }

    pub(crate) fn push(&mut self, next: Frame) -> bool {
        let has_follow = (next.get_flag() & frame::FLAG_FOLLOW) != 0;
        self.inner.push_back(next);
        !has_follow
    }
}

#[cfg(test)]
mod tests {

    use crate::frame;
    use crate::payload::Payload;
    use crate::transport::Joiner;
    use bytes::{Buf, Bytes};

    #[test]
    fn test_joiner() {
        let first = frame::Payload::builder(1, frame::FLAG_FOLLOW)
            .set_data(Bytes::from("(ROOT)"))
            .set_metadata(Bytes::from("(ROOT)"))
            .build();
        let mut joiner = Joiner::new(first);

        for i in 0..10 {
            let flag = if i == 9 { 0u16 } else { frame::FLAG_FOLLOW };
            let next = frame::Payload::builder(1, flag)
                .set_data(Bytes::from(format!("(data{:04})", i)))
                .set_metadata(Bytes::from(format!("(data{:04})", i)))
                .build();
            joiner.push(next);
        }
        let pa: Payload = joiner.into();
        println!("payload: {:?}", pa);
    }
}
