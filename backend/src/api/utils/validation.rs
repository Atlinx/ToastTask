use once_cell::sync::Lazy;
use regex::Regex;
use validator::ValidationError;

use super::Patch;

pub fn validate_patch_color(color: &Patch<String>) -> Result<(), ValidationError> {
    match color {
        Patch::Missing | Patch::Null => Ok(()),
        Patch::Value(ref color_str) => validate_color(color_str),
    }
}

pub fn validate_color(color: &str) -> Result<(), ValidationError> {
    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^#([A-Fa-f0-9]{6})$").unwrap());
    if REGEX.is_match(color) {
        return Ok(());
    }
    Err(ValidationError::new(
        "Color must follow the 6 digit hex format (#ffffff).",
    ))
}
