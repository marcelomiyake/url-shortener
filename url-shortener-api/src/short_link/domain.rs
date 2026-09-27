use sha2::{Digest, Sha256};
use url::Url;

const MAX_URL_BYTES: usize = 2048;
const FIRST_CODE_LENGTH: usize = 12;
const MAX_CODE_LENGTH: usize = 43;
const BASE62_ALPHABET: &[u8; 62] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DestinationUrl(String);

impl DestinationUrl {
    pub fn parse(input: &str) -> Result<Self, DestinationUrlError> {
        if input.is_empty() || input.len() > MAX_URL_BYTES {
            return Err(DestinationUrlError::Length);
        }
        if input.trim() != input || input.chars().any(char::is_control) {
            return Err(DestinationUrlError::WhitespaceOrControl);
        }

        let parsed = Url::parse(input).map_err(|_| DestinationUrlError::Malformed)?;
        if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
            return Err(DestinationUrlError::UnsupportedScheme);
        }

        let raw_authority = input
            .split_once("://")
            .map(|(_, remainder)| remainder)
            .and_then(|remainder| remainder.split(['/', '?', '#']).next())
            .ok_or(DestinationUrlError::Malformed)?;
        if raw_authority.is_empty() {
            return Err(DestinationUrlError::Malformed);
        }
        let serialized = parsed.to_string();
        if !parsed.username().is_empty()
            || parsed.password().is_some()
            || raw_authority.contains('@')
        {
            return Err(DestinationUrlError::Credentials);
        }
        if serialized.len() > MAX_URL_BYTES {
            return Err(DestinationUrlError::CanonicalLength);
        }

        Ok(Self(serialized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn digest(&self) -> [u8; 32] {
        Sha256::digest(self.0.as_bytes()).into()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DestinationUrlError {
    #[error("destination URL has an invalid length")]
    Length,
    #[error("destination URL contains leading/trailing whitespace or control characters")]
    WhitespaceOrControl,
    #[error("destination URL is malformed")]
    Malformed,
    #[error("destination URL must use HTTP or HTTPS and include a host")]
    UnsupportedScheme,
    #[error("destination URL must not contain credentials")]
    Credentials,
    #[error("canonical destination URL exceeds 2048 bytes")]
    CanonicalLength,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ShortCode(String);

impl ShortCode {
    pub fn parse(value: &str) -> Result<Self, ShortCodeError> {
        if !(FIRST_CODE_LENGTH..=MAX_CODE_LENGTH).contains(&value.len())
            || !value.bytes().all(|byte| BASE62_ALPHABET.contains(&byte))
        {
            return Err(ShortCodeError);
        }
        Ok(Self(value.to_owned()))
    }

    pub fn from_digest_prefix(digest: &[u8; 32], length: usize) -> Result<Self, ShortCodeError> {
        if !(FIRST_CODE_LENGTH..=MAX_CODE_LENGTH).contains(&length) {
            return Err(ShortCodeError);
        }
        let encoded = encode_base62(digest);
        Self::parse(&encoded[..length])
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ShortCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("short code must contain 12 to 43 Base62 characters")]
pub struct ShortCodeError;

fn encode_base62(digest: &[u8; 32]) -> String {
    let mut number = digest.to_vec();
    let mut digits = Vec::with_capacity(MAX_CODE_LENGTH);

    loop {
        let mut remainder = 0_u16;
        for byte in &mut number {
            let value = remainder * 256 + u16::from(*byte);
            *byte = (value / 62) as u8;
            remainder = value % 62;
        }
        digits.push(BASE62_ALPHABET[usize::from(remainder)]);

        let first_nonzero = number.iter().position(|byte| *byte != 0);
        match first_nonzero {
            Some(index) => number.drain(..index),
            None => break,
        };
    }

    digits.reverse();
    let padding = MAX_CODE_LENGTH.saturating_sub(digits.len());
    let mut encoded = String::with_capacity(MAX_CODE_LENGTH);
    encoded.extend(std::iter::repeat_n('0', padding));
    encoded.extend(digits.into_iter().map(char::from));
    encoded
}

#[cfg(test)]
mod tests {
    use super::{DestinationUrl, DestinationUrlError, ShortCode, encode_base62};

    #[test]
    fn canonicalizes_http_urls() {
        let url = DestinationUrl::parse("HTTPS://Example.COM:443/a/../b?x=1").unwrap();

        assert_eq!(url.as_str(), "https://example.com/b?x=1");
    }

    #[test]
    fn canonicalizes_http_default_port_and_path() {
        let url = DestinationUrl::parse("http://Example.com:80/a/./b").unwrap();

        assert_eq!(url.as_str(), "http://example.com/a/b");
    }

    #[test]
    fn accepts_a_url_at_the_byte_limit() {
        let url = format!("https://example.com/{}", "a".repeat(2028));

        assert!(DestinationUrl::parse(&url).is_ok());
    }

    #[test]
    fn rejects_url_over_the_byte_limit() {
        let url = format!("https://example.com/{}", "a".repeat(2033));

        assert_eq!(
            DestinationUrl::parse(&url),
            Err(DestinationUrlError::Length)
        );
    }

    #[test]
    fn rejects_canonical_form_that_expands_past_the_limit() {
        let url = format!("https://example.com/{}x", " ".repeat(680));

        assert_eq!(
            DestinationUrl::parse(&url),
            Err(DestinationUrlError::CanonicalLength)
        );
    }

    #[test]
    fn rejects_unsupported_or_malformed_urls() {
        for value in [
            "javascript:alert(1)",
            "data:text/plain,hello",
            "file:///etc/passwd",
            "https:///missing-host",
        ] {
            assert!(DestinationUrl::parse(value).is_err(), "accepted {value}");
        }
    }

    #[test]
    fn rejects_embedded_and_empty_user_information() {
        for value in [
            "https://user:password@example.com/",
            "https://user@example.com/",
            "https://@example.com/",
        ] {
            assert_eq!(
                DestinationUrl::parse(value),
                Err(DestinationUrlError::Credentials),
                "accepted {value}"
            );
        }
    }

    #[test]
    fn rejects_ambiguous_whitespace_and_control_characters() {
        for value in [
            " https://example.com",
            "https://example.com ",
            "https://example.com/\npath",
        ] {
            assert_eq!(
                DestinationUrl::parse(value),
                Err(DestinationUrlError::WhitespaceOrControl)
            );
        }
    }

    #[test]
    fn base62_encoding_is_padded_and_deterministic() {
        let digest = [0_u8; 32];
        let encoded = encode_base62(&digest);

        assert_eq!(encoded.len(), 43);
        assert_eq!(encoded, "0".repeat(43));
        assert_eq!(
            ShortCode::from_digest_prefix(&digest, 12).unwrap().as_str(),
            "000000000000"
        );
    }

    #[test]
    fn collision_prefixes_extend_without_changing_the_existing_prefix() {
        let digest = [255_u8; 32];
        let short = ShortCode::from_digest_prefix(&digest, 12).unwrap();
        let extended = ShortCode::from_digest_prefix(&digest, 13).unwrap();

        assert_eq!(extended.as_str().get(..12), Some(short.as_str()));
        assert_eq!(extended.as_str().len(), 13);
        assert!(
            extended
                .as_str()
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric())
        );
    }

    #[test]
    fn parses_only_case_sensitive_base62_codes_in_the_supported_range() {
        assert!(ShortCode::parse("0123456789AB").is_ok());
        assert!(ShortCode::parse("0123456789aB").is_ok());
        assert!(ShortCode::parse("short").is_err());
        assert!(ShortCode::parse("0123456789A-").is_err());
    }
}
