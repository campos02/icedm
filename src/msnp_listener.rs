use iced::futures::channel::mpsc;
use iced::futures::{SinkExt, Stream, StreamExt};
use iced::stream;
use std::sync::Arc;

#[derive(Clone)]
pub enum Event {
    Ready(mpsc::Sender<Input>),
    NotificationServer(msnp11_sdk::Event),
    Switchboard {
        session_id: Arc<String>,
        event: msnp11_sdk::Event,
    },
}

pub enum Input {
    NewClient(Arc<msnp11_sdk::Client>),
    NewSwitchboard(Arc<msnp11_sdk::Switchboard>),
}

pub fn listen() -> impl Stream<Item = Event> {
    stream::channel(64, |mut output| async move {
        let (sender, mut receiver) = mpsc::channel::<Input>(64);
        let _ = output.send(Event::Ready(sender)).await;

        loop {
            let input = receiver.select_next_some().await;
            match input {
                Input::NewClient(client) => {
                    let output = output.clone();

                    client.add_event_handler_closure(move |event| {
                        let mut output = output.clone();
                        async move {
                            let _ = output.send(Event::NotificationServer(event)).await;
                        }
                    })
                }

                Input::NewSwitchboard(switchboard) => {
                    let output = output.clone();
                    if let Ok(Some(session_id)) = switchboard.get_session_id() {
                        let session_id = Arc::new(session_id);

                        switchboard.add_event_handler_closure(move |event| {
                            let session_id = session_id.clone();
                            let mut output = output.clone();

                            async move {
                                let _ = output.send(Event::Switchboard { session_id, event }).await;
                            }
                        })
                    }
                }
            }
        }
    })
}
