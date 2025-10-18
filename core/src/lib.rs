use notify::Event;

pub mod constants;
pub mod nimbus_protocol;

pub use nimbus_protocol::NimbusProtocol;
pub use anyhow::Result;

pub type FsEvent = Event;