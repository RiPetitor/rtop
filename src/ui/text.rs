use crate::app::Language;

pub fn tr<'a>(lang: Language, en: &'a str, ru: &'a str) -> &'a str {
    match lang {
        Language::English => en,
        Language::Russian => ru,
    }
}
