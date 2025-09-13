use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    m_name: String,
    m_short_description: String,
    m_description: String,
    // mDevelopers
    // mPublishers
    m_icon_object_id: String,
    m_banner_object_id: String,
    m_cover_object_id: String,
    m_image_library_object_ids: Vec<String>,
    m_image_carousel_object_ids: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct User {
    id: String,
    username: String,
    admin: bool,
    display_name: String,
    profile_picture_object_id: String,
}
