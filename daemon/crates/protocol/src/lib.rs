pub mod cloudevent;
pub mod command;
pub mod error;
pub mod event;
pub mod tool;
pub mod types;

pub use cloudevent::CloudEvent;
pub use command::Command;
pub use error::ProtocolError;
pub use tool::ToolView;
pub use types::SUBPROTOCOL;

pub const SPEC_VERSION: &str = "1.0";

pub const CONTENT_TYPE: &str = "application/json";
