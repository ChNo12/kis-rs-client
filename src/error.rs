use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    Config {
        message: String,
    },
    Auth {
        message: String,
    },
    Http {
        message: String,
    },
    Api {
        code: Option<String>,
        message: String,
    },
    Parse {
        message: String,
    },
}

impl Error {
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth {
            message: message.into(),
        }
    }

    pub fn http(message: impl Into<String>) -> Self {
        Self::Http {
            message: message.into(),
        }
    }

    pub fn api(code: impl Into<Option<String>>, message: impl Into<String>) -> Self {
        Self::Api {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse {
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config { message } => write!(f, "configuration error: {message}"),
            Self::Auth { message } => write!(f, "authentication error: {message}"),
            Self::Http { message } => write!(f, "http error: {message}"),
            Self::Api {
                code: Some(code),
                message,
            } => write!(f, "KIS API error ({code}): {message}"),
            Self::Api {
                code: None,
                message,
            } => write!(f, "KIS API error: {message}"),
            Self::Parse { message } => write!(f, "parse error: {message}"),
        }
    }
}

impl std::error::Error for Error {}
