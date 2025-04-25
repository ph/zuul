// SPDX-License-Identifier: {{ license }}

use crate::config::Config;
use crate::error::ZuulErr;
use crate::fl;
use crate::form::{apply_commands, Form};
use crate::subscription::{read_external_commands_input, Event};
use assuan::{Command, Response};
use cosmic::app::{context_drawer, CosmicFlags};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::id::Id;
use cosmic::iced::platform_specific::shell::commands::{
    self,
    activation::request_token,
    layer_surface::{
        destroy_layer_surface, get_layer_surface, Anchor, KeyboardInteractivity, Layer,
    },
};
use cosmic::iced::{stream, window, Alignment, Border, Color, Length, Shadow, Subscription};
use cosmic::iced::keyboard::{self, Key, key::Named};
use cosmic::iced_runtime::core::layout::Limits;
use cosmic::iced_runtime::core::window::{Event as WindowEvent, Id as SurfaceId};
use cosmic::iced_runtime::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings;
use cosmic::iced_widget::row;
use cosmic::iced_winit::commands::overlap_notify::overlap_notify;
use cosmic::prelude::*;
use cosmic::theme::{self, Button, Container};
use cosmic::widget::{autosize, horizontal_space};
use cosmic::widget::text::body;
use cosmic::widget::text_input::StyleSheet;
use cosmic::widget::{
    self, column, container, icon, id_container, menu, nav_bar, text_input, vertical_space, Column,
};
use cosmic::widget::{button, text};
use cosmic::{cosmic_theme, surface};
use futures_util::{SinkExt, Stream};
use std::collections::HashMap;
use std::io::BufWriter;
use std::io::Write;
use std::sync::LazyLock;
use tracing::{error, info};

static AUTOSIZE_ID: LazyLock<Id> = LazyLock::new(|| Id::new("autosize"));
static MAIN_ID: LazyLock<Id> = LazyLock::new(|| Id::new("main"));
static INPUT_PASSPHRASE_ID: LazyLock<Id> = LazyLock::new(|| Id::new("input_passphrase"));

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

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
    EventOccurred(cosmic::iced::Event),
    External(Event),
    ButtonOkPressed,
    ButtonCancelPressed,
    OnPassphraseChange(String),
    OnPassphraseSubmit(String),
    //
    TogglePassphraseVisibility(bool),
    Ready,
    Fatal,
}

#[derive(Debug, Clone)]
pub enum ZuulTasks {
    Open,
}

impl std::fmt::Display for ZuulTasks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZuulTasks::Open => write!(f, "open"),
        }
    }
}

pub struct Args {}

impl CosmicFlags for Args {
    type SubCommand = ZuulTasks;
    type Args = Vec<String>;
}

#[derive(Clone)]
enum State {
    Waiting(WaitingState),
    Display(DisplayState),
}

#[derive(Default, Clone)]
struct WaitingState {
    received_commands: Vec<Command>,
}

#[derive(Default, Clone)]
struct DisplayState {
    form: Form,
    passphrase: String,
}

impl cosmic::Application for Zuul {
    type Executor = cosmic::executor::single::Executor;
    type Flags = Args;
    type Message = Message;

    const APP_ID: &'static str = "org.heyk.Zuul";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, _flags: Self::Flags) -> (Self, cosmic::app::Task<Self::Message>) {
        let mut app = Zuul {
            window_id: SurfaceId::unique(),
            state: State::Waiting(WaitingState::default()),
            core,
        };

        (app, Task::none())
    }

    fn view(&self) -> Element<Self::Message> {
        unreachable!("No main window")
    }

    fn view_window(&self, _id: SurfaceId) -> Element<Self::Message> {
        match &self.state {
            State::Display(state) => {
                let prompt = text(state.form.prompt());

                let pin = text_input::secure_input("", state.passphrase.clone(), None, true)
                    .on_input(Message::OnPassphraseChange)
                    .id(INPUT_PASSPHRASE_ID.clone())
                    .editing(true)
                    .always_active();

                let description = if let Some(d) = state.form.description() {
                    Some(text(d).align_y(Vertical::Center))
                } else {
                    None
                };

                let actions = container(row![
		    button::custom(text(state.form.button_cancel()))
			.on_press(Message::ButtonCancelPressed),
		    horizontal_space(),
		    button::custom(text(state.form.button_ok())).on_press(Message::ButtonOkPressed),
		]).align_x(Horizontal::Right);

                let content = Column::new()
                    .push(prompt)
                    .push(pin)
		    .push_maybe(description)
		    .push(vertical_space().height(Length::Fixed(16.)))
                    .push(actions);

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
                    .padding([24, 24]);

                autosize::autosize(window, AUTOSIZE_ID.clone())
                    .auto_height(true)
                    .min_width(300.)
                    .max_width(400.)
                    .max_height(1920.)
                    .into()
            }
            State::Waiting(state) => unreachable!(),
        }
    }

    fn update(&mut self, message: Self::Message) -> cosmic::app::Task<Self::Message> {
        use Message::*;

        match &mut self.state {
            State::Waiting(s) => match message {
                External(Event::Bye) => self.exit(),
                External(Event::Form(form)) => {
                    self.state = State::Display(DisplayState { form, ..Default::default() });

                    return Task::batch(vec![
                        self.show(),
                        text_input::focus(INPUT_PASSPHRASE_ID.clone()),
                    ]);
                }
                _ => {}
            },
	    State::Display(s) => match message {
		Message::EventOccurred(cosmic::iced::Event::Keyboard(keyboard::Event::KeyReleased {
		    key: Key::Named(Named::Escape),
		    ..
		})) => self.exit(),
		ButtonOkPressed => {
		    perform_response(Response::Data(s.passphrase.clone()));
		}
		ButtonCancelPressed => self.exit(),
		OnPassphraseChange(passphrase) => {
		    s.passphrase = passphrase;
		}
		OnPassphraseSubmit(passphrase) =>  {
		    s.passphrase = passphrase.clone();
		    perform_response(Response::Data(s.passphrase.clone()));
		}
		_ => {}
	    },
	}
	Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![subscribe_to_commands()])
    }
}

impl Zuul {
    fn exit(&self) {
	std::process::exit(0);
    }

    fn show(&self) -> cosmic::app::Task<Message> {
        Task::batch(vec![
            get_layer_surface(SctkLayerSurfaceSettings {
                id: self.window_id,
                keyboard_interactivity: KeyboardInteractivity::OnDemand,
                layer: Layer::Top,
                namespace: "zuul".into(),
                size: None,
                size_limits: Limits::NONE.min_width(1.0).min_height(1.0).max_width(600.0),
                exclusive_zone: -1,
                ..Default::default()
            }),
        ])
    }
}

fn subscribe_to_window_events() -> Subscription<Message> {
    cosmic::iced::event::listen().map(Message::EventOccurred)
}

pub fn subscribe_to_commands() -> Subscription<Message> {
    Subscription::run_with_id(
        std::any::TypeId::of::<Event>(),
        read_external_commands_input(),
    )
    .map(|e| match e {
        Ok(c) => Message::External(c),
        Err(_) => Message::Fatal,
    })
}

async fn perform_response(response: Response) -> Result<(), ZuulErr> {
    let mut stdout = std::io::stdout();
    let mut w = BufWriter::new(&mut stdout);
    writeln!(w, "{}", response.to_pinentry()).map_err(|_| ZuulErr::Output)?;
    w.flush().map_err(|_| ZuulErr::Output)?;
    Ok(())
}
