use std::io::ErrorKind;

use assuan::{Command, ParseErr};
use iced::{
    futures::Stream, widget::{button, column, container, row, text, text_input}, Alignment::Center, Element, Task
};
use iced::futures::sink::SinkExt;
use iced::Subscription;
use iced::stream;
use tokio::io::BufReader;
use tokio::io::AsyncBufReadExt;

mod assuan;

#[derive(Debug, Clone)]
enum ZuulErr {
    Input(ErrorKind),
    Parsing(ParseErr),
}

impl std::error::Error  for ZuulErr {}
impl std::fmt::Display for ZuulErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	match self {
	    ZuulErr::Input(e) => write!(f, "error `{}` while reading stdin input", e),
	    ZuulErr::Parsing(e) => write!(f, "error `{}` while parssing pinentry commands", e)

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

struct Application {
    passphrase: String,
}

#[derive(Debug, Clone)]
enum Message {
    PassphraseChanged(String),
    ButtonOkPressed,
    ButtonCancelPressed,
    Output(String),
    Input(Command),
    Fatal(ZuulErr),
}

struct Form {
    prompt: String,
    button_ok: String,
    button_cancel: String,
}

struct FormBuilder {
    prompt: String,
    button_ok: String,
    button_cancel: String,
}

impl FormBuilder {
    fn new() -> Self {
        Self {
            prompt: String::from("PIN:"),
            button_ok: String::from("OK"),
            button_cancel: String::from("cancel"),
        }
    }

    fn with_prompt(mut self, s: impl Into<String>) -> Self {
        self.prompt = s.into();
        self
    }

    fn with_button_ok(mut self, s: impl Into<String>) -> Self {
        self.button_ok = s.into();
        self
    }

    fn with_button_cancel(mut self, s: impl Into<String>) -> Self {
        self.button_cancel = s.into();
        self
    }

    fn build(self) -> Form {
        Form {
            prompt: self.prompt,
            button_ok: self.button_ok,
            button_cancel: self.button_cancel,
        }
    }
}

fn apply_commands(commands: &[Command]) -> Form {
    let mut b = FormBuilder::new();

    for command in commands {
        // iteratively building the form.
        b = match command {
            Command::SetPrompt(p) => b.with_prompt(p),
            Command::SetOk(t) => b.with_button_ok(t),
            Command::SetCancel(t) => b.with_button_cancel(t),
            _ => break,
        };
    }
    b.build()
}

impl Application {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                passphrase: String::new(),
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Zuul")
    }

    fn output(&self, s: &str) {
        println!("my passphrase is: {}", s)
    }

    fn update(&mut self, message: Message) {
        // debug!(message=?message, "new message");
        match message {
            Message::PassphraseChanged(p) => self.passphrase = p,
            Message::ButtonOkPressed => self.output(&self.passphrase),
            Message::ButtonCancelPressed => println!("cancel"),
            Message::Output(_) => todo!(),
            Message::Input(command) => println!("pintentry: {:?}", command),
            Message::Fatal(err) => println!("error: {}", err),
        }
    }

    fn view(&self) -> Element<Message> {
        let f = apply_commands(&vec![
            Command::SetOk(String::from("This is ok")),
            Command::SetPrompt(String::from("This is my seeecrets")),
            Command::SetCancel(String::from(" I have changed my mind")),
            Command::GetPin,
        ]);

        container(row![
            text(f.prompt),
            column![
                text_input("", &self.passphrase)
                    .on_input(Message::PassphraseChanged)
                    .secure(true),
                row![
                    button(text(f.button_cancel)).on_press(Message::ButtonCancelPressed),
                    button(text(f.button_ok)).on_press(Message::ButtonOkPressed)
                ]
                .align_y(Center)
                .spacing(10)
                .padding(10)
            ]
        ])
        .padding(10)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
	subscribe_to_commands()
    }
}

fn subscribe_to_commands() -> Subscription<Message>{
    Subscription::run_with_id(
	std::any::TypeId::of::<Command>(),
	read_external_commands_input()
    ).map(|e| {
	match e {
	    Ok(c) => Message::Input(c),
	    Err(e) => Message::Fatal(e),
	}
    })
}

fn read_external_commands_input() -> impl Stream<Item=Result<Command, ZuulErr>> {
    stream::try_channel(1, async move |mut output| {
	let stdin = tokio::io::stdin();
	let buf = BufReader::new(stdin);
	let mut lines = buf.lines();

	while let Some(line) = lines.next_line().await? {
	    let command = Command::try_from(line)?;
	    output.send(command).await;
	}

	Ok(())
    })
}

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application(Application::title, Application::update, Application::view)
        // .subscription(Application::subscription)
        .window_size((400.0, 400.0))
        .subscription(Application::subscription)
        .run_with(Application::new)
}
