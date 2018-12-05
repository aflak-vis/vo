use std::error;
use std::fmt;
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            XmlReaderError(e) => write!(f, "Error parsing VO Table XML file: {}", e),
            ContentNotFound { tag } => write!(
                f,
                "Invalid VO Table file. Could not get content on tag '{}'",
                tag
            ),
            CannotParseIntAttribute { e, attribute } => write!(
                f,
                "Invalid VO Table file. Could not parse attribute '{}'. {}",
                attribute, e
            ),
            CannotParse { got, target } => write!(
                f,
                "Invalid VO Table file. Could not parse {}, instead got {}, which is unexpected.",
                target, got
            ),
        }
    }
}

impl error::Error for Error {}
