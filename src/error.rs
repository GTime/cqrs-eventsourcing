use std::{collections::HashMap, error, fmt};

/** App Error */
#[derive(Debug, PartialEq)]
pub struct Error {
    code: Option<STR>,
    message: STR,
    extension: Extension,
}

impl Error {
    pub fn new(message: STR, code: Option<&'static str>, extension: Extension) -> Error {
        Error {
            message,
            code,
            extension,
        }
    }

    pub fn code(&self) -> STR {
        match self.code {
            Some(code) => code,
            None => return "",
        }
    }

    pub fn message(&self) -> STR {
        self.message
    }
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "User Error: {}\ncode: {:?}\n extenssion: {:?}",
            &self.message, &self.code, &self.extension
        )
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Self {
        match err.classify() {
            serde_json::error::Category::Syntax => {
                Error::new("Serde: invalid json", Some("INTERNAL"), None)
            }
            serde_json::error::Category::Io
            | serde_json::error::Category::Data
            | serde_json::error::Category::Eof => Error::new("Serde: fail", Some("INTERNAL"), None),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        match err {
            _ => Error::new("IO Isssues", Some("INTERNAL"), None),
        }
    }
}

type Extension = Option<HashMap<String, String>>;
type STR = &'static str;
