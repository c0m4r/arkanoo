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
    volume: i32,
    muted: bool,
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

        // Load oh.mp3
        let oh_sound = Chunk::from_file(Path::new("assets/oh.mp3")).ok();
        if oh_sound.is_none() {
            eprintln!("Warning: Could not load assets/oh.mp3");
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

        // Setup song playlist
        let songs = vec![
            "assets/song1.mp3".to_string(),
            "assets/song2.mp3".to_string(),
            "assets/song3.mp3".to_string(),
            "assets/song4.mp3".to_string(),
            "assets/song5.mp3".to_string(),
            "assets/song6.mp3".to_string(),
        ];

        // Verify songs exist
        let mut songs_found = 0;
        for song in &songs {
            if Path::new(song).exists() {
                songs_found += 1;
            }
        }

        if songs_found == 0 {
            eprintln!("Warning: No song files found (song1.mp3 - song4.mp3)");
        } else {
            eprintln!("Info: Found {} song(s)", songs_found);
        }

        // Start at a random song
        let mut rng = rand::thread_rng();
        let current_song_index = rng.gen_range(0..songs.len());

        Ok(AudioManager {
            bounce_sound,
            oh_sound,
            load_sound,
            breaking_glass_sound,
            songs,
            current_song_index,
            volume: 32, // Default to 50% volume (max is 128)
            muted: false,
            music_should_play: false,
        })
    }

    pub fn play_bounce(&self) {
        if !self.muted {
            if let Some(ref sound) = self.bounce_sound {
                let _ = Channel::all().play(sound, 0);
            }
        }
    }

    pub fn play_oh(&self) {
        if !self.muted {
            if let Some(ref sound) = self.oh_sound {
                let _ = Channel::all().play(sound, 0);
            }
        }
    }

    pub fn play_load(&self) {
        if !self.muted {
            if let Some(ref sound) = self.load_sound {
                let _ = Channel::all().play(sound, 0);
            }
        }
    }

    pub fn play_breaking_glass(&self) {
        if !self.muted {
            if let Some(ref sound) = self.breaking_glass_sound {
                let _ = Channel::all().play(sound, 0);
            }
        }
    }

    pub fn play_music(&mut self) {
        if !self.muted && !self.songs.is_empty() {
            let song_path = &self.songs[self.current_song_index];
            if let Ok(music) = Music::from_file(Path::new(song_path)) {
                Music::set_volume(self.volume);
                // Play repeatedly - SDL will loop it indefinitely
                let _ = music.play(-1);
                self.music_should_play = true;
                eprintln!("Now playing: {}", song_path);
                
                // Leak the music to keep it alive
                // This is necessary with SDL2's music system
                std::mem::forget(music);
            }
        }
    }

    pub fn update(&mut self) {
        // Check if music finished playing and we should play next song
        // Only do this if we expect music to be playing
        if self.music_should_play && !self.muted && !Music::is_playing() {
            // Move to next song
            self.current_song_index = (self.current_song_index + 1) % self.songs.len();
            
            // Small delay to avoid rapid switching
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            // Play next song
            self.play_music();
        }
    }
    
    pub fn play_level_music(&mut self, level: usize) {
        // For level changes, move to next song in sequence
        // Level 1 starts random, then each level advances to next song
        if level == 1 {
            // Keep the initial random song for level 1
        } else {
            // Advance to next song for subsequent levels
            self.current_song_index = (self.current_song_index + 1) % self.songs.len();
        }
        
        // Stop current music
        Music::halt();
        
        // Play new song
        self.play_music();
    }

    pub fn stop_music(&mut self) {
        Music::halt();
        self.music_should_play = false;
    }

    pub fn set_volume(&mut self, volume: i32) {
        self.volume = volume.clamp(0, 128);
        Music::set_volume(self.volume);
        Channel::all().set_volume(self.volume);
    }

    pub fn get_volume(&self) -> i32 {
        self.volume
    }

    pub fn set_muted(&mut self, muted: bool) {
        let was_muted = self.muted;
        self.muted = muted;
        
        if muted {
            Music::set_volume(0);
            Channel::all().set_volume(0);
            if Music::is_playing() {
                Music::pause();
            }
        } else {
            Music::set_volume(self.volume);
            Channel::all().set_volume(self.volume);
            if was_muted {
                Music::resume();
            }
        }
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    pub fn toggle_mute(&mut self) {
        self.set_muted(!self.muted);
    }
}
