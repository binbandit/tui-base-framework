use std::any::Any;

pub enum Message {
    Quit,
    Custom(Box<dyn Any + Send>),
}
