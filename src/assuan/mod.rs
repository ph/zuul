use std::time::Duration;

const LINE_LIMITS: usize = 1000;

#[derive(Debug, PartialEq, Clone)]
pub enum ParseErr {
    UnknownCommand(String),
    StringTooLong(usize),
    Empty,
    InvalidDuration(String),
    UnknownOption(String),
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
            UnknownOption(s) => write!(f, "unknown OPTION named `{}`", s),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Response {
    Ok,
    OkHello,
    Quit,
    Reset,
    Data(String),
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Response::Ok => write!(f, "OK"),
            Response::OkHello => write!(f, "OK Hello and waiting"),
            Response::Quit => todo!(),
            Response::Reset => todo!(),
            Response::Data(data) => write!(f, "D {}", data),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    // Not sure if this is worth it
    Ok,
    Quit,
    Reset,
    // show dialog
    // Config, 	// Ask for confirmation
    // Message, // Show a message
    GetPin, // Show dialog Ask the user for a PIN or passphrase
    SetTitle(String),
    Comment(String),
    SetTimeOut(Duration),
    SetPrompt(String),
    SetDesc(String),
    SetOk(String),
    SetCancel(String),
    SetNotOk(String),
    SetError(String),
    SetRepeat,
    SetQualityBar,
    SetQualityBarTT(String),
    Option(OptionArgs),
    SetGenPin,
    SetGenPinTT(String),
    SetKeyInfo(String),
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
            "GETPIN" => Ok(Command::GetPin),
            "QUIT" => Ok(Command::Quit),
            "RESET" => Ok(Command::Reset),
            "SETTITLE" => Ok(Command::SetTitle(remainder.to_owned())),
            "SETDESC" => Ok(Command::SetDesc(remainder.to_owned())),
            "SETPROMPT" => Ok(Command::SetPrompt(remainder.to_owned())),
            "SETOK" => Ok(Command::SetOk(remainder.to_owned())),
            "SETCANCEL" => Ok(Command::SetCancel(remainder.to_owned())),
            "SETNOTOK" => Ok(Command::SetNotOk(remainder.to_owned())),
            "SETERROR" => Ok(Command::SetError(remainder.to_owned())),
            "SETREPEAT" => Ok(Command::SetRepeat),
            "SETQUALITYBAR" => Ok(Command::SetQualityBar),
            "SETQUALITYBAR_TT" => Ok(Command::SetQualityBarTT(remainder.to_owned())),
            "SETGENPIN" => Ok(Command::SetGenPin),
            "SETGENPIN_TT" => Ok(Command::SetGenPinTT(remainder.to_owned())),
            "OPTION" => Ok(Command::Option(OptionArgs::try_from(remainder)?)),
            "SETKEYINFO" => Ok(Command::SetKeyInfo(remainder.to_owned())),
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

#[derive(Debug, PartialEq, Clone)]
pub enum OptionArgs {
    ConstraintsEnforce,
    ConstraintsHintShort(String),
    ConstraintsHintLong(String),
    FormattedPassphrase,
    FormattedPassphraseHint(String),
    // NOTE: Not sure of the inner type yet.
    TtyName(String),
    TtyType(String),
    LcCType(String),
    DefaultOk(String),
    DefaultCancel(String),
    DefaultPrompt(String),
    AllowExternalPasswordCache,
}

impl TryFrom<&str> for OptionArgs {
    type Error = ParseErr;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (option, args) = match value.split_once('=') {
            Some(v) => v,
            None => (value, ""),
        };

        use OptionArgs::*;

        match (option, args) {
            ("constraints-enforce", "") => Ok(ConstraintsEnforce),
            ("constraints-hint-short", _) => Ok(ConstraintsHintShort(args.to_owned())),
            ("constraints-hint-long", _) => Ok(ConstraintsHintLong(args.to_owned())),
            ("formatted-passphrase", "") => Ok(FormattedPassphrase),
            ("formatted-passphrase-hint", _) => Ok(FormattedPassphraseHint(args.to_owned())),
            ("ttyname", _) => Ok(TtyName(args.to_owned())),
            ("ttytype", _) => Ok(TtyType(args.to_owned())),
            ("lc-ctype", _) => Ok(LcCType(args.to_owned())),
            ("default-ok", _) => Ok(DefaultOk(args.to_owned())),
            ("default-cancel", _) => Ok(DefaultCancel(args.to_owned())),
            ("default-prompt", _) => Ok(DefaultPrompt(args.to_owned())),
            ("allow-external-password-cache", "") => Ok(AllowExternalPasswordCache),
            (_, _) => Err(ParseErr::UnknownOption(value.to_owned())),
        }
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

    #[test]
    fn parse_set_ok() {
        assert_eq!(
            Command::SetOk("hello".to_string()),
            Command::try_from("SETOK hello").unwrap()
        )
    }

    #[test]
    fn parse_set_cancel() {
        assert_eq!(
            Command::SetCancel("hello".to_string()),
            Command::try_from("SETCANCEL hello").unwrap()
        )
    }

    #[test]
    fn parse_set_not_ok() {
        assert_eq!(
            Command::SetNotOk("hello".to_string()),
            Command::try_from("SETNOTOK hello").unwrap()
        )
    }
    #[test]
    fn parse_set_error() {
        assert_eq!(
            Command::SetError("hello".to_string()),
            Command::try_from("SETERROR hello").unwrap()
        )
    }

    #[test]
    fn parse_set_repeat() {
        assert_eq!(Command::SetRepeat, Command::try_from("SETREPEAT").unwrap())
    }

    #[test]
    fn parse_set_quality_bar() {
        assert_eq!(
            Command::SetQualityBar,
            Command::try_from("SETQUALITYBAR").unwrap()
        )
    }

    #[test]
    fn parse_set_quality_bar_tt() {
        assert_eq!(
            Command::SetQualityBarTT("Hello".to_string()),
            Command::try_from("SETQUALITYBAR_TT Hello").unwrap()
        )
    }

    #[test]
    fn parse_option_constraints_enforce() {
        assert_eq!(
            Command::Option(OptionArgs::ConstraintsEnforce),
            Command::try_from("OPTION constraints-enforce").unwrap()
        )
    }

    #[test]
    fn parse_option_constraints_hint_short_text() {
        assert_eq!(
            Command::Option(OptionArgs::ConstraintsHintShort("hello".to_string())),
            Command::try_from("OPTION constraints-hint-short=hello").unwrap()
        )
    }

    #[test]
    fn parse_option_constraints_hint_long_text() {
        assert_eq!(
            Command::Option(OptionArgs::ConstraintsHintLong("hello".to_string())),
            Command::try_from("OPTION constraints-hint-long=hello").unwrap()
        )
    }

    #[test]
    fn parse_option_formatted_passphrase() {
        assert_eq!(
            Command::Option(OptionArgs::FormattedPassphrase),
            Command::try_from("OPTION formatted-passphrase").unwrap()
        )
    }

    #[test]
    fn parse_option_formatted_passphrase_hint() {
        assert_eq!(
            Command::Option(OptionArgs::FormattedPassphraseHint("hello".to_string())),
            Command::try_from("OPTION formatted-passphrase-hint=hello").unwrap()
        )
    }

    #[test]
    fn parse_option_ttyname() {
        assert_eq!(
            Command::Option(OptionArgs::TtyName("hello".to_string())),
            Command::try_from("OPTION ttyname=hello").unwrap()
        )
    }

    #[test]
    fn parse_option_ttytype() {
        assert_eq!(
            Command::Option(OptionArgs::TtyType("hello".to_string())),
            Command::try_from("OPTION ttytype=hello").unwrap()
        )
    }

    #[test]
    fn parse_option_lc_ctype() {
        assert_eq!(
            Command::Option(OptionArgs::LcCType("hello".to_string())),
            Command::try_from("OPTION lc-ctype=hello").unwrap()
        )
    }

    #[test]
    fn parse_option_default_ok() {
        assert_eq!(
            Command::Option(OptionArgs::DefaultOk("Okay".to_string())),
            Command::try_from("OPTION default-ok=Okay").unwrap()
        )
    }

    #[test]
    fn parse_option_default_cancel() {
        assert_eq!(
            Command::Option(OptionArgs::DefaultCancel("Okay".to_string())),
            Command::try_from("OPTION default-cancel=Okay").unwrap()
        )
    }

    #[test]
    fn parse_option_default_prompt() {
        assert_eq!(
            Command::Option(OptionArgs::DefaultPrompt("Okay".to_string())),
            Command::try_from("OPTION default-prompt=Okay").unwrap()
        )
    }

    #[test]
    fn parse_option_allow_external_password_cache() {
        assert_eq!(
            Command::Option(OptionArgs::AllowExternalPasswordCache),
            Command::try_from("OPTION allow-external-password-cache").unwrap()
        )
    }

    #[test]
    fn parse_getpin() {
        assert_eq!(Command::GetPin, Command::try_from("GETPIN").unwrap())
    }

    #[test]
    fn parse_set_key_info() {
        assert_eq!(
            Command::SetKeyInfo("hello".to_string()),
            Command::try_from("SETKEYINFO hello").unwrap()
        )
    }

    #[test]
    fn parse_quit() {
        assert_eq!(Command::Quit, Command::try_from("QUIT").unwrap())
    }

    #[test]
    fn parse_reset() {
        assert_eq!(Command::Reset, Command::try_from("RESET").unwrap())
    }

    #[test]
    fn parse_set_set_gen_pin() {
        assert_eq!(Command::SetGenPin, Command::try_from("SETGENPIN").unwrap())
    }

    #[test]
    fn parse_set_set_gen_pin_tt() {
        assert_eq!(
            Command::SetGenPinTT("Hello".to_string()),
            Command::try_from("SETGENPIN_TT Hello").unwrap()
        )
    }
}
