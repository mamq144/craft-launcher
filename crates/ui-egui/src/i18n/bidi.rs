//! Bidirectional text handling (UAX #9) for RTL interface languages such as Arabic.
//!
//! Reorders mixed RTL/LTR runs into visual presentation order while preserving logical
//! character order within each run so OpenType shaping (HarfRust) produces correct cursive glyphs.

use std::borrow::Cow;
use unicode_bidi::{BidiInfo, Level};

/// Reorder text with mixed directionality into visual run order for layout and display.
pub fn reorder(text: &str) -> Cow<'_, str> {
    if !has_rtl(text) {
        return Cow::Borrowed(text);
    }

    let bidi_info = BidiInfo::new(text, Some(Level::rtl()));
    if !bidi_info.has_rtl() {
        return Cow::Borrowed(text);
    }

    let mut result = String::with_capacity(text.len());
    for para in &bidi_info.paragraphs {
        let (levels, runs) = bidi_info.visual_runs(para, para.range.clone());
        if runs.iter().all(|r| levels[r.start].is_ltr()) {
            result.push_str(&text[para.range.clone()]);
            continue;
        }

        for run in runs {
            let is_rtl = levels[run.start].is_rtl();
            let slice = &text[run.clone()];
            if is_rtl {
                // Mirror paired punctuation / brackets within RTL runs.
                for c in slice.chars() {
                    result.push(mirror_char(c));
                }
            } else {
                result.push_str(slice);
            }
        }
    }
    Cow::Owned(result)
}

/// Does this text contain any right-to-left characters (Arabic, Hebrew, etc.)?
pub fn has_rtl(text: &str) -> bool {
    text.chars().any(is_rtl_char)
}

/// True if `c` is a character from a Right-to-Left script.
pub fn is_rtl_char(c: char) -> bool {
    matches!(
        c,
        '\u{0590}'..='\u{05FF}' // Hebrew
        | '\u{0600}'..='\u{06FF}' // Arabic
        | '\u{0750}'..='\u{077F}' // Arabic Supplement
        | '\u{08A0}'..='\u{08FF}' // Arabic Extended-A
        | '\u{FB50}'..='\u{FDFF}' // Arabic Presentation Forms-A
        | '\u{FE70}'..='\u{FEFF}' // Arabic Presentation Forms-B
    )
}

/// Mirror brackets/parentheses for RTL context per Unicode BidiMirroring.txt.
fn mirror_char(c: char) -> char {
    match c {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        '«' => '»',
        '»' => '«',
        _ => c,
    }
}
