//! API routes for pine-tv

pub mod check;
pub mod data;
pub mod examples;
pub mod run;
pub mod ws;

pub use check::CheckHandler;
pub use data::DataHandler;
pub use run::RunHandler;
pub use ws::WsHandler;
