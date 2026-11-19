//! Parses a `.dev.vars` file the way the registration utility reads it, then
//! parses `DISCORD_APPLICATION_ID`. Parsing must never panic, and an accepted
//! ID must be the decimal value written in the file.

#![no_main]

use std::io::Cursor;

use libfuzzer_sys::fuzz_target;
use piston_proxy_register::parse_application_id;

fuzz_target!(|data: &[u8]| {
    // `from_read_iter` uses the same parser as `from_filename` in the
    // registration utility, without changing the process environment.
    for item in dotenvy::from_read_iter(Cursor::new(data)) {
        let Ok((key, value)) = item else {
            return;
        };
        if key != "DISCORD_APPLICATION_ID" {
            continue;
        }
        if let Ok(id) = parse_application_id(&value) {
            assert_eq!(
                value.parse::<u64>().ok(),
                Some(id.get()),
                "the parsed ID differs from the configured value"
            );
        }
    }
});
