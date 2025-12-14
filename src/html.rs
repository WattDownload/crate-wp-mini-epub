use anyhow::{anyhow, Context, Result};
use lol_html::{element, html_content::ContentType, HtmlRewriter, Settings};
use quick_xml::{events::Event, Reader, Writer};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
pub(super) fn re_encode_html(html_fragment: &str) -> Result<String> {
    let wrapped_html = format!("<root>{}</root>", html_fragment);
    let mut reader = Reader::from_str(&wrapped_html);
    let config = reader.config_mut();
    config.trim_text(false);
    config.expand_empty_elements = false;
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) if e.name().as_ref() != b"root" => {
                writer.write_event(Event::Start(e))?;
            }
            Ok(Event::End(e)) if e.name().as_ref() != b"root" => {
                writer.write_event(Event::End(e))?;
            }
            Ok(_event @ Event::Start(_)) | Ok(_event @ Event::End(_)) => {}
            Ok(Event::Eof) => break,
            Ok(e) => {
                writer.write_event(e)?;
            }
            Err(e) => {
                return Err(anyhow!(
                    "XML parsing error at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ));
            }
        }
    }
    let result_bytes = writer.into_inner().into_inner();
    let final_string = String::from_utf8(result_bytes)?;
    Ok(final_string)
}

pub(super) fn rewrite_and_clean_html(
    html_in: &str,
    embed_images: bool,
    image_map: &HashMap<String, String>,
) -> Result<String> {
    let output_buffer = Arc::new(Mutex::new(String::new()));
    let output_clone = Arc::clone(&output_buffer);

    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("p[data-media-type='image']", |el| {
                    el.remove_and_keep_content();
                    Ok(())
                }),
                element!("*[data-p-id]", |el| {
                    el.remove_attribute("data-p-id");
                    Ok(())
                }),
                element!("br", |el| {
                    el.replace("<br />", ContentType::Html);
                    Ok(())
                }),
                element!("img", move |el| {
                    if let Some(src) = el.get_attribute("src")
                        && embed_images
                            && let Some(new_src) = image_map.get(&src) {
                                el.set_attribute("src", new_src)?;
                            }

                    // Remove unwanted data attributes from the image tag.
                    el.remove_attribute("data-original-width");
                    el.remove_attribute("data-original-height");

                    // This part rebuilds the tag to ensure it's self-closing (e.g., <img ... />)
                    // for XHTML compatibility in the EPUB.
                    let mut new_tag = String::from("<img");
                    for attr in el.attributes() {
                        new_tag.push_str(&format!(" {}=\"{}\"", attr.name(), attr.value()));
                    }
                    new_tag.push_str(" />");

                    el.replace(&new_tag, ContentType::Html);
                    Ok(())
                }),
            ],
            ..Settings::default()
        },
        |c: &[u8]| {
            output_clone
                .lock()
                .unwrap()
                .push_str(&String::from_utf8_lossy(c));
        },
    );

    rewriter.write(html_in.as_bytes())?;
    rewriter.end()?;

    let cleaned_html = output_buffer.lock().unwrap().clone();

    re_encode_html(&cleaned_html).context("Failed to re-encode HTML for XML compatibility")
}

pub(super) fn collect_image_urls(html: &str) -> Result<Vec<String>> {
    let urls = Arc::new(Mutex::new(Vec::new()));
    let urls_clone = Arc::clone(&urls);
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![element!("img[src]", move |el| {
                if let Some(src) = el.get_attribute("src") {
                    urls_clone.lock().unwrap().push(src);
                }
                Ok(())
            })],
            ..Settings::default()
        },
        |_: &[u8]| {},
    );
    rewriter.write(html.as_bytes())?;
    rewriter.end()?;
    Ok(Arc::try_unwrap(urls).unwrap().into_inner().unwrap())
}

pub(super) fn infer_extension_from_data(data: &[u8]) -> Option<&str> {
    match data {
        [0xFF, 0xD8, 0xFF, ..] => Some("jpg"),
        [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, ..] => Some("png"),
        [0x47, 0x49, 0x46, 0x38, ..] => Some("gif"),
        _ => None,
    }
}
