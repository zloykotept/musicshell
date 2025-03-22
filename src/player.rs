use std::{
    collections::VecDeque,
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use anyhow::{Ok, Result};
use rodio::{source, Decoder, OutputStream, Sink, Source};

pub struct Player {
    pub now_playing: String,
    pub queue: VecDeque<PathBuf>,
    pub song_index: usize,
    pub repeat: bool,
    restart: bool,
    sink: Sink,
    total_duration: usize,
}

impl Player {
    pub fn new() -> Option<(OutputStream, Self)> {
        let (_stream, stream_handle) = OutputStream::try_default().ok()?;
        let sink = Sink::try_new(&stream_handle).ok()?;

        Some((
            _stream,
            Player {
                now_playing: String::new(),
                sink,
                queue: VecDeque::new(),
                song_index: 0,
                repeat: false,
                restart: false,
                total_duration: 0,
            },
        ))
    }

    pub fn get_duration_current(&self) -> usize {
        self.sink.get_pos().as_secs() as usize
    }

    pub fn get_duration_total(&self) -> usize {
        self.total_duration
    }

    fn play(&mut self, path: PathBuf) -> Result<()> {
        let file = File::open(&path)?;
        let source = Decoder::new(BufReader::new(file))?;

        if let Some(total_duration) = source.total_duration() {
            self.total_duration = total_duration.as_secs() as usize;
        }

        self.now_playing = path.file_name().unwrap().to_str().unwrap().to_string();
        self.sink.clear();
        self.sink.append(source);
        self.sink.play();

        Ok(())
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn resume(&self) {
        self.sink.play();
    }

    pub fn increase_volume(&self, step: f32) {
        let current = self.sink.volume();
        let new = current + (step / 100.0);
        if new >= 1.0 {
            self.sink.set_volume(1.0);
        } else {
            self.sink.set_volume(new);
        }
    }

    pub fn decrease_volume(&self, step: f32) {
        let current = self.sink.volume();
        let new = current - (step / 100.0);
        if new <= 0.0 {
            self.sink.set_volume(0.0);
        } else {
            self.sink.set_volume(new);
        }
    }

    pub fn get_volume_percantage(&self) -> usize {
        (self.sink.volume() * 100.0) as usize
    }

    pub fn get_volume(&self) -> f32 {
        self.sink.volume()
    }

    pub fn set_volume(&self, x: f32) {
        self.sink.set_volume(x)
    }

    pub fn rewind_forward(&self, step: u64) {
        let current_pos = self.sink.get_pos();
        let new = Duration::from_secs(step) + current_pos;
        self.sink.try_seek(new).unwrap_or(());
    }

    pub fn rewind_back(&self, step: u64) {
        let current_pos = self.sink.get_pos().as_secs();
        if step > current_pos {
            self.sink.try_seek(Duration::from_secs(0)).unwrap();
            return;
        }
        let new = Duration::from_secs(current_pos) - Duration::from_secs(step);
        self.sink.try_seek(new).unwrap_or(());
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }

    pub fn clear(&mut self) {
        self.sink.clear();
        self.total_duration = 0;
        self.now_playing = String::new();
    }

    pub fn restart(&mut self) {
        self.restart = true;
    }
}

pub fn main_loop(player: Arc<RwLock<Player>>) {
    loop {
        let mutex = player.read().unwrap();
        if mutex.queue.is_empty() || !mutex.is_empty() {
            if mutex.restart {
                drop(mutex);
                let mut mutex = player.write().unwrap();

                mutex.clear();
                mutex.song_index = 0;
                continue;
            }

            drop(mutex);
            thread::sleep(Duration::from_millis(100));
            continue;
        }
        drop(mutex);
        let mut mutex = player.write().unwrap();

        if mutex.repeat {
            mutex.song_index -= 1;
        }
        if mutex.song_index > mutex.queue.len() - 1 {
            mutex.song_index = 0;
        }
        let track_path = mutex.queue.get(mutex.song_index).unwrap().to_path_buf();
        mutex.song_index += 1;

        mutex.restart = false;

        if !track_path.exists() {
            let i = mutex.song_index - 1;
            mutex.queue.remove(i);
            continue;
        }

        if let Err(err) = mutex.play(track_path) {
            panic!("Error while decoding file, try MPEG-4 codec");
        }
    }
}
