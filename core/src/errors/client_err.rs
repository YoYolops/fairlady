use std::fmt::{self, Display};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkerError {
    ErrReceiverChannelClosed,
}

impl WorkerError {
    fn as_str(&self) -> &str {
        match self {
            WorkerError::ErrReceiverChannelClosed => "ERR_RECEIVER_CHANNEL_CLOSED",
        }
    }
}

impl Display for WorkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str(),)
    }
}
