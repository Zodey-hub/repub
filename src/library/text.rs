//! The form a text is searched in, the same when a book is indexed and when it is searched.

use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// The `text` in lower case, without accents.
pub fn folded(text: &str) -> String {
    text.to_lowercase()
        .nfd()
        .filter(|character| !is_combining_mark(*character))
        .collect()
}

/// The words of `text`: its runs of letters and digits, the punctuation and the spaces separate them.
pub fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
}

/// The primary subtag of a language tag in lower case (`en-US` -> `en`), the key of the language index.
pub fn primary_language(tag: &str) -> String {
    tag.split('-')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_is_searched_without_case_and_accents() {
        assert_eq!(
            folded("Jókai MÓR: A KŐSZÍVŰ ember fiai, Çà"),
            "jokai mor: a koszivu ember fiai, ca"
        );
    }

    #[test]
    fn punctuation_separates_words() {
        let text = "holmes, sherlock (fictitious character) -- fiction r/kiszamolo";

        assert_eq!(
            words(text).collect::<Vec<_>>(),
            [
                "holmes",
                "sherlock",
                "fictitious",
                "character",
                "fiction",
                "r",
                "kiszamolo"
            ]
        );
    }
}
