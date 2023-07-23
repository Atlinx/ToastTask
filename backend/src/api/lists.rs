crate::api_tree_crud! {
    model_table: "lists",
    model_fields: { title, description, color, parent_id },
    get_model: GetModel,
    post_input: PostInput,
    patch_input: PatchInput
}

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
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
    pub parent_id: Patch<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PostInput {
    pub title: String,
    pub description: Option<String>,
    #[validate(custom = "validate_color")]
    pub color: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub color: String,
    pub parent_id: Option<Uuid>,
    pub child_ids: Vec<Uuid>,
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
