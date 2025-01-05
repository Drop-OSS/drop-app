use std::collections::HashMap;

use crate::download_manager::downloadable::Downloadable;

pub struct Registry<T: Downloadable> {
    tools: HashMap<String, T>,
}
