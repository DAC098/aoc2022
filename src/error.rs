pub enum ErrorKind {
    InvalidArgument,
    MissingArgument,
    BadInput,
    Unexpected,
    IoError,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::InvalidArgument => f.write_str("InvalidArgument"),
            ErrorKind::MissingArgument => f.write_str("MissingArgument"),
            ErrorKind::BadInput => f.write_str("BadInput"),
            ErrorKind::Unexpected => f.write_str("Unexpected"),
            ErrorKind::IoError => f.write_str("IoError"),
        }
    }
}

type BoxDynError = Box<dyn std::error::Error + Send + Sync>;

pub struct Error {
    pub kind: ErrorKind,
    pub message: Option<String>,
    pub source: Option<BoxDynError>
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Error {
            kind,
            message: None,
            source: None,
        }
    }

    pub fn with_error<E>(mut self, source: E) -> Self
    where
        E: Into<BoxDynError>
    {
        self.source = Some(source.into());
        self
    }

    pub fn with_message<M>(mut self, message: M) -> Self
    where
        M: Into<String>
    {
        self.message = Some(message.into());
        self
    }
}

macro_rules! generic_catch {
    ($e:path, $k:expr) => {
        impl From<$e> for Error {
            fn from(err: $e) -> Self {
                Error::new($k).with_error(err)
            }
        }
    };
    ($e:path, $k:expr, $m:expr) => {
        impl From<$e> for Error {
            fn from(err: $e) -> Self {
                Error::new($k).with_error(err).with_message($m)
            }
        }
    }
}

generic_catch!(std::io::Error, ErrorKind::IoError);

pub mod build {
    use super::{Error, ErrorKind};

    /// common error for providing an invalid argument
    #[inline]
    pub fn invalid_argument(arg: String) -> Error {
        let mut msg = String::from("given invalid argument. \"");
        msg.push_str(&arg);
        msg.push('"');

        Error::new(ErrorKind::InvalidArgument)
            .with_message(msg)
    }

    #[inline]
    pub fn no_file_provided() -> Error {
        Error::new(ErrorKind::MissingArgument)
            .with_message("no file provided for input")
    }

    #[inline]
    pub fn bad_line_input<S>(count: usize, line: S) -> Error
    where
        S: AsRef<str>
    {
        Error::new(ErrorKind::BadInput)
            .with_message(format!(
                "a line in the file could not be parsed. line: {} \"{}\"", 
                count, 
                line.as_ref()
            ))
    }
}