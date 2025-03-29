use assuan::Command;
use iced::{
    widget::{button, column, row, text, text_input},
    Element, Task,
};

mod assuan;

struct Application {
    passphrase: String,
}

#[derive(Debug, Clone)]
enum Message {
    PassphraseChanged(String),
    ButtonOkPressed,
    ButtonCancelPressed,
    Output(String),
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
        }
    }

    fn view(&self) -> Element<Message> {
        let f = apply_commands(&vec![
            Command::SetOk(String::from("This is ok")),
            Command::SetPrompt(String::from("This is my seeecrets")),
            Command::SetCancel(String::from(" I have changed my mind")),
            Command::GetPin,
        ]);

        column![
            row![
                text(f.prompt),
                text_input("", &self.passphrase)
                    .on_input(Message::PassphraseChanged)
                    .secure(true),
            ],
            row!(
                button(text(f.button_cancel)).on_press(Message::ButtonCancelPressed),
                button(text(f.button_ok)).on_press(Message::ButtonOkPressed)
            )
        ]
        .into()
    }
}

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application(Application::title, Application::update, Application::view)
        // .subscription(Application::subscription)
        .window_size((400.0, 400.0))
        .run_with(Application::new)
}
