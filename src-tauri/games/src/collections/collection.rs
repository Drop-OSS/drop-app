use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::library::Game;

pub type Collections = Vec<Collection>;

#[derive(Serialize, Deserialize, Debug, Clone, Default, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    id: String,
    name: String,
    is_default: bool,
    user_id: String,
    pub entries: Vec<CollectionObject>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct CollectionObject {
    pub collection_id: String,
    pub game_id: String,
    pub game: Game,
}
