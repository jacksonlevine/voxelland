use glam::Vec3;
use libfmod::{Channel, Sound, System, Vector};
use std::collections::HashMap;

pub struct AudioPlayer {
    system: System,
    sounds: HashMap<&'static str, Sound>,
    series: HashMap<&'static str, Vec<&'static str>>,
    series_indexes: HashMap<&'static str, usize>,
    channels: HashMap<&'static str, Vec<Channel>>, // Store channels to manage sound playback
}

impl AudioPlayer {
    pub fn new() -> Result<Self, libfmod::Error> {
        let system = System::create().unwrap();
        system.init(32, libfmod::Init::NORMAL, None)?; // Initialize system with 32 channels

        Ok(AudioPlayer {
            system,
            sounds: HashMap::new(),
            series: HashMap::new(),
            series_indexes: HashMap::new(),
            channels: HashMap::new(),
        })
    }
    pub fn update(&mut self) {
        self.system.update();
    }

    pub fn preload(&mut self, id: &'static str, file_path: &'static str) -> Result<(), libfmod::Error> {
        let sound = self.system.create_sound(file_path, libfmod::Mode::FMOD_3D, None)?;
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

    pub fn play_next_in_series(&mut self, series_name: &'static str, pos: &Vec3, vel: &Vec3) -> Result<(), libfmod::Error> {
        let index = *self.series_indexes.get(series_name).unwrap_or(&0);
        let file_path = self.series.get(series_name).unwrap()[index];

        self.play(file_path, pos, vel);
        let next_index = (index + 1) % self.series.get(series_name).unwrap().len();
        self.series_indexes.insert(series_name, next_index);

        Ok(())
    }

    pub fn play(&mut self, id: &'static str, pos: &Vec3, vel: &Vec3)  {
        if let Some(sound) = self.sounds.get(id) {
            let channel = self.system.play_sound(*sound, None, false).unwrap();
            channel.set_3d_attributes(Some(Vector::new(pos.x, pos.y, pos.z)), Some(Vector::new(vel.x, vel.y, vel.z)));
            channel.set_3d_min_max_distance(10.0, 100.0);
            self.channels.entry(id).or_insert_with(Vec::new).push(channel);
        } else {
            self.preload(id, id);
            if let Some(sound) = self.sounds.get(id) {
                let channel = self.system.play_sound(*sound, None, false).unwrap();
                channel.set_3d_attributes(Some(Vector::new(pos.x, pos.y, pos.z)), Some(Vector::new(vel.x, vel.y, vel.z)));
                channel.set_3d_min_max_distance(10.0, 100.0);
                self.channels.entry(id).or_insert_with(Vec::new).push(channel);
            }
        }
    }

    pub fn set_listener_attributes(&mut self, position: Vector, velocity: Vector, forward: Vector, up: Vector) {
        self.system.set_3d_listener_attributes(0, Some(position), Some(velocity), Some(forward), Some(up)).unwrap();
    }
}