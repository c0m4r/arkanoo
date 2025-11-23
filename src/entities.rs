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
    GhostBall,
    Rocket,
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
    pub ghost_timer: u32, // Timer for Ghost Ball mode
    pub rocket_ammo: u32, // Ammo for Rocket bonus
    pub last_x: i32,
    pub vel_x: i32,
    pub spin_intensity: f32,
}

impl Paddle {
    pub fn new() -> Self {
        let normal_width = PADDLE_WIDTH;
        Paddle {
            x: (WINDOW_WIDTH as i32 - normal_width) / 2,
            y: WINDOW_HEIGHT as i32 - 50,
            width: normal_width,
            normal_width,
            long_width: normal_width + 40,
            bonus_timer: 0,
            ghost_timer: 0,
            rocket_ammo: 0,
            last_x: (WINDOW_WIDTH as i32 - normal_width) / 2,
            vel_x: 0,
            spin_intensity: 0.0,
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

    pub fn activate_ghost_bonus(&mut self) {
        self.ghost_timer = 600; // 10 seconds at 60 FPS
    }

    pub fn add_rockets(&mut self) {
        self.rocket_ammo += 1; // Add 1 rocket
    }

    pub fn update(&mut self) {
        self.vel_x = self.x - self.last_x;
        self.last_x = self.x;
        
        // Decay spin intensity visual effect
        self.spin_intensity *= 0.9;
        if self.spin_intensity < 0.01 {
            self.spin_intensity = 0.0;
        }

        if self.bonus_timer > 0 {
            self.bonus_timer -= 1;
            if self.bonus_timer == 0 {
                self.width = self.normal_width;
            }
        }

        if self.ghost_timer > 0 {
            self.ghost_timer -= 1;
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
    pub spin: f32,
    pub trail_positions: std::collections::VecDeque<(f32, f32)>, // Recent positions for trail effect
}

impl Ball {
    pub fn new(x: f32, y: f32) -> Self {
        Ball {
            x,
            y,
            vel_x: 4.0,
            vel_y: -4.0,
            active: true,
            spin: 0.0,
            trail_positions: std::collections::VecDeque::new(),
        }
    }

    pub fn update(&mut self) {
        if !self.active {
            return;
        }
        
        // Calculate speed
        let speed = (self.vel_x.powi(2) + self.vel_y.powi(2)).sqrt();
        let speed_px_sec = speed * 60.0; // Convert to px/s
        
        // Track trail positions based on speed
        if speed_px_sec >= 1400.0 {
            // Ultra-fast: 20 positions (2x longer trail)
            self.trail_positions.push_back((self.x, self.y));
            if self.trail_positions.len() > 20 {
                self.trail_positions.pop_front();
            }
        } else if speed_px_sec >= 800.0 {
            // Normal fast: 8 positions
            self.trail_positions.push_back((self.x, self.y));
            if self.trail_positions.len() > 8 {
                self.trail_positions.pop_front();
            }
        } else {
            // Clear trail when slowing down
            self.trail_positions.clear();
        }
        
        // Apply spin (Magnus effect approximation)
        self.vel_x += self.spin * 0.05;
        // Decay spin
        self.spin *= 0.98;
        
        self.x += self.vel_x;
        self.y += self.vel_y;

        // Wall collision with stuck prevention
        if self.x <= 0.0 {
            self.x = 0.0;
            self.vel_x = self.vel_x.abs(); // Force positive
        } else if self.x >= (WINDOW_WIDTH - BALL_SIZE as u32) as f32 {
            self.x = (WINDOW_WIDTH - BALL_SIZE as u32) as f32;
            self.vel_x = -self.vel_x.abs(); // Force negative
        }
        
        if self.y <= 0.0 {
            self.y = 0.0;
            self.vel_y = self.vel_y.abs(); // Force positive
        }

        // Bottom boundary - deactivate ball
        if self.y >= WINDOW_HEIGHT as f32 {
            self.active = false;
        }
        
        // Prevent ball from getting stuck in vertical-only movement
        // Force a minimum horizontal velocity
        if self.vel_x.abs() < 2.0 {
            if self.vel_x >= 0.0 {
                self.vel_x = 2.0;
            } else {
                self.vel_x = -2.0;
            }
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
        Rect::new(self.x as i32, self.y as i32, 40, 40)
    }
}

pub struct Rocket {
    pub x: f32,
    pub y: f32,
    pub active: bool,
}

impl Rocket {
    pub fn new(x: f32, y: f32) -> Self {
        Rocket {
            x,
            y,
            active: true,
        }
    }

    pub fn update(&mut self) {
        self.y -= 8.0; // Move up fast
        if self.y < 0.0 {
            self.active = false;
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.x as i32, self.y as i32, 10, 20)
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
            
            let should_add = if level <= 9 {
                // Predefined patterns for levels 1-9
                match level {
                    1 => true, // Level 1: Full grid
                    2 => (row + col) % 2 == 0, // Level 2: Checkerboard
                    3 => row % 2 == 0, // Level 3: Horizontal Stripes
                    4 => col % 2 == 0 || col % 2 == 1 && row == 0 || row == BLOCK_ROWS - 1,
                    5 => {
                        let center_col = BLOCK_COLS as i32 / 2;
                        let dist = (col as i32 - center_col).abs();
                        dist <= row as i32
                    },
                    6 => {
                        let center_col = BLOCK_COLS as i32 / 2;
                        let center_row = BLOCK_ROWS as i32 / 2;
                        let dist_x = (col as i32 - center_col).abs();
                        let dist_y = (row as i32 - center_row).abs();
                        dist_x + dist_y <= center_row + 2
                    },
                    7 => {
                        let center_col = BLOCK_COLS as f32 / 2.0;
                        let center_row = BLOCK_ROWS as f32 / 2.0;
                        let dx = col as f32 - center_col;
                        let dy = row as f32 - center_row;
                        let angle = dy.atan2(dx);
                        let dist = (dx * dx + dy * dy).sqrt();
                        let spiral = (angle * 2.0 + dist * 0.5).sin();
                        spiral > 0.0
                    },
                    8 => {
                        let center_col = BLOCK_COLS as f32 / 2.0;
                        let center_row = BLOCK_ROWS as f32 / 2.0;
                        let dx = col as f32 - center_col;
                        let dy = row as f32 - center_row;
                        let dist = (dx * dx + dy * dy).sqrt();
                        (dist as i32) % 3 != 1
                    },
                    9 => {
                        let pattern_x = col % 4;
                        let pattern_y = row % 4;
                        !(pattern_x == 1 && pattern_y == 1) && 
                        !(pattern_x == 2 && pattern_y == 2) &&
                        !((col + row) % 7 == 0)
                    },
                    _ => true,
                }
            } else {
                // Random patterns for levels 10+ (seeded by level number)
                use rand::{Rng, SeedableRng};
                use rand::rngs::StdRng;
                
                // Use level number as seed for consistent random patterns
                let mut rng = StdRng::seed_from_u64(level as u64);
                let pattern_type = rng.gen_range(0..6);
                
                // Re-seed for this specific block position
                let block_seed = level as u64 * 1000 + row as u64 * 100 + col as u64;
                let mut block_rng = StdRng::seed_from_u64(block_seed);
                
                match pattern_type {
                    0 => {
                        // Random scatter (60-80% density)
                        let density = rng.gen_range(0.6..0.8);
                        block_rng.gen::<f32>() < density
                    },
                    1 => {
                        // Wave pattern
                        let wave = (col as f32 * 0.5 + row as f32 * 0.3).sin();
                        let threshold = rng.gen_range(-0.3..0.3);
                        wave > threshold
                    },
                    2 => {
                        // Diagonal stripes
                        let stripe_width = rng.gen_range(2..5);
                        ((row + col) / stripe_width) % 2 == 0
                    },
                    3 => {
                        // Random rings from center
                        let center_col = BLOCK_COLS as f32 / 2.0;
                        let center_row = BLOCK_ROWS as f32 / 2.0;
                        let dx = col as f32 - center_col;
                        let dy = row as f32 - center_row;
                        let dist = (dx * dx + dy * dy).sqrt();
                        let ring_size = rng.gen_range(1.5..3.0);
                        (dist / ring_size) as i32 % 2 == 0
                    },
                    4 => {
                        // Checkerboard with random offset
                        let offset = rng.gen_range(0..3);
                        (row + col + offset) % 2 == 0
                    },
                    _ => {
                        // Cellular automata-like
                        let neighbor_sum = (row % 3) + (col % 3);
                        let rule = rng.gen_range(2..6);
                        neighbor_sum == rule || neighbor_sum == rule + 1
                    },
                }
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
