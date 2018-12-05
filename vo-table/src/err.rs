use std::num;

use xml::reader;

#[derive(Debug)]
pub enum Error {
    XmlReaderError(reader::Error),
    ContentNotFound {
        tag: &'static str,
    },
    CannotParseIntAttribute {
        e: num::ParseIntError,
        attribute: &'static str,
    },
    CannotParse {
        got: String,
        target: &'static str,
    },
}

impl From<reader::Error> for Error {
    fn from(e: reader::Error) -> Self {
        Error::XmlReaderError(e)
    }
}
