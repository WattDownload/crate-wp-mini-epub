pub(super) struct ProcessedChapter {
    pub(super) index: usize,
    pub(super) title: String,
    pub(super) file_name: String,
    pub(super) html_content: String,
    pub(super) images: Vec<ImageAsset>,
}

pub(super) struct ImageAsset {
    pub(super) epub_path: String,
    pub(super) data: Vec<u8>,
}