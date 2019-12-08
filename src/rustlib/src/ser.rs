extern crate bytes;

use super::*;

use bytes::BufMut;
use rustson::ser::Writer;
use hyper::body::Sender;
use hyper::body::Chunk;
//use std::{thread, time};
//use futures::Future;

pub trait BodyWriter {
    fn write(&self, writer: &mut Writer) -> RTsonResult<()>;
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
    buf: Vec<u8>,
    sender: Sender,
}

impl SenderWriter {
    pub fn new(sender: Sender) -> SenderWriter {
        SenderWriter { buf: Vec::with_capacity(1048576), sender }
    }

    pub fn close(&mut self)  -> TsonResult<()> {
        self.flush()?;
        match self.sender.close() {
            Ok(_) => return Ok(()),
            Err(e) => return Err(TsonError::new(e.to_string())),
        }
    }

    fn on_put(&mut self) -> TsonResult<()> {
        if self.buf.len() > 1048576 {
            self.flush()?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> TsonResult<()> {
        let mut some_chunk = Some(Chunk::from(self.buf.clone()));
        self.buf.clear();
        loop {
            match self.sender.poll_ready() {
                Ok(_) => {
                    match self.sender.send_data(some_chunk.take().unwrap()) {
                        Ok(_) => return Ok(()),
                        Err(chunk) => {
                            some_chunk.replace(chunk);
//                            std::thread::yield_now();

                            let duration = std::time::Duration::from_millis(2);
                            std::thread::sleep(duration);
                        }
                    }
                }
                Err(e) => return Err(TsonError::new(e.to_string())),
            }
        }
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
}

