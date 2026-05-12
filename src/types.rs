use std::{
    fs,
    path::{Path, PathBuf},
};

use wavers::{Samples, WaversError};

use crate::utils::ByteUtils;

#[allow(dead_code)]
#[derive(Debug)]
pub struct SoundBank {
    pub preset_name: String,
    pub preset_file_name: String,
    pub sample: Option<SampleInfo>,
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
        let is_stereo: bool = !right_channel_name_data.iter().all(|&f| f as char == '\0');

        self.sample = Some(SampleInfo {
            sample_name: sample_data[0x32..0x41].to_lossy_string(),
            left_channel_name: sample_data[0x78..0x87].to_lossy_string(),
            // Use empty string if it's stereo, let's not waste memory
            right_channel_name: if is_stereo {
                right_channel_name_data.to_lossy_string()
            } else {
                String::new()
            },
            stereo: is_stereo,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct SampleInfo {
    // other data should be here but there's no documentation lmao?
    pub sample_name: String,        //x32 to x41
    pub left_channel_name: String,  // x78 to x87
    pub right_channel_name: String, // x88 to x97
    pub stereo: bool,
}

#[derive(Debug)]
pub struct PatchData {
    pub patch_name: String,
    pub patch_file_name: String,
    pub sound_banks: Vec<SoundBank>,
    pub sample_data: Vec<Sample>,
}

impl PatchData {
    pub fn from_data(data: &[u8]) -> Self {
        let name: String = data[1..16].to_lossy_string();
        let file_name: String = data[18..22].to_lossy_string();

        PatchData {
            patch_name: name,
            patch_file_name: file_name,
            sound_banks: Vec::new(),
            sample_data: Vec::new(),
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
                preset_name: chunk[1..16].to_lossy_string(),
                preset_file_name: chunk[18..22].to_lossy_string(),
                sample: None,
            })
            .collect();
    }

    pub fn load_smpl_metadata(&mut self, base_path: &Path) {
        let smpl_path: PathBuf = base_path
            .join(&self.patch_file_name)
            .join("SMPL")
            .join("0000");

        let smpl_data: Vec<u8> = fs::read(smpl_path).expect("Failed to read smpl data");

        self.sample_data = smpl_data
            .chunks_exact(32)
            .map(|chunk| Sample {
                waveform_name: chunk[1..16].to_lossy_string(),
                waveform_file_name: chunk[18..22].to_lossy_string(),
                waveform: None,
            })
            .collect();
    }
}

#[derive(Debug)]
pub struct WaveForm {
    // ALL DATA IS BIG ENDIAN!
    pub parameters: Vec<u8>, // they go from 0x00 to 0x1FF
    pub sample_rate: u16,    // 0x28 to 0x29, two bytes, samples/sec
    pub raw_waveform_data: Vec<i16>, // starts at 0x200, goes until the end of the file, should be signed 16bit
}

impl WaveForm {
    pub fn return_waveform_as_f32(&self) -> Vec<f32> {
        self.raw_waveform_data
            .iter()
            .map(|&sample| sample as f32 / 32768.0)
            .collect()
    }
}

#[derive(Debug)]
pub struct Sample {
    pub waveform_name: String,
    pub waveform_file_name: String,
    pub waveform: Option<WaveForm>,
}

impl Sample {
    pub fn load_waveform(&mut self, base_path: &PathBuf) {
        let waveform_path: PathBuf = base_path.join("SMPL").join(&self.waveform_file_name);

        let waveform_data: Vec<u8> = fs::read(&waveform_path).expect(&format!(
            "Failed to read waveform data at {}",
            waveform_path.to_string_lossy()
        ));

        let samples = waveform_data[0x200..]
            .chunks_exact(2)
            .map(|chunk| i16::from_be_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<i16>>();

        self.waveform = Some(WaveForm {
            parameters: waveform_data[0x00..0x200].to_vec(),
            sample_rate: waveform_data[0x28..0x2A].from_be_to_u16(),
            raw_waveform_data: samples,
        });

        println!(
            "Loaded waveform for sample {} with sample rate {} Hz",
            self.waveform_name,
            self.waveform.as_ref().unwrap().sample_rate
        );
    }
    
    pub fn save_waveform_as_wav(&self, base_path: &PathBuf) -> Result<(), WaversError> {
        let final_wav_path: PathBuf = base_path.join("SMPL").join(&self.waveform_file_name).with_added_extension("wav");
        println!("Using path: {}", final_wav_path.to_string_lossy());

        if self.waveform.is_some() {
            let waveform: &WaveForm = self.waveform.as_ref().unwrap();
            let waveform_f32: Vec<f32> = waveform.return_waveform_as_f32();

            let (samples, sample_rate): (Samples<f32>,i32) = (waveform_f32.into(), waveform.sample_rate.into());
            let n_channels = 1;

            println!("Writing audio to {}", final_wav_path.to_string_lossy());
            wavers::write(final_wav_path, &samples, sample_rate, n_channels)?;
        } else {
            println!("No waveform found for {}", final_wav_path.to_string_lossy());
        }

        Ok(())
    }
}
