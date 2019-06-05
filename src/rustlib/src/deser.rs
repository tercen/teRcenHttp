extern crate bytes;

use super::*;

use rustson::deser::Reader;
use bytes::{Buf, Bytes};
use rtsonlib::deser::{RDeserializer, RTsonDeserializer,
                      RJsonDeserializer, RBinaryDeserializer, RUTF8Deserializer};

type ReaderResult<T> = std::result::Result<T, TsonError>;
type Receiver = mpsc::Receiver<Result<BodyItem, hyper::error::Error>>;

struct ReceiverReader<'r> {
    pub is_done: bool,
    receiver: &'r Receiver,
    inner: Cursor<Bytes>,
}

//impl<'r> std::panic::UnwindSafe for &  ReceiverReader<'r>{}

impl<'r> ReceiverReader<'r> {
    fn new(receiver: &'r Receiver) -> ReceiverReader<'r> {
        ReceiverReader { is_done: false, receiver, inner: Cursor::new(Bytes::with_capacity(0)) }
    }

    fn ensure(&mut self, n: usize) -> ReaderResult<()> {
        if self.inner.remaining() >= n {
            return Ok(());
        }

        if (self.is_done){
            return Err(TsonError::new("teRcenHttp -- ReceiverReader -- ensure -- EOF"));
        }

        loop {

            self.next_item()?;
            if self.inner.remaining() < n   {
                if (self.is_done){
                    return Err(TsonError::new("teRcenHttp -- ReceiverReader -- ensure -- EOF"));
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    fn next_item(&mut self) -> ReaderResult<()> {
        let body_item = self.receiver.recv().map_err(TsonError::other)?.map_err(TsonError::other)?;
        match body_item {
            BodyItem::C(chunk) => {
                if self.inner.remaining() > 0 {
                    self.inner.get_mut().extend_from_slice(&chunk.into_bytes());
                } else {
                    let new_inner = Cursor::new(chunk.into_bytes());
                    std::mem::replace(&mut self.inner, new_inner);
                }

                return Ok(());
            }
            BodyItem::Done => {
                self.is_done = true;
                return Ok(());
            }
            _ => {
                return Err(TsonError::new("bad response"));
            }
        }
    }
}

impl<'r> Reader for ReceiverReader<'r> {
    fn read_all(&mut self, buf: &mut Vec<u8>) -> ReaderResult<()> {
        buf.extend_from_slice(self.inner.get_ref());
        loop {
            self.next_item()?;
            if self.is_done { break; }
            buf.extend_from_slice(self.inner.get_ref());
        }

        Ok(())
    }

    fn read_u8(&mut self) -> ReaderResult<u8> {
        self.ensure(1)?;
        Ok(self.inner.get_u8())
    }
    fn read_i8(&mut self) -> ReaderResult<i8> {
        self.ensure(1)?;
        Ok(self.inner.get_i8())
    }
    fn read_u16(&mut self) -> ReaderResult<u16> {
        self.ensure(2)?;
        Ok(self.inner.get_u16_le())
    }
    fn read_i16(&mut self) -> ReaderResult<i16> {
        self.ensure(2)?;
        Ok(self.inner.get_i16_le())
    }
    fn read_u32(&mut self) -> ReaderResult<u32> {
        self.ensure(4)?;
        Ok(self.inner.get_u32_le())
    }
    fn read_i32(&mut self) -> ReaderResult<i32> {
        self.ensure(4)?;
        Ok(self.inner.get_i32_le())
    }
    fn read_u64(&mut self) -> ReaderResult<u64> {
        self.ensure(8)?;
        Ok(self.inner.get_u64_le())
    }
    fn read_i64(&mut self) -> ReaderResult<i64> {
        self.ensure(8)?;
        Ok(self.inner.get_i64_le())
    }
    fn read_f32(&mut self) -> ReaderResult<f32> {
        self.ensure(4)?;
        Ok(self.inner.get_f32_le())
    }
    fn read_f64(&mut self) -> ReaderResult<f64> {
        self.ensure(8)?;
        Ok(self.inner.get_f64_le())
    }
}


#[derive(Clone)]
pub enum ResponseType {
    TSON,
    JSON,
    UTF8,
    BINARY,
    DEFAULT,
}


impl RDeserializer for ResponseType {
    fn read(&self, reader: &mut Reader) -> RTsonResult<SEXP> {
        match self {
            ResponseType::TSON => RTsonDeserializer {}.read(reader),
            ResponseType::JSON => RJsonDeserializer {}.read(reader),
            ResponseType::UTF8 => RUTF8Deserializer {}.read(reader),
            ResponseType::BINARY => RBinaryDeserializer {}.read(reader),
            ResponseType::DEFAULT => RBinaryDeserializer {}.read(reader),
        }
    }
}

impl From<String> for ResponseType {
    fn from(resp_type: String) -> Self {
        From::from(&resp_type as &str)
    }
}

impl From<&str> for ResponseType {
    fn from(resp_type: &str) -> Self {
        match resp_type {
            "default" => ResponseType::DEFAULT,
            "application/tson" => ResponseType::TSON,
            "application/json" => ResponseType::JSON,
            "text/html; charset=utf-8" | "text/plain; charset=utf-8" => ResponseType::UTF8,
            _ => {
                if resp_type.starts_with("text") {
                    ResponseType::UTF8
                } else {
                    ResponseType::BINARY
                }
            }
        }
    }
}

pub struct ResponseReader {
    response_type: ResponseType,
}

impl ResponseReader {
    pub fn new(response_type: ResponseType) -> ResponseReader { ResponseReader { response_type } }

    pub fn read(&self, receiver: &Receiver) -> RResult<SEXP> {
        let parts = self.read_parts(receiver)?;
        let response_type = self.response_type_from(&parts);
        let mut reader = ReceiverReader::new(receiver);
        let result = self.build_r_response(parts, response_type.read(&mut reader)?);
        return result;
    }

    pub fn read_parts(&self, receiver: &Receiver) -> RResult<Parts> {
        match (receiver.recv().map_err(RError::other)?).map_err(RError::other)? {
            BodyItem::H(parts) => {
                Ok(parts)
            }
            _ => Err(RError::unknown("bad response".to_string())),
        }
    }

    fn response_type_from(&self, parts: &Parts) -> ResponseType {
        match self.response_type {
            ResponseType::DEFAULT => {
                match parts.headers.get("content-type") {
                    None => ResponseType::BINARY,
                    Some(content_type) => {
                        let resp_type = content_type.to_str().unwrap();
                        match resp_type {
                            "application/tson" => ResponseType::TSON,
                            "application/json" => ResponseType::JSON,
                            "text/html; charset=utf-8" | "text/plain; charset=utf-8" => ResponseType::UTF8,
                            _ => {
                                if resp_type.starts_with("text") {
                                    ResponseType::UTF8
                                } else {
                                    ResponseType::BINARY
                                }
                            }
                        }
                    }
                }
            }
            _ => self.response_type.clone()
        }
    }

    fn build_r_response(&self, parts: Parts, object: SEXP) -> RResult<SEXP> {
        let mut names = CharVec::alloc(3);
        let mut values = RList::alloc(3);

        names.set(0, "status")?;
        values.set(0, parts.status.as_u16().intor()?)?;
        names.set(1, "headers")?;

        let mut h: Vec<String> = vec![];

        for (key, value) in parts.headers.iter() {
            h.push(key.as_str().to_string());
            match value.to_str() {
                Err(error) => {
                    return rraise(error);
                }
                Ok(v) => {
                    h.push(v.to_string());
                }
            }
        }

        values.set(1, h.intor()?)?;
        names.set(2, "content")?;

        values.set(2, object)?;

        unsafe {
            Rf_setAttrib(values.s(), R_NamesSymbol, names.s());
        }

        values.intor()
    }
}
