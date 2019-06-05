extern crate bytes;

use super::*;
use bytes::BufMut;
use rustson::ser::Writer;

struct Part {
    headers: HashMap<String, String>,
    content: SEXP,
}

impl Part {
    fn from_object(object: SEXP) -> RTsonResult<Part> {
        match object.rtype() {
            VECSXP => {
                /* generic vectors */
                // empty list
                let rlist = RList::new(object)?;
                let names: CharVec = RName::name(&rlist);

                let mut some_headers = None;
                let mut content = None;

                let range = std::ops::Range { start: 0 as usize, end: names.rsize() as usize };

                for i in range {
                    let name = names.at(i)?;
                    match &name as &str {
                        "headers" => {
                            let headers_value = r_to_value(rlist.at(i).unwrap())?;

                            match headers_value {
                                MAP(headers_val) => {
                                    let mut headers: HashMap<String, String> = HashMap::new();

                                    for (k, value_value) in headers_val {
                                        match value_value {
                                            STR(value) => {
                                                headers.insert(k.to_string(), value.to_string());
                                            }
                                            _ => return Err(RTsonError::new("header values must be string" )),
                                        }
                                    }

                                    if !headers.contains_key("content-type") {
                                        return Err(RTsonError::new("headers.content-type is required"));
                                    }

                                    some_headers = Some(headers);
                                }
                                _ => { return Err(RTsonError::new("headers must be a map"));}
                            }
                        }
                        "content" => {
                            content = Some(rlist.at(i).unwrap());
                        }
                        _ => {}
                    }
                }

                if some_headers.is_none() {
                    return Err(RTsonError::new("headers is required"));
                }

                if content.is_none() {
                    return Err(RTsonError::new("content is required"));
                }

                Ok(Part { headers: some_headers.unwrap(), content: content.unwrap() })
            }
            _ => {
                Err(RTsonError::new(format!("bad object type : {}", object.rtype())))
            }
        }
    }

    fn write_headers(&self, writer: &mut Writer) -> RTsonResult<()> {
        let mut bytes: Vec<u8> = Vec::new();
        for (k, v) in &self.headers {
            bytes.put(k);
            bytes.put(": ");
            bytes.put(v);
            bytes.put_u8(13);
            bytes.put_u8(10);
        }

        bytes.put_u8(13);
        bytes.put_u8(10);

        for byte in bytes {
            writer.add_u8(byte)?;
        }

        Ok(())
    }

    fn write_content(&self, writer: &mut Writer) -> RTsonResult<()> {
        let content_type = self.headers.get("content-type")
            .ok_or(RTsonError::new("headers.content-type is required"))?;

        match content_type.as_ref() {
            "application/octet-stream" => {
                match self.content.rtype() {
                    RAWSXP => {
                        let object_ = RawVec::rnew(self.content)?;

                        for x in object_ {
                            writer.add_u8(x)?;
                        }
                     }
                    _ => {
                        return Err(RTsonError::new("content-type application/octet-stream requires raw vector"));
                    }
                }
            }
            "application/tson" => {
                RSerializer::new().write(&self.content, writer)?;
            }
            "application/json" => {
                 for b in rustson::encode_json(&r_to_value(self.content)?)?.into_bytes(){
                    writer.add_u8(b)?;
                }
            }
            _ => {
                return Err(RTsonError::new("unknown content-type"));
            }
        }

        writer.add_u8(13)?;
        writer.add_u8(10)?;

        Ok(())
    }
}

pub struct MultiPart {
    pub frontier: String,
    parts: Vec<Part>,
}

impl MultiPart {
    pub fn from_r(object: SEXP) -> RTsonResult<MultiPart> {
        match object.rtype() {
            VECSXP => {
                /* generic vectors */
                // empty list
                let rlist = RList::new(object)?;

                let mut parts = Vec::new();

                for part in rlist {
                    parts.push(Part::from_object(part)?);
                }

                Ok(MultiPart { frontier: "ab63a1363ab349aa8627be56b0479de2".to_string(), parts: parts })
            }
            _ => Err(RTsonError::new("body must be a list")),
        }
    }

    pub fn write_multipart(&self, writer: &mut Writer) -> RTsonResult<()> {
        for part in &self.parts {
            self.write_frontier(writer)?;
            part.write_headers(writer)?;
            part.write_content(writer)?;
        }

        self.write_end(writer)?;
        Ok(())
    }

    fn write_frontier(&self, writer: &mut Writer) -> RTsonResult<()> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.put("--");
        bytes.put(&self.frontier);
        bytes.put_u8(13);
        bytes.put_u8(10);

        for byte in bytes {
            writer.add_u8(byte)?;
        }

        Ok(())
    }

    fn write_end(&self, writer: &mut Writer) -> RTsonResult<()> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.put("--");
        bytes.put(&self.frontier);
        bytes.put("--");
        bytes.put_u8(13);
        bytes.put_u8(10);


        for byte in bytes {
            writer.add_u8(byte)?;
        }

        Ok(())
    }
}

impl BodyWriter for MultiPart {
    fn write(&self, writer: &mut Writer) -> RTsonResult<()>{
        self.write_multipart( writer)
    }
}
