mod entities;
mod game;
mod rendering;
mod audio;
mod menu;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::image::{LoadTexture, InitFlag};
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
    
    // Set logical size for proper scaling in fullscreen
    canvas.set_logical_size(WINDOW_WIDTH, WINDOW_HEIGHT)?;
    
    // Hide cursor and lock it to the window
    sdl_context.mouse().show_cursor(false);
    let _ = canvas.window_mut().set_grab(true);
    
    let mut event_pump = sdl_context.event_pump()?;

    // Load font
    let font_path = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf";
    let font = ttf_context.load_font(font_path, 24)?;

    // Load background image (will be loaded dynamically per level)
    let texture_creator = canvas.texture_creator();
    
    // Load heart texture
    let heart_texture = texture_creator
        .load_texture("assets/heart.png")
        .ok();

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
    menu.volume_slider.set_value(audio_manager.get_volume());
    menu.set_muted(audio_manager.is_muted());
    menu.set_fullscreen(false);

    let mut mouse_down = false;
    let mut is_fullscreen = false;
    
    // Available resolutions
    let resolutions = vec![
        (1280, 720),
        (1920, 1080),
        (2560, 1440),
    ];
    let mut current_resolution_idx = 0;
    
    // Cache background and track current level
    let mut current_level = game.current_level;
    let mut background = texture_creator
        .load_texture(&game.get_background_path())
        .ok();

    'running: loop {
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
                
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    if game.state == GameState::Paused || game.state == GameState::GameOver || game.state == GameState::Victory {
                        break 'running;
                    }
                }

                Event::MouseMotion { x, y, .. } => {
                    if game.state == GameState::Paused {
                        menu.update_hover(x, y);
                        menu.update_slider(x, y, mouse_down);
                        
                        // Update audio volume from slider
                        if menu.state == MenuState::Settings {
                            let new_volume = menu.volume_slider.get_value();
                            if new_volume != audio_manager.get_volume() {
                                audio_manager.set_volume(new_volume);
                            }
                        }
                    } else if game.state == GameState::Playing {
                        // Mouse control for paddle - center paddle on mouse X position
                        let paddle_center_x = x - (game.paddle.width / 2);
                        game.paddle.set_x(paddle_center_x);
                    }
                }

                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    mouse_down = true;
                    if game.state == GameState::Paused {
                        let action = handle_menu_click(&menu, x, y);
                        match action {
                            MenuAction::Resume => {
                                game.toggle_pause();
                                // Hide cursor when resuming
                                sdl_context.mouse().show_cursor(false);
                                let _ = canvas.window_mut().set_grab(true);
                            }
                            MenuAction::Restart => {
                                game.reset();
                                audio_manager.play_level_music(1);
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
                            MenuAction::ToggleMute => {
                                audio_manager.toggle_mute();
                                menu.set_muted(audio_manager.is_muted());
                            }
                            MenuAction::CycleResolution => {
                                // Cycle to next resolution
                                current_resolution_idx = (current_resolution_idx + 1) % resolutions.len();
                                let (width, height) = resolutions[current_resolution_idx];
                                menu.resolution_button.label = format!("{}x{}", width, height);
                                
                                // Apply resolution change
                                let _ = canvas.window_mut().set_size(width, height);
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
                            MenuAction::None => {}
                        }
                    } else if game.state == GameState::LevelTransition {
                        // Click to start next level
                        game.start_next_level();
                        audio_manager.play_level_music(game.current_level);
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

        // Update game
        let mut should_play_sound = false;
        game.update(&mut || should_play_sound = true);
        
        if should_play_sound {
            audio_manager.play_bounce();
        }

        // Update audio (for song transitions)
        audio_manager.update();

        // Render
        render_game(&mut canvas, &game, &menu, background.as_mut(), heart_texture.as_ref(), &font);

        // Target 60 FPS
        std::thread::sleep(Duration::from_millis(16));
    }

    audio_manager.stop_music();
    Ok(())
}
