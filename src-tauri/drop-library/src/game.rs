use crate::libraries::LibraryProviderIdentifier;

pub struct LibraryGamePreview {
    pub library: LibraryProviderIdentifier,
    pub internal_id: String,
    pub name: String,
    pub short_description: String,
    pub icon: String,
}

pub struct LibraryGame {
    pub library: LibraryProviderIdentifier,
    pub internal_id: String,
    pub name: String,
    pub short_description: String,
    pub md_description: String,
    pub icon: String,
}

impl From<LibraryGame> for LibraryGamePreview {
    fn from(value: LibraryGame) -> Self {
        LibraryGamePreview {
            library: value.library,
            internal_id: value.internal_id,
            name: value.name,
            short_description: value.short_description,
            icon: value.icon,
        }
    }
}
