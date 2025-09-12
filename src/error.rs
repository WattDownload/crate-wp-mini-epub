use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication failed: invalid username or password")]
    AuthenticationFailed,

    #[error("User is not logged in")]
    NotLoggedIn,

    #[error("Failed to log out")]
    LogoutFailed,

    #[error("Story with ID {0} could not be found")]
    StoryNotFound(i32),

    #[error("Failed to fetch story metadata from Wattpad")]
    MetadataFetchFailed,

    #[error("Failed to download story content")]
    DownloadFailed,

    #[error("Failed to process chapter content")]
    ChapterProcessingFailed,

    #[error("Failed to generate the EPUB file")]
    EpubGenerationFailed,

    #[error("An I/O error occurred")]
    IoError(#[from] std::io::Error),
}