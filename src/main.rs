// based on https://mybunnyhug.org/fileformats/yamahaaseries/

use std::fs::read;

// TODO: Separate structs where applicable

#[derive(Debug)]
struct PatchData {
    index: usize,
    patch_name: String,
    patch_file_name: String,
}

#[derive(Debug)]
struct SoundBankInfo {
    index: usize,
    preset_name: String,
    preset_file_name: String,
}

fn read_file_as_vec_u8(file_path: &str) -> Vec<u8> {
    // TODO: Should we propagate the error?
    let data: Vec<u8> =
        read(&file_path).expect(&format!("Failed to read bytes for file in {}", file_path));
    return data;
}

fn get_disk_name(data: &[u8]) -> String {
    let disk_name: String = data[0..16]
        .iter()
        .map(|&byte| byte as char)
        .collect::<String>()
        .to_string();

    return disk_name;
}

fn get_patch_name(data: &[u8], index: usize) -> String {
    let start: usize = (index * 16) + 1;
    let end: usize = start + 16; // always 16 bytes
    let patch_name: String = data[start..end]
        .iter()
        .map(|&byte| byte as char)
        .collect::<String>()
        .to_string();

    return patch_name;
}

fn get_patch_file_name(data: &[u8], index: usize) -> String {
    let start: usize = (index * 16) + 18;
    let end: usize = start + 4; // always 4 bytes
    let patch_fname: String = data[start..end]
        .iter()
        .map(|&byte| byte as char)
        .collect::<String>()
        .to_string();

    return patch_fname;
}

fn get_all_patch_data(data: &[u8], patch_count: usize) -> Vec<PatchData> {
    let mut i: usize = 0;
    let mut patches: Vec<PatchData> = Vec::new();

    // FIXME: We shouldn't count the last one, as it's just metadata (or garbage?)
    // TODO: Perhaps turn this into a singular function to get the entire PatchData struct as one?
    while i < (patch_count * 2) {
        patches.push(PatchData {
            index: (i / 2) + 1,
            patch_name: get_patch_name(&data, i),
            patch_file_name: get_patch_file_name(&data, i),
        });
        i += 2;
    }

    return patches;
}

fn read_soundbanks(directory: &str) {
    let metadata: Vec<SoundBankInfo> = read_soundbank_metadata(&directory);
    for bank in metadata {
        println!("{:?}", bank);
    }
}

fn get_soundbank_info(data: &[u8], index: usize) -> SoundBankInfo {
    let start_name: usize = (index * 16) + 1;
    let end_name: usize = start_name + 16;

    if end_name > data.len() {
        panic!(
            "Ending index {} greater than data length {}!",
            end_name,
            data.len()
        );
    }

    let bank_name: String = data[start_name..end_name]
        .iter()
        .map(|&byte| byte as char)
        .collect::<String>()
        .to_string();

    let bank_file_name: String = data[end_name + 1..end_name + 5]
        .iter()
        .map(|&byte| byte as char)
        .collect::<String>()
        .to_string();

    return SoundBankInfo {
        index: index,
        preset_name: bank_name,
        preset_file_name: bank_file_name,
    };
}

fn read_soundbank_metadata(directory: &str) -> Vec<SoundBankInfo> {
    let mut soundbank_info: Vec<SoundBankInfo> = Vec::new();
    let raw_soundbank_info: Vec<u8> = read_file_as_vec_u8(&format!("{}\\0000", directory));

    let mut i: usize = 0;
    // Length of file / block_size
    let block_count = raw_soundbank_info.len() / 32;

    while i < block_count {
        soundbank_info.push(get_soundbank_info(&raw_soundbank_info, i));
        i += 2;
    }

    soundbank_info
}

fn main() {
    let metadata_dir: &str = "V:\\24297D08";

    let bank_metadata_raw: Vec<u8> = read_file_as_vec_u8(&format!("{}\\0000", metadata_dir));
    let patch_count = bank_metadata_raw.len() / 32;

    println!("\n === Patch Data ===\n");
    let mut patch_data: Vec<PatchData> = get_all_patch_data(&bank_metadata_raw, patch_count);
    for data in &patch_data {
        println!("{:?}", data);
    }

    // The patch_data contains patch_data, but also includes the disc_metadata inside its file as the last entry.
    // Sometimes the name on that patch isn't set so we actually grab it from its own file. Which can't be dynamically discovered.
    // e.g: PatchData { index: 15, patch_name: "_DSKNAME\0\0\0\0\0\0\0\0", patch_file_name: "F015" }
    let disc_metadata: PatchData = patch_data
        .pop()
        .expect("Failed to grab disc metadata from patch_data.");
    
    let disc_metadata_raw: Vec<u8> = read_file_as_vec_u8(&format!(
        "{}\\{}",
        metadata_dir, disc_metadata.patch_file_name
    ));
    
    let disc_name: String = get_disk_name(&disc_metadata_raw);
    println!("Disc Name from File: {} | Disc Name from Patch_Data: {}", disc_name, disc_metadata.patch_name);

    let patch_names = patch_data.iter().map(|p| &p.patch_file_name);
    
    println!("\n=== SoundBank Info ===\n");
    for dir_name in patch_names {
        let base_path: String = format!("{}\\{}", metadata_dir, dir_name);
        read_soundbanks(&format!("{}\\SBNK", base_path));
    }
}
