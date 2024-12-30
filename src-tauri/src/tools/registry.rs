use std::collections::HashMap;

use super::external_component::ExternalComponent;

pub struct Registry<T: ExternalComponent> {
    tools: HashMap<String, T>
}
