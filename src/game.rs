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
    pub particles: Vec<Particle>,
    pub score: u32,
    pub lives: u32,
    pub current_level: usize,
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
            score: 0,
            lives: 3,
            current_level: level,
        }
    }

    pub fn reset(&mut self) {
        *self = Game::new();
    }

    pub fn next_level(&mut self) {
        if self.current_level < 6 {
            self.state = GameState::LevelTransition;
        } else {
            self.state = GameState::Victory;
        }
    }
    
    pub fn start_next_level(&mut self) {
        if self.current_level < 6 {
            self.current_level += 1;
            self.paddle = Paddle::new();
            self.balls = vec![Ball::new(
                WINDOW_WIDTH as f32 / 2.0,
                WINDOW_HEIGHT as f32 / 2.0,
            )];
            self.blocks = create_blocks(self.current_level);
            self.bonuses.clear();
            self.particles.clear();
            self.state = GameState::Playing;
        }
    }

    pub fn get_background_path(&self) -> String {
        format!("assets/background{}.png", self.current_level)
    }

    pub fn update(&mut self, play_sound: &mut dyn FnMut()) {
        if self.state != GameState::Playing {
            return;
        }

        // Update paddle
        self.paddle.update();

        // Track particles to spawn
        let mut particles_to_spawn = Vec::new();

        // Update balls
        for ball in &mut self.balls {
            ball.update();

            // Paddle collision
            if ball.active && check_collision(ball.rect(), self.paddle.rect()) {
                ball.vel_y = -ball.vel_y.abs();
                // Add horizontal velocity based on where ball hits paddle
                let paddle_center = self.paddle.x + self.paddle.width / 2;
                let ball_center = ball.x as i32 + BALL_SIZE / 2;
                let offset = ball_center - paddle_center;
                ball.vel_x += offset as f32 * 0.1;
                play_sound();
            }

            // Block collision
            for block in &mut self.blocks {
                if block.active && ball.active && check_collision(ball.rect(), block.rect()) {
                    block.active = false;
                    ball.vel_y = -ball.vel_y;
                    self.score += 10;
                    play_sound();

                    // Queue particles to spawn
                    particles_to_spawn.push((
                        block.x as f32 + BLOCK_WIDTH as f32 / 2.0,
                        block.y as f32 + BLOCK_HEIGHT as f32 / 2.0,
                        block.color,
                    ));

                    // Random bonus drop (15% chance)
                    let mut rng = rand::thread_rng();
                    if rng.gen::<f32>() < 0.15 {
                        let bonus_type = if rng.gen::<bool>() {
                            BonusType::ExtraBall
                        } else {
                            BonusType::LongPaddle
                        };
                        self.bonuses.push(Bonus::new(
                            block.x as f32 + BLOCK_WIDTH as f32 / 2.0,
                            block.y as f32,
                            bonus_type,
                        ));
                    }
                    break;
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

        // Check if all balls are gone
        if self.balls.is_empty() {
            self.lives -= 1;
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

        // Check if all blocks are destroyed
        if self.blocks.iter().all(|block| !block.active) {
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
