//! The phone's forms and links in a few lines of the tool's own (D-15 addendum; EP-30a): a form's
//! body as browsers post it (`application/x-www-form-urlencoded`) read into its fields, and a
//! project's name written into a link's path. axum keeps its `http1`, `json` and `tokio`
//! features, and nothing joins the lock.

/// The content type a browser posts a form with.
pub const CONTENT_TYPE: &str = "application/x-www-form-urlencoded";

/// Whether a `Content-Type` header names a form's body, its parameters aside.
pub fn is_form(content_type: &str) -> bool {
    content_type
        .split(';')
        .next()
        .is_some_and(|kind| kind.trim().eq_ignore_ascii_case(CONTENT_TYPE))
}

/// A form's body or a query read into its fields, in their order: `+` a space, `%XX` a byte, the
/// bytes read as UTF-8 (a broken sequence replaced); a field without `=` has an empty value.
pub fn decode(body: &[u8]) -> Vec<(String, String)> {
    body.split(|&b| b == b'&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (name, value) = match pair.iter().position(|&b| b == b'=') {
                Some(at) => (&pair[..at], &pair[at + 1..]),
                None => (pair, &[][..]),
            };
            (unescape(name), unescape(value))
        })
        .collect()
}

/// The first value of field `name`.
pub fn field<'a>(fields: &'a [(String, String)], name: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

fn unescape(bytes: &[u8]) -> String {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => match (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                (Some(high), Some(low)) => {
                    out.push(high << 4 | low);
                    i += 2;
                }
                _ => out.push(b'%'),
            },
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// `text` as one segment of a link's path: every byte but the unreserved letters, digits and
/// `-._~` written as `%XX`, so a name with a space, a slash or a `?` stays one segment.
pub fn segment(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_forms_body_reads_into_its_fields() {
        let fields = decode(b"id=OT-3&disposition=rolled+into+EP-4%3A+done&urgent&empty=");
        assert_eq!(
            fields,
            [
                ("id".to_owned(), "OT-3".to_owned()),
                (
                    "disposition".to_owned(),
                    "rolled into EP-4: done".to_owned()
                ),
                ("urgent".to_owned(), String::new()),
                ("empty".to_owned(), String::new()),
            ]
        );
        assert_eq!(
            field(&fields, "disposition"),
            Some("rolled into EP-4: done")
        );
        assert_eq!(field(&fields, "missing"), None);
        // UTF-8 by its bytes, a plus written as %2B, a broken escape kept as written.
        let fields = decode("text=caf%C3%A9+%2B+%E2%86%92&odd=100%&bad=%zz".as_bytes());
        assert_eq!(field(&fields, "text"), Some("café + →"));
        assert_eq!(field(&fields, "odd"), Some("100%"));
        assert_eq!(field(&fields, "bad"), Some("%zz"));
        // Repeated fields keep their order (a question of several answers).
        let fields = decode(b"pick1=0&pick1=2");
        assert_eq!(
            fields.iter().map(|(_, v)| v.as_str()).collect::<Vec<_>>(),
            ["0", "2"]
        );
        assert!(decode(b"").is_empty());
    }

    #[test]
    fn a_name_is_one_segment_of_a_path() {
        assert_eq!(segment("harbor"), "harbor");
        assert_eq!(segment("my project/2?x"), "my%20project%2F2%3Fx");
        assert_eq!(segment("café"), "caf%C3%A9");
        // What the router decodes again.
        assert_eq!(
            decode(segment("my project/2?x").as_bytes())[0].0,
            "my project/2?x"
        );
    }

    #[test]
    fn a_form_is_named_by_its_content_type() {
        assert!(is_form("application/x-www-form-urlencoded"));
        assert!(is_form("Application/X-WWW-Form-Urlencoded; charset=UTF-8"));
        assert!(!is_form("multipart/form-data; boundary=x"));
        assert!(!is_form("application/json"));
    }
}
