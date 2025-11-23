use sdl2::rect::Rect;

#[derive(Clone, Copy, PartialEq)]
pub enum MenuState {
    Main,
    Settings,
    ResolutionConfirm,
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
    pub back_button: Button,
    pub quit_button: Button,
    pub mute_button: Button,
    pub resolution_button: Button,
    pub fullscreen_button: Button,
    pub volume_slider: VolumeSlider,
    pub confirm_button: Button,
    pub cancel_button: Button,
    pub muted: bool,
    pub is_fullscreen: bool,
    pub resolutions: Vec<(u32, u32)>,
    pub current_resolution_idx: usize,
    pub pending_resolution_idx: Option<usize>,
    pub confirmation_timer: f32,
}

impl Menu {
    pub fn new(window_width: u32, window_height: u32) -> Self {
        let center_x = window_width as i32 / 2 - 100;
        let center_y = window_height as i32 / 2;

        Menu {
            state: MenuState::Main,
            resume_button: Button::new(center_x, center_y - 80, 200, 40, "Resume"),
            restart_button: Button::new(center_x, center_y - 30, 200, 40, "Restart"),
            settings_button: Button::new(center_x, center_y + 20, 200, 40, "Settings"),
            quit_button: Button::new(center_x, center_y + 70, 200, 40, "Quit"),
            back_button: Button::new(center_x, center_y + 120, 200, 40, "Back"),
            mute_button: Button::new(center_x, center_y - 80, 200, 40, "Sound: ON"),
            resolution_button: Button::new(center_x, center_y - 30, 200, 40, "1280x720"),
            fullscreen_button: Button::new(center_x, center_y + 70, 200, 40, "Windowed"),
            volume_slider: VolumeSlider::new(center_x, center_y + 20, 200),
            confirm_button: Button::new(center_x - 110, center_y + 20, 100, 40, "Keep"),
            cancel_button: Button::new(center_x + 10, center_y + 20, 100, 40, "Revert"),
            muted: false,
            is_fullscreen: false,
            resolutions: vec![
                (1280, 720),
                (1920, 1080),
                (2560, 1440),
            ],
            current_resolution_idx: 0,
            pending_resolution_idx: None,
            confirmation_timer: 0.0,
        }
    }

    pub fn update_hover(&mut self, mouse_x: i32, mouse_y: i32) {
        match self.state {
            MenuState::Main => {
                self.resume_button.update_hover(mouse_x, mouse_y);
                self.restart_button.update_hover(mouse_x, mouse_y);
                self.settings_button.update_hover(mouse_x, mouse_y);
                self.quit_button.update_hover(mouse_x, mouse_y);
            }
            MenuState::Settings => {
                self.mute_button.update_hover(mouse_x, mouse_y);
                self.resolution_button.update_hover(mouse_x, mouse_y);
                self.fullscreen_button.update_hover(mouse_x, mouse_y);
                self.back_button.update_hover(mouse_x, mouse_y);
            }
            MenuState::ResolutionConfirm => {
                self.confirm_button.update_hover(mouse_x, mouse_y);
                self.cancel_button.update_hover(mouse_x, mouse_y);
            }
        }
    }

    pub fn update_slider(&mut self, mouse_x: i32, mouse_y: i32, mouse_down: bool) {
        if self.state == MenuState::Settings {
            self.volume_slider.update(mouse_x, mouse_y, mouse_down);
        }
    }

    pub fn update_timer(&mut self, dt: f32) -> bool {
        if self.state == MenuState::ResolutionConfirm {
            self.confirmation_timer -= dt;
            if self.confirmation_timer <= 0.0 {
                return true; // Timer expired, revert
            }
        }
        false
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        self.mute_button.label = if muted {
            "Sound: OFF".to_string()
        } else {
            "Sound: ON".to_string()
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
}

pub enum MenuAction {
    None,
    Resume,
    Restart,
    Quit,
    OpenSettings,
    CloseSettings,
    ToggleMute,
    CycleResolution,
    ToggleFullscreen,
    ConfirmResolution,
    RevertResolution,
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
            if menu.settings_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::OpenSettings;
            }
            if menu.quit_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::Quit;
            }
        }
        MenuState::Settings => {
            if menu.mute_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::ToggleMute;
            }
            if menu.resolution_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::CycleResolution;
            }
            if menu.fullscreen_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::ToggleFullscreen;
            }
            if menu.back_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::CloseSettings;
            }
        }
        MenuState::ResolutionConfirm => {
            if menu.confirm_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::ConfirmResolution;
            }
            if menu.cancel_button.is_clicked(mouse_x, mouse_y) {
                return MenuAction::RevertResolution;
            }
        }
    }
    MenuAction::None
}
