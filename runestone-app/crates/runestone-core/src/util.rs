pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Longest prefix of `s` that is at most `max_chars` characters (never splits a UTF-8 sequence).
pub fn truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

/// Longest prefix of `s` that is at most `max_bytes` bytes and ends on a char boundary.
pub fn truncate_bytes(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Inverse of [`html_escape`] for the entities the editor emits (plus `&nbsp;` and numeric
/// references). `&amp;` is decoded last so `&amp;lt;` becomes the literal text `&lt;`.
pub fn decode_html_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(pos) = rest.find('&') {
        out.push_str(&rest[..pos]);
        let tail = &rest[pos..];
        let decoded = tail.find(';').filter(|&end| end <= 10).and_then(|end| {
            let name = &tail[1..end];
            let ch = match name {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                "nbsp" => Some(' '),
                _ => name
                    .strip_prefix('#')
                    .and_then(|num| match num.strip_prefix(['x', 'X']) {
                        Some(hex) => u32::from_str_radix(hex, 16).ok(),
                        None => num.parse::<u32>().ok(),
                    })
                    .and_then(char::from_u32),
            };
            ch.map(|c| (c, end + 1))
        });
        match decoded {
            Some((c, consumed)) => {
                out.push(c);
                rest = &tail[consumed..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Strip HTML tags, decode entities and collapse whitespace.
pub fn plain_text(html: &str) -> String {
    let mut stripped = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                stripped.push(' ');
            }
            '>' if in_tag => in_tag = false,
            _ if !in_tag => stripped.push(c),
            _ => {}
        }
    }
    decode_html_entities(&stripped)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_chars_respects_char_boundaries() {
        assert_eq!(truncate_chars("héllo wörld", 4), "héll");
        assert_eq!(truncate_chars("日本語テキスト", 3), "日本語");
        assert_eq!(truncate_chars("abc", 10), "abc");
        assert_eq!(truncate_chars("", 3), "");
    }

    #[test]
    fn truncate_bytes_never_panics_on_multibyte() {
        let s = "日本語"; // 3 bytes each
        assert_eq!(truncate_bytes(s, 4), "日");
        assert_eq!(truncate_bytes(s, 2), "");
        assert_eq!(truncate_bytes(s, 100), s);
    }

    #[test]
    fn decode_html_entities_round_trips_html_escape() {
        for s in [
            "A & B",
            "<b>\"x\" 'y'</b>",
            "C# notes",
            "&amp;lt;",
            "5 < 6 && 7 > 3",
        ] {
            assert_eq!(decode_html_entities(&html_escape(s)), s);
        }
        assert_eq!(
            decode_html_entities("&#65;&#x42;&nbsp;&bogus; &"),
            "AB &bogus; &"
        );
    }

    #[test]
    fn plain_text_strips_markup() {
        assert_eq!(
            plain_text("<h1>Title</h1><p>Tom &amp; <strong>Jerry</strong></p>"),
            "Title Tom & Jerry"
        );
    }
}

/// Word count of note content. Notes are stored as editor HTML where blocks are concatenated
/// without whitespace (`<p>a</p><p>b</p>`), so count words of the tag-stripped text instead of
/// splitting the raw markup on whitespace.
pub fn word_count(html: &str) -> i32 {
    plain_text(html).split_whitespace().count() as i32
}

#[cfg(test)]
mod word_count_tests {
    use super::*;

    #[test]
    fn word_count_counts_words_across_blocks() {
        assert_eq!(
            word_count("<p>one two</p><p>three</p><ul><li>four</li><li>five</li></ul>"),
            5
        );
        assert_eq!(word_count(""), 0);
        assert_eq!(word_count("plain text works too"), 4);
    }
}
