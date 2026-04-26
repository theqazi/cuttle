//! PII redactor trait + default implementation per D-2026-04-26-30 (OQ-12).
//!
//! Per `docs/TDD.md` §5.4. Default redactor masks email-shaped strings
//! using a simple regex. Operator-configurable additional regexes live in
//! `~/.cuttle/config.toml` `[audit.redact]`. Redacted content digest is
//! computed on the redacted text, NOT the original.

/// Trait for PII redaction. Implementations are stateless and pure.
pub trait Redactor: Send + Sync {
    /// Return a redacted copy of `content`. The default implementation in
    /// [`DefaultRedactor`] handles emails, IPv4 addresses, and SSN-shaped
    /// digit groups.
    fn redact(&self, content: &str) -> String;
}

/// Default redactor: emails, IPv4, SSN-shaped sequences.
pub struct DefaultRedactor;

impl Redactor for DefaultRedactor {
    fn redact(&self, content: &str) -> String {
        let mut out = String::with_capacity(content.len());
        let mut chars = content.chars().peekable();
        let mut buf = String::new();

        while let Some(c) = chars.next() {
            if is_token_char(c) {
                buf.push(c);
                if chars.peek().map(|p| !is_token_char(*p)).unwrap_or(true) {
                    out.push_str(&redact_token(&buf));
                    buf.clear();
                }
            } else {
                if !buf.is_empty() {
                    out.push_str(&redact_token(&buf));
                    buf.clear();
                }
                out.push(c);
            }
        }
        if !buf.is_empty() {
            out.push_str(&redact_token(&buf));
        }
        out
    }
}

fn is_token_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '+' | '-' | '@')
}

fn redact_token(token: &str) -> String {
    if looks_like_email(token) {
        return "<email-redacted>".to_string();
    }
    if looks_like_ipv4(token) {
        return "<ipv4-redacted>".to_string();
    }
    if looks_like_ssn(token) {
        return "<ssn-redacted>".to_string();
    }
    token.to_string()
}

fn looks_like_email(s: &str) -> bool {
    let parts: Vec<&str> = s.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let (local, domain) = (parts[0], parts[1]);
    if local.is_empty() || domain.is_empty() {
        return false;
    }
    if !domain.contains('.') {
        return false;
    }
    domain
        .rsplit('.')
        .next()
        .map(|tld| tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic()))
        .unwrap_or(false)
}

fn looks_like_ipv4(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts
        .iter()
        .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()) && p.parse::<u8>().is_ok())
}

fn looks_like_ssn(s: &str) -> bool {
    if s.len() != 11 {
        return false;
    }
    let bytes = s.as_bytes();
    bytes.iter().enumerate().all(|(i, b)| match i {
        3 | 6 => *b == b'-',
        _ => b.is_ascii_digit(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_email() {
        let r = DefaultRedactor;
        let out = r.redact("contact me at alice@example.com please");
        assert!(out.contains("<email-redacted>"));
        assert!(!out.contains("alice@example.com"));
    }

    #[test]
    fn redacts_ipv4() {
        let r = DefaultRedactor;
        let out = r.redact("server 192.168.1.1 responds slowly");
        assert!(out.contains("<ipv4-redacted>"));
    }

    #[test]
    fn redacts_ssn() {
        let r = DefaultRedactor;
        let out = r.redact("ssn 123-45-6789 should not be logged");
        assert!(out.contains("<ssn-redacted>"));
    }

    #[test]
    fn passes_through_innocent_text() {
        let r = DefaultRedactor;
        let out = r.redact("the quick brown fox jumps over the lazy dog");
        assert_eq!(out, "the quick brown fox jumps over the lazy dog");
    }
}
