crate::api_crud! {
  model_table: "lists",
  model_fields: { title, description, color },
  get_model: crate::models::list::ListModel,
  post_input: PostInput,
  patch_input: PatchInput
}

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use super::utils::Patch;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PatchInput {
    #[serde(default)]
    pub title: Patch<String>,
    #[serde(default)]
    pub description: Patch<String>,
    #[serde(default)]
    #[validate(custom = "validate_patch_color")]
    pub color: Patch<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PostInput {
    pub title: String,
    pub description: Option<String>,
    #[validate(custom = "validate_color")]
    pub color: String,
}

fn validate_patch_color(color: &Patch<String>) -> Result<(), ValidationError> {
    match color {
        Patch::Missing | Patch::Null => Ok(()),
        Patch::Value(ref color_str) => validate_color(color_str),
    }
}

fn validate_color(color: &str) -> Result<(), ValidationError> {
    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^#([A-Fa-f0-9]{6})$").unwrap());
    if REGEX.is_match(color) {
        return Ok(());
    }
    Err(ValidationError::new(
        "Color must follow the 6 digit hex format (#ffffff).",
    ))
}
