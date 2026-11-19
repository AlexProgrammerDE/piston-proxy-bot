//! Configuration parsing for the command-registration utility.

#![warn(clippy::indexing_slicing, clippy::arithmetic_side_effects)]

use std::io::{self, ErrorKind};

use twilight_model::id::{Id, marker::ApplicationMarker};

/// Parses the `DISCORD_APPLICATION_ID` value from the local configuration.
///
/// # Errors
///
/// Returns an error if the value is not a nonzero decimal `u64`.
pub fn parse_application_id(value: &str) -> io::Result<Id<ApplicationMarker>> {
    value
        .parse::<u64>()
        .ok()
        .and_then(Id::new_checked)
        .ok_or_else(|| {
            io::Error::new(
                ErrorKind::InvalidData,
                "the DISCORD_APPLICATION_ID environment variable must be a nonzero integer",
            )
        })
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, ErrorKind};

    use twilight_model::id::Id;

    use super::parse_application_id;

    #[test]
    fn rejects_a_zero_application_id_from_the_development_file() {
        // Minimized fuzz input. It used to panic in `Id::new`.
        let (key, value) = dotenvy::from_read_iter(Cursor::new(b"DISCORD_APPLICATION_ID=0\n"))
            .next()
            .expect("the file should contain one variable")
            .expect("the variable should parse");
        assert_eq!(key, "DISCORD_APPLICATION_ID", "the key should be preserved");

        let error = parse_application_id(&value).expect_err("zero is not a Discord ID");
        assert_eq!(
            error.kind(),
            ErrorKind::InvalidData,
            "a zero ID should be invalid data"
        );
    }

    #[test]
    fn parses_decimal_application_ids() {
        assert_eq!(
            parse_application_id("1332702321397727303")
                .ok()
                .map(Id::get),
            Some(1_332_702_321_397_727_303),
            "a decimal ID should parse"
        );
        assert!(
            parse_application_id("-1").is_err_and(|error| error.kind() == ErrorKind::InvalidData),
            "a negative ID should be invalid data"
        );
    }
}
