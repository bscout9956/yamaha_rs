use std::{fs, path::{Path, PathBuf}};

use crate::utils::bytes_to_str;

#[allow(dead_code)]
#[derive(Debug)]
pub struct SoundBank {
    pub preset_name: String,
    pub preset_file_name: String,
    pub sample: Option<Sample>,
}

impl SoundBank {
    pub fn load_sample_params(&mut self, base_path: &PathBuf) {
        let sample_path: PathBuf = base_path.join("SBNK").join(&self.preset_file_name);
        
        let sample_data: Vec<u8> = fs::read(&sample_path).expect(&format!(
            "Failed to read sample data at {}",
            sample_path.to_string_lossy()
        ));

        let right_channel_name_data: &[u8] = &sample_data[0x88..0x97];
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
pub struct Sample {
    // other data should be here but there's no documentation lmao?
    pub sample_name: String,        //x32 to x41
    pub left_channel_name: String,  // x78 to x87
    pub right_channel_name: String, // x88 to x97
    stereo: bool,
}

#[derive(Debug)]
pub struct PatchData {
    pub patch_name: String,
    pub patch_file_name: String,
    pub sound_banks: Vec<SoundBank>,
}

impl PatchData {
    pub fn from_data(data: &[u8]) -> Self {
        let name: String = bytes_to_str(&data[1..16]);
        let file_name: String = bytes_to_str(&data[18..22]);

        PatchData {
            patch_name: name,
            patch_file_name: file_name,
            sound_banks: Vec::new(),
        }
    }

    pub fn load_soundbanks(&mut self, base_path: &Path) {
        let sbnk_path: PathBuf = base_path
            .join(&self.patch_file_name)
            .join("SBNK")
            .join("0000");
        
        let sbnk_data: Vec<u8> = fs::read(sbnk_path).expect("Failed to read sbnk data");
        
        self.sound_banks = sbnk_data
            .chunks_exact(32)
            .map(|chunk| SoundBank {
                preset_name: bytes_to_str(&chunk[1..16]),
                preset_file_name: bytes_to_str(&chunk[18..22]),
                sample: None,
            })
            .collect();
    }
}