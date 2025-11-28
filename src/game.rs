use crate::entities::*;
use rand::Rng;

#[derive(PartialEq, Clone, Copy)]
pub enum GameState {
    SplashScreen,
    Playing,
    Paused,
    GameOver,
    Victory,
    LevelTransition,
    LevelEditor,
}

pub struct Game {
    pub state: GameState,
    pub paddle: Paddle,
    pub balls: Vec<Ball>,
    pub blocks: Vec<Block>,
    pub bonuses: Vec<Bonus>,
    pub rockets: Vec<Rocket>, // New field for rockets
    pub particles: Vec<Particle>,
    pub penguin: Option<Penguin>, // Penguin animation for heart theft
    pub stolen_heart_position: Option<(f32, f32)>, // Position of heart being stolen
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
    pub gravity_mode: bool, // Gravity mode enabled (heavier physics, no spin)
    pub is_test_mode: bool, // Whether we are in editor test mode
}

#[derive(Clone, Copy)]
pub enum SoundEffect {
    Bounce,
    Oh,
    Load,
    BreakingGlass,
    Explosion,
}

impl Game {
    pub fn new() -> Self {
        Game::new_level(1)
    }

    pub fn new_level(level: usize) -> Self {
        let paddle = Paddle::new();
        // Ball starts on top of paddle
        let initial_ball = Ball::new(
            paddle.x as f32 + paddle.width as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
            paddle.y as f32 - BALL_SIZE as f32,
        );
        
        Game {
            state: GameState::SplashScreen,
            paddle,
            balls: vec![initial_ball],
            blocks: create_blocks(level),
            bonuses: Vec::new(),
            particles: Vec::new(),
            rockets: Vec::new(),
            penguin: None,
            stolen_heart_position: None,
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
            gravity_mode: false,
            is_test_mode: false,
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
        self.current_level += 1;
        self.paddle = Paddle::new();
        // Ball starts on top of paddle
        self.balls = vec![Ball::new(
            self.paddle.x as f32 + self.paddle.width as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
            self.paddle.y as f32 - BALL_SIZE as f32,
        )];
        self.blocks = create_blocks(self.current_level);
        self.bonuses.clear();
        self.particles.clear();
        self.rockets.clear();
        self.penguin = None;
        self.stolen_heart_position = None;
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
    
    pub fn launch_balls(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        for ball in &mut self.balls {
            if ball.attached_to_paddle {
                ball.launch();
                
                // Create particle burst effect at launch
                let cx = ball.x + BALL_SIZE as f32 / 2.0;
                let cy = ball.y + BALL_SIZE as f32 / 2.0;
                
                for _ in 0..20 {
                    let angle = rng.gen::<f32>() * std::f32::consts::PI * 2.0;
                    let speed = rng.gen::<f32>() * 4.0 + 2.0;
                    
                    self.particles.push(Particle::new(
                        cx,
                        cy,
                        angle.cos() * speed,
                        angle.sin() * speed,
                        Color { r: 255, g: 200, b: 50 }, // Golden/yellow launch effect
                    ));
                }
            }
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

            // If ball is attached to paddle, keep it on the paddle
            if ball.attached_to_paddle {
                ball.x = self.paddle.x as f32 + self.paddle.width as f32 / 2.0 - BALL_SIZE as f32 / 2.0;
                ball.y = self.paddle.y as f32 - BALL_SIZE as f32;
            }

            ball.update(self.gravity_mode);
            
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
                
                // Icy wave trail effect
                // Calculate direction opposite to movement
                let speed_len = (ball.vel_x * ball.vel_x + ball.vel_y * ball.vel_y).sqrt();
                if speed_len > 0.1 {
                    let dir_x = -ball.vel_x / speed_len;
                    let dir_y = -ball.vel_y / speed_len;
                    
                    // Spawn a few particles behind the ball to form a trail
                    for _ in 0..5 {
                        let mut rng = rand::thread_rng();
                        
                        // Spread angle slightly for "wave" look
                        let spread_angle = (rng.gen::<f32>() - 0.5) * 1.0; // +/- 0.5 radians
                        let angle = dir_y.atan2(dir_x) + spread_angle;
                        
                        let speed = rng.gen::<f32>() * 2.0 + 1.0; // Slower, drifting particles
                        
                        // Icy colors: Cyan, Light Blue, White
                        let color = match rng.gen_range(0..12) {
                            0 => Color { r: 0, g: 255, b: 255 },   // Cyan
                            1 => Color { r: 100, g: 200, b: 255 }, // Light Blue
                            _ => Color { r: 200, g: 255, b: 255 }, // White-ish Cyan
                        };

                        self.particles.push(Particle::new(
                            cx - dir_x * 10.0, // Spawn slightly behind center
                            cy - dir_y * 10.0,
                            angle.cos() * speed,
                            angle.sin() * speed,
                            color,
                        ));
                    }
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
        }
        
        // Ball-to-ball collisions (only when not in portal mode)
        if !self.portal_active {
            // Collect collision data first to avoid borrow issues
            let mut collisions: Vec<(usize, usize, f32, f32)> = Vec::new(); // i, j, collision_x, collision_y
            
            for i in 0..self.balls.len() {
                for j in (i + 1)..self.balls.len() {
                    if self.balls[i].active && self.balls[j].active {
                        let ball1 = &self.balls[i];
                        let ball2 = &self.balls[j];
                        
                        let dx = ball2.x - ball1.x;
                        let dy = ball2.y - ball1.y;
                        let distance = (dx * dx + dy * dy).sqrt();
                        let min_dist = BALL_SIZE as f32;
                        
                        if distance < min_dist {
                            // Collision detected!
                            let collision_x = ball1.x + dx / 2.0;
                            let collision_y = ball1.y + dy / 2.0;
                            collisions.push((i, j, collision_x, collision_y));
                        }
                    }
                }
            }
            
            // Apply collision responses
            for (i, j, col_x, col_y) in collisions {
                // 1. Eject Upwards & Separate Horizontally
                {
                    let ball1 = &mut self.balls[i];
                    ball1.vel_y = -ball1.vel_y.abs().max(8.0); // Force UP, min speed 8.0
                    // Push left if it was on the left, or just random/away
                    ball1.vel_x = if ball1.x < col_x { -5.0 } else { 5.0 };
                }
                {
                    let ball2 = &mut self.balls[j];
                    ball2.vel_y = -ball2.vel_y.abs().max(8.0); // Force UP, min speed 8.0
                    ball2.vel_x = if ball2.x < col_x { -5.0 } else { 5.0 };
                }

                // 2. Sonic Boom Effect (Expanding Ring)
                // Spawn 36 particles in a circle expanding outward
                for k in 0..36 {
                    let angle = (k as f32 * 10.0).to_radians();
                    let speed = 6.0; // Fast expansion
                    
                    self.particles.push(Particle::new(
                        col_x + BALL_SIZE as f32 / 2.0, 
                        col_y + BALL_SIZE as f32 / 2.0,
                        angle.cos() * speed,
                        angle.sin() * speed,
                        Color { r: 200, g: 255, b: 255 }, // Cyan/White shockwave
                    ));
                }
                
                // Play collision sound
                play_sound(SoundEffect::Bounce);
            }
        }

        // Paddle and block collisions (per ball, still inside original ball iteration context)
        for ball in &mut self.balls {
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
            let mut explosions = Vec::new();
            for block in &mut self.blocks {
                if !block.active || !ball.active {
                    continue;
                }

                if let Some(overlap) = ball.rect().intersection(block.rect()) {
                    // Handle block hit based on type
                    let destroyed = match block.block_type {
                        BlockType::Undestroyable => {
                            play_sound(SoundEffect::Bounce); // Metal sound ideally
                            false
                        },
                        BlockType::Ice => {
                            block.health -= 1;
                            if block.health == 0 {
                                true
                            } else {
                                play_sound(SoundEffect::BreakingGlass); // Crack sound
                                false
                            }
                        },
                        BlockType::Explosive => {
                            true // Explodes immediately
                        },
                        BlockType::Normal => {
                            true
                        }
                    };

                    if destroyed {
                        block.active = false;
                    }
                    
                    // Collision Resolution
                    // If ghost mode is ON, we pass through EVERYTHING (no bounce).
                    // Otherwise (ghost mode OFF), we bounce off EVERYTHING (even if destroyed).
                    let should_bounce = self.paddle.ghost_timer == 0;
                    
                    if should_bounce {
                        // Determine collision side based on overlap dimensions
                        // Smaller overlap dimension indicates the axis of collision
                        if overlap.width() < overlap.height() {
                            // Horizontal Collision (Side hit)
                            // Push ball out horizontally
                            if ball.x + (BALL_SIZE as f32 / 2.0) < block.x as f32 + (BLOCK_WIDTH as f32 / 2.0) {
                                // Hit from left
                                ball.x -= overlap.width() as f32;
                            } else {
                                // Hit from right
                                ball.x += overlap.width() as f32;
                            }
                            ball.vel_x = -ball.vel_x;
                        } else {
                            // Vertical Collision (Top/Bottom hit)
                            // Push ball out vertically
                            if ball.y + (BALL_SIZE as f32 / 2.0) < block.y as f32 + (BLOCK_HEIGHT as f32 / 2.0) {
                                // Hit from top
                                ball.y -= overlap.height() as f32;
                            } else {
                                // Hit from bottom
                                ball.y += overlap.height() as f32;
                            }
                            ball.vel_y = -ball.vel_y;
                        }
                    }
                    
                    if destroyed {
                        self.score += 10;
                        play_sound(SoundEffect::Bounce);

                        // Queue particles to spawn
                        particles_to_spawn.push((
                            block.x as f32 + BLOCK_WIDTH as f32 / 2.0,
                            block.y as f32 + BLOCK_HEIGHT as f32 / 2.0,
                            block.color,
                        ));
                        
                        // Handle Explosion
                        if block.block_type == BlockType::Explosive {
                             // Explosion radius logic (2 blocks radius approx 120px)
                            let explosion_center = (
                                block.x as f32 + BLOCK_WIDTH as f32 / 2.0,
                                block.y as f32 + BLOCK_HEIGHT as f32 / 2.0,
                            );
                            explosions.push(explosion_center);
                        }
                    }

                    // Random bonus drop (15% chance) with 1-second cooldown
                    // Only drop bonuses from destroyed blocks
                    if destroyed {
                        let mut rng = rand::thread_rng();
                        let cooldown_frames = 60; // 1 seconds at 60 FPS
                        
                        if rng.gen::<f32>() < 0.15 && self.bonus_cooldown >= cooldown_frames {
                            // Weighted bonus distribution:
                            // LongPaddle: 50%, ExtraBall: 25%, GhostBall: 15%, Rocket: 10%
                            let bonus_type = match rng.gen_range(0..100) {
                                0..=49 => BonusType::LongPaddle,     // 40%
                                50..=74 => BonusType::ExtraBall,     // 35%
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
                    }
                    
                    // If not ghost mode, break after first collision to prevent destroying multiple blocks in one frame
                    // unless we want to allow corner hits. Standard breakout behavior is break.
                    if self.paddle.ghost_timer == 0 {
                        break;
                    }
                }
            }

            // Process explosions
            for (exp_x, exp_y) in explosions {
                // Explosion radius: 120px (approx 2 blocks)
                let radius_sq = 60.0 * 60.0;
                
                for block in &mut self.blocks {
                    if !block.active {
                        continue;
                    }
                    
                    let block_center_x = block.x as f32 + BLOCK_WIDTH as f32 / 2.0;
                    let block_center_y = block.y as f32 + BLOCK_HEIGHT as f32 / 2.0;
                    
                    let dx = block_center_x - exp_x;
                    let dy = block_center_y - exp_y;
                    let dist_sq = dx*dx + dy*dy;
                    
                    if dist_sq <= radius_sq {
                        // Destroy block
                        block.active = false;
                        self.score += 10;
                        
                        // Add particles for destroyed block
                        particles_to_spawn.push((
                            block_center_x,
                            block_center_y,
                            block.color,
                        ));
                    }
                }
                
                // Play explosion sound
                play_sound(SoundEffect::Explosion);
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

        // Update penguin animation
        if let Some(ref mut penguin) = self.penguin {
            penguin.update();
            
            // Clear stolen heart when penguin grabs it
            if penguin.state == PenguinState::Grabbing && self.stolen_heart_position.is_some() {
                self.stolen_heart_position = None;
            }
            
            // Remove penguin when animation is done
            if penguin.is_done() {
                self.penguin = None;
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
            
            // Penguin animation instead of heart shatter particles
            // Calculate position of the lost heart (it was at index self.lives)
            // Logic: WINDOW_WIDTH - 30 - index * 25
            // Since we just decremented lives, the lost heart index is the current self.lives value
            // e.g. had 3 lives (indices 0,1,2). Lost one -> lives=2. Lost heart was at index 2.
            let heart_x = WINDOW_WIDTH as f32 - 30.0 - (self.lives as f32 * 25.0);
            let heart_y = 25.0; // Heart center Y position
            
            // Store the stolen heart position so it stays visible
            self.stolen_heart_position = Some((heart_x, heart_y));
            
            // Spawn penguin to steal the heart
            self.penguin = Some(Penguin::new(heart_x, heart_y));


            if self.lives == 0 {
                self.state = GameState::GameOver;
            } else {
                // Spawn new ball on paddle
                self.balls.push(Ball::new(
                    self.paddle.x as f32 + self.paddle.width as f32 / 2.0 - BALL_SIZE as f32 / 2.0,
                    self.paddle.y as f32 - BALL_SIZE as f32,
                ));
            }
        }

        // Check if all destroyable blocks are destroyed (only if portal is not active)
        // If portal is active, it handles the transition after animation
        if !self.portal_active && self.blocks.iter().all(|block| !block.active || block.block_type == BlockType::Undestroyable) {
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
            GameState::SplashScreen => GameState::SplashScreen,
            GameState::LevelEditor => GameState::LevelEditor,
        };
    }

    pub fn toggle_gravity_mode(&mut self) {
        self.gravity_mode = !self.gravity_mode;
    }
}
