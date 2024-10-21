pub(crate) struct DropManifest {

}
pub struct DropChunk {
    permissions: usize,
    ids: Vec<String>,
    checksums: Vec<String>,
    lengths: Vec<String>
}

type Manifest = (DropManifest, DropChunk);