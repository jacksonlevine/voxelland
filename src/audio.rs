use rodio::{source::{Source, Buffered}, Decoder, OutputStream, OutputStreamHandle, buffer::SamplesBuffer};
use std::{fs::File, io::{BufReader, Cursor}, time::Duration, sync::Arc, collections::HashMap};
use std::io::Read;

pub struct AudioPlayer {
    streams: Vec<OutputStream>,                // Holds OutputStreams to keep them alive
    handles: Vec<OutputStreamHandle>,          // Handles to control playback on each stream
    cache: HashMap<String, Arc<Vec<u8>>>,      // Cache to store loaded and decoded audio data
    series_map: HashMap<String, Vec<String>>,  // Map series names to a list of audio paths
    series_index: HashMap<String, usize>,      // Current index for each series
}

impl AudioPlayer {
    pub fn new(num_streams: usize) -> Self {
        let mut streams = Vec::with_capacity(num_streams);
        let mut handles = Vec::with_capacity(num_streams);
        let cache = HashMap::new();
        let series_map = HashMap::new();
        let series_index = HashMap::new();

        for _ in 0..num_streams {
            let (stream, handle) = OutputStream::try_default().unwrap();
            streams.push(stream);
            handles.push(handle);
        }

        AudioPlayer { streams, handles, cache, series_map, series_index }
    }

    pub fn preload(&mut self, file_path: &str) {
        let audio_data = self.load_audio(file_path);
        self.cache.insert(file_path.to_string(), Arc::new(audio_data));
    }

    fn load_audio(&self, file_path: &str) -> Vec<u8> {
        let file = File::open(file_path).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut buffer = Vec::new();
        buf_reader.read_to_end(&mut buffer).unwrap();
        buffer
    }

    pub fn preload_series(&mut self, name: &str, paths: Vec<String>) {
        for path in &paths {
            self.preload(path);
        }
        self.series_map.insert(name.to_string(), paths);
        self.series_index.insert(name.to_string(), 0);
    }

    pub fn play_next_in_series(&mut self, name: &str, stream_index: usize) {
        if let Some(paths) = self.series_map.get(name) {
            let index = self.series_index.get(name).cloned().unwrap_or(0);
            let file_path = &paths[index];
            self.play(file_path, stream_index, false);  // Call to play the file once
            let next_index = (index + 1) % paths.len();
            self.series_index.insert(name.to_string(), next_index);
        } else {
            panic!("Series name not found!");
        }
    }

    pub fn play(&self, file_path: &str, stream_index: usize, looped: bool) {
        if stream_index >= self.handles.len() {
            panic!("Stream index out of bounds!");
        }
        
        let audio_data = self.cache.get(file_path).cloned().unwrap_or_else(|| {
            let data = self.load_audio(file_path);
            Arc::new(data)
        });

        let cursor = Cursor::new((*audio_data).clone());
        let decoder = Decoder::new(cursor).unwrap();

        let source: Box<dyn Source<Item = _> + Send> = if looped {
            Box::new(decoder.repeat_infinite())
        } else {
            Box::new(decoder)
        };

        self.handles[stream_index].play_raw(source.convert_samples()).unwrap();
    }
}