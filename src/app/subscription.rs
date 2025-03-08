use iced::{Subscription, event};

use super::{message::Message, state::State};

pub fn handle(_: &State) -> Subscription<Message> {
    event::listen().map(Message::EventOccurred)
}
