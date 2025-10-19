use super::{
    html, lang_util,
    models::{ImageAsset, ProcessedChapter},
};
use crate::error::AppError;
use crate::types::StoryDownload;
use anyhow::{anyhow, Result};
use futures::stream::{self, StreamExt};
use iepub::prelude::{EpubBuilder, EpubHtml};
use reqwest::Client;
use sanitize_filename::{sanitize_with_options, Options};
#[cfg(not(target_arch = "wasm32"))] // Excluded for wasm32
use std::path::PathBuf;
use std::{
    collections::HashMap,
    io::{Cursor, Read},
    path::Path,
};
use tracing::{info, instrument, warn};
use wp_mini::field::{LanguageField, PartStubField, StoryField, UserStubField};
use wp_mini::types::StoryResponse;
use wp_mini::WattpadClient;
use zip::ZipArchive;

static PLACEHOLDER_IMAGE_DATA: &[u8] = include_bytes!("../assets/placeholder.jpg");
static PLACEHOLDER_EPUB_PATH: &str = "images/placeholder.jpg";

// --- PUBLIC API FUNCTIONS ---

/// Downloads and processes a Wattpad story, saving the result as an EPUB file.
///
/// Excluded for wasm32
///
/// # Arguments
/// * `output_path` - The directory where the final `.epub` file will be saved.
///
/// # Returns
/// A `Result` containing the full `PathBuf` to the generated file.
#[cfg(not(target_arch = "wasm32"))]
#[instrument(skip(client, concurrent_requests), fields(id = story_id, path = %output_path.display()))]
pub async fn download_story_to_folder(
    client: &Client,
    story_id: u64,
    embed_images: bool,
    concurrent_requests: usize,
    output_path: &Path,
    extra_fields: Option<&[StoryField]>,
) -> Result<StoryDownload<PathBuf>> {
    let (epub_builder, sanitized_title, story_metadata) = prepare_epub_builder(
        client,
        story_id,
        embed_images,
        concurrent_requests,
        extra_fields,
    )
    .await?;

    let final_path = output_path.join(format!("{}.epub", sanitized_title));
    epub_builder
        .file(&final_path)
        .map_err(|e| anyhow!("Failed to generate EPUB file: {:?}", e))?;

    info!(path = %final_path.display(), "Successfully generated EPUB file");
    Ok(StoryDownload {
        sanitized_title,
        epub_response: final_path,
        metadata: story_metadata,
    })
}

/// Downloads and processes a Wattpad story, saving the result to provided file.
///
/// Excluded for wasm32
///
/// # Arguments
/// * `output_file` - The file of the final `.epub` file.
///
/// # Returns
/// A `Result` containing the full `PathBuf` to the generated file.
#[cfg(not(target_arch = "wasm32"))]
#[instrument(skip(client, concurrent_requests), fields(id = story_id, path = %output_file.display()))]
pub async fn download_story_to_file(
    client: &Client,
    story_id: u64,
    embed_images: bool,
    concurrent_requests: usize,
    output_file: &Path,
    extra_fields: Option<&[StoryField]>,
) -> Result<StoryDownload<PathBuf>> {
    let (epub_builder, sanitized_title, story_metadata) = prepare_epub_builder(
        client,
        story_id,
        embed_images,
        concurrent_requests,
        extra_fields,
    )
    .await?;

    epub_builder
        .file(&output_file)
        .map_err(|e| anyhow!("Failed to generate EPUB file: {:?}", e))?;

    info!(path = %output_file.display(), "Successfully generated EPUB file");
    Ok(StoryDownload {
        sanitized_title,
        epub_response: output_file.to_path_buf(),
        metadata: story_metadata,
    })
}

/// Downloads and processes a Wattpad story, returning the EPUB as an in-memory byte vector.
///
/// # Returns
/// A `Result` containing the `Vec<u8>` of the generated EPUB file.
#[instrument(skip(client, concurrent_requests), fields(id = story_id))]
pub async fn download_story_to_memory(
    client: &Client,
    story_id: u64,
    embed_images: bool,
    concurrent_requests: usize,
    extra_fields: Option<&[StoryField]>,
) -> Result<StoryDownload<Vec<u8>>> {
    let (epub_builder, sanitized_title, story_metadata) = prepare_epub_builder(
        client,
        story_id,
        embed_images,
        concurrent_requests,
        extra_fields,
    )
    .await?;

    let epub_bytes = epub_builder
        .mem()
        .map_err(|e| anyhow!("Failed to generate EPUB in memory: {:?}", e))?;

    info!(
        bytes = epub_bytes.len(),
        "Successfully generated EPUB in memory"
    );
    Ok(StoryDownload {
        sanitized_title,
        epub_response: epub_bytes,
        metadata: story_metadata,
    })
}

// --- PRIVATE CORE LOGIC ---

/// Core internal function to fetch, process, and prepare an EpubBuilder instance.
/// This function is not concerned with the final output format (file or memory).
/// It returns the builder and a sanitized title for potential filename usage.
async fn prepare_epub_builder(
    client: &Client,
    story_id: u64,
    embed_images: bool,
    concurrent_requests: usize,
    extra_fields: Option<&[StoryField]>,
) -> Result<(EpubBuilder, String, StoryResponse)> {
    info!("Starting story download and processing");
    let wp_client = WattpadClient::builder()
        .reqwest_client(client.clone())
        .build();

    // --- 1. Fetch Story Info ---
    let mut story_fields: Vec<StoryField> = vec![
        StoryField::Title,
        StoryField::Description,
        StoryField::Cover,
        StoryField::Language(vec![LanguageField::Id]),
        StoryField::User(vec![UserStubField::Username]),
        StoryField::Parts(vec![PartStubField::Id, PartStubField::Title]),
    ];

    if let Some(fields) = extra_fields {
        story_fields.extend_from_slice(fields);
    }

    // Remove duplicates (I guess this's not needed, though)
    story_fields.sort();
    story_fields.dedup();

    let story = wp_client
        .story
        .get_story_info(story_id, Some(&story_fields))
        .await
        .map_err(|_| AppError::MetadataFetchFailed)?;

    info!(title = ?story.title, "Successfully fetched story metadata");

    // --- 2. Fetch Story Content as a ZIP ---
    let zip_bytes = wp_client
        .story
        .get_story_content_zip(story_id)
        .await
        .map_err(|_| AppError::DownloadFailed)?;

    info!("Successfully downloaded story content ZIP");

    // --- 3. Process ZIP in Memory ---
    let mut chapter_html_map: HashMap<i64, String> = HashMap::new();
    let zip_cursor = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(zip_cursor)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = match Path::new(file.name()).file_name() {
            Some(name) => name.to_string_lossy().into_owned(),
            None => continue,
        };

        if let Ok(part_id) = file_name.parse::<i64>() {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            chapter_html_map.insert(part_id, contents);
        }
    }

    // --- 4. Process Chapters Concurrently ---
    let chapter_metadata = story.parts.clone().ok_or(AppError::MetadataFetchFailed)?;
    let total_chapter_count = chapter_metadata.len(); // <-- GET THE COUNT HERE
    info!(count = total_chapter_count, "Starting chapter processing");

    // Consume `chapter_metadata` and `chapter_html_map` to get owned values.
    let chapters_to_process = chapter_metadata.into_iter().filter_map(|part| {
        // chapter_metadata is moved here
        part.id.and_then(|id_u64| {
            let id_i64 = id_u64 as i64;
            // Use .remove() to take ownership of the String from the HashMap.
            chapter_html_map.remove(&id_i64).map(|html| (part, html))
        })
    });

    let processed_chapters_results: Vec<Result<ProcessedChapter>> =
        stream::iter(chapters_to_process.enumerate())
            .map(|(i, (metadata, html_content))| async move {
                // `metadata` is owned, `html_content` is owned
                process_chapter(
                    client,
                    i + 1,
                    metadata.title.as_deref().unwrap_or("Untitled Chapter"),
                    &html_content,
                    embed_images,
                    concurrent_requests,
                )
                .await
            })
            .buffer_unordered(concurrent_requests)
            .collect()
            .await;

    let mut successfully_processed: Vec<ProcessedChapter> = Vec::new();
    for result in processed_chapters_results {
        match result {
            Ok(chapter) => successfully_processed.push(chapter),
            Err(e) => warn!("Failed to process a chapter: {}", e),
        }
    }

    successfully_processed.sort_by_key(|c| c.index);
    info!(
        success_count = successfully_processed.len(),
        total_count = total_chapter_count,
        "Finished chapter processing"
    );

    // --- 5. Build EPUB ---
    let author = story
        .user
        .as_ref()
        .and_then(|u| u.username.as_deref())
        .unwrap_or("Unknown Author");

    let story_title = story.title.as_deref().unwrap_or("Untitled Story");
    let story_description = story.description.as_deref().unwrap_or("");

    let language_id = story
        .language
        .as_ref() // Safely get an Option<&Language>
        .and_then(|lang| lang.id) // Chain to get the inner Option<u64>
        .unwrap_or(1); // Provide a default if any part of the chain was None

    let language_code = lang_util::get_lang_code(language_id);
    let language_dir = lang_util::get_direction_for_lang_id(language_id);

    info!(author, title = story_title, "Building EPUB file");

    let mut epub_builder = EpubBuilder::default()
        .with_title(story_title)
        .with_creator(author)
        .with_description(story_description)
        .with_direction(language_dir)
        .add_assets(PLACEHOLDER_EPUB_PATH, PLACEHOLDER_IMAGE_DATA.to_vec());

    if let Some(cover_url) = story.cover.as_deref() {
        if let Ok(Some(cover_data)) = download_image(client, cover_url).await {
            info!("Adding cover image to EPUB");
            epub_builder = epub_builder.cover("cover.jpg", cover_data);
        }
    }

    for chapter in successfully_processed {
        for image in chapter.images {
            epub_builder = epub_builder.add_assets(&image.epub_path, image.data);
        }
        epub_builder = epub_builder.add_chapter(
            EpubHtml::default()
                .with_title(&chapter.title)
                .with_file_name(&chapter.file_name)
                .with_language(language_code)
                .with_data(chapter.html_content.as_bytes().to_vec()),
        );
    }

    let sanitized_title = format!(
        "{}-{}",
        story_id,
        sanitize_with_options(
            story_title,
            Options {
                replacement: "_",     // Set the replacement to an underscore
                ..Default::default() // Use default values for other options like `windows` and `truncate`
            }
        )
    );

    Ok((epub_builder, sanitized_title, story))
}

// --- PRIVATE HELPER FUNCTIONS ---

#[instrument(skip(client, html_in), fields(index, title))]
async fn process_chapter(
    client: &Client,
    index: usize,
    title: &str,
    html_in: &str,
    embed_images: bool,
    concurrent_requests: usize,
) -> Result<ProcessedChapter> {
    let mut images = Vec::new();
    let image_map = if embed_images {
        let image_urls = html::collect_image_urls(html_in)?;

        let image_download_futures = stream::iter(image_urls)
            .map(|url| async move {
                let download_result = download_image(client, &url).await.unwrap_or(None);
                (url, download_result)
            })
            .buffer_unordered(concurrent_requests)
            .collect::<Vec<(String, Option<Vec<u8>>)>>()
            .await;

        let mut map = HashMap::new();
        let mut successful_image_index = 0;
        for (original_url, data_option) in image_download_futures {
            if let Some(data) = data_option {
                // --- SUCCESSFUL DOWNLOAD ---
                let extension = html::infer_extension_from_data(&data).unwrap_or("jpg");
                let epub_path = format!(
                    "images/chapter_{}/image_{}.{}",
                    index, successful_image_index, extension
                );

                // Add the new asset to be bundled with the chapter
                images.push(ImageAsset {
                    epub_path: epub_path.clone(),
                    data,
                });

                // Map the original URL to the new, unique path for this image
                map.insert(original_url, epub_path);

                successful_image_index += 1;
            } else {
                // --- FAILED OR INVALID URL ---
                // Map the original URL to the global placeholder path.
                map.insert(original_url, PLACEHOLDER_EPUB_PATH.to_string());
            }
        }
        map
    } else {
        HashMap::new()
    };

    let cleaned_html = html::rewrite_and_clean_html(html_in, embed_images, &image_map)?;

    Ok(ProcessedChapter {
        index,
        title: title.to_string(),
        file_name: format!("{}.xhtml", index),
        html_content: cleaned_html,
        images,
    })
}

async fn download_image(client: &Client, url: &str) -> Result<Option<Vec<u8>>> {
    if reqwest::Url::parse(url).is_err() {
        warn!(
            url,
            "Invalid image URL found. It will be replaced by a placeholder."
        );
        return Ok(None); // Signal failure for invalid URLs.
    }

    let response = client.get(url).send().await;

    match response {
        Ok(resp) if resp.status().is_success() => Ok(Some(resp.bytes().await?.to_vec())),
        Ok(resp) => {
            warn!(status = %resp.status(), url, "Failed to download image (non-success status). Replacing with placeholder.");
            Ok(None)
        }
        Err(e) => {
            warn!(error = %e, url, "Failed to download image (request error). Replacing with placeholder.");
            Ok(None)
        }
    }
}
