use sdl2::rect::Rect;

#[derive(Clone, Copy, PartialEq)]
pub enum MenuState {
    Main,
    Settings,

}

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

    pub fullscreen_button: Button,
    pub gravity_mode_button: Button,
    pub music_slider: VolumeSlider,
    pub sfx_slider: VolumeSlider,

    pub music_muted: bool,
    pub sfx_muted: bool,
    pub is_fullscreen: bool,
    pub gravity_mode: bool,
}

impl Menu {
    pub fn new(window_width: u32, window_height: u32) -> Self {
        let center_x = window_width as i32 / 2 - 100;
        let center_y = window_height as i32 / 2;

        Menu {
            state: MenuState::Main,
            resume_button: Button::new(center_x, center_y - 125, 200, 40, "Resume"),
            restart_button: Button::new(center_x, center_y - 75, 200, 40, "Restart"),
            gravity_mode_button: Button::new(center_x, center_y - 25, 200, 40, "Gravity Mode"),
            level_editor_button: Button::new(center_x, center_y + 25, 200, 40, "Level Editor"),
            settings_button: Button::new(center_x, center_y + 75, 200, 40, "Settings"),
            quit_button: Button::new(center_x, center_y + 125, 200, 40, "Quit"),
            back_button: Button::new(center_x, center_y + 170, 200, 40, "Back"),
            music_toggle_button: Button::new(center_x, center_y - 120, 200, 40, "Music: ON"),
            sfx_toggle_button: Button::new(center_x, center_y - 20, 200, 40, "SFX: ON"),

            fullscreen_button: Button::new(center_x, center_y + 130, 200, 40, "Windowed"),
            music_slider: VolumeSlider::new(center_x, center_y - 70, 200),
            sfx_slider: VolumeSlider::new(center_x, center_y + 30, 200),

            music_muted: false,
            sfx_muted: false,
            is_fullscreen: false,
            gravity_mode: false,
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
            }
            MenuState::Settings => {
                self.music_toggle_button.update_hover(mouse_x, mouse_y);
                self.sfx_toggle_button.update_hover(mouse_x, mouse_y);
                self.fullscreen_button.update_hover(mouse_x, mouse_y);
                self.back_button.update_hover(mouse_x, mouse_y);
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
}

pub enum MenuAction {
    None,
    Resume,
    Restart,
    Quit,
    OpenSettings,
    CloseSettings,
    ToggleMusic,
    ToggleSFX,
    ToggleFullscreen,
    ToggleGravity,
    EnterLevelEditor,
}

pub fn handle_menu_click(menu: &Menu, mouse_x: i32, mouse_y: i32) -> MenuAction {
    match menu.state {
        MenuState::Main => {
            if menu.resume_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::Resume;
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
        }
        MenuState::Settings => {
            if menu.music_toggle_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::ToggleMusic;
            }
            if menu.sfx_toggle_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::ToggleSFX;
            }
            if menu.fullscreen_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::ToggleFullscreen;
            }
            if menu.back_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::CloseSettings;
            }
        }

    }
    MenuAction::None
}
