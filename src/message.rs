use std::any::Any;
use std::fmt;

pub enum Message {
    Quit,
    Custom(Box<dyn Any + Send>),
}

impl Message {
    pub fn custom<T>(message: T) -> Self
    where
        T: Any + Send,
    {
        Self::Custom(Box::new(message))
    }

    pub fn downcast<T>(self) -> Result<Box<T>, Self>
    where
        T: Any + Send,
    {
        match self {
            Self::Custom(message) => message.downcast::<T>().map_err(Self::Custom),
            message => Err(message),
        }
    }
}

impl fmt::Debug for Message {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Quit => formatter.write_str("Quit"),
            Self::Custom(_) => formatter.write_str("Custom(..)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Message;

    #[test]
    fn downcasts_custom_messages() {
        let message = Message::custom(String::from("saved"));

        let value = message.downcast::<String>().expect("custom string message");

        assert_eq!(*value, "saved");
    }

    #[test]
    fn returns_original_message_when_downcast_fails() {
        let message = Message::custom(42_u32);

        let message = message
            .downcast::<String>()
            .expect_err("u32 should not downcast to String");

        assert!(matches!(message, Message::Custom(_)));
    }
}
