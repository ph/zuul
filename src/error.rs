use std::io::ErrorKind;

use assuan::ParseErr;

#[derive(Debug, Clone)]
pub enum ZuulErr {
    Input(ErrorKind),
    Parsing(ParseErr),
    Output,
}

impl std::error::Error for ZuulErr {}
impl std::fmt::Display for ZuulErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZuulErr::Input(e) => write!(f, "error `{}` while reading stdin input", e),
            ZuulErr::Parsing(e) => write!(f, "error `{}` while parsing pinentry commands", e),
            ZuulErr::Output => write!(f, "todo output"),
        }
    }
}

impl From<std::io::Error> for ZuulErr {
    fn from(value: std::io::Error) -> Self {
        ZuulErr::Input(value.kind())
    }
}

impl From<ParseErr> for ZuulErr {
    fn from(value: ParseErr) -> Self {
        ZuulErr::Parsing(value)
    }
}
