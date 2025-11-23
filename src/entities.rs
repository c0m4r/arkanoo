use sdl2::rect::Rect;

/// Game constants
pub const WINDOW_WIDTH: u32 = 1280;
pub const WINDOW_HEIGHT: u32 = 720;
pub const PADDLE_WIDTH: i32 = 140;
pub const PADDLE_HEIGHT: i32 = 22;
pub const PADDLE_SPEED: i32 = 18;
pub const BALL_SIZE: i32 = 12;
pub const BALL_SPEED: i32 = 6;
pub const BLOCK_WIDTH: i32 = 60;
pub const BLOCK_HEIGHT: i32 = 20;
pub const BLOCK_ROWS: usize = 10;
pub const BLOCK_COLS: usize = 20;
pub const BLOCK_OFFSET_Y: i32 = 80;

#[derive(Clone, Copy, PartialEq)]
pub enum BonusType {
    ExtraBall,
    LongPaddle,
}

#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
}

pub struct Paddle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub normal_width: i32,
    pub long_width: i32,
    pub bonus_timer: u32,
}

impl Paddle {
    pub fn new() -> Self {
        let normal_width = PADDLE_WIDTH;
        Paddle {
            x: (WINDOW_WIDTH as i32 - normal_width) / 2,
            y: WINDOW_HEIGHT as i32 - 40,
            width: normal_width,
            normal_width,
            long_width: normal_width + 40,
            bonus_timer: 0,
        }
    }

    pub fn move_left(&mut self) {
        self.x = (self.x - PADDLE_SPEED).max(0);
    }

    pub fn move_right(&mut self) {
        self.x = (self.x + PADDLE_SPEED).min(WINDOW_WIDTH as i32 - self.width);
    }
    
    pub fn set_x(&mut self, x: i32) {
        self.x = x.clamp(0, WINDOW_WIDTH as i32 - self.width);
    }

    pub fn activate_long_bonus(&mut self) {
        self.width = self.long_width;
        self.bonus_timer = 300; // 5 seconds at 60 FPS
    }

    pub fn update(&mut self) {
        if self.bonus_timer > 0 {
            self.bonus_timer -= 1;
            if self.bonus_timer == 0 {
                self.width = self.normal_width;
            }
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width as u32, PADDLE_HEIGHT as u32)
    }
}

pub struct Ball {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub active: bool,
}

impl Ball {
    pub fn new(x: f32, y: f32) -> Self {
        Ball {
            x,
            y,
            vel_x: BALL_SPEED as f32,
            vel_y: -BALL_SPEED as f32,
            active: true,
        }
    }

    pub fn update(&mut self) {
        if !self.active {
            return;
        }
        
        self.x += self.vel_x;
        self.y += self.vel_y;

        // Wall collision
        if self.x <= 0.0 || self.x >= (WINDOW_WIDTH - BALL_SIZE as u32) as f32 {
            self.vel_x = -self.vel_x;
        }
        if self.y <= 0.0 {
            self.vel_y = -self.vel_y;
        }

        // Bottom boundary - deactivate ball
        if self.y >= WINDOW_HEIGHT as f32 {
            self.active = false;
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.x as i32, self.y as i32, BALL_SIZE as u32, BALL_SIZE as u32)
    }
}

pub struct Block {
    pub x: i32,
    pub y: i32,
    pub color: Color,
    pub active: bool,
}

impl Block {
    pub fn new(x: i32, y: i32, color: Color) -> Self {
        Block {
            x,
            y,
            color,
            active: true,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, BLOCK_WIDTH as u32, BLOCK_HEIGHT as u32)
    }
}

pub struct Bonus {
    pub x: f32,
    pub y: f32,
    pub bonus_type: BonusType,
    pub active: bool,
}

impl Bonus {
    pub fn new(x: f32, y: f32, bonus_type: BonusType) -> Self {
        Bonus {
            x,
            y,
            bonus_type,
            active: true,
        }
    }

    pub fn update(&mut self) {
        self.y += 2.0;
        if self.y > WINDOW_HEIGHT as f32 {
            self.active = false;
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.x as i32, self.y as i32, 20, 20)
    }
}

/// Particle for glass-shattering effect
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub rotation: f32,
    pub rotation_speed: f32,
    pub lifetime: u32,
    pub max_lifetime: u32,
    pub size: i32,
    pub color: Color,
}

impl Particle {
    pub fn new(x: f32, y: f32, vel_x: f32, vel_y: f32, color: Color) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        Particle {
            x,
            y,
            vel_x,
            vel_y,
            rotation: rng.gen_range(0.0..360.0),
            rotation_speed: rng.gen_range(-10.0..10.0),
            lifetime: 0,
            max_lifetime: rng.gen_range(20..40),
            size: rng.gen_range(3..8),
            color,
        }
    }

    pub fn update(&mut self) {
        self.x += self.vel_x;
        self.y += self.vel_y;
        self.vel_y += 0.3; // Gravity
        self.rotation += self.rotation_speed;
        self.lifetime += 1;
    }

    pub fn is_alive(&self) -> bool {
        self.lifetime < self.max_lifetime
    }

    pub fn alpha(&self) -> u8 {
        let progress = self.lifetime as f32 / self.max_lifetime as f32;
        ((1.0 - progress) * 255.0) as u8
    }
}

// Block colors (rainbow pattern)
pub const BLOCK_COLORS: [Color; 6] = [
    Color::new(255, 0, 0),     // Red
    Color::new(255, 165, 0),   // Orange
    Color::new(255, 255, 0),   // Yellow
    Color::new(0, 255, 0),     // Green
    Color::new(0, 0, 255),     // Blue
    Color::new(138, 43, 226),  // Violet
];

pub fn create_blocks(level: usize) -> Vec<Block> {
    let mut blocks = Vec::new();
    let total_blocks_width = BLOCK_COLS as i32 * BLOCK_WIDTH;
    let offset_x = (WINDOW_WIDTH as i32 - total_blocks_width) / 2;

    for row in 0..BLOCK_ROWS {
        for col in 0..BLOCK_COLS {
            let x = offset_x + col as i32 * BLOCK_WIDTH;
            let y = BLOCK_OFFSET_Y + row as i32 * BLOCK_HEIGHT;
            let color = BLOCK_COLORS[row % BLOCK_COLORS.len()];
            
            let should_add = match level {
                1 => true, // Level 1: Full grid
                2 => (row + col) % 2 == 0, // Level 2: Checkerboard
                3 => row % 2 == 0, // Level 3: Horizontal Stripes
                4 => col % 2 == 0 || col % 2 == 1 && row == 0 || row == BLOCK_ROWS - 1, // Level 4: Pillars with top/bottom
                5 => {
                    // Level 5: Pyramid / Triangle
                    let center_col = BLOCK_COLS as i32 / 2;
                    let dist = (col as i32 - center_col).abs();
                    dist <= row as i32
                },
                6 => {
                    // Level 6: Diamond / X shape
                    let center_col = BLOCK_COLS as i32 / 2;
                    let center_row = BLOCK_ROWS as i32 / 2;
                    let dist_x = (col as i32 - center_col).abs();
                    let dist_y = (row as i32 - center_row).abs();
                    dist_x + dist_y <= center_row + 2
                },
                _ => true, // Default
            };

            if should_add {
                blocks.push(Block::new(x, y, color));
            }
        }
    }

    blocks
}

pub fn check_collision(rect1: Rect, rect2: Rect) -> bool {
    rect1.has_intersection(rect2)
}
