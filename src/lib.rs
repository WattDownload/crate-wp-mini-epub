// Keep modules private to the crate
mod auth;
mod html;
mod models;
mod processor;
mod error;
mod types;
mod lang_util;

// Expose own items
pub use auth::{login, logout};
pub use error::AppError;
pub use crate::types::StoryDownload;

// Re-export the necessary types from the wp-mini crate
pub use wp_mini::field::StoryField;
pub use wp_mini::types::StoryResponse; // We return this, so re-export it too!

// Be explicit with the processor module's public API
#[cfg(not(target_arch = "wasm32"))]
pub use processor::download_story_to_file; // Only expose `download_story_to_file` in non-WASM builds

pub use processor::download_story_to_memory;

// Your prelude would then also be explicit
pub mod prelude {
    pub use crate::auth::{login, logout};
    pub use crate::error::AppError;
    pub use crate::types::StoryDownload;

    // Re-export from the prelude as well for convenience
    pub use wp_mini::field::StoryField;
    pub use wp_mini::types::StoryResponse;

    // Only expose `download_story_to_file` in non-WASM builds
    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::processor::download_story_to_file;
    
    pub use crate::processor::download_story_to_memory;
}