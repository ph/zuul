// SPDX-FileCopyrightText: 2025 Pier-Hugues Pellerin <ph@heykimo.com>
//
// SPDX-License-Identifier: MIT

use std::io::{BufWriter, ErrorKind};

use assuan::{Command, ParseErr, Response};
use iced::Subscription;
use iced::Theme;
use iced::futures::sink::SinkExt;
use iced::stream;
use iced::{
    Element, Event, Task, event,
    futures::Stream,
    keyboard::{self, Key, key::Named},
    widget::{button, column, container, horizontal_space, row, text, text_input},
    window::{self, Position, settings::PlatformSpecific},
};
use std::io::Write;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;

static INPUT_PASSPHRASE_ID: &str = "INPUT_PASSPHRASE_ID";

mod assuan;

#[derive(Debug, Clone)]
enum ZuulErr {
    Input(ErrorKind),
    Parsing(ParseErr),
    Output,
}

impl std::error::Error for ZuulErr {}
impl std::fmt::Display for ZuulErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZuulErr::Input(e) => write!(f, "error `{}` while reading stdin input", e),
            ZuulErr::Parsing(e) => write!(f, "error `{}` while parssing pinentry commands", e),
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

#[derive(Debug, Clone)]
enum Message {
    PassphraseChanged(String),
    ButtonOkPressed,
    ButtonCancelPressed,
    Input(Command),
    EventOccurred(Event),
    Result(Result<(), ZuulErr>),
    Fatal,
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

#[derive(Default)]
struct WaitingState {
    title: String,
    received_commands: Vec<Command>,
}

struct DisplayState {
    form: Form,
    passphrase: String,
}

enum Application {
    Waiting(WaitingState),
    Display(DisplayState),
}

impl Application {
    fn new() -> (Self, Task<Message>) {
        (
            Self::Waiting(WaitingState {
                title: "".to_string(),
                received_commands: Vec::new(),
            }),
            Task::none(),
        )
    }

    fn title(&self) -> String {
        match self {
            Application::Waiting(state) => state.title.clone(),
            Application::Display(_state) => String::from("display"),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match self {
            Application::Waiting(state) => match message {
                Message::Result(_r) => Task::none(),
                Message::Input(command) => match command {
                    Command::Bye => self.close(),
                    Command::GetPin => {
                        state.received_commands.push(command);
                        let f = apply_commands(&state.received_commands);

                        *self = Application::Display(DisplayState {
                            form: f,
                            passphrase: String::new(),
                        });

                        text_input::focus(INPUT_PASSPHRASE_ID)
                    }
                    _ => {
                        state.received_commands.push(command);
                        Task::perform(perform_response(Response::Ok), Message::Result)
                    }
                },
                _ => Task::none(),
            },
            Application::Display(state) => match message {
                Message::EventOccurred(Event::Keyboard(keyboard::Event::KeyReleased {
                    key: Key::Named(Named::Escape),
                    ..
                })) => self.close(),
                Message::PassphraseChanged(p) => {
                    state.passphrase = p;
                    Task::none()
                }
                Message::ButtonOkPressed => {
                    let passphrase = state.passphrase.clone();
                    *self = Application::Waiting(WaitingState::default());

                    Task::future(perform_response(Response::Data(passphrase)))
                        .then(|_| Task::future(perform_response(Response::Ok)))
                        .map(Message::Result)
                }
                Message::ButtonCancelPressed => window::get_latest().and_then(window::close),
                Message::Result(_) => Task::none(),
                Message::Input(_command) => Task::none(),
                _ => Task::none(),
            },
        }
    }

    fn close(&self) -> Task<Message> {
        window::get_latest().and_then(window::close)
    }

    fn view(&self) -> Element<Message> {
        match self {
            Application::Waiting(_state) => row![].into(),
            Application::Display(state) => container(
                column![]
                    .push(text(state.form.prompt.clone()))
                    .push(
                        text_input("", &state.passphrase)
                            .id(INPUT_PASSPHRASE_ID)
                            .on_input(Message::PassphraseChanged)
                            .on_submit(Message::ButtonOkPressed)
                            .secure(true),
                    )
                    .push(
                        row![]
                            .push(horizontal_space())
                            .push(
                                button(text(state.form.button_cancel.clone()))
                                    .on_press(Message::ButtonCancelPressed),
                            )
                            .push(horizontal_space().width(iced::Length::Fixed(10.0)))
                            .push(
                                button(text(state.form.button_ok.clone()))
                                    .on_press(Message::ButtonOkPressed),
                            )
                            .width(iced::Length::Fill)
                            .padding(iced::Padding::from([10, 0])),
                    ),
            )
            .padding(10)
            .into(),
        }
    }

    fn theme(&self) -> Theme {
        Theme::CatppuccinMocha
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![subscribe_to_commands(), subscribe_to_window_events()])
    }
}

fn subscribe_to_window_events() -> Subscription<Message> {
    event::listen().map(Message::EventOccurred)
}

fn subscribe_to_commands() -> Subscription<Message> {
    Subscription::run_with_id(
        std::any::TypeId::of::<Command>(),
        read_external_commands_input(),
    )
    .map(|e| match e {
        Ok(c) => Message::Input(c),
        Err(_) => Message::Fatal,
    })
}

fn read_external_commands_input() -> impl Stream<Item = Result<Command, ZuulErr>> {
    stream::try_channel(1, async move |mut output| {
        let stdin = tokio::io::stdin();
        let buf = BufReader::new(stdin);
        let mut lines = buf.lines();

        let stdout = std::io::stdout();
        let mut writer = BufWriter::new(&stdout);
        writeln!(writer, "{}", Response::OkHello.to_pinentry()).map_err(|_| ZuulErr::Output)?;
        writer.flush().map_err(|_| ZuulErr::Output)?;

        while let Some(line) = lines.next_line().await? {
            let command = Command::try_from(line)?;
            let _ = output.send(command).await;
        }

        Ok(())
    })
}

async fn perform_response(response: Response) -> Result<(), ZuulErr> {
    let stdout = std::io::stdout();
    let mut writer = BufWriter::new(&stdout);
    writeln!(writer, "{}", response.to_pinentry()).map_err(|_| ZuulErr::Output)?;
    writer.flush().map_err(|_| ZuulErr::Output)?;

    Ok(())
}

fn main() -> iced::Result {
    iced::application(Application::title, Application::update, Application::view)
        .window(iced::window::Settings {
            position: Position::Centered,
            platform_specific: PlatformSpecific {
                application_id: String::from("zuul"),
                override_redirect: true,
            },
            ..Default::default()
        })
        .theme(Application::theme)
        .window_size((400.0, 400.0))
        .subscription(Application::subscription)
        .run_with(Application::new)
}
