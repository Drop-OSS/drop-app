use ciborium::from_reader;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    io::{self, BufReader, Error, Seek},
    os::unix::fs::PermissionsExt,
    path::Path,
};
use tauri::Runtime;
use xz2::bufread::XzDecoder;

#[derive(Deserialize)]
struct ManifestChunk {
    uuid: String,
    index: i64,
}

#[derive(Deserialize)]
struct ManifestRecord {
    chunks: Vec<ManifestChunk>,
    permissions: u32,
}

#[derive(Deserialize)]
struct Manifest {
    record: HashMap<String, ManifestRecord>,
}

fn generate_permissions(permissions: Vec<bool>) -> u32 {
    // Base 8
    let mut perms: u32 = 0;

    // Read
    if permissions[0] {
        perms += 4;
    }

    // Write
    if permissions[1] {
        perms += 2;
    }

    // Execute
    if permissions[2] {
        perms += 1;
    }

    perms *= 8 * 8;
    perms += 4 * 8 + 4;

    return perms;
}

pub fn unpack() -> Result<(), Error> {
    let chunk_size: u64 = 1024 * 1024 * 16;

    let input = Path::new("/home/decduck/Dev/droplet-output");
    let output = Path::new("/home/decduck/Dev/droplet-rebuilt");

    let manifest_path = input.join("manifest.drop");
    let manifest_file_handle = File::open(manifest_path).unwrap();

    let manifest: Manifest = from_reader(manifest_file_handle).unwrap();
    manifest.record.into_par_iter().for_each(|(key, value)| {
        let file = output.join(key.clone());
        create_dir_all(file.parent().unwrap()).unwrap();
        let mut file_handle = File::create(file).unwrap();

        #[cfg(unix)]
        {
            let mut file_permissions = file_handle.metadata().unwrap().permissions();
            file_permissions.set_mode(value.permissions);
            file_handle.set_permissions(file_permissions).unwrap();
        }

        for chunk in value.chunks {
            let chunk_path = input.join(chunk.uuid + ".bin");
            let chunk_handle = File::open(chunk_path).unwrap();

            let chunk_reader = BufReader::new(chunk_handle);
            let mut decompressor = XzDecoder::new(chunk_reader);

            let offset = u64::try_from(chunk.index).unwrap() * chunk_size;
            file_handle.seek(io::SeekFrom::Start(offset)).unwrap();

            io::copy(&mut decompressor, &mut file_handle).unwrap();
        }
    });

    return Ok(());
}
