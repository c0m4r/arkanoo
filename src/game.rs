use crate::entities::*;
use rand::Rng;

#[derive(PartialEq, Clone, Copy)]
pub enum GameState {
    Playing,
    Paused,
    GameOver,
    Victory,
    LevelTransition,
}

pub struct Game {
    pub state: GameState,
    pub paddle: Paddle,
    pub balls: Vec<Ball>,
    pub blocks: Vec<Block>,
    pub bonuses: Vec<Bonus>,
    pub rockets: Vec<Rocket>, // New field for rockets
    pub particles: Vec<Particle>,
    pub score: u32,
    pub lives: u32,
    pub current_level: usize,
    pub frame_count: u64,  // For animations
    pub bonus_cooldown: u64, // Frames since last bonus drop (for 5-second cooldown)
    pub lost_life_this_level: bool, // Track if player lost a life this level
    pub max_speed: f32, // Maximum ball speed ever reached (px/s)
    pub max_speed_record_frame: u64, // Frame when new record was set (for effects)
    pub portal_active: bool, // Portal activated at 3600 px/s
    pub portal_completion_timer: u64, // Frames since all blocks consumed (for animation delay)
}

#[derive(Clone, Copy)]
pub enum SoundEffect {
    Bounce,
    Oh,
    Load,
    BreakingGlass,
}

impl Game {
    pub fn new() -> Self {
        Game::new_level(1)
    }

    pub fn new_level(level: usize) -> Self {
        let paddle = Paddle::new();
        let initial_ball = Ball::new(
            WINDOW_WIDTH as f32 / 2.0,
            WINDOW_HEIGHT as f32 / 2.0,
        );
        
        Game {
            state: GameState::Playing,
            paddle,
            balls: vec![initial_ball],
            blocks: create_blocks(level),
            bonuses: Vec::new(),
            particles: Vec::new(),
            rockets: Vec::new(),
            score: 0,
            lives: 3,
            current_level: level,
            frame_count: 0,
            bonus_cooldown: 0,
            lost_life_this_level: false,
            max_speed: 0.0,
            max_speed_record_frame: 0,
            portal_active: false,
            portal_completion_timer: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Game::new();
    }

    pub fn next_level(&mut self) {
        // Restore 1 life if lost during this level (up to max 3)
        if self.lost_life_this_level && self.lives < 3 {
            self.lives += 1;
        }
        
        if self.current_level == 9 {
            self.state = GameState::Victory;
        } else {
            self.state = GameState::LevelTransition;
        }
    }    
    pub fn start_next_level(&mut self) {
        // Infinite levels - always allow progression
        self.current_level += 1;
        self.paddle = Paddle::new();
        self.balls = vec![Ball::new(
            WINDOW_WIDTH as f32 / 2.0,
            WINDOW_HEIGHT as f32 / 2.0,
        )];
        self.blocks = create_blocks(self.current_level);
        self.bonuses.clear();
        self.particles.clear();
        self.rockets.clear();
        self.state = GameState::Playing;
        self.lost_life_this_level = false; // Reset flag for new level
        self.portal_active = false; // Reset portal for new level
        self.portal_completion_timer = 0; // Reset timer for new level
        self.max_speed = 0.0; // Reset max speed so portal can trigger again
    }

    pub fn get_background_path(&self) -> String {
        format!("assets/background{}.png", self.current_level)
    }

    pub fn fire_rocket(&mut self, play_sound: &mut dyn FnMut(SoundEffect)) {
        if self.paddle.rocket_ammo > 0 {
            self.paddle.rocket_ammo -= 1;
            // Spawn rocket at center of paddle
            self.rockets.push(Rocket::new(
                self.paddle.x as f32 + self.paddle.width as f32 / 2.0 - 5.0,
                self.paddle.y as f32 - 20.0,
            ));
            play_sound(SoundEffect::Load);
        }
    }

    pub fn update(&mut self, play_sound: &mut dyn FnMut(SoundEffect)) {
        if self.state != GameState::Playing {
            return;
        }
        
        // Increment frame counter for animations
        self.frame_count = self.frame_count.wrapping_add(1);
        
        // Increment bonus cooldown
        self.bonus_cooldown = self.bonus_cooldown.saturating_add(1);

        // Update paddle
        self.paddle.update();

        // Track particles to spawn
        let mut particles_to_spawn = Vec::new();
        let mut portal_just_activated = false;

        // Update balls
        for (i, ball) in self.balls.iter_mut().enumerate() {
            // If portal is active, override physics with orbital movement
            if self.portal_active {
                let cx = WINDOW_WIDTH as f32 / 2.0;
                let cy = WINDOW_HEIGHT as f32 / 2.0;
                let radius = 150.0; // Match portal outer ring
                
                // Calculate angle based on time and ball index for distribution
                // Spin speed: 0.1 radians per frame
                let angle = (self.frame_count as f32 * 0.1) + (i as f32 * (std::f32::consts::PI * 2.0 / 3.0));
                
                ball.x = cx + angle.cos() * radius - BALL_SIZE as f32 / 2.0;
                ball.y = cy + angle.sin() * radius - BALL_SIZE as f32 / 2.0;
                
                // Skip normal physics
                continue;
            }

            ball.update();
            
            // Calculate current speed
            let speed_px_frame = (ball.vel_x.powi(2) + ball.vel_y.powi(2)).sqrt();
            let speed_px_sec = speed_px_frame * 60.0;
            
            // Check if new record
            if speed_px_sec > self.max_speed {
                self.max_speed = speed_px_sec;
                self.max_speed_record_frame = self.frame_count;
                
                // Create fancy particle burst effect for new record
                let cx = ball.x + BALL_SIZE as f32 / 2.0;
                let cy = ball.y + BALL_SIZE as f32 / 2.0;
                
                for _ in 0..30 {
                    let mut rng = rand::thread_rng();
                    let angle = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
                    let speed = rng.gen::<f32>() * 8.0 + 4.0;
                    
                    self.particles.push(Particle::new(
                        cx,
                        cy,
                        angle.cos() * speed,
                        angle.sin() * speed,
                        Color { r: 255, g: 255, b: 100 }, // Gold color for record
                    ));
                }
                
                // Activate portal at 3600 px/s (only once per level)
                if self.max_speed >= 3600.0 && !self.portal_active {
                    self.portal_active = true;
                    portal_just_activated = true;
                    
                    // Create massive particle burst for portal activation
                    let portal_x = WINDOW_WIDTH as f32 / 2.0;
                    let portal_y = WINDOW_HEIGHT as f32 / 2.0;
                    
                    for _ in 0..100 {
                        let mut rng = rand::thread_rng();
                        let angle = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
                        let speed = rng.gen::<f32>() * 15.0 + 5.0;
                        
                        self.particles.push(Particle::new(
                            portal_x,
                            portal_y,
                            angle.cos() * speed,
                            angle.sin() * speed,
                            Color { r: 150, g: 50, b: 255 }, // Purple for portal
                        ));
                    }
                }
            }
            // Paddle collision
        if ball.active && check_collision(ball.rect(), self.paddle.rect()) {
            ball.vel_y = -ball.vel_y.abs();
            // Add horizontal velocity based on where ball hits paddle
            let paddle_center = self.paddle.x + self.paddle.width / 2;
            let ball_center = ball.x as i32 + BALL_SIZE / 2;
            let offset = ball_center - paddle_center;
            ball.vel_x += offset as f32 * 0.1;
            
            // Add spin based on paddle velocity and offset
            // REFINED: Less sensitive, requires minimum velocity
            let paddle_vel = self.paddle.vel_x as f32;
            
            if paddle_vel.abs() > 2.0 {
                // Only apply spin if moving fast enough
                ball.spin = (paddle_vel * 0.3) + (offset as f32 * 0.05);
                // Trigger visual discharge effect
                self.paddle.spin_intensity = 1.0;
            } else {
                // Minimal spin from just position offset
                ball.spin = offset as f32 * 0.02;
            }
            
            // Scoring: +5 points for reflecting ball
            self.score += 5;
            play_sound(SoundEffect::Bounce);
        }

            // Block collision
            for block in &mut self.blocks {
                if block.active && ball.active && check_collision(ball.rect(), block.rect()) {
                    block.active = false;
                    
                    // Ghost Ball Logic: If ghost mode is active, ball continues through block
                    // Otherwise, it bounces
                    if self.paddle.ghost_timer == 0 {
                        ball.vel_y = -ball.vel_y;
                    }
                    
                    self.score += 10;
                    play_sound(SoundEffect::Bounce);

                    // Queue particles to spawn
                    particles_to_spawn.push((
                        block.x as f32 + BLOCK_WIDTH as f32 / 2.0,
                        block.y as f32 + BLOCK_HEIGHT as f32 / 2.0,
                        block.color,
                    ));

                    // Random bonus drop (15% chance) with 5-second cooldown
                    let mut rng = rand::thread_rng();
                    let cooldown_frames = 300; // 5 seconds at 60 FPS
                    
                    if rng.gen::<f32>() < 0.15 && self.bonus_cooldown >= cooldown_frames {
                        // Weighted bonus distribution:
                        // LongPaddle: 50%, ExtraBall: 25%, GhostBall: 15%, Rocket: 10%
                        let bonus_type = match rng.gen_range(0..100) {
                            0..=49 => BonusType::LongPaddle,     // 50%
                            50..=74 => BonusType::ExtraBall,     // 25%
                            75..=89 => BonusType::GhostBall,     // 15%
                            90..=99 => BonusType::Rocket,        // 10%
                            _ => BonusType::LongPaddle,          // Fallback to most common
                        };
                        self.bonuses.push(Bonus::new(
                            block.x as f32 + BLOCK_WIDTH as f32 / 2.0,
                            block.y as f32,
                            bonus_type,
                        ));
                        // Reset cooldown timer
                        self.bonus_cooldown = 0;
                    }
                    
                    // If not ghost mode, break after first collision to prevent destroying multiple blocks in one frame
                    // unless we want to allow corner hits. Standard breakout behavior is break.
                    if self.paddle.ghost_timer == 0 {
                        break;
                    }
                }
            }
        }

        if portal_just_activated {
            // self.balls.clear(); // Don't remove balls, let them orbit
            self.score += 5000;
        }

        // Update Rockets
        for rocket in &mut self.rockets {
            rocket.update();
            
            if rocket.active {
                // Check collision with blocks
                let mut hit_block = false;
                let mut explosion_center = (0.0, 0.0);
                
                for block in &mut self.blocks {
                    if block.active && check_collision(rocket.rect(), block.rect()) {
                        block.active = false;
                        hit_block = true;
                        explosion_center = (
                            block.x as f32 + BLOCK_WIDTH as f32 / 2.0,
                            block.y as f32 + BLOCK_HEIGHT as f32 / 2.0,
                        );
                        self.score += 10;
                        particles_to_spawn.push((explosion_center.0, explosion_center.1, block.color));
                        break; // Rocket hits one block then explodes
                    }
                }
                
                if hit_block {
                    rocket.active = false;
                    play_sound(SoundEffect::BreakingGlass); // Breaking glass sound for explosion
                    
                    // Explosion radius logic (2 blocks radius approx 120px)
                    let radius = 120.0;
                    for block in &mut self.blocks {
                        if block.active {
                            let block_center_x = block.x as f32 + BLOCK_WIDTH as f32 / 2.0;
                            let block_center_y = block.y as f32 + BLOCK_HEIGHT as f32 / 2.0;
                            let dx = block_center_x - explosion_center.0;
                            let dy = block_center_y - explosion_center.1;
                            let dist = (dx*dx + dy*dy).sqrt();
                            
                            if dist <= radius {
                                block.active = false;
                                self.score += 10;
                                particles_to_spawn.push((block_center_x, block_center_y, block.color));
                            }
                        }
                    }
                }
            }
        }

        // Create all queued particles
        for (x, y, color) in particles_to_spawn {
            self.create_particles(x, y, color);
        }

        // Update bonuses
        for bonus in &mut self.bonuses {
            bonus.update();

            // Check bonus collection
            if bonus.active && check_collision(bonus.rect(), self.paddle.rect()) {
                bonus.active = false;
                self.score += 2; // Scoring: +2 points for bonus collection
                
                match bonus.bonus_type {
                    BonusType::ExtraBall => {
                        // Add a new ball
                        self.balls.push(Ball::new(
                            self.paddle.x as f32 + self.paddle.width as f32 / 2.0,
                            self.paddle.y as f32 - 20.0,
                        ));
                    }
                    BonusType::LongPaddle => {
                        self.paddle.activate_long_bonus();
                    }
                    BonusType::GhostBall => {
                        self.paddle.activate_ghost_bonus();
                    }
                    BonusType::Rocket => {
                        self.paddle.add_rockets();
                    }
                }
            }
        }

        // Portal effect: suck blocks into center
        if self.portal_active {
            let portal_x = WINDOW_WIDTH as f32 / 2.0;
            let portal_y = WINDOW_HEIGHT as f32 / 2.0;
            
            let mut all_blocks_consumed = true;
            
            for block in &mut self.blocks {
                if block.active {
                    all_blocks_consumed = false;
                    let bx = block.x as f32 + BLOCK_WIDTH as f32 / 2.0;
                    let by = block.y as f32 + BLOCK_HEIGHT as f32 / 2.0;
                    
                    // Calculate direction to portal
                    let dx = portal_x - bx;
                    let dy = portal_y - by;
                    let dist = (dx * dx + dy * dy).sqrt();
                    
                    if dist > 5.0 {
                        // Move block toward portal
                        let speed = 8.0;
                        block.x += (dx / dist * speed) as i32;
                        block.y += (dy / dist * speed) as i32;
                    } else {
                        // Block reached portal, destroy it
                        block.active = false;
                        
                        // Spawn purple particles
                        for _ in 0..5 {
                            let mut rng = rand::thread_rng();
                            let angle = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
                            let speed = rng.gen::<f32>() * 3.0;
                            
                            self.particles.push(Particle::new(
                                portal_x,
                                portal_y,
                                angle.cos() * speed,
                                angle.sin() * speed,
                                Color { r: 150, g: 50, b: 255 },
                            ));
                        }
                    }
                }
            }
            
            // If all blocks are consumed, start completion timer
            if all_blocks_consumed {
                self.portal_completion_timer += 1;
                
                // Wait 4.5 seconds (270 frames at 60 FPS) for animation to finish
                if self.portal_completion_timer >= 270 {
                    self.next_level();
                }
            }
        }

        // Update particles
        for particle in &mut self.particles {
            particle.update();
        }

        // Remove inactive elements
        self.balls.retain(|ball| ball.active);
        self.bonuses.retain(|bonus| bonus.active);
        self.particles.retain(|p| p.is_alive());
        self.rockets.retain(|r| r.active);

        // Check if all balls are gone (only if portal is not active)
        if self.balls.is_empty() && !self.portal_active {
            self.lives -= 1;
            self.lost_life_this_level = true; // Mark that a life was lost this level
            
            // Scoring: -20 points for losing life (ensure score doesn't go negative)
            if self.score >= 20 {
                self.score -= 20;
            } else {
                self.score = 0;
            }
            
            play_sound(SoundEffect::Oh);
            
            // Heart shatter effect
            // Calculate position of the lost heart (it was at index self.lives)
            // Logic: WINDOW_WIDTH - 30 - index * 25
            // Since we just decremented lives, the lost heart index is the current self.lives value
            // e.g. had 3 lives (indices 0,1,2). Lost one -> lives=2. Lost heart was at index 2.
            let heart_x = WINDOW_WIDTH as f32 - 30.0 - (self.lives as f32 * 25.0);
            let heart_y = 15.0 + 10.0; // Center of the heart (y=15, size=20)
            
            // Crimson/Red color for heart
            let heart_color = Color { r: 220, g: 20, b: 60 };
            self.create_particles(heart_x, heart_y, heart_color);

            if self.lives == 0 {
                self.state = GameState::GameOver;
            } else {
                // Spawn new ball
                self.balls.push(Ball::new(
                    WINDOW_WIDTH as f32 / 2.0,
                    WINDOW_HEIGHT as f32 / 2.0,
                ));
            }
        }

        // Check if all blocks are destroyed (only if portal is not active)
        // If portal is active, it handles the transition after animation
        if !self.portal_active && self.blocks.iter().all(|block| !block.active) {
            self.next_level();
        }
    }

    fn create_particles(&mut self, x: f32, y: f32, color: Color) {
        let mut rng = rand::thread_rng();
        
        // Create 10-15 glass shard particles
        for _ in 0..rng.gen_range(10..16) {
            let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
            let speed = rng.gen_range(2.0..6.0);
            let vel_x = angle.cos() * speed;
            let vel_y = angle.sin() * speed - 2.0; // Slight upward bias
            
            self.particles.push(Particle::new(x, y, vel_x, vel_y, color));
        }
    }

    pub fn toggle_pause(&mut self) {
        self.state = match self.state {
            GameState::Playing => GameState::Paused,
            GameState::Paused => GameState::Playing,
            GameState::GameOver => GameState::GameOver,
            GameState::Victory => GameState::Victory,
            GameState::LevelTransition => GameState::LevelTransition,
        };
    }
}
