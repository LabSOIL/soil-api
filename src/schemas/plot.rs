use serde::Deserialize;

// List
#[derive(Deserialize, Default)]
pub struct FilterOptions {
    pub filter: Option<String>, // JSON-encoded filter
    pub range: Option<String>,  // range in the format "[0,24]"
    pub sort: Option<String>,   // sort in the format '["id", "ASC"]'
}

// // Create
// #[derive(Serialize, Deserialize, Debug)]
// pub struct CreateNoteSchema {
//     pub title: String,
//     pub content: String,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub is_published: Option<bool>,
// }

// // Update
// #[derive(Serialize, Deserialize, Debug)]
// pub struct UpdateNoteSchema {
//     pub title: Option<String>,
//     pub content: Option<String>,
//     pub is_published: Option<bool>,
// }
