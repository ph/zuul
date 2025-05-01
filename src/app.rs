// SPDX-License-Identfier: {{ license }}

use crate::error::ZuulErr;
use crate::form::Form;
use crate::subscription::{Event, read_external_commands_input};
use assuan::Response;
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::id::Id;
use cosmic::iced::keyboard::{self, Key, key::Named};
use cosmic::iced::platform_specific::shell::commands::layer_surface::{
    KeyboardInteractivity, Layer, get_layer_surface,
};
use cosmic::iced::{Border, Color, Length, Shadow, Subscription, window};
use cosmic::iced_runtime::core::layout::Limits;
use cosmic::iced_runtime::core::window::Id as SurfaceId;
use cosmic::iced_runtime::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings;
use cosmic::iced_widget::row;
use cosmic::iced_winit::commands::layer_surface::destroy_layer_surface;
use cosmic::prelude::*;
use cosmic::theme::{self, Container};
use cosmic::widget::{Column, container, id_container, text_input, vertical_space};
use cosmic::widget::{autosize, horizontal_space};
use cosmic::widget::{button, text};
use std::io::BufWriter;
use std::io::Write;
use std::sync::LazyLock;
use tracing::error;

static AUTOSIZE_ID: LazyLock<Id> = LazyLock::new(|| Id::new("autosize"));
static MAIN_ID: LazyLock<Id> = LazyLock::new(|| Id::new("main"));
static INPUT_PASSPHRASE_ID: LazyLock<Id> = LazyLock::new(|| Id::new("input_passphrase"));

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct Zuul {
    core: cosmic::Core,
    window_id: window::Id,
    state: State,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    External(Event),
    ButtonOkPressed,
    ButtonCancelPressed,
    OnPassphraseChange(String),
    OnPassphraseSubmit(String),
    Exit,
    Result(Result<(), ZuulErr>),
    TogglePassphraseVisibility,
}

#[derive(Clone)]
enum State {
    WaitingForm(WaitingState),
    Display(DisplayState),
    WaitingValidation,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::WaitingForm(_) => write!(f, "State::WaitingForm"),
            State::Display(_) => write!(f, "State::Display"),
            State::WaitingValidation => write!(f, "State::WaitingValidation"),
        }
    }
}

#[derive(Default, Clone)]
struct WaitingState {}

#[derive(Default, Clone)]
struct DisplayState {
    form: Form,
    passphrase: String,
    passphrase_is_visible: bool,
}

impl cosmic::Application for Zuul {
    type Executor = cosmic::executor::single::Executor;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "org.heyk.Zuul";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, _flags: Self::Flags) -> (Self, cosmic::app::Task<Self::Message>) {
        let app = Zuul {
            window_id: SurfaceId::unique(),
            state: State::WaitingForm(WaitingState::default()),
            core,
        };

        (app, Task::none())
    }

    fn view(&self) -> Element<Self::Message> {
        unreachable!("No main window")
    }

    fn view_window(&self, _id: SurfaceId) -> Element<Self::Message> {
        let Spacing { space_s, .. } = theme::active().cosmic().spacing;

        match &self.state {
            State::Display(state) => {
                let prompt = text(state.form.prompt());

                let pin = text_input::secure_input(
                    "",
                    state.passphrase.clone(),
                    Some(Message::TogglePassphraseVisibility),
                    !state.passphrase_is_visible,
                )
                .id(INPUT_PASSPHRASE_ID.clone())
                .editing(true)
                .always_active()
                .on_input(Message::OnPassphraseChange)
                .on_submit(Message::OnPassphraseSubmit);

                let description = state
                    .form
                    .description()
                    .map(|d| text(d).align_y(Vertical::Center));

                let actions = container(
                    row![
                        horizontal_space().width(Length::Fill),
                        button::standard(state.form.button_cancel())
                            .on_press(Message::ButtonCancelPressed),
                        button::suggested(state.form.button_ok())
                            .on_press(Message::ButtonOkPressed),
                    ]
                    .spacing(space_s),
                )
                .align_x(Horizontal::Right);

                let content = Column::new()
                    .push(prompt)
                    .push(pin)
                    .push_maybe(description)
                    .push(vertical_space().height(Length::Fixed(16.)))
                    .push(actions)
                    .spacing(space_s);

                let window = container(id_container(content, MAIN_ID.clone()))
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .class(Container::Custom(Box::new(|theme| container::Style {
                        text_color: Some(theme.cosmic().on_bg_color().into()),
                        icon_color: Some(theme.cosmic().on_bg_color().into()),
                        background: Some(Color::from(theme.cosmic().background.base).into()),
                        border: Border {
                            radius: theme.cosmic().corner_radii.radius_m.into(),
                            width: 1.0,
                            color: theme.cosmic().bg_divider().into(),
                        },
                        shadow: Shadow::default(),
                    })))
                    .padding(space_s);

                autosize::autosize(window, AUTOSIZE_ID.clone())
                    .auto_height(true)
                    .min_width(300.)
                    .max_width(400.)
                    .max_height(1920.)
                    .into()
            }
            State::WaitingForm(_) | State::WaitingValidation => row![].into(),
        }
    }

    fn update(&mut self, message: Self::Message) -> cosmic::app::Task<Self::Message> {
        use Message::*;
        match &mut self.state {
            State::WaitingForm(_) | State::WaitingValidation => match message {
                External(Event::Bye) => self.exit(),
                External(Event::Form(form)) => {
                    return self.transition(State::Display(DisplayState {
                        form,
                        ..Default::default()
                    }));
                }
                _ => {}
            },
            State::Display(s) => match message {
                Message::Exit | ButtonCancelPressed => self.exit(),
                ButtonOkPressed => {
                    return self.transition(State::WaitingValidation);
                }
                OnPassphraseChange(passphrase) => {
                    s.passphrase = passphrase;
                }
                OnPassphraseSubmit(passphrase) => {
                    s.passphrase = passphrase;
                    return self.transition(State::WaitingValidation);
                }
                Message::Result(r) => match r {
                    Ok(_) => self.exit(),
                    Err(err) => {
                        error!("Error: {err}");
                        std::process::exit(exitcode::DATAERR);
                    }
                },
                Message::TogglePassphraseVisibility => {
                    s.passphrase_is_visible = !s.passphrase_is_visible;
                }
                _ => {}
            },
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            subscribe_to_commands(),
            subscribe_to_specific_events(),
        ])
    }
}

impl Zuul {
    fn transition(&mut self, new_state: State) -> cosmic::app::Task<Message> {
        match (self.state.clone(), new_state.clone()) {
            (State::WaitingForm(..), State::Display(..))
            | (State::WaitingValidation, State::Display(..)) => {
		self.state = new_state;
		self.show().chain(text_input::focus(INPUT_PASSPHRASE_ID.clone()))
            }
            (State::Display(s), State::WaitingValidation) => {
                self.state = new_state;
                Task::batch(vec![self.hide(), send_passphrase(s.passphrase.clone())])
            }
            _ => {
                error!(
                    "Error: This is a bug, unexpected transition from `{}` to `{new_state}`",
                    self.state
                );
                std::process::exit(exitcode::DATAERR);
            }
        }
    }

    fn exit(&self) {
        std::process::exit(exitcode::OK);
    }

    fn show(&self) -> cosmic::app::Task<Message> {
	Task::batch(vec![
	    get_layer_surface(SctkLayerSurfaceSettings {
		id: self.window_id,
		keyboard_interactivity: KeyboardInteractivity::Exclusive,
		layer: Layer::Top,
		namespace: "zuul".into(),
		size: None,
		size_limits: Limits::NONE.min_width(1.0).min_height(1.0).max_width(600.0),
		exclusive_zone: -1,
		..Default::default()
	    }),
	])
    }

    fn hide(&self) -> cosmic::app::Task<Message> {
        destroy_layer_surface(self.window_id)
    }
}

fn subscribe_to_specific_events() -> Subscription<Message> {
    cosmic::iced::event::listen_raw(|e, _status, _id| match e {
        cosmic::iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Named(Named::Escape),
            ..
        }) => Some(Message::Exit),
        _ => None,
    })
}

pub fn subscribe_to_commands() -> Subscription<Message> {
    Subscription::run_with_id(
        std::any::TypeId::of::<Event>(),
        read_external_commands_input(),
    )
    .map(|e| match e {
        Ok(c) => Message::External(c),
        Err(err) => Message::Result(Err(err)),
    })
}

async fn reply(responses: Vec<Response>) -> Result<(), ZuulErr> {
    let mut stdout = std::io::stdout();
    let mut w = BufWriter::new(&mut stdout);
    for response in responses {
        writeln!(w, "{}", response.to_pinentry()).map_err(|_| ZuulErr::Output)?;
    }

    w.flush().map_err(|_| ZuulErr::Output)?;
    Ok(())
}

fn send_passphrase(passphrase: String) -> cosmic::app::Task<Message> {
    Task::perform(reply(vec![Response::Data(passphrase), Response::Ok]), |r| {
        cosmic::action::app(Message::Result(r))
    })
}
