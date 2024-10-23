use crate::app::App;

#[derive(Clone, Debug)]
pub enum BackgroundEvent {
    InfoMessage(String),
    ErrorMessage(String),
}

impl App {
    pub fn check_background_events(&mut self) {
        self.background_event_channel.rx.try_recv().ok().map(
            |event: BackgroundEvent| match event {
                BackgroundEvent::InfoMessage(message) => self.show_info(message),
                BackgroundEvent::ErrorMessage(message) => {
                    self.error_message = Some(message);
                }
            },
        );
    }
}
