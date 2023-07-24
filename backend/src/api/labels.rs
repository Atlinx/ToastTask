crate::api_crud! {
    model_table: "labels",
    model_fields: { title, description, color },
    get_model: crate::models::label::LabelModel,
    post_input: PostInput,
    patch_input: PatchInput
}

use super::utils::validation::{validate_color, validate_patch_color};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

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
