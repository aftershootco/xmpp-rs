use std::convert::From;
use std::io;
use std::num;

use base64;
use minidom;

#[derive(Debug)]
pub enum Error {
    ParseError(&'static str),
    IoError(io::Error),
    XMLError(minidom::Error),
    Base64Error(base64::Base64Error),
    ParseIntError(num::ParseIntError),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<minidom::Error> for Error {
    fn from(err: minidom::Error) -> Error {
        Error::XMLError(err)
    }
}

impl From<base64::Base64Error> for Error {
    fn from(err: base64::Base64Error) -> Error {
        Error::Base64Error(err)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Error {
        Error::ParseIntError(err)
    }
}