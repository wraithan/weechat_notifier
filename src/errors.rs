
use std::error;
use std::fmt;

#[macro_export]
macro_rules! fail {
    ($EXPR:expr) => (
        return Err(::std::convert::From::from($expr));
    )
}

pub struct WeechatRelayError {
    repr: ErrorRepr,
}

#[derive(Debug)]
pub enum ErrorRepr {
    WithDescription(ErrorKind, &'static str),
    WithDescriptionAndDetail(ErrorKind, &'static str, String),
}


#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ErrorKind {
    UnknownType,
}

impl WeechatRelayError {
    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            ErrorRepr::WithDescription(kind, _) => kind,
            ErrorRepr::WithDescriptionAndDetail(kind, _, _) => kind
        }
    }
}

impl PartialEq for WeechatRelayError {
    fn eq(&self, other: &WeechatRelayError) -> bool {
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

impl error::Error for WeechatRelayError {
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

impl fmt::Display for WeechatRelayError {
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

impl fmt::Debug for WeechatRelayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt::Display::fmt(self, f)
    }
}

impl From<(ErrorKind, &'static str)> for WeechatRelayError {
    fn from((kind, description): (ErrorKind, &'static str)) -> WeechatRelayError {
        WeechatRelayError { repr: ErrorRepr::WithDescription(kind, description) }
    }
}

impl From<(ErrorKind, &'static str, String)> for WeechatRelayError {
    fn from((kind, description, detail): (ErrorKind, &'static str, String)) -> WeechatRelayError {
        WeechatRelayError { repr: ErrorRepr::WithDescriptionAndDetail(kind, description, detail) }
    }
}
