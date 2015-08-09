use std::error;
use std::error::Error as CoreError;
use std::fmt;
use std::io::Error as IOError;
use byteorder::Error as ByteOrderError;

#[macro_export]
macro_rules! fail {
    ($expr:expr) => (
        return Err(::std::convert::From::from($expr));
    )
}

#[macro_export]
macro_rules! try_result {
    ($expr:expr) => (
        match $expr {
            Ok(value) => Ok(value),
            Err(error) => fail!(error)
        }
    )
}

pub struct WeechatParseError {
    repr: ErrorRepr,
}

#[derive(Debug)]
pub enum ErrorRepr {
    WithDescription(ErrorKind, &'static str),
    WithDescriptionAndDetail(ErrorKind, &'static str, String),
}


#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ErrorKind {
    MalformedBinaryParse,
    NotImplemented,
    UnknownId,
    UnknownType
}

impl WeechatParseError {
    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            ErrorRepr::WithDescription(kind, _) => kind,
            ErrorRepr::WithDescriptionAndDetail(kind, _, _) => kind
        }
    }
}

impl PartialEq for WeechatParseError {
    fn eq(&self, other: &WeechatParseError) -> bool {
        match (&self.repr, &other.repr) {
            (&ErrorRepr::WithDescription(kind_a, _),
             &ErrorRepr::WithDescription(kind_b, _)) => {
                kind_a == kind_b
            },
            (&ErrorRepr::WithDescriptionAndDetail(kind_a, _, _),
             &ErrorRepr::WithDescriptionAndDetail(kind_b, _, _)) => {
                kind_a == kind_b
            },
            _ => false,
        }
    }
}

impl error::Error for WeechatParseError {
    fn description(&self) -> &str {
        match self.repr {
            ErrorRepr::WithDescription(_, description) => description,
            ErrorRepr::WithDescriptionAndDetail(_, description, _) => description,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl fmt::Display for WeechatParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.repr {
            ErrorRepr::WithDescription(_, description) => {
                description.fmt(f)
            }
            ErrorRepr::WithDescriptionAndDetail(_, description, ref detail) => {
                try!(description.fmt(f));
                try!(f.write_str(": "));
                detail.fmt(f)
            }
        }
    }
}

impl fmt::Debug for WeechatParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}

impl From<(ErrorKind, &'static str)> for WeechatParseError {
    fn from((kind, description): (ErrorKind, &'static str)) -> WeechatParseError {
        WeechatParseError { repr: ErrorRepr::WithDescription(kind, description) }
    }
}

impl From<(ErrorKind, &'static str, String)> for WeechatParseError {
    fn from((kind, description, detail): (ErrorKind, &'static str, String)) -> WeechatParseError {
        WeechatParseError { repr: ErrorRepr::WithDescriptionAndDetail(kind, description, detail) }
    }
}

impl From<ByteOrderError> for WeechatParseError {
    fn from (error: ByteOrderError) -> WeechatParseError {
        WeechatParseError {
            repr: ErrorRepr::WithDescriptionAndDetail(ErrorKind::MalformedBinaryParse, "failed to parse binary data", error.description().to_owned())
        }
    }
}

impl From<IOError> for WeechatParseError {
    fn from (error: IOError) -> WeechatParseError {
        WeechatParseError {
            repr: ErrorRepr::WithDescriptionAndDetail(ErrorKind::MalformedBinaryParse, "failed to parse binary data", error.description().to_owned())
        }
    }
}
