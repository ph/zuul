use std::time::Duration;

const LINE_LIMITS: usize = 1000;

#[derive(Debug, PartialEq)]
enum ParseErr {
    UnknownCommand(String),
    StringTooLong(usize),
    Empty,
    InvalidDuration(String),
}

impl std::error::Error for ParseErr {}
impl std::fmt::Display for ParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ParseErr::*;

        match self {
            UnknownCommand(c) => write!(f, "unknown command with name `{}`", c),
            StringTooLong(l) => write!(
                f,
                "string too long, limit is {} bytes, received a command of size {}",
                LINE_LIMITS, l
            ),
            Empty => write!(f, "empty string"),
            InvalidDuration(s) => write!(f, "invalid duration, error converting `{}`", s),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Command {
    Ok,
    SetTitle(String),
    Comment(String),
    //
    SetTimeOut(Duration),
    SetPrompt(String),
    SetDesc(String),
}

impl TryFrom<String> for Command {
    type Error = ParseErr;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ParseErr::Empty);
        }

        if value.as_bytes().len() > LINE_LIMITS {
            return Err(ParseErr::StringTooLong(value.as_bytes().len()));
        }

        let (c, remainder) = match value.split_once(' ') {
            Some(v) => v,
            None => (value.as_str(), ""),
        };

        match c {
            "#" => Ok(Command::Comment(remainder.to_owned())),
            "SETTIMEOUT" => {
                let d = Duration::from_secs(
                    remainder
                        .parse::<u64>()
                        .map_err(|_| ParseErr::InvalidDuration(remainder.to_owned()))?,
                );
                Ok(Command::SetTimeOut(d))
            }
            "OK" => Ok(Command::Ok),
            "SETTITLE" => Ok(Command::SetTitle(remainder.to_owned())),
            "SETDESC" => Ok(Command::SetDesc(remainder.to_owned())),
            "SETPROMPT" => Ok(Command::SetPrompt(remainder.to_owned())),
            _ => Err(ParseErr::UnknownCommand(value)),
        }
    }
}

impl TryFrom<&str> for Command {
    type Error = ParseErr;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Command::try_from(value.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    fn random_string(l: usize) -> String {
        let mut s = String::new();
        for _ in 0..l {
            s.push('-');
        }
        s
    }

    #[test]
    fn error_on_empty_string() {
        assert_eq!(Err(ParseErr::Empty), Command::try_from(""))
    }

    #[test]
    fn error_on_string_over_limit() {
        let size = LINE_LIMITS + 1;
        let s = random_string(size);
        assert_eq!(Err(ParseErr::StringTooLong(size)), Command::try_from(s));
    }

    #[test]
    fn parse_comment() {
        assert_eq!(
            Command::Comment("Hello la famille".to_string()),
            Command::try_from("# Hello la famille").unwrap()
        );
    }

    #[test]
    fn parse_ok() {
        assert_eq!(Command::Ok, Command::try_from("OK").unwrap())
    }

    #[test]
    fn parse_set_title() {
        assert_eq!(
            Command::SetTitle("hello".to_string()),
            Command::try_from("SETTITLE hello").unwrap()
        )
    }

    #[test]
    fn parse_set_timeout() {
        assert_eq!(
            Command::SetTimeOut(Duration::from_secs(20)),
            Command::try_from("SETTIMEOUT 20").unwrap()
        )
    }

    #[test]
    fn parse_set_prompt() {
        assert_eq!(
            Command::SetPrompt("hello".to_string()),
            Command::try_from("SETPROMPT hello").unwrap()
        )
    }

    #[test]
    fn parse_set_desc() {
        assert_eq!(
            Command::SetDesc("hello".to_string()),
            Command::try_from("SETDESC hello").unwrap()
        )
    }
}
