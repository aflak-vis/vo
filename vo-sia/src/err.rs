use std::io;

#[derive(Debug)]
pub enum Error {
    Hyper(hyper::Error),
    VOTable(vo_table::Error),
    IoError(io::Error, &'static str),
}
