extern crate bytes;

use super::*;

use bytes::BufMut;
use rustson::ser::Writer;
// use hyper::body::Sender;
// use hyper::body::Chunk;
//use std::{thread, time};
//use futures::Future;

pub trait BodyWriter {
    fn write(&self, writer: &mut dyn Writer) -> RTsonResult<()>;
}

pub struct TsonBodyWriter {
    object: SEXP,
}

impl TsonBodyWriter {
    pub fn new(object: SEXP) -> TsonBodyWriter {
        TsonBodyWriter { object }
    }
}

impl BodyWriter for TsonBodyWriter {
    fn write(&self, writer: &mut dyn Writer) -> RTsonResult<()> {
        RSerializer::new().write(&self.object, writer)
    }
}

impl BodyWriter for String {
    fn write(&self, writer: &mut dyn Writer) -> RTsonResult<()> {
        for b in self.as_bytes() {
            writer.add_u8(*b)?;
        }
        Ok(())
    }
}

impl BodyWriter for RawVec {
    fn write(&self, writer: &mut dyn Writer) -> RTsonResult<()> {
        let range = std::ops::Range { start: 0 as usize, end: self.rsize() as usize };
        unsafe {
            for i in range {
                writer.add_u8(self.uat(i))?;
            }
        }
        Ok(())
    }
}

pub struct SenderWriter {
    pub buf: Vec<u8>,
    pub sender: Request<Streaming>,
}

impl SenderWriter {
    pub fn new(sender: Request<Streaming>) -> SenderWriter {
        SenderWriter { buf: Vec::with_capacity(1048576), sender }
    }


    pub fn close(&mut self)  -> TsonResult<()> {
        self.flush()
        // self.sender.flush().map_err(|e| TsonError::new(e.to_string()))
    }

    fn on_put(&mut self) -> TsonResult<()> {
        if self.buf.len() > 1048576 {
            self.flush()?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> TsonResult<()> {
        if !self.buf.is_empty() {
            let mut buf = Cursor::new(&mut self.buf);
            std::io::copy(&mut buf, &mut self.sender).map_err(|e| TsonError::new("flush failed".to_string()))?;
            self.buf.clear();
        }

        Ok(())
    }
}

impl Writer for SenderWriter {
    fn add_u8(&mut self, value: u8) -> TsonResult<()> {
        self.buf.put_u8(value);
        self.on_put()
    }
    fn add_i8(&mut self, value: i8) -> TsonResult<()> {
        self.buf.put_i8(value);
        self.on_put()
    }
    fn add_u32(&mut self, value: u32) -> TsonResult<()> {
        self.buf.put_u32_le(value);
        self.on_put()
    }
    fn add_i32(&mut self, value: i32) -> TsonResult<()> {
        self.buf.put_i32_le(value);
        self.on_put()
    }
    fn add_f64(&mut self, value: f64) -> TsonResult<()> {
        self.buf.put_f64_le(value);
        self.on_put()
    }
    fn add_u16(&mut self, value: u16) -> TsonResult<()> {
        self.buf.put_u16_le(value);
        self.on_put()
    }
    fn add_i16(&mut self, value: i16) -> TsonResult<()> {
        self.buf.put_i16_le(value);
        self.on_put()
    }
    fn add_u64(&mut self, value: u64) -> TsonResult<()> {
        self.buf.put_u64_le(value);
        self.on_put()
    }
    fn add_i64(&mut self, value: i64) -> TsonResult<()> {
        self.buf.put_i64_le(value);
        self.on_put()
    }
    fn add_f32(&mut self, value: f32) -> TsonResult<()> {
        self.buf.put_f32_le(value);
        self.on_put()
    }

    fn put_slice(&mut self, src: &[u8]) -> TsonResult<()> {
        BufMut::put_slice(&mut self.buf, src);
        self.on_put()
    }
}

