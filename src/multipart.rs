use serde::Serialize;

const BOUNDARY_PREFIX: &str = "----piston-proxy-bot-boundary";

pub struct MultipartFile<'a> {
    pub field_name: &'a str,
    pub filename: &'a str,
    pub content_type: &'a str,
    pub data: &'a [u8],
}

pub struct EncodedMultipart {
    pub body: Vec<u8>,
    pub content_type: String,
}

pub fn encode<T: Serialize>(
    payload: &T,
    file: MultipartFile<'_>,
) -> serde_json::Result<EncodedMultipart> {
    let payload = serde_json::to_vec(payload)?;
    let boundary = select_boundary(&payload, file.data);
    let mut body = Vec::with_capacity(payload.len() + file.data.len() + 512);

    append_boundary(&mut body, &boundary);
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"payload_json\"\r\n");
    body.extend_from_slice(b"Content-Type: application/json\r\n\r\n");
    body.extend_from_slice(&payload);
    body.extend_from_slice(b"\r\n");

    append_boundary(&mut body, &boundary);
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"");
    body.extend_from_slice(file.field_name.as_bytes());
    body.extend_from_slice(b"\"; filename=\"");
    body.extend_from_slice(file.filename.as_bytes());
    body.extend_from_slice(b"\"\r\nContent-Type: ");
    body.extend_from_slice(file.content_type.as_bytes());
    body.extend_from_slice(b"\r\n\r\n");
    body.extend_from_slice(file.data);
    body.extend_from_slice(b"\r\n--");
    body.extend_from_slice(boundary.as_bytes());
    body.extend_from_slice(b"--\r\n");

    Ok(EncodedMultipart {
        content_type: format!("multipart/form-data; boundary={boundary}"),
        body,
    })
}

fn append_boundary(body: &mut Vec<u8>, boundary: &str) {
    body.extend_from_slice(b"--");
    body.extend_from_slice(boundary.as_bytes());
    body.extend_from_slice(b"\r\n");
}

fn select_boundary(payload: &[u8], file: &[u8]) -> String {
    for suffix in 0_u64.. {
        let boundary = format!("{BOUNDARY_PREFIX}-{suffix:x}");
        if !contains_bytes(payload, boundary.as_bytes())
            && !contains_bytes(file, boundary.as_bytes())
        {
            return boundary;
        }
    }

    unreachable!("the multipart boundary suffix space is exhausted")
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::{BOUNDARY_PREFIX, MultipartFile, encode};

    #[derive(Serialize)]
    struct Payload<'a> {
        kind: u8,
        content: &'a str,
    }

    #[test]
    fn encodes_json_and_file_parts_without_changing_file_bytes() {
        let file_data = b"first\r\nsecond\0third";
        let encoded = encode(
            &Payload {
                kind: 4,
                content: "proxy response",
            },
            MultipartFile {
                field_name: "files[0]",
                filename: "proxies.txt",
                content_type: "text/plain",
                data: file_data,
            },
        )
        .expect("the multipart response should encode");

        let boundary = encoded
            .content_type
            .strip_prefix("multipart/form-data; boundary=")
            .expect("the content type should expose its boundary");
        let delimiter = format!("--{boundary}");
        let parts = encoded
            .body
            .split(|byte| *byte == b'\n')
            .collect::<Vec<_>>();

        assert_eq!(
            encoded
                .body
                .windows(file_data.len())
                .filter(|window| *window == file_data)
                .count(),
            1
        );
        assert_eq!(
            encoded
                .body
                .windows(delimiter.len())
                .filter(|window| *window == delimiter.as_bytes())
                .count(),
            3
        );
        assert!(parts.last().is_some_and(|line| line.is_empty()));
    }

    #[test]
    fn changes_the_boundary_when_content_contains_the_first_candidate() {
        let colliding_data = format!("{BOUNDARY_PREFIX}-0").into_bytes();
        let encoded = encode(
            &Payload {
                kind: 4,
                content: "proxy response",
            },
            MultipartFile {
                field_name: "files[0]",
                filename: "proxies.txt",
                content_type: "text/plain",
                data: &colliding_data,
            },
        )
        .expect("the multipart response should encode");

        assert_eq!(
            encoded.content_type,
            format!("multipart/form-data; boundary={BOUNDARY_PREFIX}-1")
        );
    }
}
