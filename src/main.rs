// based on https://mybunnyhug.org/fileformats/yamahaaseries/

use std::fs::read;

// TODO: Separate structs where applicable

#[derive(Debug)]
struct YamahaDiscData {
    patch_data: PatchDataInfo,
}

#[derive(Debug)]
struct PatchData {
    patch_name: String,
    patch_file_name: String,
    sound_banks: Vec<SoundBank>,
}

#[derive(Debug)]
struct SoundBank {
    preset_name: String,
    preset_file_name: String,
}

#[allow(dead_code)]
#[derive(Debug)]
struct PatchDataInfo {
    index: usize,
    patch_name: String,
    patch_file_name: String,
}

#[allow(dead_code)]
#[derive(Debug)]
struct SoundBankInfo {
    index: usize,
    preset_name: String,
    preset_file_name: String,
}

impl SoundBankInfo {
    pub fn to_sound_bank(self) -> SoundBank {
        return SoundBank {
            preset_name: self.preset_name,
            preset_file_name: self.preset_file_name
        }
    }
}


// Reads any binary file as a Vector of u8s, given a file_path
fn read_file_as_vec_u8(file_path: &str) -> Vec<u8> {
    // TODO: Should we propagate the error?
    let data: Vec<u8> =
        read(&file_path).expect(&format!("Failed to read bytes for file in {}", file_path));
    return data;
}

// Gets the disk name from the data
fn get_disk_name(data: &[u8]) -> String {
    let disk_name: String = data[0..16]
        .iter()
        .map(|&byte| byte as char)
        .collect::<String>()
        .to_string();

    return disk_name;
}

// Returns a singular PatchDataInfo from the raw data at a given index
fn get_patch_data(data: &[u8], index: usize) -> PatchDataInfo {
    let start: usize = (index * 16) + 1;
    let end: usize = start + 16;
    PatchDataInfo {
        index: index,
        patch_name: {
            data[start..end]
                .iter()
                .map(|&byte| byte as char)
                .collect::<String>()
                .to_string()
        },
        patch_file_name: {
            data[end + 1..end + 5]
                .iter()
                .map(|&byte| byte as char)
                .collect::<String>()
                .to_string()
        },
    }
}

// Returns a Vector of PatchDataInfo, these do not contain SoundBanks inside
fn get_all_patch_data(data: &[u8], patch_count: usize) -> Vec<PatchDataInfo> {
    let mut i: usize = 0;
    let mut patches: Vec<PatchDataInfo> = Vec::new();

    while i < (patch_count * 2) {
        patches.push(get_patch_data(data, i));
        i += 2;
    }

    return patches;
}

// Returns SoundBankInfo containing Bank Name, Bank File Name (F???) and Index
fn get_soundbank_info(data: &[u8], index: usize) -> SoundBankInfo {
    let start_name: usize = (index * 16) + 1;
    let end_name: usize = start_name + 16;

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
        index: index/2,
        preset_name: bank_name,
        preset_file_name: bank_file_name,
    };
}

fn read_soundbank_metadata(directory: &str) -> Vec<SoundBankInfo> {
    let mut soundbank_info: Vec<SoundBankInfo> = Vec::new();
    println!("Reading from {directory}");
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
    let disc_dir: &str = "V:\\24297D08";

    let bank_metadata_raw: Vec<u8> = read_file_as_vec_u8(&format!("{}\\0000", disc_dir));
    let patch_count = bank_metadata_raw.len() / 32;

    println!("\n === Patch Data ===\n");
    let mut patch_data: Vec<PatchDataInfo> = get_all_patch_data(&bank_metadata_raw, patch_count);
    let mut full_patch_data: Vec<PatchData> = Vec::new();

    // The patch_data contains patch_data, but also includes the disc_metadata inside its file as the last entry.
    // Sometimes the name on that patch isn't set so we actually grab it from its own file. Which can't be dynamically discovered.
    // e.g: PatchData { index: 15, patch_name: "_DSKNAME\0\0\0\0\0\0\0\0", patch_file_name: "F015" }
    let disc_metadata: PatchDataInfo = patch_data
        .pop()
        .expect("Failed to grab disc metadata from patch_data.");

    let disc_metadata_raw: Vec<u8> =
        read_file_as_vec_u8(&format!("{disc_dir}\\{}", disc_metadata.patch_file_name));

    let disc_name: String = get_disk_name(&disc_metadata_raw);
    println!(
        "Disc Name from File: {disc_name} | Disc Name from Patch_Data: {}",
        disc_metadata.patch_name
    );

    for data in &patch_data {
        let sound_bank_info: Vec<SoundBankInfo> = read_soundbank_metadata(&format!("{disc_dir}\\{}\\SBNK", data.patch_file_name));
        let sound_banks: Vec<SoundBank> = sound_bank_info.into_iter().map(|sbi| sbi.to_sound_bank()).collect();

        full_patch_data.push(PatchData {
            patch_name: data.patch_name.clone(),
            patch_file_name: data.patch_file_name.clone(),
            sound_banks: sound_banks,
        });

        println!("{:#?}", full_patch_data);
    }
}
