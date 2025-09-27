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
#[cfg(not(target_arch = "wasm32"))]
pub use processor::download_story_to_file; // Only expose `download_story_to_file` in non-WASM builds

pub use processor::download_story_to_memory;

// Your prelude would then also be explicit
pub mod prelude {
    pub use crate::auth::{login, logout};
    pub use crate::error::AppError;

    // Only expose `download_story_to_file` in non-WASM builds
    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::processor::download_story_to_file;
    
    pub use crate::processor::download_story_to_memory;
}