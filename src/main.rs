// based on https://mybunnyhug.org/fileformats/yamahaaseries/

pub mod types;
pub mod utils;

use std::{
    fs::{self},
    path::Path,
    path::PathBuf,
};

use crate::{types::PatchData, utils::bytes_to_str};


fn main() {
    let disc_dir: &Path = Path::new("V:\\24297D08");
    let index_path: PathBuf = disc_dir.join("0000");

    let index_file: Vec<u8> = fs::read(index_path).expect("Failed to read index_file");

    let mut patch_data: Vec<PatchData> = index_file
        .chunks_exact(32)
        .map(PatchData::from_data)
        .collect();

    println!("{:?}", patch_data);

    // The patch_data contains patch_data, but also includes the disc_metadata inside its file as the last entry.
    // Sometimes the name on that patch isn't set so we actually grab it from its own file. Which can't be dynamically discovered.
    // e.g: PatchData { index: 15, patch_name: "_DSKNAME\0\0\0\0\0\0\0\0", patch_file_name: "F015" }
    let disc_metadata: PatchData = patch_data
        .pop()
        .expect("Failed to grab disc metadata from patch_data.");

    let disc_metadata_contents = fs::read(disc_dir.join(disc_metadata.patch_file_name))
        .expect("Failed to read disc metadata content");
    let disc_name: &String = &disc_metadata_contents[0..16].to_lossy_string();

    for patch in &mut patch_data {
        patch.load_soundbanks(disc_dir);
    }

    println!(
        "Disc Name from File: {disc_name} | Disc Name from Patch_Data: {}",
        disc_metadata.patch_name
    );

    for patch in &mut patch_data {
        for sbnk in &mut patch.sound_banks {
            sbnk.load_sample_params(&disc_dir.join(&patch.patch_file_name));
        }
    }

    println!("{:#?}", patch_data);
}
