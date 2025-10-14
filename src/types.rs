pub use wp_mini::types::StoryResponse;
pub struct StoryDownload<T> {
    /// The generated EPUB file, either as a PathBuf or Vec<u8>.
    pub epub_response: T,
    /// The full story metadata fetched from Wattpad.
    pub metadata: StoryResponse,
}
