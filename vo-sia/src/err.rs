use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    Hyper(hyper::Error),
    VOTable(vo_table::Error),
    RuntimeError(io::Error, &'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match self {
            Hyper(e) => write!(f, "HTTP error. {}", e),
            VOTable(e) => write!(f, "VOTable error. {}", e),
            RuntimeError(e, msg) => write!(f, "Runtime error. {}, caused by {}", msg, e),
        }
    }
}

impl error::Error for Error {}
