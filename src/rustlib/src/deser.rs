extern crate bytes;

use super::*;

use rustson::deser::Reader;
use bytes::{Buf, Bytes};
use rtsonlib::deser::{RDeserializer, RTsonDeserializer,
                      RJsonDeserializer, RBinaryDeserializer, RUTF8Deserializer};
use std::io::{BufRead, Read};
use hyper::client::Response;
use std::str::from_utf8;
use hyper::header::ContentType;

type ReaderResult<T> = std::result::Result<T, TsonError>;

struct ReceiverReader<'r> {
    pub is_done: bool,
    receiver: &'r mut Response,
    inner: Cursor<Bytes>,
}

impl<'r> ReceiverReader<'r> {
    fn new(receiver: &'r mut Response) -> ReceiverReader<'r> {
        ReceiverReader { is_done: false, receiver, inner: Cursor::new(Bytes::with_capacity(0)) }
    }

    fn ensure(&mut self, n: usize) -> ReaderResult<()> {
        if self.inner.remaining() >= n {
            return Ok(());
        }

        if self.is_done {
            return Err(TsonError::new("teRcenHttp -- ReceiverReader -- ensure -- EOF"));
        }

        loop {
            self.next_item()?;
            if self.inner.remaining() < n {
                if self.is_done {
                    return Err(TsonError::new("teRcenHttp -- ReceiverReader -- ensure -- EOF"));
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    fn next_item(&mut self) -> ReaderResult<()> {
        let mut buffer = [0; 4096];
        let n = self.receiver.read(&mut buffer[..]).map_err(TsonError::other )?;
        if n == 0 {
            self.is_done = true;
            return Ok(());
        }
        if self.inner.remaining() > 0 {
            self.inner.get_mut().extend_from_slice(&buffer[0..n]);
        } else {
            let new_inner = Cursor::new(buffer[0..n].into());
            let _old = std::mem::replace(&mut self.inner, new_inner);
        }

        Ok(())
    }
}

impl<'r> Reader for ReceiverReader<'r> {
    fn read_all(&mut self, buf: &mut Vec<u8>) -> ReaderResult<()> {
        buf.extend_from_slice(self.inner.get_ref());
        self.inner.consume(   self.inner.get_ref().len());
        // self.inner.consume(self.inner.bytes().len());

        loop {
            self.next_item()?;
            if self.is_done { break; }
            buf.extend_from_slice(self.inner.get_ref());
            self.inner.consume(self.inner.get_ref().len());
            // self.inner.consume(self.inner.bytes().len());
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
    fn read(&self, reader: &mut dyn Reader) -> RTsonResult<SEXP> {
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

    pub fn read(&self, response: &mut Response) -> RResult<SEXP> {

        // let parts = self.read_parts(receiver)?;
        let response_type = self.response_type_from(&response.headers);
        let mut reader = ReceiverReader::new(response);
        let obj = response_type.read(&mut reader)?;
        let result = self.build_r_response(response, obj);
        return result;
    }

    fn response_type_from(&self, headers: &Headers) -> ResponseType {
        match self.response_type {
            ResponseType::DEFAULT => {

                match headers.get::<ContentType>() {
                    None => ResponseType::BINARY,
                    Some(content_type) => {

                        let resp_type = content_type.to_string();
                        match resp_type.as_str() {
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
    //
    fn build_r_response(&self, response: &Response, object: SEXP) -> RResult<SEXP> {
        let mut names = CharVec::alloc(3);
        let mut values = RList::alloc(3);

        names.set(0, "status")?;
        values.set(0, response.status.to_u16().intor()?)?;
        names.set(1, "headers")?;

        let mut h: Vec<String> = vec![];

        for header_view in response.headers.iter() {
            h.push(header_view.name().to_string());
            h.push(header_view.value_string());
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
