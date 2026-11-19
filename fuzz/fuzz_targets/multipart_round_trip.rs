//! Encodes a proxy response the way the Worker does, then decodes the
//! multipart body and checks that the JSON payload and the attachment bytes
//! come back unchanged.
//!
//! Input layout: the first byte selects the proxy protocol. The bytes up to
//! the next NUL byte are the message content. The remaining bytes are the
//! attachment data, which comes from the third-party proxy API in production.

#![no_main]

use libfuzzer_sys::fuzz_target;
use piston_proxy_bot::fuzzing::{MultipartFile, encode};
use piston_proxy_commands::ProxyProtocol;
use twilight_model::http::{
    attachment::Attachment,
    interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};

const PROTOCOLS: [ProxyProtocol; 5] = [
    ProxyProtocol::Http,
    ProxyProtocol::Https,
    ProxyProtocol::Socks4,
    ProxyProtocol::Socks5,
    ProxyProtocol::All,
];
const CONTENT_TYPE_PREFIX: &str = "multipart/form-data; boundary=";

struct Part<'a> {
    headers: Vec<&'a [u8]>,
    body: &'a [u8],
}

fuzz_target!(|data: &[u8]| {
    let Some((&selector, rest)) = data.split_first() else {
        return;
    };
    let protocol = PROTOCOLS[usize::from(selector) % PROTOCOLS.len()];
    let (content, file_data) = match rest.iter().position(|byte| *byte == 0) {
        Some(index) => (&rest[..index], &rest[index + 1..]),
        None => (rest, &[][..]),
    };
    let content = String::from_utf8_lossy(content).into_owned();

    // Build the response the same way `proxy_response` in src/lib.rs does.
    let mut attachment =
        Attachment::from_bytes(protocol.filename().to_owned(), file_data.to_vec(), 0);
    attachment.description(protocol.attachment_description().to_owned());
    let response = InteractionResponse {
        kind: InteractionResponseType::ChannelMessageWithSource,
        data: Some(InteractionResponseData {
            attachments: Some(vec![attachment]),
            content: Some(content),
            ..InteractionResponseData::default()
        }),
    };
    let encoded = encode(
        &response,
        &MultipartFile {
            field_name: "files[0]",
            filename: protocol.filename(),
            content_type: "text/plain",
            data: file_data,
        },
    )
    .expect("encoding a serializable response must succeed");

    let boundary = encoded
        .content_type
        .strip_prefix(CONTENT_TYPE_PREFIX)
        .expect("the content type must name a multipart boundary");
    assert!(
        is_valid_boundary(boundary),
        "the boundary {boundary:?} violates RFC 2046"
    );

    let parts = decode(boundary.as_bytes(), &encoded.body)
        .expect("the encoded body must be a well-formed multipart body");
    assert_eq!(
        parts.len(),
        2,
        "the body must have a payload and a file part"
    );

    let expected_payload = serde_json::to_vec(&response).expect("the payload must serialize");
    assert_eq!(
        parts[0].headers,
        [
            &b"Content-Disposition: form-data; name=\"payload_json\""[..],
            b"Content-Type: application/json",
        ],
        "the payload part headers changed"
    );
    assert_eq!(
        parts[0].body, expected_payload,
        "the decoded payload differs from the original"
    );

    let disposition = format!(
        "Content-Disposition: form-data; name=\"files[0]\"; filename=\"{}\"",
        protocol.filename()
    );
    assert_eq!(
        parts[1].headers,
        [disposition.as_bytes(), b"Content-Type: text/plain"],
        "the file part headers changed"
    );
    assert_eq!(
        parts[1].body, file_data,
        "the decoded attachment differs from the original"
    );
});

/// RFC 2046 section 5.1.1: 1 to 70 characters from `bchars`, not ending in a space.
fn is_valid_boundary(boundary: &str) -> bool {
    (1..=70).contains(&boundary.len())
        && !boundary.ends_with(' ')
        && boundary
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"'()+_,-./:=? ".contains(&byte))
}

/// A strict RFC 2046 multipart parser that accepts no preamble, padding or
/// epilogue, because the encoder never writes them.
fn decode<'a>(boundary: &[u8], body: &'a [u8]) -> Option<Vec<Part<'a>>> {
    let delimiter = [&b"\r\n--"[..], boundary].concat();
    let mut rest = body.strip_prefix(&delimiter[2..])?;
    let mut parts = Vec::new();

    loop {
        if let Some(epilogue) = rest.strip_prefix(b"--") {
            return (epilogue == b"\r\n").then_some(parts);
        }
        rest = rest.strip_prefix(b"\r\n")?;
        let part_length = find(rest, &delimiter)?;
        let part = &rest[..part_length];
        rest = &rest[part_length + delimiter.len()..];

        let header_length = find(part, b"\r\n\r\n")?;
        parts.push(Part {
            headers: split(&part[..header_length], b"\r\n"),
            body: &part[header_length + 4..],
        });
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn split<'a>(mut bytes: &'a [u8], separator: &[u8]) -> Vec<&'a [u8]> {
    let mut pieces = Vec::new();
    while let Some(index) = find(bytes, separator) {
        pieces.push(&bytes[..index]);
        bytes = &bytes[index + separator.len()..];
    }
    pieces.push(bytes);
    pieces
}
