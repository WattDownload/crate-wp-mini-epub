// Keep modules private to the crate
mod auth;
mod html;
mod models;
mod processor;
mod error;

// Expose only the necessary public items
pub use auth::{login, logout};
pub use error::AppError;

// Be explicit with the processor module's public API
pub use processor::{
    download_story_to_file,
    download_story_to_memory,
    // Any other public functions or structs from processor
};

// Your prelude would then also be explicit
pub mod prelude {
    pub use crate::auth::{login, logout};
    pub use crate::error::AppError;
    pub use crate::processor::{
        download_story_to_file,
        download_story_to_memory,
    };
}