use bitcode::{Decode, Encode};
use database::models::Game;
use serde::{Deserialize, Serialize};

pub type Collections = Vec<Collection>;

#[derive(Serialize, Deserialize, Debug, Clone, Default, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    id: String,
    name: String,
    is_default: bool,
    user_id: String,
    entries: Vec<CollectionObject>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct CollectionObject {
    collection_id: String,
    game_id: String,
    game: Game,
}
