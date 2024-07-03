use glam::Vec3;
use libfmod::ffi::FMOD_DSP_STATE;
use libfmod::{Channel, ChannelGroup, DspDescription, DspParameterDesc, Sound, System, Vector};
use libfmod::{Dsp, DspCallbackType, DspState};
use std::ffi::c_void;
use std::f32::consts::PI;
use std::ptr;
use std::{collections::HashMap, sync::atomic::AtomicBool};

static mut SINE_WAVE_STATE: Option<SineWaveState> = None;

struct SineWaveState {
    frequency: f32,
    phase: f32,
    sample_rate: f32,
}

extern "C" fn dsp_callback(
    dsp_state: *mut FMOD_DSP_STATE,
    inbuffer: *mut f32,
    outbuffer: *mut f32,
    length: u32,
    inchannels: i32,
    outchannels: *mut i32,
) -> i32 {
    unsafe {
        if let Some(state) = &mut SINE_WAVE_STATE {
            let sample_rate = state.sample_rate;
            let frequency = state.frequency;
            let mut phase = state.phase;

            let samples = length * *outchannels as u32;
            for i in 0..samples {
                let sample_index = i as usize;
                let sample = (2.0 * PI * frequency * phase / sample_rate).sin();
                *outbuffer.add(sample_index) = sample as f32;

                phase += 1.0;
                if phase >= sample_rate {
                    phase -= sample_rate;
                }
            }

            state.phase = phase;
        }

        libfmod::FmodResult::Ok as i32
    }
}
pub struct AudioPlayer {
    system: System,
    sounds: HashMap<&'static str, Sound>,
    series: HashMap<&'static str, Vec<&'static str>>,
    series_indexes: HashMap<&'static str, usize>,
    channels: HashMap<&'static str, Vec<Channel>>, // Store channels to manage sound playback
    head_group: ChannelGroup,
    spatial_group: ChannelGroup,
    dsp_group: ChannelGroup,
    voicechannelsplaying: AtomicBool,
}

impl AudioPlayer {

    fn create_channel_groups(system: &libfmod::System) -> (libfmod::ChannelGroup, libfmod::ChannelGroup, ChannelGroup) {
        let master_group = system.get_master_channel_group().unwrap();
        let dsp_group = system.create_channel_group(Some(String::from("dsp"))).unwrap();
        let head_group = system.create_channel_group(Some(String::from("head"))).unwrap();
        let spatial_group = system.create_channel_group(Some(String::from("spatial"))).unwrap();
        
        master_group.add_group(head_group, false).unwrap();
        master_group.add_group(spatial_group, false).unwrap();
        
        (head_group, spatial_group, dsp_group)
    }


    pub fn new() -> Result<Self, libfmod::Error> {
        let system = System::create().unwrap();
        system.init(128, libfmod::Init::NORMAL, None)?; // Initialize system with 32 channels
        let (head_group, spatial_group, dsp_group) = Self::create_channel_groups(&system);

        let name: [i8; 32] = [b'f'.try_into().unwrap(), b'u'.try_into().unwrap(), b's'.try_into().unwrap(), b't'.try_into().unwrap(), b'o'.try_into().unwrap(), b'm'.try_into().unwrap(), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // DSP name
    
        // let dsp = system
        //     .create_dsp(libfmod::DspDescription {
        //         pluginsdkversion: 110,
        //         name,
        //         version: 0x00010000, // Version number
        //         numinputbuffers: 1,  // Number of input buffers
        //         numoutputbuffers: 1, // Number of output buffers
        //         read: Some(dsp_callback), // Set the DSP callback function
        //         // Set other callbacks to None or default as needed
        //         create: None,
        //         release: None,
        //         reset: None,
        //         process: None,
        //         setposition: None,
        //         paramdesc: Vec::new(),
        //         setparameterfloat: None,
        //         setparameterint: None,
        //         setparameterbool: None,
        //         setparameterdata: None,
        //         getparameterfloat: None,
        //         getparameterint: None,
        //         getparameterbool: None,
        //         getparameterdata: None,
        //         shouldiprocess: None,
        //         userdata: std::ptr::null_mut(),
        //         sys_register: None,
        //         sys_deregister: None,
        //         sys_mix: None,
        //     })
        //     .expect("Failed to create DSP");

        // // Initialize the sine wave state
        // unsafe {
        //     SINE_WAVE_STATE = Some(SineWaveState {
        //         frequency: 440.0, // A4 note
        //         phase: 0.0,
        //         sample_rate: 44100.0,
        //     });
        // }


        // dsp_group.add_dsp(0, dsp).expect("Heyoo! It didn't work!");

        


        Ok(AudioPlayer {
            system,
            sounds: HashMap::new(),
            series: HashMap::new(),
            series_indexes: HashMap::new(),
            channels: HashMap::new(),
            head_group,
            spatial_group,
            dsp_group,
            voicechannelsplaying: AtomicBool::new(true),
        })
    }
    pub fn update(&mut self) {
        self.system.update();
        self.cleanup_channels();
    }

    pub fn preload(
        &mut self,
        id: &'static str,
        file_path: &'static str,
    ) -> Result<(), libfmod::Error> {
        let sound = self
            .system
            .create_sound(file_path, libfmod::Mode::FMOD_3D, None).unwrap();

        self.sounds.insert(id, sound);
        Ok(())
    }

    pub fn preload_series(&mut self, series_name: &'static str, paths: Vec<&'static str>) {
        self.series.insert(series_name, paths.clone());
        self.series_indexes.insert(series_name, 0);
        for path in paths {
            if !self.sounds.contains_key(path) {
                self.preload(path, path).unwrap(); // Simplified error handling
            }
        }
    }

    pub fn play_next_in_series(
        &mut self,
        series_name: &'static str,
        pos: &Vec3,
        vel: &Vec3,
        vol: f32
    ) -> Result<(), libfmod::Error> {
        let index = *self.series_indexes.get(series_name).unwrap_or(&0);
        let file_path = self.series.get(series_name).unwrap()[index];

        self.play(file_path, pos, vel, vol);
        let next_index = (index + 1) % self.series.get(series_name).unwrap().len();
        self.series_indexes.insert(series_name, next_index);

        Ok(())
    }

    pub fn play_in_head(&mut self, id: &'static str) {
        if let Some(sound) = self.sounds.get(id) {
            let channel = self.system.play_sound(*sound, Some(self.head_group), false).unwrap();
            channel.set_mode(libfmod::Mode::FMOD_2D).unwrap();  // Ensure the sound is 2D
            channel.set_volume(0.2).unwrap();  // Ensure the volume is set
            self.channels
                .entry(id)
                .or_insert_with(Vec::new)
                .push(channel);
        } else {
            self.preload(id, id).unwrap();
            if let Some(sound) = self.sounds.get(id) {
                let channel = self.system.play_sound(*sound, Some(self.head_group), false).unwrap();
                channel.set_mode(libfmod::Mode::FMOD_2D).unwrap();  // Ensure the sound is 2D
                channel.set_volume(0.2).unwrap();  // Ensure the volume is set
                self.channels
                    .entry(id)
                    .or_insert_with(Vec::new)
                    .push(channel);
            }
        }
    }

    pub fn play(&mut self, id: &'static str, pos: &Vec3, vel: &Vec3, vol: f32) {
        if let Some(sound) = self.sounds.get(id) {
            let channel = self.system.play_sound(*sound, Some(self.spatial_group), false).unwrap();
            channel.set_mode(libfmod::Mode::FMOD_3D).unwrap();  // Ensure the sound is 3D
            channel.set_3d_attributes(
                Some(Vector::new(pos.x, pos.y, pos.z)),
                Some(Vector::new(vel.x, vel.y, vel.z)),
            ).unwrap();
            channel.set_volume(vol).unwrap();
            channel.set_3d_min_max_distance(1.0, 30.0).unwrap();  // Set min and max distances
            self.channels
                .entry(id)
                .or_insert_with(Vec::new)
                .push(channel);
        } else {
            self.preload(id, id).unwrap();
            if let Some(sound) = self.sounds.get(id) {
                let channel = self.system.play_sound(*sound, Some(self.spatial_group), false).unwrap();
                channel.set_mode(libfmod::Mode::FMOD_3D).unwrap();  // Ensure the sound is 3D
                channel.set_3d_attributes(
                    Some(Vector::new(pos.x, pos.y, pos.z)),
                    Some(Vector::new(vel.x, vel.y, vel.z)),
                ).unwrap();
                channel.set_volume(vol).unwrap();
                channel.set_3d_min_max_distance(1.0, 30.0).unwrap();  // Set min and max distances
                self.channels
                    .entry(id)
                    .or_insert_with(Vec::new)
                    .push(channel);
            }
        }
    }

    pub fn cleanup_channels(&mut self) {
        for (name, channels) in &mut self.channels {
            // Collect the indices of channels that are not playing
            let mut chanstoremove = Vec::new();
            for (index, channel) in channels.iter().enumerate() {
                if !channel.is_playing().unwrap_or(true) {
                    chanstoremove.push(index);
                    //println!("Cleaned up channel {index}");
                }
            }
    
            // Remove channels that are not playing, in reverse order
            for index in chanstoremove.iter().rev() {
                channels.remove(*index);
            }
        }
    }

    pub fn set_listener_attributes(
        &mut self,
        position: Vector,
        velocity: Vector,
        forward: Vector,
        up: Vector,
    ) {
        self.system
            .set_3d_listener_attributes(0, Some(position), Some(velocity), Some(forward), Some(up))
            .unwrap();
    }
}
