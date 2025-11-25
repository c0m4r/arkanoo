use sdl2::mixer::{Channel, Chunk, Music};
use std::path::Path;
use rand::Rng;

    pub struct AudioManager {
    bounce_sound: Option<Chunk>,
    oh_sound: Option<Chunk>,
    load_sound: Option<Chunk>,
    breaking_glass_sound: Option<Chunk>,
    songs: Vec<String>,
    current_song_index: usize,
    music_volume: i32,
    sfx_volume: i32,
    music_muted: bool,
    sfx_muted: bool,
    music_should_play: bool,
}

impl AudioManager {
    pub fn new() -> Result<Self, String> {
        // Initialize SDL2 mixer
        sdl2::mixer::open_audio(44100, sdl2::mixer::AUDIO_S16LSB, 2, 1024)?;
        sdl2::mixer::allocate_channels(4);

        // Try to load MP3 bounce sound (fallback to WAV if MP3 not found)
        let bounce_sound = Chunk::from_file(Path::new("assets/ball.mp3"))
            .or_else(|_| Chunk::from_file(Path::new("assets/ball_bounce.mp3")))
            .or_else(|_| Chunk::from_file(Path::new("assets/ball_bounce.wav")))
            .ok();

        if bounce_sound.is_none() {
            eprintln!("Warning: Could not load ball.mp3, ball_bounce.mp3, or ball_bounce.wav");
        }

        // Load drop-sound-effect-240899.mp3
        let oh_sound = Chunk::from_file(Path::new("assets/drop-sound-effect-240899.mp3")).ok();
        if oh_sound.is_none() {
            eprintln!("Warning: Could not load assets/drop-sound-effect-240899.mp3");
        }

        // Load load.mp3
        let load_sound = Chunk::from_file(Path::new("assets/load.mp3")).ok();
        if load_sound.is_none() {
            eprintln!("Warning: Could not load assets/load.mp3");
        }

        // Load breaking-glass.mp3
        let breaking_glass_sound = Chunk::from_file(Path::new("assets/breaking-glass.mp3")).ok();
        if breaking_glass_sound.is_none() {
            eprintln!("Warning: Could not load assets/breaking-glass.mp3");
        }

        // Setup song playlist - dynamically load all .mp3 files from assets directory
        let mut songs = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir("assets/music") {
            for entry in entries.flatten() {
                if let Ok(path) = entry.path().canonicalize() {
                    if let Some(ext) = path.extension() {
                        if ext == "mp3" {
                            if let Some(path_str) = path.to_str() {
                                // Convert to relative path for consistency
                                if let Ok(rel_path) = path.strip_prefix(std::env::current_dir().unwrap_or_default()) {
                                    songs.push(rel_path.to_string_lossy().to_string());
                                } else {
                                    songs.push(path_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Sort songs for consistent ordering
        songs.sort();

        if songs.is_empty() {
            eprintln!("Warning: No .mp3 music files found in assets directory");
        }

        // Start at a random song if we have any
        let current_song_index = if !songs.is_empty() {
            let mut rng = rand::thread_rng();
            rng.gen_range(0..songs.len())
        } else {
            0
        };


        Ok(AudioManager {
            bounce_sound,
            oh_sound,
            load_sound,
            breaking_glass_sound,
            songs,
            current_song_index,
            music_volume: 64, // Default to 50% volume (max is 128)
            sfx_volume: 64,   // Default to 50% volume (max is 128)
            music_muted: false,
            sfx_muted: false,
            music_should_play: false,
        })
    }

    pub fn play_bounce(&self) {
        if !self.sfx_muted {
            if let Some(ref sound) = self.bounce_sound {
                let _ = Channel::all().play(sound, 0);
            }
        }
    }

    pub fn play_oh(&self) {
        if !self.sfx_muted {
            if let Some(ref sound) = self.oh_sound {
                let _ = Channel::all().play(sound, 0);
            }
        }
    }

    pub fn play_load(&self) {
        if !self.sfx_muted {
            if let Some(ref sound) = self.load_sound {
                let _ = Channel::all().play(sound, 0);
            }
        }
    }

    pub fn play_breaking_glass(&self) {
        if !self.sfx_muted {
            if let Some(ref sound) = self.breaking_glass_sound {
                let _ = Channel::all().play(sound, 0);
            }
        }
    }

    pub fn update(&mut self) {
        if !self.music_muted && self.music_should_play && !self.songs.is_empty() {
            // Auto-advance to next random song when current finishes
            if !Music::is_playing() {
                // Pick a random song
                let mut rng = rand::thread_rng();
                self.current_song_index = rng.gen_range(0..self.songs.len());
                
                let song_path = &self.songs[self.current_song_index];
                if let Ok(music) = Music::from_file(song_path) {
                    Music::set_volume(self.music_volume);
                    // Play ONCE (1), not loop (-1)
                    // This allows is_playing() to return false when done
                    let _ = music.play(1); 
                    
                    // Leak the music to keep it alive
                    std::mem::forget(music);
                } else {
                    eprintln!("Warning: Could not load {}", song_path);
                }
            }
        }
    }

    pub fn play_music(&mut self) {
        if self.songs.is_empty() || self.music_muted {
            return;
        }

        self.music_should_play = true;
        
        // Start playing the current song
        let song_path = &self.songs[self.current_song_index];
        if let Ok(music) = Music::from_file(song_path) {
            Music::set_volume(self.music_volume);
            // Play ONCE (1), not loop (-1)
            let _ = music.play(1); 
            
            // Leak the music to keep it alive (SDL2 requirement)
            std::mem::forget(music);
        } else {
            eprintln!("Warning: Could not load music file: {}", song_path);
        }
    }

    
    pub fn stop_music(&mut self) {
        Music::halt();
        self.music_should_play = false;
    }

    // Music volume and mute controls
    pub fn set_music_volume(&mut self, volume: i32) {
        self.music_volume = volume.clamp(0, 128);
        Music::set_volume(self.music_volume);
    }

    pub fn get_music_volume(&self) -> i32 {
        self.music_volume
    }

    pub fn set_music_muted(&mut self, muted: bool) {
        let was_muted = self.music_muted;
        self.music_muted = muted;
        
        if muted {
            Music::set_volume(0);
            if Music::is_playing() {
                Music::pause();
            }
        } else {
            Music::set_volume(self.music_volume);
            if was_muted {
                Music::resume();
            }
        }
    }

    pub fn is_music_muted(&self) -> bool {
        self.music_muted
    }

    pub fn toggle_music_mute(&mut self) {
        self.set_music_muted(!self.music_muted);
    }

    // SFX volume and mute controls
    pub fn set_sfx_volume(&mut self, volume: i32) {
        self.sfx_volume = volume.clamp(0, 128);
        Channel::all().set_volume(self.sfx_volume);
    }

    pub fn get_sfx_volume(&self) -> i32 {
        self.sfx_volume
    }

    pub fn set_sfx_muted(&mut self, muted: bool) {
        self.sfx_muted = muted;
        
        if muted {
            Channel::all().set_volume(0);
        } else {
            Channel::all().set_volume(self.sfx_volume);
        }
    }

    pub fn is_sfx_muted(&self) -> bool {
        self.sfx_muted
    }

    pub fn toggle_sfx_mute(&mut self) {
        self.set_sfx_muted(!self.sfx_muted);
    }
}
