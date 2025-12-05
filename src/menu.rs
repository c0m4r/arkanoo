use sdl2::rect::Rect;

#[derive(Clone, Copy, PartialEq)]
pub enum MenuState {
    Main,
    Settings,

}

#[derive(Clone)]
pub struct Button {
    pub rect: Rect,
    pub label: String,
    pub hovered: bool,
}

impl Button {
    pub fn new(x: i32, y: i32, width: u32, height: u32, label: &str) -> Self {
        Button {
            rect: Rect::new(x, y, width, height),
            label: label.to_string(),
            hovered: false,
        }
    }

    pub fn update_hover(&mut self, mouse_x: i32, mouse_y: i32) {
        self.hovered = self.rect.contains_point((mouse_x, mouse_y));
    }

    pub fn is_clicked(&self, mouse_x: i32, mouse_y: i32) -> bool {
        self.rect.contains_point((mouse_x, mouse_y))
    }
}

pub struct VolumeSlider {
    pub rect: Rect,
    pub value: i32, // 0-128
    pub dragging: bool,
}

impl VolumeSlider {
    pub fn new(x: i32, y: i32, width: u32) -> Self {
        VolumeSlider {
            rect: Rect::new(x, y, width, 20),
            value: 64,
            dragging: false,
        }
    }

    pub fn update(&mut self, mouse_x: i32, mouse_y: i32, mouse_down: bool) {
        if mouse_down && self.rect.contains_point((mouse_x, mouse_y)) {
            self.dragging = true;
        }

        if !mouse_down {
            self.dragging = false;
        }

        if self.dragging {
            let relative_x = (mouse_x - self.rect.x()).max(0).min(self.rect.width() as i32);
            self.value = (relative_x * 128) / self.rect.width() as i32;
        }
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }

    pub fn set_value(&mut self, value: i32) {
        self.value = value.clamp(0, 128);
    }
}

pub struct Menu {
    pub state: MenuState,
    pub resume_button: Button,
    pub restart_button: Button,
    pub settings_button: Button,
    pub level_editor_button: Button,
    pub back_button: Button,
    pub quit_button: Button,
    pub music_toggle_button: Button,
    pub sfx_toggle_button: Button,
    pub github_button: Button,

    pub fullscreen_button: Button,
    pub vsync_button: Button,
    pub gravity_mode_button: Button,
    pub music_slider: VolumeSlider,
    pub sfx_slider: VolumeSlider,

    // Resolution selection - list of clickable resolution buttons
    pub resolution_label: String,
    pub resolution_buttons: Vec<Button>,
    pub available_resolutions: Vec<(u32, u32)>,
    pub selected_resolution_index: usize,
    pub pending_resolution: Option<(u32, u32)>,
    pub resolution_confirm_timer: Option<u32>, // frames remaining (5 sec = 300 frames)
    pub confirm_button: Button,
    pub cancel_button: Button,

    pub version_string: String,

    pub music_muted: bool,
    pub sfx_muted: bool,
    pub is_fullscreen: bool,
    pub vsync_enabled: bool,
    pub gravity_mode: bool,
    pub game_started: bool, // Track if game has been started (for New Game vs Resume)
}

impl Menu {
    pub fn new(window_width: u32, window_height: u32) -> Self {
        let center_x = window_width as i32 / 2 - 100;
        let center_y = window_height as i32 / 2;

        // Available resolutions (common ones)
        let available_resolutions = vec![
            (1280, 720),   // 720p
            (1366, 768),   // Common laptop
            (1600, 900),   // 900p
            (1920, 1080),  // 1080p
            (2560, 1440),  // 1440p
        ];

        Menu {
            state: MenuState::Main,
            // Main menu - use "New Game" initially, will change to "Resume" once game starts
            resume_button: Button::new(center_x, center_y - 125, 200, 40, "New Game"),
            restart_button: Button::new(center_x, center_y - 75, 200, 40, "Restart"),
            gravity_mode_button: Button::new(center_x, center_y - 25, 200, 40, "Gravity Mode"),
            level_editor_button: Button::new(center_x, center_y + 25, 200, 40, "Level Editor"),
            settings_button: Button::new(center_x, center_y + 75, 200, 40, "Settings"),
            quit_button: Button::new(center_x, center_y + 125, 200, 40, "Quit"),
            
            // Settings menu - improved layout with proper spacing
            // Row 1: Music toggle and slider (y offset: -140 and -100)
            music_toggle_button: Button::new(center_x, center_y - 140, 200, 40, "Music: ON"),
            music_slider: VolumeSlider::new(center_x, center_y - 90, 200),
            
            // Row 2: SFX toggle and slider (y offset: -50 and -10)
            sfx_toggle_button: Button::new(center_x, center_y - 50, 200, 40, "SFX: ON"),
            sfx_slider: VolumeSlider::new(center_x, center_y, 200),
            
            // Row 3: Fullscreen toggle (y offset: +40)
            fullscreen_button: Button::new(center_x, center_y + 40, 200, 40, "Windowed"),
            
            // Row 4: VSync toggle (y offset: +90)
            vsync_button: Button::new(center_x, center_y + 90, 200, 40, "VSync: ON"),
            
            // Row 5: Resolution selection - list of resolution buttons
            // Create buttons for each resolution
            resolution_buttons: {
                let mut buttons = Vec::new();
                let resolutions = [
                    (1280, 720),   // 720p
                    (1366, 768),   // Common laptop
                    (1600, 900),   // 900p
                    (1920, 1080),  // 1080p
                    (2560, 1440),  // 1440p
                ];
                for (i, (w, h)) in resolutions.iter().enumerate() {
                    let label = format!("{}x{}", w, h);
                    // Stack buttons vertically starting below vsync
                    let btn = Button::new(center_x, center_y + 150 + (i as i32 * 35), 200, 30, &label);
                    buttons.push(btn);
                }
                buttons
            },
            resolution_label: "1280x720".to_string(),
            available_resolutions,
            selected_resolution_index: 0,
            pending_resolution: None,
            resolution_confirm_timer: None,
            
            // Confirmation dialog buttons (centered, shown only when confirming)
            confirm_button: Button::new(center_x - 60, center_y + 350, 100, 35, "Keep"),
            cancel_button: Button::new(center_x + 60, center_y + 350, 100, 35, "Revert"),
            
            // Back button (y offset: +400)
            back_button: Button::new(center_x, center_y + 400, 200, 40, "Back"),

            // Bottom right corner
            github_button: Button::new(window_width as i32 - 110, window_height as i32 - 50, 100, 40, "Github"),
            version_string: format!("Version: {}", env!("CARGO_PKG_VERSION")),

            music_muted: false,
            sfx_muted: false,
            is_fullscreen: false,
            vsync_enabled: true,
            gravity_mode: false,
            game_started: false, // Initially false - shows "New Game"
        }
    }

    pub fn update_hover(&mut self, mouse_x: i32, mouse_y: i32) {
        match self.state {
            MenuState::Main => {
                self.resume_button.update_hover(mouse_x, mouse_y);
                self.restart_button.update_hover(mouse_x, mouse_y);
                self.gravity_mode_button.update_hover(mouse_x, mouse_y);
                self.level_editor_button.update_hover(mouse_x, mouse_y);
                self.settings_button.update_hover(mouse_x, mouse_y);
                self.quit_button.update_hover(mouse_x, mouse_y);
                self.github_button.update_hover(mouse_x, mouse_y);
            }
            MenuState::Settings => {
                self.music_toggle_button.update_hover(mouse_x, mouse_y);
                self.sfx_toggle_button.update_hover(mouse_x, mouse_y);
                self.fullscreen_button.update_hover(mouse_x, mouse_y);
                self.vsync_button.update_hover(mouse_x, mouse_y);
                self.back_button.update_hover(mouse_x, mouse_y);
                
                // Resolution list buttons (only when not confirming)
                if self.resolution_confirm_timer.is_none() {
                    for btn in &mut self.resolution_buttons {
                        btn.update_hover(mouse_x, mouse_y);
                    }
                }
                
                // Confirmation dialog buttons (only when confirming)
                if self.resolution_confirm_timer.is_some() {
                    self.confirm_button.update_hover(mouse_x, mouse_y);
                    self.cancel_button.update_hover(mouse_x, mouse_y);
                }
            }

        }
    }

    pub fn update_slider(&mut self, mouse_x: i32, mouse_y: i32, mouse_down: bool) {
        if self.state == MenuState::Settings {
            self.music_slider.update(mouse_x, mouse_y, mouse_down);
            self.sfx_slider.update(mouse_x, mouse_y, mouse_down);
        }
    }



    pub fn set_music_muted(&mut self, muted: bool) {
        self.music_muted = muted;
        self.music_toggle_button.label = if muted {
            "Music: OFF".to_string()
        } else {
            "Music: ON".to_string()
        };
    }
    
    pub fn set_sfx_muted(&mut self, muted: bool) {
        self.sfx_muted = muted;
        self.sfx_toggle_button.label = if muted {
            "SFX: OFF".to_string()
        } else {
            "SFX: ON".to_string()
        };
    }
    
    pub fn set_fullscreen(&mut self, is_fullscreen: bool) {
        self.is_fullscreen = is_fullscreen;
        self.fullscreen_button.label = if is_fullscreen {
            "Fullscreen".to_string()
        } else {
            "Windowed".to_string()
        };
    }
    
    pub fn set_gravity_mode(&mut self, gravity_mode: bool) {
        self.gravity_mode = gravity_mode;
        // Keep label as "Gravity Mode" - don't change it
    }
    
    pub fn set_vsync(&mut self, enabled: bool) {
        self.vsync_enabled = enabled;
        self.vsync_button.label = if enabled {
            "VSync: ON".to_string()
        } else {
            "VSync: OFF".to_string()
        };
    }
    
    pub fn set_game_started(&mut self, started: bool) {
        self.game_started = started;
        self.resume_button.label = if started {
            "Resume".to_string()
        } else {
            "New Game".to_string()
        };
    }
    
    pub fn set_resolution(&mut self, width: u32, height: u32) {
        self.resolution_label = format!("{}x{}", width, height);
        // Find index if it matches a preset
        for (i, &(w, h)) in self.available_resolutions.iter().enumerate() {
            if w == width && h == height {
                self.selected_resolution_index = i;
                break;
            }
        }
    }
    
    pub fn get_selected_resolution(&self) -> (u32, u32) {
        self.available_resolutions[self.selected_resolution_index]
    }
    
    pub fn start_resolution_confirmation(&mut self, old_resolution: (u32, u32)) {
        self.pending_resolution = Some(old_resolution);
        self.resolution_confirm_timer = Some(300); // 5 seconds at 60 FPS
    }
    
    pub fn update_resolution_timer(&mut self) -> bool {
        // Returns true if timer expired (should revert)
        if let Some(ref mut timer) = self.resolution_confirm_timer {
            if *timer > 0 {
                *timer -= 1;
                false
            } else {
                true // Expired
            }
        } else {
            false
        }
    }
    
    pub fn confirm_resolution(&mut self) {
        self.pending_resolution = None;
        self.resolution_confirm_timer = None;
    }
    
    pub fn cancel_resolution(&mut self) -> Option<(u32, u32)> {
        let old = self.pending_resolution;
        self.pending_resolution = None;
        self.resolution_confirm_timer = None;
        old
    }
}

pub enum MenuAction {
    None,
    Resume,
    NewGame,
    Restart,
    Quit,
    OpenSettings,
    CloseSettings,
    ToggleMusic,
    ToggleSFX,
    ToggleFullscreen,
    ToggleVSync,
    ToggleGravity,
    EnterLevelEditor,
    OpenGithub,
    SelectResolution(usize), // Selected resolution index
    ConfirmResolution,
    CancelResolution,
}

pub fn handle_menu_click(menu: &Menu, mouse_x: i32, mouse_y: i32) -> MenuAction {
    match menu.state {
        MenuState::Main => {
            if menu.resume_button.is_clicked(mouse_x, mouse_y) {
                // Return different action based on game state
                return if menu.game_started {
                    MenuAction::Resume
                } else {
                    MenuAction::NewGame
                };
            }
            if menu.restart_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::Restart;
            }
            if menu.gravity_mode_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::ToggleGravity;
            }
            if menu.level_editor_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::EnterLevelEditor;
            }
            if menu.settings_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::OpenSettings;
            }
            if menu.quit_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::Quit;
            }
            if menu.github_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::OpenGithub;
            }
        }
        MenuState::Settings => {
            // Check confirmation dialog first if active
            if menu.resolution_confirm_timer.is_some() {
                if menu.confirm_button.is_clicked(mouse_x, mouse_y) {
                    return MenuAction::ConfirmResolution;
                }
                if menu.cancel_button.is_clicked(mouse_x, mouse_y) {
                    return MenuAction::CancelResolution;
                }
                // Block other interactions during confirmation
                return MenuAction::None;
            }
            
            if menu.music_toggle_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::ToggleMusic;
            }
            if menu.sfx_toggle_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::ToggleSFX;
            }
            if menu.fullscreen_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::ToggleFullscreen;
            }
            if menu.vsync_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::ToggleVSync;
            }
            // Check resolution buttons
            for (i, btn) in menu.resolution_buttons.iter().enumerate() {
                if btn.is_clicked(mouse_x, mouse_y) {
                    return MenuAction::SelectResolution(i);
                }
            }
            if menu.back_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::CloseSettings;
            }
        }

    }
    MenuAction::None
}
