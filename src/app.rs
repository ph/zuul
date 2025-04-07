use std::sync::LazyLock;

use cosmic::{iced::{self, Size}, widget::autosize};
use cosmic::iced::widget::column;
use cosmic::app::{Core, Settings, Task};

const MIN_WIDTH: f64 = 400.;
const MIN_HEIGHT: f64 = 130.;
const MAX_WIDTH: f64 = 400.;
const MAX_HEIGHT: f64 = 250.;

static AUTOSIZE_ID: LazyLock<iced::id::Id> = LazyLock::new(|| iced::id::Id::new("pinentry_dialog"));

pub struct App {
    core: Core,
}

#[derive(Clone, Debug)]
pub enum Message {
    Start
}

impl cosmic::Application for App {
    type Executor = cosmic::executor::Default;

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
	(Self { core }, Task::none())

    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Start => Task::none(),
        }
    }

    fn view(&self) -> cosmic::Element<Self::Message> {
	let c = cosmic::widget::container(
	    column![
		cosmic::widget::text("title"),
		cosmic::widget::text("Description"),
	    ]
	);

	autosize::autosize(c, AUTOSIZE_ID.clone())
	    .min_width(200.)
	    .min_height(100.)
	    .max_width(300.)
	    .max_height(1920.)
	    .into()
    }
}

pub fn run() -> iced::Result {
    let settings = Settings::default()
	.size(Size::new(1024., 768.))
	.client_decorations(false)
	.transparent(true)
	.autosize(true);

    cosmic::app::run::<App>(settings, ())
}

