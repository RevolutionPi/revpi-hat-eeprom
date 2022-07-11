// SPDX-License-Identifier: GPL-2.0-or-later
// SPDX-FileCopyrightText: Copyright 2022 KUNBUS GmbH

use crate::RevPiError;

/// Validate a string to be max 255 bytes long.
///
/// Check if a given string can fit into a 255 byte buffer.
///
/// # Examples
/// ```
/// assert!(validate_string_max255("foo bar").is_ok());
/// ```
pub fn validate_string_max255(src: &str) -> Result<(), RevPiError> {
    if src.as_bytes().len() > 255 {
        return Err(RevPiError::ValidationError("String to long (> 255 bytes)".to_string()).into());
    }
    Ok(())
}

#[test]
fn test_validate_string_max255() {
    assert!(validate_string_max255("foo bar").is_ok());
    assert!(
        validate_string_max255(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Duis ut diam quam nulla porttitor massa id neque. Facilisis volutpat est velit egestas dui id ornare arcu. Hac habitasse platea dict"
        ).is_err()
    )
}
