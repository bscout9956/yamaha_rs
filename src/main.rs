// based on https://mybunnyhug.org/fileformats/yamahaaseries/

use std::{
    fs::{self},
    path::Path,
    path::PathBuf,
};

#[allow(dead_code)]
#[derive(Debug)]
struct SoundBank {
    preset_name: String,
    preset_file_name: String,
    sample: Option<Sample>,
}

impl SoundBank {
    fn load_sample_params(&mut self, base_path: &PathBuf) {
        let sample_path: PathBuf = base_path.join("SBNK").join(&self.preset_file_name);
        
        let sample_data = fs::read(&sample_path).expect(&format!(
            "Failed to read sample data at {}",
            sample_path.to_string_lossy()
        ));

        let right_channel_name_data = &sample_data[0x88..0x97];
        // If all characters are \0, that means it's all empty, thus it's Mono (not Stereo)
        let is_stereo = !right_channel_name_data.iter().all(|&f| f as char == '\0');

        self.sample = Some(Sample {
            sample_name : bytes_to_str(&sample_data[0x32..0x41]),
            left_channel_name : bytes_to_str(&sample_data[0x78..0x87]),
            // Use empty string if it's stereo, let's not waste memory
            right_channel_name : if is_stereo { bytes_to_str(right_channel_name_data)} else { String::new() },
            stereo: is_stereo,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct Sample {
    // other data should be here but there's no documentation lmao?
    sample_name: String,        //x32 to x41
    left_channel_name: String,  // x78 to x87
    right_channel_name: String, // x88 to x97
}

#[derive(Debug)]
struct PatchData {
    patch_name: String,
    patch_file_name: String,
    sound_banks: Vec<SoundBank>,
}

fn bytes_to_str(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

impl PatchData {
    fn from_data(data: &[u8]) -> Self {
        let name = bytes_to_str(&data[1..16]);
        let file_name = bytes_to_str(&data[18..22]);

        PatchData {
            patch_name: name,
            patch_file_name: file_name,
            sound_banks: Vec::new(),
        }
    }

    fn load_soundbanks(&mut self, base_path: &Path) {
        let sbnk_path: PathBuf = base_path
            .join(&self.patch_file_name)
            .join("SBNK")
            .join("0000");
        let sbnk_data: Vec<u8> = fs::read(sbnk_path).expect("Failed to read sbnk data");
        self.sound_banks = sbnk_data
            .chunks_exact(32)
            .map(|chunk| SoundBank {
                preset_file_name: bytes_to_str(&chunk[1..16]),
                preset_name: bytes_to_str(&chunk[18..22]),
                sample: None
            })
            .collect();
    }
}

fn main() {
    let disc_dir: &Path = Path::new("V:\\24297D08");
    let index_path = disc_dir.join("0000");

    let index_file = fs::read(index_path).expect("Failed to read index_file");

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
    let disc_name = bytes_to_str(&disc_metadata_contents[0..16]);

    for patch in &mut patch_data {
        patch.load_soundbanks(disc_dir);
    }

    println!(
        "Disc Name from File: {disc_name} | Disc Name from Patch_Data: {}",
        disc_metadata.patch_name
    );

    println!("{:#?}", patch_data);
}
