use iepub::prelude::Direction;

/// Maps the API's numeric ID to a standard IETF language code.
/// Defaults to "en" for unknown codes.
pub(crate) fn get_lang_code(lang_id: u64) -> &'static str {
    match lang_id {
        1 => "en",       // English
        2 => "fr",       // Français (French)
        3 => "it",       // Italiano (Italian)
        4 => "de",       // Deutsch (German)
        5 => "es",       // Español (Spanish)
        6 => "pt-PT",    // Português (Portugal)
        7 => "ru",       // Русский (Russian)
        8 => "zh-Hant",  // 繁體中文 (Traditional Chinese)
        9 => "ja",       // 日本語 (Japanese)
        10 => "ko",      // 한국어 (Korean)
        11 => "en",      // Other (defaults to English)
        12 => "zh-Hans", // 简体中文 (Simplified Chinese)
        13 => "nl",      // Nederlands (Dutch)
        14 => "pl",      // Polski (Polish)
        15 => "ro",      // Română (Romanian)
        16 => "ar",      // العربية (Arabic)
        17 => "he",      // עברית (Hebrew)
        18 => "fil",     // Filipino
        19 => "vi",      // Tiếng Việt (Vietnamese)
        20 => "id",      // Bahasa Indonesia
        21 => "hi",      // हिन्दी (Hindi)
        22 => "ms",      // Bahasa Melayu (Malay)
        23 => "tr",      // Türkçe (Turkish)
        24 => "cs",      // Česky (Czech)
        25 => "ml",      // മലയാളം (Malayalam)
        26 => "sv",      // Svenska (Swedish)
        27 => "no",      // Norsk (Norwegian)
        28 => "hu",      // Magyar (Hungarian)
        29 => "da",      // Dansk (Danish)
        30 => "el",      // ελληνικά (Greek)
        31 => "fa",      // فارسی (Persian)
        32 => "th",      // ภาษาไทย (Thai)
        33 => "is",      // Íslenska (Icelandic)
        34 => "fi",      // Suomi (Finnish)
        35 => "et",      // Eesti (Estonian)
        36 => "lv",      // Latviešu (Latvian)
        37 => "lt",      // Lietuvių (Lithuanian)
        38 => "ca",      // Català (Catalan)
        39 => "bs",      // Босански (Bosnian)
        40 => "sr",      // Српски (Serbian)
        41 => "hr",      // Hrvatski (Croatian)
        42 => "sl",      // Slovenščina (Slovenian)
        43 => "bg",      // Български (Bulgarian)
        44 => "sk",      // Slovenčina (Slovak)
        45 => "be",      // Беларускі (Belarusian)
        46 => "uk",      // Українська (Ukrainian)
        47 => "bn",      // বাংলা (Bengali)
        48 => "ur",      // اُردُو‎ (Urdu)
        49 => "ta",      // தமிழ் (Tamil)
        50 => "sw",      // Kiswahili
        51 => "af",      // Afrikaans
        52 => "pt-BR",   // Português Brasileiro (Brazilian Portuguese)
        53 => "gu",      // ગુજરાતી (Gujarati)
        54 => "or",      // ଓଡ଼ିଆ (Odia)
        55 => "pa",      // ਪੰਜਾਬੀ (Punjabi)
        56 => "as",      // অসমীয়া (Assamese)
        57 => "mr",      // मराठी (Marathi)
        _ => "en",       // Default for any unrecognized ID
    }
}

/// Maps the API's language ID to a text direction (LTR or RTL).
/// Defaults to LTR.
pub(crate) fn get_direction_for_lang_id(lang_id: u64) -> Direction {
    match lang_id {
        16 | 17 | 31 | 48 => Direction::RTL, // Arabic, Hebrew, Persian, Urdu
        _ => Direction::LTR, // All other languages are Left-to-Right
    }
}

/// Maps Language Codes to a text direction (LTR or RTL).
/// Defaults to LTR.
pub(crate) fn get_direction_for_lang_code(lang_code: &str) -> Direction {
    match lang_code {
        "ar" | "he" | "fa" | "ur" => Direction::RTL, // Arabic, Hebrew, Persian, Urdu
        _ => Direction::LTR, // All other languages are Left-to-Right
    }
}
