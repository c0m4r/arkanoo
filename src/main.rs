mod entities;
mod game;
mod rendering;
mod audio;
mod menu;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::image::{LoadTexture, LoadSurface, InitFlag};
use std::time::Duration;

use crate::entities::{WINDOW_WIDTH, WINDOW_HEIGHT};
use crate::game::{Game, GameState};
use crate::rendering::render_game;
use crate::audio::AudioManager;
use crate::menu::{Menu, MenuState, MenuAction, handle_menu_click};

fn main() -> Result<(), Box<dyn std::error::Error>> {


    // Initialize SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG)?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    // Create window
    let window = video_subsystem
        .window("Arkanoo", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    
    // Set window icon
    if let Ok(icon) = sdl2::surface::Surface::from_file("assets/icon-64.png") {
        canvas.window_mut().set_icon(icon);
    }
    
    // Set initial scale
    let _ = canvas.set_scale(1.0, 1.0);
    
    // Hide cursor and lock it to the window
    sdl_context.mouse().show_cursor(false);
    let _ = canvas.window_mut().set_grab(true);
    
    let mut event_pump = sdl_context.event_pump()?;

    // Load font
    let font_path = if cfg!(target_os = "windows") {
        r"C:\Windows\Fonts\Arial.ttf"
    } else {
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
    };
    
    // Helper to load font with scaling
    let load_font = |scale: f32| -> Result<sdl2::ttf::Font, String> {
        let font_size = (24.0 * scale) as u16;
        ttf_context.load_font(font_path, font_size).map_err(|e| e.to_string())
    };

    let mut font = load_font(1.0)?;

    // Load background image (will be loaded dynamically per level)
    let texture_creator = canvas.texture_creator();
    
    // Initialize texture cache
    let mut texture_cache = crate::rendering::TextureCache::new(&mut canvas, &texture_creator)?;

    // Load heart texture
    let heart_texture = texture_creator
        .load_texture("assets/heart.png")
        .ok();

    // Load splash screen texture
    let splash_texture = texture_creator
        .load_texture("assets/antigravity.webp")
        .ok();
    let mut splash_timer: u64 = 0; // Timer for splash screen (in frames)

    // Initialize audio
    let mut audio_manager = AudioManager::new().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to initialize audio: {}", e);
        eprintln!("The game will continue without sound.");
        AudioManager::new().unwrap()
    });

    // Start background music
    audio_manager.play_music();

    // Create game and menu
    let mut game = Game::new();
    let mut menu = Menu::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    menu.music_slider.set_value(audio_manager.get_music_volume());
    menu.sfx_slider.set_value(audio_manager.get_sfx_volume());
    menu.set_music_muted(audio_manager.is_music_muted());
    menu.set_sfx_muted(audio_manager.is_sfx_muted());
    menu.set_fullscreen(false);

    // Start playing music
    audio_manager.play_music();

    let mut mouse_down = false;

    let mut is_fullscreen = false;
    
    // FPS tracking
    let mut frame_times: Vec<std::time::Instant> = Vec::new();
    let mut current_fps = 60.0;
    
    // Cache background and track current level
    let mut current_level = game.current_level;
    let mut background = texture_creator
        .load_texture(&game.get_background_path())
        .ok();

    let target_frame_time = Duration::from_micros(1_000_000 / 60);

    'running: loop {
        let frame_start = std::time::Instant::now();

        // Reload background only if level changed
        if game.current_level != current_level {
            current_level = game.current_level;
            background = texture_creator
                .load_texture(&game.get_background_path())
                .ok();
        }

        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    if game.state != GameState::GameOver && game.state != GameState::Victory {
                        game.toggle_pause();
                        menu.state = MenuState::Main;
                        
                        // Show/Hide cursor based on pause state
                        if game.state == GameState::Paused {
                            sdl_context.mouse().show_cursor(true);
                            let _ = canvas.window_mut().set_grab(false);
                        } else {
                            sdl_context.mouse().show_cursor(false);
                            let _ = canvas.window_mut().set_grab(true);
                        }
                    }
                }
                
                Event::Window { win_event, .. } => {
                    if let sdl2::event::WindowEvent::Maximized = win_event {
                        // Get the current window size
                        let (w, h) = canvas.window().size();
                        
                        // Update resolution to match window size
                        let scale_x = w as f32 / WINDOW_WIDTH as f32;
                        let scale_y = h as f32 / WINDOW_HEIGHT as f32;
                        let _ = canvas.set_scale(scale_x, scale_y);
                        
                        // Reload font with new scale
                        if let Ok(new_font) = load_font(scale_y) {
                            font = new_font;
                        }
                    }
                }
                
                Event::KeyDown { keycode: Some(Keycode::F11), .. } => {
                    is_fullscreen = !is_fullscreen;
                    if is_fullscreen {
                        let _ = canvas.window_mut().set_fullscreen(sdl2::video::FullscreenType::Desktop);
                    } else {
                        let _ = canvas.window_mut().set_fullscreen(sdl2::video::FullscreenType::Off);
                    }
                }
                
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    if game.state == GameState::Paused || game.state == GameState::GameOver || game.state == GameState::Victory {
                        game.reset();
                        menu.state = MenuState::Main;
                        // Ensure cursor is hidden/grabbed when restarting
                        sdl_context.mouse().show_cursor(false);
                        let _ = canvas.window_mut().set_grab(true);
                    }
                }
                
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
                    if game.state == GameState::SplashScreen {
                        // Skip splash screen, show menu
                        game.state = GameState::Paused;
                        menu.state = MenuState::Main;
                        sdl_context.mouse().show_cursor(true);
                        let _ = canvas.window_mut().set_grab(false);
                    } else if game.state == GameState::Victory {
                        game.start_next_level(); // Starts level 10 (Infinite Mode)
                    } else if game.state == GameState::LevelTransition {
                        game.start_next_level();
                        // Music continues playing, no change needed
                    }
                }

                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    if game.state == GameState::Paused || game.state == GameState::GameOver || game.state == GameState::Victory {
                        break 'running;
                    } else if game.state == GameState::Playing {
                        // Cheat: Skip to next level
                        if game.blocks.iter().any(|b| b.active) {
                            // Clear all blocks to trigger level transition
                            for block in &mut game.blocks {
                                block.active = false;
                            }
                        }
                    }
                }

                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    if game.state == GameState::Playing {
                        // Check if any balls are attached to paddle
                        let has_attached_balls = game.balls.iter().any(|b| b.attached_to_paddle);
                        
                        if has_attached_balls {
                            // Launch attached balls
                            game.launch_balls();
                        } else {
                            // Fire rocket if no balls are attached
                            let mut sound_to_play = None;
                            game.fire_rocket(&mut |effect| sound_to_play = Some(effect));
                            if let Some(effect) = sound_to_play {
                                match effect {
                                    crate::game::SoundEffect::Bounce => audio_manager.play_bounce(),
                                    crate::game::SoundEffect::Oh => audio_manager.play_oh(),
                                    crate::game::SoundEffect::Load => audio_manager.play_load(),
                                    crate::game::SoundEffect::BreakingGlass => audio_manager.play_breaking_glass(),
                                }
                            }
                        }
                    }
                }

                Event::MouseMotion { x, y, .. } => {
                    // Adjust mouse coordinates for scaling
                    let (scale_x, scale_y) = canvas.scale();
                    let adj_x = (x as f32 / scale_x) as i32;
                    let adj_y = (y as f32 / scale_y) as i32;

                    if game.state == GameState::Paused {
                        menu.update_hover(adj_x, adj_y);
                        menu.update_slider(adj_x, adj_y, mouse_down);
                        
                        // Update audio volume from sliders
                        if menu.state == MenuState::Settings {
                            let new_music_volume = menu.music_slider.get_value();
                            if new_music_volume != audio_manager.get_music_volume() {
                                audio_manager.set_music_volume(new_music_volume);
                            }
                            
                            let new_sfx_volume = menu.sfx_slider.get_value();
                            if new_sfx_volume != audio_manager.get_sfx_volume() {
                                audio_manager.set_sfx_volume(new_sfx_volume);
                            }
                        }
                    } else if game.state == GameState::Playing {
                        // Mouse control for paddle - center paddle on mouse X position
                        let paddle_center_x = adj_x - (game.paddle.width / 2);
                        game.paddle.set_x(paddle_center_x);
                    }
                }

                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    mouse_down = true;
                    // Adjust mouse coordinates for scaling
                    let (scale_x, scale_y) = canvas.scale();
                    let adj_x = (x as f32 / scale_x) as i32;
                    let adj_y = (y as f32 / scale_y) as i32;

                    if game.state == GameState::SplashScreen {
                        // Skip splash screen, show menu
                        game.state = GameState::Paused;
                        menu.state = MenuState::Main;
                        sdl_context.mouse().show_cursor(true);
                        let _ = canvas.window_mut().set_grab(false);
                    } else if game.state == GameState::Paused {
                        let action = handle_menu_click(&menu, adj_x, adj_y);
                        match action {
                            MenuAction::Resume => {
                                game.toggle_pause();
                                // Hide cursor when resuming
                                sdl_context.mouse().show_cursor(false);
                                let _ = canvas.window_mut().set_grab(true);
                            }
                            MenuAction::Restart => {
                                game.reset();
                                // Music continues playing, no change needed
                                // Hide cursor when restarting
                                sdl_context.mouse().show_cursor(false);
                                let _ = canvas.window_mut().set_grab(true);
                            }
                            MenuAction::Quit => {
                                break 'running;
                            }
                            MenuAction::OpenSettings => {
                                menu.state = MenuState::Settings;
                            }
                            MenuAction::CloseSettings => {
                                menu.state = MenuState::Main;
                            }
                            MenuAction::ToggleMusic => {
                                audio_manager.toggle_music_mute();
                                menu.set_music_muted(audio_manager.is_music_muted());
                            }
                            MenuAction::ToggleSFX => {
                                audio_manager.toggle_sfx_mute();
                                menu.set_sfx_muted(audio_manager.is_sfx_muted());
                            }
                            MenuAction::ToggleFullscreen => {
                                is_fullscreen = !is_fullscreen;
                                menu.set_fullscreen(is_fullscreen);
                                if is_fullscreen {
                                    let _ = canvas.window_mut().set_fullscreen(sdl2::video::FullscreenType::Desktop);
                                } else {
                                    let _ = canvas.window_mut().set_fullscreen(sdl2::video::FullscreenType::Off);
                                }
                            }
                            MenuAction::ToggleGravity => {
                                game.toggle_gravity_mode();
                                menu.set_gravity_mode(game.gravity_mode);
                            }
                            MenuAction::None => {}
                        }
                    } else if game.state == GameState::LevelTransition {
                        // Click to start next level
                        game.start_next_level();
                        // Music continues playing
                    } else if game.state == GameState::Victory {
                        // Click to start infinite mode (level 10)
                        game.start_next_level();
                    }

                }

                Event::MouseButtonUp { mouse_btn: MouseButton::Left, .. } => {
                    mouse_down = false;
                }

                _ => {}
            }
        }

        // Handle continuous input (arrow keys)
        if game.state == GameState::Playing {
            let keyboard_state = event_pump.keyboard_state();
            if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Left) {
                game.paddle.move_left();
            }
            if keyboard_state.is_scancode_pressed(sdl2::keyboard::Scancode::Right) {
                game.paddle.move_right();
            }
        }

        // Update splash screen timer
        if game.state == GameState::SplashScreen {
            splash_timer += 1;
            // Auto-advance to menu after 3 seconds (180 frames at 60 FPS)
            if splash_timer >= 180 {
                game.state = GameState::Paused;
                menu.state = MenuState::Main;
                sdl_context.mouse().show_cursor(true);
                let _ = canvas.window_mut().set_grab(false);
            }
        }

        // Update game
        let mut sound_to_play = None;
        game.update(&mut |effect| sound_to_play = Some(effect));
        
        if let Some(effect) = sound_to_play {
            match effect {
                crate::game::SoundEffect::Bounce => audio_manager.play_bounce(),
                crate::game::SoundEffect::Oh => audio_manager.play_oh(),
                crate::game::SoundEffect::Load => audio_manager.play_load(),
                crate::game::SoundEffect::BreakingGlass => audio_manager.play_breaking_glass(),
            }
        }

        // Update audio (for song transitions)
        audio_manager.update();
        
        // Calculate FPS
        let now = std::time::Instant::now();
        frame_times.push(now);
        frame_times.retain(|t| now.duration_since(*t).as_secs_f32() < 1.0);
        current_fps = frame_times.len() as f32;

        // Render
        render_game(&mut canvas, &game, &menu, background.as_mut(), heart_texture.as_ref(), splash_texture.as_ref(), &font, current_fps, &mut texture_cache);

        // Target 60 FPS
        let elapsed = frame_start.elapsed();
        if elapsed < target_frame_time {
            std::thread::sleep(target_frame_time - elapsed);
        }
    }

    audio_manager.stop_music();
    Ok(())
}
