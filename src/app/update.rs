use super::{message::Message, state::State};

/// iced update function
pub fn handle(_state: &mut State, message: Message) {
    match message {
        Message::EventOccurred(event) => match event {
            iced::Event::Window(event) => match event {
                iced::window::Event::FileDropped(path) => {
                    if !path.extension().is_some_and(|x| x.to_str() == Some("xlsx")) {
                        return;
                    }

                    let _filepath = path.to_string_lossy().to_string();
                }
                _ => (),
            },
            _ => (),
        },
        Message::DummyButton => {}
    }
}
