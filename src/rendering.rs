use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::pixels::{Color as SdlColor, PixelFormatEnum};
use sdl2::rect::{Rect, Point};
use sdl2::ttf::Font;
use crate::game::{Game, GameState};
use crate::entities::*;
use crate::menu::{Menu, MenuState, Button, VolumeSlider};

pub struct TextureCache<'a> {
    pub ball: Texture<'a>,
    pub paddle_normal: Texture<'a>,
    pub paddle_long: Texture<'a>,
    pub blocks: Vec<Texture<'a>>,
    pub speed_text: Option<Texture<'a>>,
    pub last_speed_text: String,
    pub creator: &'a TextureCreator<WindowContext>,
}

impl<'a> TextureCache<'a> {
    pub fn new(canvas: &mut Canvas<Window>, texture_creator: &'a TextureCreator<WindowContext>) -> Result<Self, String> {
        // Create ball texture
        let mut ball = texture_creator.create_texture_target(PixelFormatEnum::RGBA8888, BALL_SIZE as u32, BALL_SIZE as u32)
            .map_err(|e| e.to_string())?;
        ball.set_blend_mode(sdl2::render::BlendMode::Blend);
        
        canvas.with_texture_canvas(&mut ball, |canvas| {
            canvas.set_draw_color(SdlColor::RGBA(0, 0, 0, 0));
            canvas.clear();
            draw_shiny_ball_texture(canvas);
        }).map_err(|e| e.to_string())?;

        // Create paddle textures
        let mut paddle_normal = texture_creator.create_texture_target(PixelFormatEnum::RGBA8888, PADDLE_WIDTH as u32, PADDLE_HEIGHT as u32)
            .map_err(|e| e.to_string())?;
        paddle_normal.set_blend_mode(sdl2::render::BlendMode::Blend);

        canvas.with_texture_canvas(&mut paddle_normal, |canvas| {
            canvas.set_draw_color(SdlColor::RGBA(0, 0, 0, 0));
            canvas.clear();
            draw_paddle_texture(canvas, PADDLE_WIDTH, PADDLE_HEIGHT);
        }).map_err(|e| e.to_string())?;

        let long_width = PADDLE_WIDTH + 40;
        let mut paddle_long = texture_creator.create_texture_target(PixelFormatEnum::RGBA8888, long_width as u32, PADDLE_HEIGHT as u32)
            .map_err(|e| e.to_string())?;
        paddle_long.set_blend_mode(sdl2::render::BlendMode::Blend);

        canvas.with_texture_canvas(&mut paddle_long, |canvas| {
            canvas.set_draw_color(SdlColor::RGBA(0, 0, 0, 0));
            canvas.clear();
            draw_paddle_texture(canvas, long_width, PADDLE_HEIGHT);
        }).map_err(|e| e.to_string())?;

        // Create block textures
        let mut blocks = Vec::new();
        for color in BLOCK_COLORS.iter() {
            let mut block = texture_creator.create_texture_target(PixelFormatEnum::RGBA8888, BLOCK_WIDTH as u32, BLOCK_HEIGHT as u32)
                .map_err(|e| e.to_string())?;
            block.set_blend_mode(sdl2::render::BlendMode::Blend);

            canvas.with_texture_canvas(&mut block, |canvas| {
                canvas.set_draw_color(SdlColor::RGBA(0, 0, 0, 0));
                canvas.clear();
                draw_block_texture(canvas, *color);
            }).map_err(|e| e.to_string())?;
            blocks.push(block);
        }

        Ok(TextureCache {
            ball,
            paddle_normal,
            paddle_long,
            blocks,
            speed_text: None,
            last_speed_text: String::new(),
            creator: texture_creator,
        })
    }
}

// Helper functions for texture generation (moved from original draw functions)

fn draw_shiny_ball_texture(canvas: &mut Canvas<Window>) {
    let radius = BALL_SIZE / 2;
    let cx = radius;
    let cy = radius;

    // Draw filled circle with gradient
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= radius * radius {
                let dist = (dist_sq as f32).sqrt();
                let factor = 1.0 - (dist / radius as f32);
                
                let brightness = (160.0 + factor * 95.0) as u8;
                
                let highlight_x = dx + radius / 2;
                let highlight_y = dy + radius / 2;
                let highlight_dist_sq = highlight_x * highlight_x + highlight_y * highlight_y;
                let highlight = if highlight_dist_sq < (radius * radius) / 4 {
                    ((1.0 - (highlight_dist_sq as f32).sqrt() / (radius as f32 / 2.0)) * 100.0) as u8
                } else {
                    0
                };
                
                let final_brightness = (brightness as u16 + highlight as u16).min(255) as u8;
                canvas.set_draw_color(SdlColor::RGB(final_brightness, final_brightness, final_brightness));
                let _ = canvas.draw_point(Point::new(cx + dx, cy + dy));
            }
        }
    }
}

fn draw_paddle_texture(canvas: &mut Canvas<Window>, w: i32, h: i32) {
    let radius = 10;
    
    // 1. Main Body
    canvas.set_draw_color(SdlColor::RGB(40, 50, 70));
    let _ = canvas.fill_rect(Rect::new(radius, 0, (w - 2 * radius) as u32, h as u32));
    let _ = canvas.fill_rect(Rect::new(0, radius, radius as u32, (h - 2 * radius) as u32));
    let _ = canvas.fill_rect(Rect::new(w - radius, radius, radius as u32, (h - 2 * radius) as u32));
    
    let corners = [
        (radius, radius),
        (w - radius, radius),
        (radius, h - radius),
        (w - radius, h - radius),
    ];
    
    for &(cx, cy) in &corners {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx*dx + dy*dy <= radius*radius {
                    let _ = canvas.draw_point(Point::new(cx + dx, cy + dy));
                }
            }
        }
    }
    
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

    // 3. Metallic Gradient on body
    for line_y in 0..h {
        if line_y >= (h - 4) / 2 && line_y < (h + 4) / 2 {
            continue; // Skip core area
        }
        
        let factor = line_y as f32 / h as f32;
        let alpha = ((1.0 - (factor - 0.5).abs() * 2.0) * 100.0) as u8;
        canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, alpha));
        
        let mut start_x = 0;
        let mut end_x = w;
        
        if line_y < radius {
            let dy = radius - line_y;
            let dx = ((radius * radius - dy * dy) as f32).sqrt() as i32;
            start_x = radius - dx;
            end_x = w - radius + dx;
        } else if line_y >= h - radius {
            let dy = line_y - (h - radius);
            let dx = ((radius * radius - dy * dy) as f32).sqrt() as i32;
            start_x = radius - dx;
            end_x = w - radius + dx;
        }
        
        let _ = canvas.draw_line(
            Point::new(start_x, line_y),
            Point::new(end_x, line_y),
        );
    }

    // 5. Tech Borders (Outline)
    canvas.set_draw_color(SdlColor::RGBA(100, 200, 255, 150));
    let _ = canvas.draw_line(Point::new(radius, 0), Point::new(w - radius, 0));
    let _ = canvas.draw_line(Point::new(radius, h - 1), Point::new(w - radius, h - 1));
    let _ = canvas.draw_line(Point::new(0, radius), Point::new(0, h - radius));
    let _ = canvas.draw_line(Point::new(w - 1, radius), Point::new(w - 1, h - radius));
    
    // Corner arcs
    for i in 0..=90 {
        let rad = (i as f32 + 180.0).to_radians();
        let px = radius + (radius as f32 * rad.cos()) as i32;
        let py = radius + (radius as f32 * rad.sin()) as i32;
        let _ = canvas.draw_point(Point::new(px, py));
    }
    for i in 0..=90 {
        let rad = (i as f32 + 270.0).to_radians();
        let px = w - radius + (radius as f32 * rad.cos()) as i32;
        let py = radius + (radius as f32 * rad.sin()) as i32;
        let _ = canvas.draw_point(Point::new(px, py));
    }
    for i in 0..=90 {
        let rad = (i as f32).to_radians();
        let px = w - radius + (radius as f32 * rad.cos()) as i32;
        let py = h - radius + (radius as f32 * rad.sin()) as i32;
        let _ = canvas.draw_point(Point::new(px, py));
    }
    for i in 0..=90 {
        let rad = (i as f32 + 90.0).to_radians();
        let px = radius + (radius as f32 * rad.cos()) as i32;
        let py = h - radius + (radius as f32 * rad.sin()) as i32;
        let _ = canvas.draw_point(Point::new(px, py));
    }
}

fn draw_block_texture(canvas: &mut Canvas<Window>, color: Color) {
    let w = BLOCK_WIDTH;
    let h = BLOCK_HEIGHT;
    
    // 1. Base fill
    let r = (color.r as f32 * 0.7) as u8;
    let g = (color.g as f32 * 0.7) as u8;
    let b = (color.b as f32 * 0.7) as u8;
    canvas.set_draw_color(SdlColor::RGB(r, g, b));
    let _ = canvas.fill_rect(Rect::new(0, 0, w as u32, h as u32));
    
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

    // 2. Metallic/Glass Shine
    for y in 0..h {
        let factor = y as f32 / h as f32;
        let alpha = if factor < 0.4 {
            ((1.0 - factor / 0.4) * 120.0) as u8
        } else if factor < 0.5 {
            ((factor - 0.4) / 0.1 * 200.0) as u8
        } else {
            ((1.0 - (factor - 0.5) / 0.5) * 80.0) as u8
        };
        
        canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, alpha));
        let _ = canvas.draw_line(
            Point::new(0, y),
            Point::new(w, y),
        );
    }
    
    // 3. Inner Glow
    let glow_rect = Rect::new(2, 2, (w - 4) as u32, (h - 4) as u32);
    canvas.set_draw_color(SdlColor::RGBA(color.r, color.g, color.b, 150));
    let _ = canvas.draw_rect(glow_rect);

    // 4. 3D Bevel Borders
    canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, 200));
    let _ = canvas.draw_line(Point::new(0, 0), Point::new(w, 0));
    let _ = canvas.draw_line(Point::new(0, 0), Point::new(0, h));
    
    canvas.set_draw_color(SdlColor::RGBA(0, 0, 0, 180));
    let _ = canvas.draw_line(Point::new(0, h-1), Point::new(w, h-1));
    let _ = canvas.draw_line(Point::new(w-1, 0), Point::new(w-1, h));
}


/// Draw a shiny metal ball with speed text and fireball effect
fn draw_shiny_ball(canvas: &mut Canvas<Window>, ball: &Ball, font: &Font, cache: &mut TextureCache, frame_count: u64) {
    let cx = ball.x as i32 + BALL_SIZE / 2;
    let cy = ball.y as i32 + BALL_SIZE / 2;
    let radius = BALL_SIZE / 2;
    
    // Calculate ball speed
    let speed_px_frame = (ball.vel_x.powi(2) + ball.vel_y.powi(2)).sqrt();
    let speed_px_sec = speed_px_frame * 60.0; // Assuming 60 FPS
    
    // Draw ball trail for ultra-fast speeds (using stored positions)
    if !ball.trail_positions.is_empty() {
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        
        let positions: Vec<_> = ball.trail_positions.iter().collect();
        for (i, (tx, ty)) in positions.iter().enumerate() {
            let alpha = ((i + 1) as f32 / positions.len() as f32 * 180.0) as u8;
            let trail_size = (BALL_SIZE as f32 * 0.8) as i32;
            
            // Draw semi-transparent ball copy
            canvas.set_draw_color(SdlColor::RGBA(200, 200, 255, alpha));
            for dy in 0..trail_size {
                for dx in 0..trail_size {
                    let cdx = dx - trail_size / 2;
                    let cdy = dy - trail_size / 2;
                    if cdx*cdx + cdy*cdy <= (trail_size/2)*(trail_size/2) {
                        let _ = canvas.draw_point(Point::new(*tx as i32 + dx, *ty as i32 + dy));
                    }
                }
            }
        }
        
        canvas.set_blend_mode(sdl2::render::BlendMode::None);
    }
    
    // Draw fireball/comet trail effect if speed >= 800px/s
    // Super fireball at >= 1400px/s with brighter glow, pulsing, and rapid spinning
    if speed_px_sec >= 800.0 {
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        
        let is_super_fireball = speed_px_sec >= 1400.0;
        
        // Calculate direction opposite to velocity
        let vel_mag = speed_px_frame;
        let dir_x = -ball.vel_x / vel_mag;
        let dir_y = -ball.vel_y / vel_mag;
        
        // Determine trail properties based on speed tier
        let (trail_length, base_intensity) = if is_super_fireball {
            (30, 220.0) // Longer, brighter trail for super fireball
        } else {
            (20, 180.0) // Normal fireball
        };
        
        // Draw flame trail particles behind the ball
        for i in 1..=trail_length {
            let trail_dist = i as f32 * 2.0;
            let trail_x = cx + (dir_x * trail_dist) as i32;
            let trail_y = cy + (dir_y * trail_dist) as i32;
            
            // Pulsing effect for super fireball
            let pulse_factor = if is_super_fireball {
                (frame_count as f32 * 0.3).sin() * 0.2 + 0.8
            } else {
                1.0
            };
            
            // Fade out and shrink as we go back
            let alpha = ((trail_length - i) as f32 / trail_length as f32 * base_intensity * pulse_factor) as u8;
            let trail_radius = radius - (i / 4);
            
            // Color gradient - brighter for super fireball
            let (r, g, b) = if is_super_fireball {
                // Super fireball: brighter, more yellow-white
                if i < 10 {
                    (255u8, 255u8, 200u8) // Bright white-yellow
                } else if i < 20 {
                    (255u8, (220 - (i - 10) * 8) as u8, 100u8) // Bright orange
                } else {
                    (255u8, 100u8, 50u8) // Bright red-orange
                }
            } else {
                // Regular fireball
                if i < 7 {
                    (255u8, (255 - i * 20) as u8, 100u8) // Yellow-white
                } else if i < 14 {
                    (255u8, (140 - (i - 7) * 15) as u8, 50u8) // Orange
                } else {
                    (200u8, 50u8, 20u8) // Red
                }
            };
            
            // Draw flame particle as filled circle
            for dy in -trail_radius..=trail_radius {
                for dx in -trail_radius..=trail_radius {
                    if dx*dx + dy*dy <= trail_radius*trail_radius {
                        canvas.set_draw_color(SdlColor::RGBA(r, g, b, alpha));
                        let _ = canvas.draw_point(Point::new(trail_x + dx, trail_y + dy));
                    }
                }
            }
        }
        
        // Add spinning particle ring for super fireball
        if is_super_fireball {
            let spin_speed = frame_count as f32 * 0.2; // Rapid spin
            for i in 0..12 {
                let angle_offset = (i as f32 / 12.0) * std::f32::consts::PI * 2.0 + spin_speed;
                let ring_radius = 18.0;
                let px = cx + (angle_offset.cos() * ring_radius) as i32;
                let py = cy + (angle_offset.sin() * ring_radius) as i32;
                
                // Pulsing alpha
                let pulse_alpha = ((frame_count as f32 * 0.3 + i as f32 * 0.5).sin() * 80.0 + 175.0) as u8;
                
                // Draw glowing particle
                for dy in -3..=3 {
                    for dx in -3..=3 {
                        if dx*dx + dy*dy <= 9 {
                            canvas.set_draw_color(SdlColor::RGBA(255, 255, 200, pulse_alpha));
                            let _ = canvas.draw_point(Point::new(px + dx, py + dy));
                        }
                    }
                }
            }
        }
        
        canvas.set_blend_mode(sdl2::render::BlendMode::None);
    }
    
    // Draw ball from cache
    let _ = canvas.copy(&cache.ball, None, Some(ball.rect()));

    // Draw speed text
    let speed_text = format!("{} px/s", speed_px_sec as i32);
    
    if speed_text != cache.last_speed_text {
        if let Ok(surface) = font.render(&speed_text).blended(SdlColor::RGB(200, 200, 200)) {
            // Use the cached texture creator
            if let Ok(texture) = cache.creator.create_texture_from_surface(&surface) {
                cache.speed_text = Some(texture);
                cache.last_speed_text = speed_text;
            }
        }
    }

    if let Some(texture) = &cache.speed_text {
        let query = texture.query();
        let text_width = query.width;
        let text_height = query.height;
        // Scale down the font for the speed display
        let scale = 0.6;
        let scaled_width = (text_width as f32 * scale) as u32;
        let scaled_height = (text_height as f32 * scale) as u32;
        
        let target = Rect::new(
            cx - (scaled_width as i32 / 2),
            cy - radius - scaled_height as i32 - 5,
            scaled_width,
            scaled_height
        );
        let _ = canvas.copy(texture, None, Some(target));
    }
}

/// Draw a filled heart shape for lives
fn draw_heart(canvas: &mut Canvas<Window>, cx: i32, cy: i32, size: i32) {
    canvas.set_draw_color(SdlColor::RGB(220, 20, 60)); // Crimson red
    
    // Simple heart shape using filled circles and triangle
    let half_size = size / 2;
    
    // Top two circles
    for dx in -half_size..=half_size {
        for dy in -half_size..=0 {
            let dist_left = ((dx + half_size/2).pow(2) + dy.pow(2)) as f32;
            let dist_right = ((dx - half_size/2).pow(2) + dy.pow(2)) as f32;
            let radius_sq = (half_size as f32 / 1.5).powf(2.0);
            
            if dist_left <= radius_sq || dist_right <= radius_sq {
                let _ = canvas.draw_point(Point::new(cx + dx, cy + dy));
            }
        }
    }
    
    // Bottom triangle
    for dy in 0..=size {
        let width = size - dy;
        for dx in -(width/2)..=(width/2) {
            let _ = canvas.draw_point(Point::new(cx + dx, cy + dy));
        }
    }
}

/// Draw block with "eye candy" aesthetics (3D bevel, metallic shine)
fn draw_block_with_gradient(canvas: &mut Canvas<Window>, block: &Block, cache: &TextureCache) {
    let color_idx = BLOCK_COLORS.iter().position(|&c| c.r == block.color.r && c.g == block.color.g && c.b == block.color.b).unwrap_or(0);
    let _ = canvas.copy(&cache.blocks[color_idx], None, Some(block.rect()));
}

/// Draw paddle with enhanced sci-fi/metallic aesthetics and rounded corners
fn draw_paddle_with_glass(canvas: &mut Canvas<Window>, paddle: &Paddle, cache: &TextureCache) {
    let x = paddle.x;
    let y = paddle.y;
    let w = paddle.width;
    let h = 20; 

    // 1. Draw cached body
    let texture = if paddle.width > paddle.normal_width { &cache.paddle_long } else { &cache.paddle_normal };
    let _ = canvas.copy(texture, None, Some(paddle.rect()));
    
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

    // 2. Energy Core (Glowing center strip)
    let core_height = 4;
    let core_y = y + (h - core_height) / 2;
    let radius = 10;
    let core_rect = Rect::new(x + radius + 2, core_y, (w - 2 * radius - 4) as u32, core_height as u32);
    
    // Pulsating core glow
    // SPIN EFFECT: Change color based on spin_intensity
    if paddle.spin_intensity > 0.1 {
        // Electric Purple/White discharge
        let intensity = (paddle.spin_intensity * 255.0) as u8;
        canvas.set_draw_color(SdlColor::RGBA(200, 100, 255, intensity)); // Purple glow
        let _ = canvas.fill_rect(core_rect);
        
        // White hot center
        canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, intensity));
        let _ = canvas.fill_rect(Rect::new(core_rect.x(), core_rect.y() + 1, core_rect.width(), 2));
    } else {
        // Standard Cyan glow
        canvas.set_draw_color(SdlColor::RGBA(0, 200, 255, 200));
        let _ = canvas.fill_rect(core_rect);
    }
    
    // 4. Thruster/Engine Lights on ends
    let light_width = 4;
    let left_light = Rect::new(x + 2, y + h/2 - 6, light_width, 12);
    let right_light = Rect::new(x + w - 2 - light_width as i32, y + h/2 - 6, light_width, 12);
    
    // SPIN EFFECT: Engines flare up
    if paddle.spin_intensity > 0.1 {
        canvas.set_draw_color(SdlColor::RGBA(255, 50, 255, 255)); // Purple flare
    } else {
        canvas.set_draw_color(SdlColor::RGBA(255, 100, 50, 200)); // Orange engine glow
    }
    let _ = canvas.fill_rect(left_light);
    let _ = canvas.fill_rect(right_light);

    // SPIN EFFECT: Outer Aura
    if paddle.spin_intensity > 0.2 {
        let aura_alpha = (paddle.spin_intensity * 100.0) as u8;
        canvas.set_draw_color(SdlColor::RGBA(200, 100, 255, aura_alpha));
        let _ = canvas.draw_rect(Rect::new(x - 2, y - 2, (w + 4) as u32, (h + 4) as u32));
    }
    
    canvas.set_blend_mode(sdl2::render::BlendMode::None);
}

/// Draw a clean glass capsule/bulb with symbol inside
fn draw_bonus_icon(canvas: &mut Canvas<Window>, bonus: &Bonus) {
    let rect = bonus.rect();
    let cx = rect.x() + rect.width() as i32 / 2;
    let cy = rect.y() + rect.height() as i32 / 2;
    let radius = 20;  // Capsule radius
    
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    
    // Determine color based on bonus type
    let (r, g, b) = match bonus.bonus_type {
        BonusType::ExtraBall => (255, 50, 50),   // Red
        BonusType::LongPaddle => (100, 255, 100), // Green
        BonusType::GhostBall => (200, 200, 200),  // Grey
        BonusType::Rocket => (255, 165, 0),       // Orange
    };

    // Draw capsule body - transparent glass with color tint
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let dist_sq = dx*dx + dy*dy;
            if dist_sq <= radius*radius {
                let dist = (dist_sq as f32).sqrt();
                let edge_factor = dist / radius as f32;
                
                // Glass transparency - more transparent in center, more opaque at edges
                let alpha = if edge_factor > 0.85 {
                    // Outer rim - more opaque
                    200
                } else {
                    // Inner area - very transparent
                    (30.0 + edge_factor * 50.0) as u8
                };
                
                // Light tint for glass
                canvas.set_draw_color(SdlColor::RGBA(r, g, b, alpha));
                let _ = canvas.draw_point(Point::new(cx + dx, cy + dy));
            }
        }
    }
    
    // Draw glass highlight (light reflection on top-left)
    let highlight_offset_x = -radius / 3;
    let highlight_offset_y = -radius / 3;
    let highlight_radius = radius / 2;
    
    for dy in -highlight_radius..=highlight_radius {
        for dx in -highlight_radius..=highlight_radius {
            let dist_sq = dx*dx + dy*dy;
            if dist_sq <= highlight_radius*highlight_radius {
                let dist = (dist_sq as f32).sqrt();
                let factor = 1.0 - (dist / highlight_radius as f32);
                let alpha = (factor * 120.0) as u8;
                
                canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, alpha));
                let _ = canvas.draw_point(Point::new(
                    cx + highlight_offset_x + dx,
                    cy + highlight_offset_y + dy
                ));
            }
        }
    }
    
    // Draw clean outline (double ring for glass effect)
    canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, 180));
    for angle in 0..360 {
        let rad = (angle as f32).to_radians();
        // Outer ring
        let x1 = cx + (radius as f32 * rad.cos()) as i32;
        let y1 = cy + (radius as f32 * rad.sin()) as i32;
        let _ = canvas.draw_point(Point::new(x1, y1));
        
        // Inner ring (slightly inside)
        let x2 = cx + ((radius - 1) as f32 * rad.cos()) as i32;
        let y2 = cy + ((radius - 1) as f32 * rad.sin()) as i32;
        canvas.set_draw_color(SdlColor::RGBA(200, 200, 200, 100));
        let _ = canvas.draw_point(Point::new(x2, y2));
        canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, 180));
    }
    
    // Draw symbol inside (with shadow for depth)
    // Shadow
    canvas.set_draw_color(SdlColor::RGBA(0, 0, 0, 80));
    match bonus.bonus_type {
        BonusType::ExtraBall => {
            // Small dot shadow
            let inner_radius = 5;
             for dy in -inner_radius..=inner_radius {
                for dx in -inner_radius..=inner_radius {
                    if dx*dx + dy*dy <= inner_radius*inner_radius {
                        let _ = canvas.draw_point(Point::new(cx + dx + 1, cy + dy + 1));
                    }
                }
            }
        }
        BonusType::LongPaddle => {
            // Horizontal bar shadow
            let _ = canvas.fill_rect(Rect::new(cx - 8, cy - 2 + 1, 16, 5));
        }
        BonusType::GhostBall => {
            // Ghost icon shadow (circle)
            let _ = canvas.fill_rect(Rect::new(cx - 6, cy - 6 + 1, 12, 12));
        }
        BonusType::Rocket => {
            // Rocket shadow (triangle)
            let _ = canvas.fill_rect(Rect::new(cx - 3, cy - 6 + 1, 6, 12));
        }
    }
    
    // Actual symbol (bright and clear)
    canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, 255));
    match bonus.bonus_type {
        BonusType::ExtraBall => {
            // Small dot
            let inner_radius = 5;
             for dy in -inner_radius..=inner_radius {
                for dx in -inner_radius..=inner_radius {
                    if dx*dx + dy*dy <= inner_radius*inner_radius {
                        let _ = canvas.draw_point(Point::new(cx + dx, cy + dy));
                    }
                }
            }
        }
        BonusType::LongPaddle => {
            // Horizontal bar shadow
            let _ = canvas.fill_rect(Rect::new(cx - 8, cy - 2 + 1, 16, 5));
        }
        BonusType::GhostBall => {
            // Ghost icon (circle)
            let _ = canvas.fill_rect(Rect::new(cx - 6, cy - 6, 12, 12));
        }
        BonusType::Rocket => {
            // Rocket (triangle)
            let _ = canvas.fill_rect(Rect::new(cx - 3, cy - 6, 6, 12));
        }
    }
    
    // Actual symbol (bright and clear)
    canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, 255));
    match bonus.bonus_type {
        BonusType::ExtraBall => {
            // Small dot
            let inner_radius = 5;
             for dy in -inner_radius..=inner_radius {
                for dx in -inner_radius..=inner_radius {
                    if dx*dx + dy*dy <= inner_radius*inner_radius {
                        let _ = canvas.draw_point(Point::new(cx + dx, cy + dy));
                    }
                }
            }
        }
        BonusType::LongPaddle => {
            // Horizontal bar
            let _ = canvas.fill_rect(Rect::new(cx - 8, cy - 2, 16, 5));
        }
        BonusType::GhostBall => {
            // Ghost icon
            let _ = canvas.fill_rect(Rect::new(cx - 6, cy - 6, 12, 12));
        }
        BonusType::Rocket => {
            // Rocket
            let _ = canvas.fill_rect(Rect::new(cx - 3, cy - 6, 6, 12));
        }
    }
    
    canvas.set_blend_mode(sdl2::render::BlendMode::None);
}

/// Draw animated background for levels 7-9
fn draw_animated_background(canvas: &mut Canvas<Window>, level: usize, frame: u64) {
    // Use frame counter for animation timing
    let time = frame as f32;
    
    match level {
        7 => {
            // Level 7: Animated starfield
            canvas.set_draw_color(SdlColor::RGB(5, 5, 20));
            canvas.clear();
            
            canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
            
            // Draw animated stars
            for i in 0..100 {
                let x = ((i * 137 + (time * 0.5 * i as f32 * 0.01) as i32) % WINDOW_WIDTH as i32) as i32;
                let y = ((i * 241) % WINDOW_HEIGHT as i32) as i32;
                let brightness = ((time * 0.05 + i as f32 * 0.5).sin() * 127.0 + 128.0) as u8;
                let size = 1 + (i % 3) as i32;
                
                canvas.set_draw_color(SdlColor::RGBA(brightness, brightness, 255, brightness));
                let _ = canvas.fill_rect(Rect::new(x, y, size as u32, size as u32));
            }
            
            canvas.set_blend_mode(sdl2::render::BlendMode::None);
        },
        8 => {
            // Level 8: Pulsing circles
            canvas.set_draw_color(SdlColor::RGB(10, 5, 15));
            canvas.clear();
            
            canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
            
            // Draw pulsing circles
            for ring in 1..=6 {
                let pulse = (time * 0.05 + ring as f32 * 0.5).sin();
                let radius = (ring as f32 * 80.0 + pulse * 20.0) as i32;
                let alpha = ((pulse + 1.0) * 60.0) as u8;
                
                canvas.set_draw_color(SdlColor::RGBA(100, 50, 150, alpha));
                
                // Draw circle
                for angle in 0..360 {
                    let rad = (angle as f32).to_radians();
                    let x = WINDOW_WIDTH as i32 / 2 + (radius as f32 * rad.cos()) as i32;
                    let y = WINDOW_HEIGHT as i32 / 2 + (radius as f32 * rad.sin()) as i32;
                    let _ = canvas.draw_point(Point::new(x, y));
                }
            }
            
            canvas.set_blend_mode(sdl2::render::BlendMode::None);
        },
        9 => {
            // Level 9: Matrix-style digital rain
            canvas.set_draw_color(SdlColor::RGB(0, 0, 0));
            canvas.clear();
            
            canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
            
            // Draw digital rain effect
            for col in 0..40 {
                let offset = ((time * 2.0 + col as f32 * 10.0) as i32) % (WINDOW_HEIGHT as i32 + 100);
                
                for i in 0..10 {
                    let y = offset - i * 20;
                    if y >= 0 && y < WINDOW_HEIGHT as i32 {
                        let alpha = if i == 0 { 255 } else { 255 - i * 25 };
                        let green = if i == 0 { 255 } else { 150 - i * 15 };
                        
                        canvas.set_draw_color(SdlColor::RGBA(0, green as u8, 0, alpha as u8));
                        let x = col * 32;
                        let _ = canvas.fill_rect(Rect::new(x, y, 2, 15));
                    }
                }
            }
            
            canvas.set_blend_mode(sdl2::render::BlendMode::None);
        },
        _ => {
            // Structured, non-distracting procedural backgrounds for levels 10+
            use rand::{Rng, SeedableRng};
            use rand::rngs::StdRng;
            
            // Use multiple entropy sources for better randomization
            let seed = (level as u64).wrapping_mul(12345)
                .wrapping_add((level as u64 / 3).wrapping_mul(67890))
                .wrapping_add((level as u64 % 7).wrapping_mul(11111));
            let mut rng = StdRng::seed_from_u64(seed);
            
            // Randomly select one of the 20 themes
            let theme = rng.gen_range(0..20);
            
            match theme {
                0 => {
                    // THEME 1: CYBER GRID (Tron-like 3D perspective)
                    // ... (keep existing code) ...
                    // Randomize grid color
                    let r_base = rng.gen_range(0..50);
                    let g_base = rng.gen_range(0..50);
                    let b_base = rng.gen_range(40..100);
                    canvas.set_draw_color(SdlColor::RGB(r_base/2, g_base/2, b_base/2));
                    canvas.clear();
                    
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    // Randomize grid parameters
                    let horizon_y = WINDOW_HEIGHT as i32 / rng.gen_range(2..4);
                    let grid_spacing = rng.gen_range(60..120);
                    let speed = rng.gen_range(0.3..0.8);
                    let offset = (time * speed) % 40.0;
                    
                    // Random line color
                    let r_line = rng.gen_range(r_base..255);
                    let g_line = rng.gen_range(g_base..255);
                    let b_line = rng.gen_range(b_base..255);
                    canvas.set_draw_color(SdlColor::RGBA(r_line, g_line, b_line, 100));
                    
                    // Vertical perspective lines
                    for i in -15..25 {
                        let x_start = (WINDOW_WIDTH as i32 / 2) + i * grid_spacing;
                        // Perspective projection
                        let _ = canvas.draw_line(
                            Point::new(x_start, WINDOW_HEIGHT as i32),
                            Point::new(WINDOW_WIDTH as i32 / 2 + i * 10, horizon_y)
                        );
                    }
                    
                    // Horizontal moving lines (scanlines)
                    for i in 0..25 {
                        let z = 1.0 + i as f32 * 0.5 - (offset / 40.0) * 0.5;
                        if z > 0.1 {
                            let y_pos = horizon_y as f32 + (WINDOW_HEIGHT as f32 - horizon_y as f32) / z;
                            
                            if y_pos < WINDOW_HEIGHT as f32 {
                                let _ = canvas.draw_line(
                                    Point::new(0, y_pos as i32),
                                    Point::new(WINDOW_WIDTH as i32, y_pos as i32)
                                );
                            }
                        }
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                1 => {
                    // THEME 2: STAR VOYAGE (Radial warp)
                    // ... (keep existing code) ...
                    // Random deep space color
                    let bg_r = rng.gen_range(0..20);
                    let bg_g = rng.gen_range(0..20);
                    let bg_b = rng.gen_range(10..40);
                    canvas.set_draw_color(SdlColor::RGB(bg_r, bg_g, bg_b));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    let num_stars = rng.gen_range(80..200);
                    let center_x = WINDOW_WIDTH as f32 / 2.0;
                    let center_y = WINDOW_HEIGHT as f32 / 2.0;
                    let speed_mult = rng.gen_range(0.5..2.5);
                    
                    // Random star tint
                    let tint_r = rng.gen_range(150..255);
                    let tint_g = rng.gen_range(150..255);
                    let tint_b = rng.gen_range(150..255);
                    
                    for i in 0..num_stars {
                        let seed = level as u64 * 1000 + i;
                        let mut star_rng = StdRng::seed_from_u64(seed);
                        
                        let angle = star_rng.gen_range(0.0..std::f32::consts::PI * 2.0);
                        let speed = star_rng.gen_range(0.5..2.0) * speed_mult;
                        let start_dist = star_rng.gen_range(0.0..800.0);
                        
                        let dist = (start_dist + time * speed) % 700.0;
                        
                        let x = center_x + angle.cos() * dist;
                        let y = center_y + angle.sin() * dist;
                        
                        let size = (dist / 200.0).max(1.0) as u32;
                        let alpha = (dist / 700.0 * 255.0) as u8;
                        
                        if x >= 0.0 && x < WINDOW_WIDTH as f32 && y >= 0.0 && y < WINDOW_HEIGHT as f32 {
                            canvas.set_draw_color(SdlColor::RGBA(tint_r, tint_g, tint_b, alpha));
                            let _ = canvas.fill_rect(Rect::new(x as i32, y as i32, size, size));
                        }
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                2 => {
                    // THEME 3: HEX PULSE (Geometric)
                    // ... (keep existing code) ...
                    // Random background color
                    let bg_r = rng.gen_range(5..40);
                    let bg_g = rng.gen_range(5..40);
                    let bg_b = rng.gen_range(5..40);
                    canvas.set_draw_color(SdlColor::RGB(bg_r, bg_g, bg_b));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    // Random hex parameters
                    let hex_size = rng.gen_range(20..60);
                    let cols = WINDOW_WIDTH / hex_size + 2;
                    let rows = WINDOW_HEIGHT / hex_size + 2;
                    let pulse_speed = rng.gen_range(0.01..0.05);
                    
                    // Random grid color
                    let grid_r = rng.gen_range(50..200);
                    let grid_g = rng.gen_range(50..200);
                    let grid_b = rng.gen_range(50..200);
                    
                    for row in 0..rows {
                        for col in 0..cols {
                            let x_offset = if row % 2 == 0 { 0 } else { hex_size / 2 };
                            let x = (col * hex_size) as i32 + x_offset as i32;
                            let y = (row * hex_size) as i32;
                            
                            let pulse = ((time * pulse_speed + col as f32 * 0.2 + row as f32 * 0.2).sin() + 1.0) / 2.0;
                            let alpha = (10.0 + pulse * 40.0) as u8;
                            
                            canvas.set_draw_color(SdlColor::RGBA(grid_r, grid_g, grid_b, alpha));
                            let _ = canvas.draw_rect(Rect::new(x, y, hex_size - 2, hex_size - 2));
                        }
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                3 => {
                    // THEME 4: AURORA WAVES (Smooth)
                    // ... (keep existing code) ...
                    let bg_r = rng.gen_range(5..25);
                    let bg_g = rng.gen_range(5..25);
                    let bg_b = rng.gen_range(10..35);
                    canvas.set_draw_color(SdlColor::RGB(bg_r, bg_g, bg_b));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    let num_waves = rng.gen_range(3..8);
                    let vertical = rng.gen_bool(0.5); // Random orientation
                    
                    for i in 0..num_waves {
                        let wave_r = rng.gen_range(0..100);
                        let wave_g = rng.gen_range(50..200);
                        let wave_b = rng.gen_range(100..255);
                        let alpha = rng.gen_range(20..50);
                        
                        canvas.set_draw_color(SdlColor::RGBA(wave_r, wave_g, wave_b, alpha));
                        
                        let offset = i as f32 * 40.0;
                        let speed = rng.gen_range(0.01..0.03);
                        let amplitude = rng.gen_range(30.0..80.0);
                        
                        if vertical {
                            // Vertical strips (original)
                            for x in (0..WINDOW_WIDTH).step_by(5) {
                                let y_base = WINDOW_HEIGHT as f32 / 2.0 + offset - (num_waves as f32 * 20.0);
                                let y_wave = (x as f32 * 0.01 + time * speed).sin() * amplitude;
                                let y = (y_base + y_wave) as i32;
                                let _ = canvas.draw_line(Point::new(x as i32, y), Point::new(x as i32, y + 100));
                            }
                        } else {
                            // Horizontal strips (new variation)
                            for y in (0..WINDOW_HEIGHT).step_by(5) {
                                let x_base = WINDOW_WIDTH as f32 / 2.0 + offset - (num_waves as f32 * 20.0);
                                let x_wave = (y as f32 * 0.01 + time * speed).sin() * amplitude;
                                let x = (x_base + x_wave) as i32;
                                let _ = canvas.draw_line(Point::new(x, y as i32), Point::new(x + 100, y as i32));
                            }
                        }
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                4 => {
                    // THEME 5: MATRIX RAIN (Digital)
                    // Falling code-like streams
                    
                    canvas.set_draw_color(SdlColor::RGB(0, 10, 0));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    let num_streams = rng.gen_range(30..60);
                    let speed_mult = rng.gen_range(0.5..1.5);
                    
                    for i in 0..num_streams {
                        let seed = level as u64 * 1000 + i;
                        let mut stream_rng = StdRng::seed_from_u64(seed);
                        
                        let x = stream_rng.gen_range(0..WINDOW_WIDTH as i32);
                        let speed = stream_rng.gen_range(2.0..5.0) * speed_mult;
                        let len = stream_rng.gen_range(5..15);
                        let y_head = ((time * speed) as i32 + stream_rng.gen_range(0..500)) % (WINDOW_HEIGHT as i32 + 200) - 100;
                        
                        for j in 0..len {
                            let y = y_head - j * 15;
                            if y >= 0 && y < WINDOW_HEIGHT as i32 {
                                let alpha = if j == 0 { 255 } else { (255 - j * 20).max(0) as u8 };
                                let green = if j == 0 { 255 } else { 150 };
                                
                                canvas.set_draw_color(SdlColor::RGBA(0, green, 0, alpha));
                                // Draw a small rect to simulate a character
                                let _ = canvas.fill_rect(Rect::new(x, y, 8, 12));
                            }
                        }
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                5 => {
                    // THEME 6: NEBULA CLOUDS (Soft Noise)
                    // Colorful, drifting soft particles
                    
                    let bg_r = rng.gen_range(10..30);
                    let bg_g = rng.gen_range(10..30);
                    let bg_b = rng.gen_range(20..40);
                    canvas.set_draw_color(SdlColor::RGB(bg_r, bg_g, bg_b));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    let num_clouds = rng.gen_range(15..30);
                    
                    for i in 0..num_clouds {
                        let seed = level as u64 * 1000 + i;
                        let mut cloud_rng = StdRng::seed_from_u64(seed);
                        
                        let x_base = cloud_rng.gen_range(0..WINDOW_WIDTH as i32);
                        let y_base = cloud_rng.gen_range(0..WINDOW_HEIGHT as i32);
                        let size = cloud_rng.gen_range(50..150);
                        
                        let r = cloud_rng.gen_range(50..150);
                        let g = cloud_rng.gen_range(0..100);
                        let b = cloud_rng.gen_range(100..200);
                        
                        // Slow drift
                        let x_drift = ((time * 0.05 + i as f32).sin() * 50.0) as i32;
                        let y_drift = ((time * 0.03 + i as f32 * 0.5).cos() * 30.0) as i32;
                        
                        let x = x_base + x_drift;
                        let y = y_base + y_drift;
                        
                        // Draw soft circle (simulated by multiple transparent rects)
                        canvas.set_draw_color(SdlColor::RGBA(r, g, b, 10)); // Very transparent
                        for s in (0..size).step_by(10) {
                            let rect_size = size - s;
                            let offset = s / 2;
                            let _ = canvas.fill_rect(Rect::new(x + offset, y + offset, rect_size as u32, rect_size as u32));
                        }
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                6 => {
                    // THEME 7: RAIN & THUNDER
                    // Dark stormy background
                    canvas.set_draw_color(SdlColor::RGB(10, 10, 20));
                    canvas.clear();
                    
                    // Thunder flash - DISABLED (removed lightning effect)
                    /*let thunder_seed = (time * 0.01) as u64; // Change seed slowly
                    let mut thunder_rng = StdRng::seed_from_u64(thunder_seed + level as u64);
                    
                    // Occasional flash (0.5% chance per frame check, but seed changes slower so it lasts a bit)
                    if thunder_rng.gen_bool(0.02) && (time as u64 % 10 < 3) { 
                         
                         
                         // Lightning bolt
                         canvas.set_draw_color(SdlColor::RGB(255, 255, 255));
                         let start_x = thunder_rng.gen_range(100..WINDOW_WIDTH as i32 - 100);
                         let mut curr_x = start_x;
                         let mut curr_y = 0;
                         while curr_y < WINDOW_HEIGHT as i32 {
                             let next_x = curr_x + thunder_rng.gen_range(-30..30);
                             let next_y = curr_y + thunder_rng.gen_range(20..50);
                             let _ = canvas.draw_line(Point::new(curr_x, curr_y), Point::new(next_x, next_y));
                             // Branch
                             if thunder_rng.gen_bool(0.3) {
                                  let branch_x = next_x + thunder_rng.gen_range(-20..20);
                                  let branch_y = next_y + thunder_rng.gen_range(20..40);
                                  let _ = canvas.draw_line(Point::new(next_x, next_y), Point::new(branch_x, branch_y));
                             }
                             curr_x = next_x;
                             curr_y = next_y;
                         }
                    }*/
                    
                    
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    // Rain
                    let num_drops = 400;
                    canvas.set_draw_color(SdlColor::RGBA(150, 150, 200, 150));
                    
                    for i in 0..num_drops {
                        let seed = level as u64 * 2000 + i;
                        let mut rain_rng = StdRng::seed_from_u64(seed);
                        
                        let x_base = rain_rng.gen_range(0..WINDOW_WIDTH as i32 + 200); // Extra width for slant
                        let speed = rain_rng.gen_range(15.0..25.0);
                        let len = rain_rng.gen_range(10..20);
                        
                        // Animate y
                        let y_anim = (time * speed + rain_rng.gen_range(0.0..1000.0)) % (WINDOW_HEIGHT as f32 + 50.0);
                        let y = y_anim as i32 - 20;
                        
                        // Slant rain
                        let x = x_base - (y as f32 * 0.2) as i32;
                        
                        if x >= 0 && x < WINDOW_WIDTH as i32 && y >= 0 && y < WINDOW_HEIGHT as i32 {
                            let _ = canvas.draw_line(Point::new(x, y), Point::new(x - 2, y + len));
                        }
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                7 => {
                    // THEME 8: SNOW
                    // Dark winter sky
                    canvas.set_draw_color(SdlColor::RGB(5, 10, 20));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    let num_flakes = 300;
                    
                    for i in 0..num_flakes {
                        let seed = level as u64 * 3000 + i;
                        let mut snow_rng = StdRng::seed_from_u64(seed);
                        
                        let x_base = snow_rng.gen_range(0..WINDOW_WIDTH as i32);
                        let speed = snow_rng.gen_range(1.0..3.0);
                        let size = snow_rng.gen_range(1..3);
                        
                        // Animate
                        let y = (time * speed + snow_rng.gen_range(0.0..1000.0)) % (WINDOW_HEIGHT as f32 + 10.0);
                        
                        // Horizontal drift
                        let drift = (time * 0.02 + i as f32).sin() * 20.0;
                        let x = (x_base as f32 + drift) as i32;
                        
                        let alpha = snow_rng.gen_range(100..255);
                        canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, alpha));
                        
                        let _ = canvas.fill_rect(Rect::new(x, y as i32 - 5, size as u32, size as u32));
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                8 => {
                    // THEME 9: STARRY NIGHT (Shooting Stars & Satellite)
                    canvas.set_draw_color(SdlColor::RGB(0, 0, 10));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    // Static stars (twinkling)
                    let num_stars = 200;
                    for i in 0..num_stars {
                        let seed = level as u64 * 4000 + i;
                        let mut star_rng = StdRng::seed_from_u64(seed);
                        
                        let x = star_rng.gen_range(0..WINDOW_WIDTH as i32);
                        let y = star_rng.gen_range(0..WINDOW_HEIGHT as i32);
                        let size = if star_rng.gen_bool(0.1) { 2 } else { 1 };
                        
                        // Twinkle
                        let twinkle_speed = star_rng.gen_range(0.05..0.2);
                        let alpha = ((time * twinkle_speed + i as f32).sin() * 100.0 + 155.0) as u8;
                        
                        canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, alpha));
                        let _ = canvas.fill_rect(Rect::new(x, y, size, size));
                    }
                    
                    // Satellite
                    let sat_speed = 0.5;
                    let sat_x = (time * sat_speed) % (WINDOW_WIDTH as f32 + 50.0) - 20.0;
                    let sat_y = 100.0 + (time * 0.01).sin() * 20.0;
                    
                    canvas.set_draw_color(SdlColor::RGBA(200, 200, 255, 255));
                    let _ = canvas.fill_rect(Rect::new(sat_x as i32, sat_y as i32, 3, 3));
                    // Blinking light on satellite
                    if (time as u64 / 30) % 2 == 0 {
                        canvas.set_draw_color(SdlColor::RGBA(255, 0, 0, 255));
                        let _ = canvas.draw_point(Point::new(sat_x as i32 + 1, sat_y as i32 + 1));
                    }
                    
                    // Shooting stars
                    // Random occurrence based on time
                    let shooting_seed = (time * 0.005) as u64; // Change every 200 frames approx
                    let mut shoot_rng = StdRng::seed_from_u64(shooting_seed + level as u64);
                    
                    if shoot_rng.gen_bool(0.3) { // 30% chance per interval
                        let progress = (time % 200.0) / 200.0; // 0 to 1 loop
                        if progress < 0.3 { // Only show for first 30% of interval
                             let start_x = shoot_rng.gen_range(0..WINDOW_WIDTH as i32);
                             let start_y = shoot_rng.gen_range(0..WINDOW_HEIGHT as i32 / 2);
                             
                             let shoot_x = start_x as f32 + progress * 800.0; // Fast
                             let shoot_y = start_y as f32 + progress * 600.0;
                             
                             // Trail
                             canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, 200));
                             let _ = canvas.draw_line(
                                 Point::new(shoot_x as i32, shoot_y as i32), 
                                 Point::new((shoot_x - 40.0) as i32, (shoot_y - 30.0) as i32)
                             );
                        }
                    }
                    
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                9 => {
                    // THEME 10: SMOKE
                    canvas.set_draw_color(SdlColor::RGB(20, 20, 20));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    let num_particles = 100;
                    
                    for i in 0..num_particles {
                        let seed = level as u64 * 5000 + i;
                        let mut smoke_rng = StdRng::seed_from_u64(seed);
                        
                        let x_base = smoke_rng.gen_range(0..WINDOW_WIDTH as i32);
                        let speed = smoke_rng.gen_range(0.5..1.5);
                        
                        // Rising
                        let y_anim = (time * speed + smoke_rng.gen_range(0.0..1000.0)) % (WINDOW_HEIGHT as f32 + 100.0);
                        let y = WINDOW_HEIGHT as f32 - y_anim; // Go up
                        
                        // Expand and fade as it goes up
                        let progress = 1.0 - (y / WINDOW_HEIGHT as f32); // 0 at bottom, 1 at top
                        let size = (20.0 + progress * 60.0) as u32;
                        let alpha = ((1.0 - progress) * 100.0) as u8;
                        
                        // Drift
                        let drift = (time * 0.01 + i as f32).sin() * 30.0 * progress;
                        let x = x_base as f32 + drift;
                        
                        canvas.set_draw_color(SdlColor::RGBA(100, 100, 100, alpha));
                        // Draw soft rect (simulated smoke puff)
                        let _ = canvas.fill_rect(Rect::new(x as i32 - size as i32 / 2, y as i32 - size as i32 / 2, size, size));
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },

                10 => {
                    // THEME 11: BUBBLES
                    // Underwater Blue
                    canvas.set_draw_color(SdlColor::RGB(0, 20, 40));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    let num_bubbles = 150;
                    for i in 0..num_bubbles {
                        let seed = level as u64 * 6000 + i;
                        let mut bubble_rng = StdRng::seed_from_u64(seed);
                        
                        let x_base = bubble_rng.gen_range(0..WINDOW_WIDTH as i32);
                        let speed = bubble_rng.gen_range(0.5..2.0);
                        let size = bubble_rng.gen_range(2..10);
                        
                        // Rising
                        let y_anim = (time * speed + bubble_rng.gen_range(0.0..1000.0)) % (WINDOW_HEIGHT as f32 + 50.0);
                        let y = WINDOW_HEIGHT as f32 - y_anim;
                        
                        // Swaying
                        let sway = (time * 0.02 + i as f32).sin() * 10.0;
                        let x = x_base as f32 + sway;
                        
                        // Draw bubble outline
                        canvas.set_draw_color(SdlColor::RGBA(100, 200, 255, 100));
                        let rect = Rect::new(x as i32, y as i32, size as u32, size as u32);
                        let _ = canvas.draw_rect(rect);
                        // Shine
                        canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, 150));
                        let _ = canvas.draw_point(Point::new(x as i32 + 1, y as i32 + 1));
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                11 => {
                    // THEME 12: FIRE
                    // Dark Red/Black
                    canvas.set_draw_color(SdlColor::RGB(20, 5, 0));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    let num_flames = 300;
                    for i in 0..num_flames {
                        let seed = level as u64 * 7000 + i;
                        let mut fire_rng = StdRng::seed_from_u64(seed);
                        
                        let x_base = fire_rng.gen_range(0..WINDOW_WIDTH as i32);
                        let speed = fire_rng.gen_range(2.0..5.0);
                        
                        // Rising fast
                        let y_anim = (time * speed + fire_rng.gen_range(0.0..1000.0)) % (WINDOW_HEIGHT as f32 / 1.5);
                        let y = WINDOW_HEIGHT as f32 - y_anim;
                        
                        let progress = 1.0 - (y_anim / (WINDOW_HEIGHT as f32 / 1.5)); // 1 at bottom, 0 at top
                        
                        // Color gradient: White -> Yellow -> Orange -> Red -> Dark Grey
                        let (r, g, b, a) = if progress > 0.8 {
                            (255, 255, 200, 255) // White/Yellow
                        } else if progress > 0.6 {
                            (255, 200, 0, 200) // Yellow/Orange
                        } else if progress > 0.3 {
                            (255, 50, 0, 150) // Red
                        } else {
                            (100, 50, 50, 100) // Smoke
                        };
                        
                        // Wiggle
                        let wiggle = (time * 0.1 + i as f32).sin() * 5.0;
                        let x = x_base as f32 + wiggle;
                        
                        let size = (progress * 8.0) as u32 + 2;
                        
                        canvas.set_draw_color(SdlColor::RGBA(r, g, b, a));
                        let _ = canvas.fill_rect(Rect::new(x as i32, y as i32, size, size));
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                12 => {
                    // THEME 13: DIGITAL CIRCUIT
                    // Dark Green/Black
                    canvas.set_draw_color(SdlColor::RGB(0, 10, 0));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    // Grid nodes
                    let spacing = 100;
                    let cols = WINDOW_WIDTH / spacing as u32 + 1;
                    let rows = WINDOW_HEIGHT / spacing as u32 + 1;
                    
                    for r in 0..rows {
                        for c in 0..cols {
                            let cx = (c * spacing) as i32;
                            let cy = (r * spacing) as i32;
                            
                            // Pulse
                            let pulse = ((time * 0.05 + (c + r) as f32).sin() * 100.0 + 100.0) as u8;
                            
                            // Draw node
                            canvas.set_draw_color(SdlColor::RGBA(0, 255, 0, pulse));
                            let _ = canvas.fill_rect(Rect::new(cx - 2, cy - 2, 5, 5));
                            
                            // Draw connections (randomly active)
                            let seed = level as u64 * 8000 + r as u64 * 100 + c as u64;
                            let mut circuit_rng = StdRng::seed_from_u64(seed);
                            
                            // Horizontal
                            if circuit_rng.gen_bool(0.7) {
                                let signal_pos = (time * 2.0) % spacing as f32;
                                canvas.set_draw_color(SdlColor::RGBA(0, 100, 0, 100));
                                let _ = canvas.draw_line(Point::new(cx, cy), Point::new(cx + spacing as i32, cy));
                                
                                // Moving signal packet
                                canvas.set_draw_color(SdlColor::RGBA(100, 255, 100, 200));
                                let sx = cx + signal_pos as i32;
                                let _ = canvas.fill_rect(Rect::new(sx, cy - 1, 4, 3));
                            }
                            
                            // Vertical
                            if circuit_rng.gen_bool(0.7) {
                                let signal_pos = (time * 2.0 + 50.0) % spacing as f32;
                                canvas.set_draw_color(SdlColor::RGBA(0, 100, 0, 100));
                                let _ = canvas.draw_line(Point::new(cx, cy), Point::new(cx, cy + spacing as i32));
                                
                                // Moving signal packet
                                canvas.set_draw_color(SdlColor::RGBA(100, 255, 100, 200));
                                let sy = cy + signal_pos as i32;
                                let _ = canvas.fill_rect(Rect::new(cx - 1, sy, 3, 4));
                            }
                        }
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                13 => {
                    // THEME 14: KALEIDOSCOPE
                    canvas.set_draw_color(SdlColor::RGB(10, 0, 10));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    let cx = WINDOW_WIDTH as f32 / 2.0;
                    let cy = WINDOW_HEIGHT as f32 / 2.0;
                    let num_petals = 12;
                    let layers = 5;
                    
                    for l in 0..layers {
                        let radius = 100.0 + l as f32 * 80.0;
                        let rotation = time * (0.01 + l as f32 * 0.005) * if l % 2 == 0 { 1.0 } else { -1.0 };
                        
                        let r = (100 + l * 30) as u8;
                        let g = (50 + l * 20) as u8;
                        let b = (200 - l * 20) as u8;
                        
                        for i in 0..num_petals {
                            let angle = (i as f32 / num_petals as f32) * std::f32::consts::PI * 2.0 + rotation;
                            
                            let x = cx + angle.cos() * radius;
                            let y = cy + angle.sin() * radius;
                            
                            let size = 20 + l * 5;
                            
                            canvas.set_draw_color(SdlColor::RGBA(r, g, b, 100));
                            
                            // Draw diamond shape
                            let _ = canvas.draw_line(Point::new(x as i32, (y - size as f32) as i32), Point::new((x + size as f32) as i32, y as i32));
                            let _ = canvas.draw_line(Point::new((x + size as f32) as i32, y as i32), Point::new(x as i32, (y + size as f32) as i32));
                            let _ = canvas.draw_line(Point::new(x as i32, (y + size as f32) as i32), Point::new((x - size as f32) as i32, y as i32));
                            let _ = canvas.draw_line(Point::new((x - size as f32) as i32, y as i32), Point::new(x as i32, (y - size as f32) as i32));
                        }
                    }
                    
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                14 => {
                    // THEME 15: WARP
                    // High speed starfield
                    canvas.set_draw_color(SdlColor::RGB(0, 0, 0));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    let cx = WINDOW_WIDTH as f32 / 2.0;
                    let cy = WINDOW_HEIGHT as f32 / 2.0;
                    let num_stars = 300;
                    
                    for i in 0..num_stars {
                        let seed = level as u64 * 9000 + i;
                        let mut warp_rng = StdRng::seed_from_u64(seed);
                        
                        let angle = warp_rng.gen_range(0.0..std::f32::consts::PI * 2.0);
                        let speed = warp_rng.gen_range(2.0..10.0);
                        
                        // Move stars outward
                        let dist_base = (time * speed + warp_rng.gen_range(0.0..1000.0)) % 1000.0;
                        // Exponential distance for warp effect
                        let dist = dist_base.powf(1.5) * 0.05;
                        
                        let x = cx + angle.cos() * dist;
                        let y = cy + angle.sin() * dist;
                        
                        if x >= 0.0 && x < WINDOW_WIDTH as f32 && y >= 0.0 && y < WINDOW_HEIGHT as f32 {
                            let size = (dist / 100.0) as u32 + 1;
                            let alpha = (dist / 2.0).min(255.0) as u8;
                            
                            // Color shift based on speed (Redshift/Blueshift)
                            let (r, g, b) = if speed > 8.0 {
                                (200, 200, 255) // Blueish
                            } else {
                                (255, 200, 200) // Reddish
                            };
                            
                            canvas.set_draw_color(SdlColor::RGBA(r, g, b, alpha));
                            
                            // Draw streak
                            let tail_len = (speed * 2.0) as f32;
                            let tail_x = x - angle.cos() * tail_len;
                            let tail_y = y - angle.sin() * tail_len;
                            
                            let _ = canvas.draw_line(Point::new(x as i32, y as i32), Point::new(tail_x as i32, tail_y as i32));
                        }
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                15 => {
                    // THEME 16: MAZE
                    // Procedural grid/maze background
                    canvas.set_draw_color(SdlColor::RGB(10, 10, 20));
                    canvas.clear();
                    
                    let cell_size = 40;
                    let cols = WINDOW_WIDTH / cell_size + 1;
                    let rows = WINDOW_HEIGHT / cell_size + 1;
                    
                    canvas.set_draw_color(SdlColor::RGB(30, 30, 60));
                    
                    for r in 0..rows {
                        for c in 0..cols {
                            let seed = level as u64 * 10000 + r as u64 * 100 + c as u64;
                            let mut maze_rng = StdRng::seed_from_u64(seed);
                            
                            let x = (c * cell_size) as i32;
                            let y = (r * cell_size) as i32;
                            
                            if maze_rng.gen_bool(0.5) {
                                // Diagonal 1
                                let _ = canvas.draw_line(Point::new(x, y), Point::new(x + cell_size as i32, y + cell_size as i32));
                            } else {
                                // Diagonal 2
                                let _ = canvas.draw_line(Point::new(x + cell_size as i32, y), Point::new(x, y + cell_size as i32));
                            }
                        }
                    }
                    
                    // Moving solver
                    let solver_x = (time * 2.0) % (WINDOW_WIDTH as f32);
                    let solver_y = (time * 1.5) % (WINDOW_HEIGHT as f32);
                    canvas.set_draw_color(SdlColor::RGB(0, 255, 0));
                    let _ = canvas.fill_rect(Rect::new(solver_x as i32 - 3, solver_y as i32 - 3, 6, 6));
                },
                16 => {
                    // THEME 17: SUPERNOVA
                    // Explosive center
                    canvas.set_draw_color(SdlColor::RGB(5, 0, 5));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    let cx = WINDOW_WIDTH as f32 / 2.0;
                    let cy = WINDOW_HEIGHT as f32 / 2.0;
                    
                    // Core
                    let pulse = (time * 0.1).sin() * 20.0 + 50.0;
                    canvas.set_draw_color(SdlColor::RGBA(255, 200, 100, 200));
                    let _ = canvas.fill_rect(Rect::new(cx as i32 - pulse as i32 / 2, cy as i32 - pulse as i32 / 2, pulse as u32, pulse as u32));
                    
                    // Shockwaves
                    let num_waves = 5;
                    for i in 0..num_waves {
                        let wave_dist = (time * 2.0 + i as f32 * 100.0) % 600.0;
                        let alpha = (255.0 * (1.0 - wave_dist / 600.0)) as u8;
                        
                        canvas.set_draw_color(SdlColor::RGBA(255, 100, 50, alpha));
                        // Draw circle approximation
                        let points = 32;
                        for p in 0..points {
                             let angle = (p as f32 / points as f32) * std::f32::consts::PI * 2.0;
                             let x = cx + angle.cos() * wave_dist;
                             let y = cy + angle.sin() * wave_dist;
                             let _ = canvas.draw_point(Point::new(x as i32, y as i32));
                        }
                    }
                    
                    // Debris
                    let num_debris = 100;
                    for i in 0..num_debris {
                        let seed = level as u64 * 11000 + i;
                        let mut debris_rng = StdRng::seed_from_u64(seed);
                        
                        let angle = debris_rng.gen_range(0.0..std::f32::consts::PI * 2.0);
                        let speed = debris_rng.gen_range(1.0..5.0);
                        let dist = (time * speed + debris_rng.gen_range(0.0..1000.0)) % 800.0;
                        
                        let x = cx + angle.cos() * dist;
                        let y = cy + angle.sin() * dist;
                        
                        let r = debris_rng.gen_range(100..255);
                        let g = debris_rng.gen_range(50..150);
                        let b = debris_rng.gen_range(50..100);
                        
                        canvas.set_draw_color(SdlColor::RGB(r, g, b));
                        let _ = canvas.fill_rect(Rect::new(x as i32, y as i32, 3, 3));
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                17 => {
                    // THEME 18: SPACE WARS
                    // Lasers
                    canvas.set_draw_color(SdlColor::RGB(0, 0, 10));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    let num_lasers = 20;
                    for i in 0..num_lasers {
                        let seed = level as u64 * 12000 + i;
                        let mut laser_rng = StdRng::seed_from_u64(seed);
                        
                        // Random firing intervals
                        let interval = laser_rng.gen_range(50..200);
                        let offset = laser_rng.gen_range(0..200);
                        
                        if (time as u64 + offset) % interval < 10 {
                            // Firing
                            let y = laser_rng.gen_range(0..WINDOW_HEIGHT as i32);
                            let color_type = laser_rng.gen_bool(0.5); // Red or Green
                            
                            if color_type {
                                canvas.set_draw_color(SdlColor::RGBA(255, 0, 0, 200)); // Rebel?
                            } else {
                                canvas.set_draw_color(SdlColor::RGBA(0, 255, 0, 200)); // Empire?
                            }
                            
                            let speed = 30.0;
                            let x_start = ((time as u64 + offset) % interval) as f32 * speed;
                            
                            let _ = canvas.fill_rect(Rect::new(x_start as i32, y, 100, 4));
                        }
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                18 => {
                    // THEME 19: DEEP SEA
                    // Dark blue, bubbles, organic shapes
                    canvas.set_draw_color(SdlColor::RGB(0, 5, 20));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    // Light shafts from top
                    for i in 0..5 {
                         let x = (i * 200) as i32 + (time * 0.1).sin() as i32 * 50;
                         let w = 100 + (time * 0.05 + i as f32).cos() as i32 * 20;
                         
                         // Gradient fade down
                         for h in (0..WINDOW_HEIGHT).step_by(20) {
                             let alpha = (50.0 * (1.0 - h as f32 / WINDOW_HEIGHT as f32)) as u8;
                             canvas.set_draw_color(SdlColor::RGBA(100, 200, 255, alpha));
                             let _ = canvas.fill_rect(Rect::new(x, h as i32, w as u32, 20));
                         }
                    }
                    
                    // Floating plankton
                    let num_plankton = 200;
                    for i in 0..num_plankton {
                        let seed = level as u64 * 13000 + i;
                        let mut sea_rng = StdRng::seed_from_u64(seed);
                        
                        let x_base = sea_rng.gen_range(0..WINDOW_WIDTH as i32);
                        let y_base = sea_rng.gen_range(0..WINDOW_HEIGHT as i32);
                        
                        let drift_x = (time * 0.02 + i as f32).sin() * 20.0;
                        let drift_y = (time * 0.03 + i as f32).cos() * 20.0;
                        
                        let x = x_base as f32 + drift_x;
                        let y = y_base as f32 + drift_y;
                        
                        canvas.set_draw_color(SdlColor::RGBA(200, 255, 200, 150));
                        let _ = canvas.draw_point(Point::new(x as i32, y as i32));
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                19 => {
                    // THEME 20: TORNADO
                    // Swirling funnel
                    canvas.set_draw_color(SdlColor::RGB(30, 30, 30));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    let cx = WINDOW_WIDTH as f32 / 2.0;
                    let num_particles = 300;
                    
                    for i in 0..num_particles {
                        let seed = level as u64 * 14000 + i;
                        let mut wind_rng = StdRng::seed_from_u64(seed);
                        
                        let height_pct = wind_rng.gen_range(0.0..1.0);
                        let y = height_pct * WINDOW_HEIGHT as f32;
                        
                        // Funnel width increases with height
                        let width = 50.0 + height_pct * 300.0;
                        
                        let angle_speed = 0.1 + (1.0 - height_pct) * 0.2; // Faster at bottom
                        let angle = time * angle_speed + wind_rng.gen_range(0.0..std::f32::consts::PI * 2.0);
                        
                        let radius = wind_rng.gen_range(width * 0.8..width);
                        let x = cx + angle.cos() * radius;
                        
                        // Color based on speed/chaos
                        let gray = wind_rng.gen_range(100..200);
                        canvas.set_draw_color(SdlColor::RGBA(gray, gray, gray, 150));
                        
                        let size = wind_rng.gen_range(2..5);
                        let _ = canvas.fill_rect(Rect::new(x as i32, y as i32, size, size));
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                },
                _ => {
                    // Fallback: Simple starfield (shouldn't normally reach here with 0..20 range)
                    canvas.set_draw_color(SdlColor::RGB(0, 0, 10));
                    canvas.clear();
                    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
                    
                    for i in 0..150 {
                        let seed = level as u64 * 15000 + i;
                        let mut star_rng = StdRng::seed_from_u64(seed);
                        
                        let x = star_rng.gen_range(0..WINDOW_WIDTH as i32);
                        let y = star_rng.gen_range(0..WINDOW_HEIGHT as i32);
                        let brightness = star_rng.gen_range(100..255);
                        
                        canvas.set_draw_color(SdlColor::RGB(brightness, brightness, brightness));
                        let _ = canvas.draw_point(Point::new(x, y));
                    }
                    canvas.set_blend_mode(sdl2::render::BlendMode::None);
                }
            }
        }
    }
}

pub fn render_game(
    canvas: &mut Canvas<Window>,
    game: &Game,
    menu: &Menu,
    background: Option<&mut Texture>,
    heart_texture: Option<&Texture>,
    splash_texture: Option<&Texture>,
    font: &Font,
    fps: f32,
    cache: &mut TextureCache,
) {
    // Handle splash screen state
    if game.state == GameState::SplashScreen {
        canvas.set_draw_color(SdlColor::RGB(0, 0, 0));
        canvas.clear();
        
        if let Some(splash) = splash_texture {
            // Get window size
            let (window_width, window_height) = canvas.output_size().unwrap_or((WINDOW_WIDTH, WINDOW_HEIGHT));
            
            // Get splash texture size
            let query = splash.query();
            let splash_width = query.width;
            let splash_height = query.height;
            
            // Center the splash image
            let x = (window_width as i32 - splash_width as i32) / 2;
            let y = (window_height as i32 - splash_height as i32) / 2;
            
            let target_rect = Rect::new(x, y, splash_width, splash_height);
            let _ = canvas.copy(splash, None, Some(target_rect));
        }
        
        canvas.present();
        return;
    }

    // Draw background
    if game.current_level > 6 {
        // Animated backgrounds for levels 7-9
        draw_animated_background(canvas, game.current_level, game.frame_count);
    } else {
        // Image backgrounds for levels 1-6
        canvas.set_draw_color(SdlColor::RGB(0, 0, 0));
        canvas.clear();

        if let Some(bg) = background {
            bg.set_blend_mode(sdl2::render::BlendMode::Blend);
            bg.set_alpha_mod(64);
            let _ = canvas.copy(bg, None, None);
        } else {
            canvas.set_draw_color(SdlColor::RGB(20, 20, 40));
            canvas.clear();
        }
    }

    // Draw blocks with gradient and glass effects
    for block in &game.blocks {
        if block.active {
            draw_block_with_gradient(canvas, block, cache);
        }
    }

    // Draw paddle with glass effect
    draw_paddle_with_glass(canvas, &game.paddle, cache);
    
    // Draw cannon on paddle if rocket ammo is available
    if game.paddle.rocket_ammo > 0 {
        let cannon_x = game.paddle.x + game.paddle.width / 2 - 5;
        let cannon_y = game.paddle.y - 15;
        
        // Cannon barrel (dark grey)
        canvas.set_draw_color(SdlColor::RGB(100, 100, 100));
        let _ = canvas.fill_rect(Rect::new(cannon_x, cannon_y, 10, 15));
        
        // Cannon highlight
        canvas.set_draw_color(SdlColor::RGB(150, 150, 150));
        let _ = canvas.fill_rect(Rect::new(cannon_x + 2, cannon_y, 2, 15));
        
        // Blinking text: "press space to launch"
        // Blink every 30 frames (about 0.5 seconds at 60 FPS)
        if (game.frame_count / 30) % 2 == 0 {
            let text = "press space to launch";
            if let Ok(surface) = font.render(text).blended(SdlColor::RGB(255, 255, 100)) {
                let texture_creator = canvas.texture_creator();
                if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
                    let text_x = game.paddle.x + game.paddle.width / 2 - surface.width() as i32 / 2;
                    let text_y = game.paddle.y - 35;
                    let target = Rect::new(text_x, text_y, surface.width(), surface.height());
                    let _ = canvas.copy(&texture, None, Some(target));
                };
            }
        }
    }

    // Draw balls (shiny circular metal balls)
    for ball in &game.balls {
        if ball.active {
            draw_shiny_ball(canvas, ball, font, cache, game.frame_count);
        }
    }

    // Draw bonuses with symbolic icons
    for bonus in &game.bonuses {
        if bonus.active {
            draw_bonus_icon(canvas, bonus);
        }
    }

    // Draw rocket projectiles
    for rocket in &game.rockets {
        if rocket.active {
            let rx = rocket.x as i32;
            let ry = rocket.y as i32;
            
            // Rocket body (orange)
            canvas.set_draw_color(SdlColor::RGB(255, 100, 0));
            let _ = canvas.fill_rect(Rect::new(rx + 2, ry + 4, 6, 12));
            
            // Rocket nose cone (red triangle)
            canvas.set_draw_color(SdlColor::RGB(255, 50, 50));
            for i in 0..4 {
                let _ = canvas.draw_line(
                    Point::new(rx + 5, ry + i),
                    Point::new(rx + 2 + i, ry + 4),
                );
                let _ = canvas.draw_line(
                    Point::new(rx + 5, ry + i),
                    Point::new(rx + 8 - i, ry + 4),
                );
            }
            
            // Rocket fins (dark orange)
            canvas.set_draw_color(SdlColor::RGB(200, 80, 0));
            let _ = canvas.fill_rect(Rect::new(rx, ry + 12, 3, 4));      // Left fin
            let _ = canvas.fill_rect(Rect::new(rx + 7, ry + 12, 3, 4));  // Right fin
            
            // Flame trail (yellow/orange gradient)
            canvas.set_draw_color(SdlColor::RGB(255, 255, 100));
            let _ = canvas.fill_rect(Rect::new(rx + 3, ry + 16, 4, 2));
            canvas.set_draw_color(SdlColor::RGB(255, 200, 50));
            let _ = canvas.fill_rect(Rect::new(rx + 3, ry + 18, 4, 2));
        }
    }

    // Draw particles
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    for particle in &game.particles {
        draw_particle(canvas, particle);
    }
    
    // Draw penguin animation if active
    if let Some(ref penguin) = game.penguin {
        draw_penguin(canvas, penguin);
    }

    canvas.set_blend_mode(sdl2::render::BlendMode::None);
    
    // Draw portal if active
    if game.portal_active {
        draw_portal(canvas, game.frame_count, game.portal_completion_timer);
    }

    // Draw HUD
    render_hud(canvas, game, heart_texture, font, fps);

    // Draw menu if paused or game over
    if game.state == GameState::Paused {
        render_pause_menu(canvas, menu, font);
    } else if game.state == GameState::GameOver {
        render_game_over_menu(canvas, game, font);
    } else if game.state == GameState::Victory {
        render_victory_menu(canvas, game, font);
    } else if game.state == GameState::LevelTransition {
        render_level_transition(canvas, game, font);
    }

    canvas.present();
}

/// Draw swirling portal at center of screen with multi-stage animation
/// Stages: Opening/Consuming (0-480), Closing (480-540), Flash (540-560), Fade (560-600)
fn draw_portal(canvas: &mut Canvas<Window>, frame_count: u64, completion_timer: u64) {
    let cx = WINDOW_WIDTH as i32 / 2;
    let cy = WINDOW_HEIGHT as i32 / 2;
    
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    
    // Animation stages based on completion_timer (Total 270 frames / 4.5 seconds)
    // Timer == 0: Portal consuming blocks
    // Timer 1-30: Portal stays open (0.5 seconds)
    // Timer 31-150: Portal closing (2 seconds)
    // Timer 151-180: Flash of light (0.5 seconds)
    // Timer 181-270: Fade out (1.5 seconds)
    
    if completion_timer == 0 || (completion_timer > 0 && completion_timer <= 30) {
        // Stage 1: Normal swirling portal consuming blocks
        for i in 0..10 {
            let radius = 150 - i * 10;
            let rotation = (frame_count as f32 * 0.1) + (i as f32 * 0.3);
            
            // Pulsing alpha
            let alpha = ((frame_count as f32 * 0.05 + i as f32 * 0.5).sin() * 100.0 + 155.0) as u8;
            
            // Purple gradient
            let color_shift = (i as f32 / 10.0 * 100.0) as u8;
            
            // Draw ring segments
            for j in 0..32 {
                let angle = (j as f32 / 32.0) * std::f32::consts::PI * 2.0 + rotation;
                let x = cx + (angle.cos() * radius as f32) as i32;
                let y = cy + (angle.sin() * radius as f32) as i32;
                
                // Draw filled circle using pixel drawing
                canvas.set_draw_color(SdlColor::RGBA(150 + color_shift, 50, 255 - color_shift, alpha));
                for dy in -4..=4 {
                    for dx in -4..=4 {
                        if dx*dx + dy*dy <= 16 {
                            let _ = canvas.draw_point(Point::new(x + dx, y + dy));
                        }
                    }
                }
            }
        }
    } else if completion_timer >= 31 && completion_timer <= 150 {
        // Stage 2: Portal closing - rings shrink inward (2 seconds)
        let close_progress = (completion_timer - 31) as f32 / 120.0; // 0.0 to 1.0 over 120 frames
        
        for i in 0..10 {
            let base_radius = 150 - i * 10;
            // Shrink radius to 0
            let radius = (base_radius as f32 * (1.0 - close_progress)).max(0.0) as i32;
            
            if radius <= 0 { continue; }

            let rotation = (frame_count as f32 * 0.15) + (i as f32 * 0.3); // Faster spin
            
            // Alpha fades slightly as it closes
            let base_alpha = ((frame_count as f32 * 0.05 + i as f32 * 0.5).sin() * 100.0 + 155.0) as u8;
            let alpha = (base_alpha as f32 * (1.0 - close_progress * 0.3)) as u8;
            
            // Purple gradient intensifies
            let color_shift = (i as f32 / 10.0 * 100.0) as u8;
            let r = (150 + color_shift + (close_progress * 50.0) as u8).min(255);
            let b = (255 - color_shift).saturating_sub((close_progress * 100.0) as u8);
            
            // Draw ring segments
            for j in 0..32 {
                let angle = (j as f32 / 32.0) * std::f32::consts::PI * 2.0 + rotation;
                let x = cx + (angle.cos() * radius as f32) as i32;
                let y = cy + (angle.sin() * radius as f32) as i32;
                
                canvas.set_draw_color(SdlColor::RGBA(r, 50, b, alpha));
                for dy in -4..=4 {
                    for dx in -4..=4 {
                        if dx*dx + dy*dy <= 16 {
                            let _ = canvas.draw_point(Point::new(x + dx, y + dy));
                        }
                    }
                }
            }
        }
    } else if completion_timer >= 151 && completion_timer <= 180 {
        // Stage 3: Bright flash of light
        let flash_progress = (completion_timer - 151) as f32 / 30.0; // 0.0 to 1.0
        
        // Bright expanding circle
        let flash_radius = (50.0 + flash_progress * 200.0) as i32;
        let flash_alpha = ((1.0 - flash_progress) * 255.0) as u8;
        
        // Draw expanding flash
        for dy in -flash_radius..=flash_radius {
            for dx in -flash_radius..=flash_radius {
                let dist_sq = dx*dx + dy*dy;
                if dist_sq <= flash_radius*flash_radius {
                    let dist = (dist_sq as f32).sqrt();
                    let edge_factor = 1.0 - (dist / flash_radius as f32);
                    let alpha = (flash_alpha as f32 * edge_factor) as u8;
                    
                    // Bright white-purple light
                    canvas.set_draw_color(SdlColor::RGBA(255, 200, 255, alpha));
                    let _ = canvas.draw_point(Point::new(cx + dx, cy + dy));
                }
            }
        }
        
        // Core bright spot
        let core_radius = 30;
        for dy in -core_radius..=core_radius {
            for dx in -core_radius..=core_radius {
                if dx*dx + dy*dy <= core_radius*core_radius {
                    canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, flash_alpha));
                    let _ = canvas.draw_point(Point::new(cx + dx, cy + dy));
                }
            }
        }
    } else if completion_timer >= 181 && completion_timer < 270 {
        // Stage 4: Fade out
        let fade_progress = (completion_timer - 181) as f32 / 89.0; // 0.0 to 1.0
        let fade_alpha = ((1.0 - fade_progress) * 150.0) as u8;
        
        // Gentle purple glow fading
        let glow_radius = 80;
        for dy in -glow_radius..=glow_radius {
            for dx in -glow_radius..=glow_radius {
                let dist_sq = dx*dx + dy*dy;
                if dist_sq <= glow_radius*glow_radius {
                    let dist = (dist_sq as f32).sqrt();
                    let edge_factor = 1.0 - (dist / glow_radius as f32);
                    let alpha = (fade_alpha as f32 * edge_factor) as u8;
                    
                    canvas.set_draw_color(SdlColor::RGBA(200, 150, 255, alpha));
                    let _ = canvas.draw_point(Point::new(cx + dx, cy + dy));
                }
            }
        }
    } else if completion_timer == 0 {
        // Portal just activated - normal swirling animation
        for i in 0..10 {
            let radius = 150 - i * 10;
            let rotation = (frame_count as f32 * 0.1) + (i as f32 * 0.3);
            
            // Pulsing alpha
            let alpha = ((frame_count as f32 * 0.05 + i as f32 * 0.5).sin() * 100.0 + 155.0) as u8;
            
            // Purple gradient
            let color_shift = (i as f32 / 10.0 * 100.0) as u8;
            
            // Draw ring segments
            for j in 0..32 {
                let angle = (j as f32 / 32.0) * std::f32::consts::PI * 2.0 + rotation;
                let x = cx + (angle.cos() * radius as f32) as i32;
                let y = cy + (angle.sin() * radius as f32) as i32;
                
                canvas.set_draw_color(SdlColor::RGBA(150 + color_shift, 50, 255 - color_shift, alpha));
                for dy in -4..=4 {
                    for dx in -4..=4 {
                        if dx*dx + dy*dy <= 16 {
                            let _ = canvas.draw_point(Point::new(x + dx, y + dy));
                        }
                    }
                }
            }
        }
    }
    
    canvas.set_blend_mode(sdl2::render::BlendMode::None);
}

fn render_hud(canvas: &mut Canvas<Window>, game: &Game, heart_texture: Option<&Texture>, font: &Font, fps: f32) {
    // Draw score text
    let score_text = format!("Score: {}", game.score);
    if let Ok(surface) = font.render(&score_text).blended(SdlColor::RGB(255, 255, 255)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(10, 10, surface.width(), surface.height());
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }
    
    // Draw lives as hearts
    if let Some(heart_tex) = heart_texture {
        // Use heart texture
        let heart_size = 20;
        for i in 0..game.lives {
            let x = WINDOW_WIDTH as i32 - 40 - i as i32 * 25;
            let y = 15;
            let _ = canvas.copy(
                heart_tex,
                None,
                Some(Rect::new(x, y, heart_size, heart_size)),
            );
        }
        
        // Draw stolen heart (if being stolen by penguin)
        if let Some((heart_x, heart_y)) = game.stolen_heart_position {
            let x = heart_x as i32 - (heart_size as i32 / 2);
            let y = heart_y as i32 - (heart_size as i32 / 2);
            let _ = canvas.copy(
                heart_tex,
                None,
                Some(Rect::new(x, y, heart_size, heart_size)),
            );
        }
    } else {
        // Fallback to drawn hearts
        for i in 0..game.lives {
            draw_heart(canvas, WINDOW_WIDTH as i32 - 40 - i as i32 * 25, 20, 12);
        }
        
        // Draw stolen heart (if being stolen by penguin)
        if let Some((heart_x, heart_y)) = game.stolen_heart_position {
            draw_heart(canvas, heart_x as i32, heart_y as i32, 12);
        }
    }

    
    // Draw level indicator (CENTER TOP)
    let level_text = if game.current_level <= 9 {
        format!("Level {}/9", game.current_level)
    } else {
        format!("Level {}/∞", game.current_level)
    };
    if let Ok(surface) = font.render(&level_text).blended(SdlColor::RGB(255, 255, 255)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(WINDOW_WIDTH as i32 / 2 - surface.width() as i32 / 2, 10, surface.width(), surface.height());
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }
    
    // Draw MAX SPEED indicator (BOTTOM LEFT)
    let max_speed_text = format!("MAX SPEED: {} px/s", game.max_speed as i32);
    
    // Check if new record was just broken (within last 2 seconds / 120 frames)
    let frames_since_record = game.frame_count.saturating_sub(game.max_speed_record_frame);
    let is_fresh_record = frames_since_record < 120;
    
    // Use gold pulsing color if fresh record, otherwise normal white
    let color = if is_fresh_record {
        let pulse = (game.frame_count as f32 * 0.15).sin() * 0.3 + 0.7;
        SdlColor::RGB(
            (255.0 * pulse) as u8,
            (255.0 * pulse) as u8,
            (100.0 * pulse + 100.0) as u8,
        )
    } else {
        SdlColor::RGB(200, 200, 200)
    };
    
    if let Ok(surface) = font.render(&max_speed_text).blended(color) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            // Position at bottom left
            let target = Rect::new(10, WINDOW_HEIGHT as i32 - 40, surface.width(), surface.height());
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }
    
    // Draw FPS counter (bottom-right)
    let fps_text = format!("FPS: {:.0}", fps);
    if let Ok(surface) = font.render(&fps_text).blended(SdlColor::RGB(200, 200, 200)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(
                WINDOW_WIDTH as i32 - surface.width() as i32 - 10,
                WINDOW_HEIGHT as i32 - surface.height() as i32 - 10,
                surface.width(),
                surface.height()
            );
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }
    
    // Draw GRAVITY MODE indicator (if active)
    if game.gravity_mode {
        let gravity_text = "GRAVITY MODE";
        // Pulsing orange/red color to make it stand out
        let pulse = (game.frame_count as f32 * 0.1).sin() * 0.3 + 0.7;
        let gravity_color = SdlColor::RGB(
            (255.0 * pulse) as u8,
            (100.0 * pulse) as u8,
            0,
        );
        
        if let Ok(surface) = font.render(gravity_text).blended(gravity_color) {
            let texture_creator = canvas.texture_creator();
            if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
                // Position below level indicator
                let target = Rect::new(
                    WINDOW_WIDTH as i32 / 2 - surface.width() as i32 / 2, 
                    35, 
                    surface.width(), 
                    surface.height()
                );
                let _ = canvas.copy(&texture, None, Some(target));
            };
        }
    }
}

/// Draw a particle (glass shard)
fn draw_particle(canvas: &mut Canvas<Window>, particle: &Particle) {
    let alpha = particle.alpha();
    
    // Draw rotated rectangle for glass shard
    let half_size = particle.size / 2;
    let angle = particle.rotation.to_radians();
    
    canvas.set_draw_color(SdlColor::RGBA(
        particle.color.r,
        particle.color.g,
        particle.color.b,
        alpha,
    ));
    
    // Simple diamond/shard shape
    for dx in -half_size..=half_size {
        for dy in -half_size..=half_size {
            if dx.abs() + dy.abs() <= half_size {
                let rotated_x = (dx as f32 * angle.cos() - dy as f32 * angle.sin()) as i32;
                let rotated_y = (dx as f32 * angle.sin() + dy as f32 * angle.cos()) as i32;
                let _ = canvas.draw_point(Point::new(
                    particle.x as i32 + rotated_x,
                    particle.y as i32 + rotated_y,
                ));
            }
        }
    }
}

/// Draw animated penguin with jetpack stealing a heart
fn draw_penguin(canvas: &mut Canvas<Window>, penguin: &Penguin) {
    let x = penguin.x as i32;
    let y = penguin.y as i32;
    
    // Gentler hover animation for flying
    let hover_offset = if matches!(penguin.state, PenguinState::WalkingIn | PenguinState::RunningAway) {
        ((penguin.frame_count as f32 * 0.15).sin() * 3.0) as i32
    } else {
        0
    };
    
    let body_y = y + hover_offset;
    
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    
    // Jetpack flames (if flying)
    if matches!(penguin.state, PenguinState::WalkingIn | PenguinState::RunningAway) {
        let flame_flicker = (penguin.frame_count % 4) as i32;
        
        // Yellow/orange flames
        canvas.set_draw_color(SdlColor::RGB(255, 200, 0));
        let _ = canvas.fill_rect(Rect::new(x + 8, body_y + 42, 4, (8 + flame_flicker) as u32));
        let _ = canvas.fill_rect(Rect::new(x + 13, body_y + 42, 4, (8 + flame_flicker) as u32));
        
        // Inner bright flame
        canvas.set_draw_color(SdlColor::RGB(255, 255, 150));
        let _ = canvas.fill_rect(Rect::new(x + 9, body_y + 42, 2, (5 + flame_flicker) as u32));
        let _ = canvas.fill_rect(Rect::new(x + 14, body_y + 42, 2, (5 + flame_flicker) as u32));
    }

    
    // Body (black oval)
    canvas.set_draw_color(SdlColor::RGB(30, 30, 40));
    let _ = canvas.fill_rect(Rect::new(x, body_y + 8, 25, 30));
    
    // Belly (white oval)
    canvas.set_draw_color(SdlColor::RGB(240, 240, 250));
    let _ = canvas.fill_rect(Rect::new(x + 6, body_y + 12, 13, 22));
    
    // Jetpack (gray rectangles on back)
    canvas.set_draw_color(SdlColor::RGB(100, 100, 120));
    let _ = canvas.fill_rect(Rect::new(x + 5, body_y + 20, 5, 15));
    let _ = canvas.fill_rect(Rect::new(x + 15, body_y + 20, 5, 15));
    
    // Head (black circle)
    canvas.set_draw_color(SdlColor::RGB(30, 30, 40));
    let _ = canvas.fill_rect(Rect::new(x + 3, body_y, 19, 18));
    
    // Eyes (white dots)
    canvas.set_draw_color(SdlColor::RGB(255, 255, 255));
    let _ = canvas.fill_rect(Rect::new(x + 8, body_y + 5, 3, 3));
    let _ = canvas.fill_rect(Rect::new(x + 14, body_y + 5, 3, 3));
    
    // Beak (orange triangle)
    canvas.set_draw_color(SdlColor::RGB(255, 140, 0));
    let _ = canvas.fill_rect(Rect::new(x + 10, body_y + 10, 5, 3));
    
    // Feet (orange - smaller when flying)
    canvas.set_draw_color(SdlColor::RGB(255, 140, 0));
    let _ = canvas.fill_rect(Rect::new(x + 7, body_y + 38, 5, 2));
    let _ = canvas.fill_rect(Rect::new(x + 13, body_y + 38, 5, 2));
    
    // Draw heart if grabbing or running away
    if matches!(penguin.state, PenguinState::Grabbing | PenguinState::RunningAway) {
        let heart_x = if penguin.state == PenguinState::Grabbing {
            x + 20 // Heart in grabbing position
        } else {
            x + 18 // Heart held while flying
        };
        let heart_y = body_y + 15;
        
        // Simple heart shape (red)
        canvas.set_draw_color(SdlColor::RGB(220, 20, 60));
        let _ = canvas.fill_rect(Rect::new(heart_x, heart_y + 2, 8, 6));
        let _ = canvas.fill_rect(Rect::new(heart_x + 1, heart_y, 3, 3));
        let _ = canvas.fill_rect(Rect::new(heart_x + 4, heart_y, 3, 3));
        let _ = canvas.fill_rect(Rect::new(heart_x + 2, heart_y + 8, 4, 2));
    }
    
    canvas.set_blend_mode(sdl2::render::BlendMode::None);
}

fn render_button(canvas: &mut Canvas<Window>, button: &Button, font: &Font) {
    // Button background
    let color = if button.hovered {
        SdlColor::RGBA(100, 100, 150, 200)
    } else {
        SdlColor::RGBA(60, 60, 100, 180)
    };
    
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.set_draw_color(color);
    let _ = canvas.fill_rect(button.rect);
    
    // Button border
    canvas.set_draw_color(SdlColor::RGB(200, 200, 200));
    let _ = canvas.draw_rect(button.rect);
    canvas.set_blend_mode(sdl2::render::BlendMode::None);
    
    // Button text
    if let Ok(surface) = font.render(&button.label).blended(SdlColor::RGB(255, 255, 255)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let text_x = button.rect.x() + (button.rect.width() as i32 - surface.width() as i32) / 2;
            let text_y = button.rect.y() + (button.rect.height() as i32 - surface.height() as i32) / 2;
            let target = Rect::new(text_x, text_y, surface.width(), surface.height());
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }
}

fn render_volume_slider(canvas: &mut Canvas<Window>, slider: &VolumeSlider, font: &Font) {
    // Slider background
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.set_draw_color(SdlColor::RGBA(80, 80, 80, 200));
    let _ = canvas.fill_rect(slider.rect);
    
    // Slider fill
    let fill_width = (slider.rect.width() as i32 * slider.value) / 128;
    let fill_rect = Rect::new(slider.rect.x(), slider.rect.y(), fill_width as u32, slider.rect.height());
    canvas.set_draw_color(SdlColor::RGBA(100, 200, 100, 220));
    let _ = canvas.fill_rect(fill_rect);
    
    // Border
    canvas.set_draw_color(SdlColor::RGB(200, 200, 200));
    let _ = canvas.draw_rect(slider.rect);
    canvas.set_blend_mode(sdl2::render::BlendMode::None);
    
    // Volume text
    let vol_text = format!("Volume: {}%", (slider.value * 100) / 128);
    if let Ok(surface) = font.render(&vol_text).blended(SdlColor::RGB(255, 255, 255)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(
                slider.rect.x(), 
                slider.rect.y() - 25, 
                surface.width(), 
                surface.height()
            );
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }
}

fn render_pause_menu(canvas: &mut Canvas<Window>, menu: &Menu, font: &Font) {
    // Semi-transparent overlay
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.set_draw_color(SdlColor::RGBA(0, 0, 0, 150));
    canvas.fill_rect(None).unwrap();
    canvas.set_blend_mode(sdl2::render::BlendMode::None);

    match menu.state {
        MenuState::Main => {
            // Title
            if let Ok(surface) = font.render("PAUSED").blended(SdlColor::RGB(255, 255, 255)) {
                let texture_creator = canvas.texture_creator();
                if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
                    let target = Rect::new(
                        WINDOW_WIDTH as i32 / 2 - surface.width() as i32 / 2,
                        WINDOW_HEIGHT as i32 / 2 - 150,
                        surface.width(),
                        surface.height(),
                    );
                    let _ = canvas.copy(&texture, None, Some(target));
                };
            }
            
            render_button(canvas, &menu.resume_button, font);
            render_button(canvas, &menu.restart_button, font);
            render_button(canvas, &menu.gravity_mode_button, font);
            render_button(canvas, &menu.settings_button, font);
            render_button(canvas, &menu.quit_button, font);
        }
        MenuState::Settings => {
            // Render settings menu
            render_button(canvas, &menu.music_toggle_button, font);
            render_volume_slider(canvas, &menu.music_slider, font);
            render_button(canvas, &menu.sfx_toggle_button, font);
            render_volume_slider(canvas, &menu.sfx_slider, font);
            render_button(canvas, &menu.fullscreen_button, font);
            render_button(canvas, &menu.back_button, font);
        }

    }
}

fn render_game_over_menu(canvas: &mut Canvas<Window>, game: &Game, font: &Font) {
    // Semi-transparent overlay
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.set_draw_color(SdlColor::RGBA(0, 0, 0, 180));
    canvas.fill_rect(None).unwrap();
    canvas.set_blend_mode(sdl2::render::BlendMode::None);

    let all_blocks_destroyed = game.blocks.iter().all(|b| !b.active);
    let title = if all_blocks_destroyed { "VICTORY!" } else { "GAME OVER" };
    let color = if all_blocks_destroyed {
        SdlColor::RGB(0, 255, 0)
    } else {
        SdlColor::RGB(255, 100, 100)
    };
    
    // Title
    if let Ok(surface) = font.render(title).blended(color) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(
                WINDOW_WIDTH as i32 / 2 - surface.width() as i32 / 2,
                WINDOW_HEIGHT as i32 / 2 - 100,
                surface.width(),
                surface.height(),
            );
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }
    
    // Score
    let score_text = format!("Final Score: {}", game.score);
    if let Ok(surface) = font.render(&score_text).blended(SdlColor::RGB(255, 255, 255)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(
                WINDOW_WIDTH as i32 / 2 - surface.width() as i32 / 2,
                WINDOW_HEIGHT as i32 / 2 - 40,
                surface.width(),
                surface.height(),
            );
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }
    
    // Instructions
    let inst_text = "Press R to Restart or Q to Quit";
    if let Ok(surface) = font.render(inst_text).blended(SdlColor::RGB(200, 200, 200)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(
                WINDOW_WIDTH as i32 / 2 - surface.width() as i32 / 2,
                WINDOW_HEIGHT as i32 / 2 + 20,
                surface.width(),
                surface.height(),
            );
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }
}

fn render_level_transition(canvas: &mut Canvas<Window>, game: &Game, font: &Font) {
    // Semi-transparent overlay
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.set_draw_color(SdlColor::RGBA(0, 0, 0, 150));
    canvas.fill_rect(None).unwrap();
    canvas.set_blend_mode(sdl2::render::BlendMode::None);

    // Level complete title
    let title = format!("Level {} Complete!", game.current_level);
    if let Ok(surface) = font.render(&title).blended(SdlColor::RGB(0, 255, 100)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(
                WINDOW_WIDTH as i32 / 2 - surface.width() as i32 / 2,
                WINDOW_HEIGHT as i32 / 2 - 100,
                surface.width(),
                surface.height(),
            );
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }
    
    // Score
    let score_text = format!("Score: {}", game.score);
    if let Ok(surface) = font.render(&score_text).blended(SdlColor::RGB(255, 255, 255)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(
                WINDOW_WIDTH as i32 / 2 - surface.width() as i32 / 2,
                WINDOW_HEIGHT as i32 / 2 - 40,
                surface.width(),
                surface.height(),
            );
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }
    
    // Click to continue prompt
    let prompt_text = "Click to start next level";
    if let Ok(surface) = font.render(prompt_text).blended(SdlColor::RGB(255, 255, 100)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(
                WINDOW_WIDTH as i32 / 2 - surface.width() as i32 / 2,
                WINDOW_HEIGHT as i32 / 2 + 40,
                surface.width(),
                surface.height(),
            );
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }
}

fn render_victory_menu(canvas: &mut Canvas<Window>, game: &Game, font: &Font) {
    // Semi-transparent overlay
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.set_draw_color(SdlColor::RGBA(0, 0, 0, 180));
    canvas.fill_rect(None).unwrap();
    canvas.set_blend_mode(sdl2::render::BlendMode::None);

    // Victory title
    if let Ok(surface) = font.render("CONGRATULATIONS!").blended(SdlColor::RGB(255, 215, 0)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(
                WINDOW_WIDTH as i32 / 2 - surface.width() as i32 / 2,
                WINDOW_HEIGHT as i32 / 2 - 120,
                surface.width(),
                surface.height(),
            );
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }
    
    // Subtitle
    let subtitle = "Campaign Complete!";
    if let Ok(surface) = font.render(subtitle).blended(SdlColor::RGB(0, 255, 0)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(
                WINDOW_WIDTH as i32 / 2 - surface.width() as i32 / 2,
                WINDOW_HEIGHT as i32 / 2 - 70,
                surface.width(),
                surface.height(),
            );
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }
    
    // Score
    let score_text = format!("Final Score: {}", game.score);
    if let Ok(surface) = font.render(&score_text).blended(SdlColor::RGB(255, 255, 255)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(
                WINDOW_WIDTH as i32 / 2 - surface.width() as i32 / 2,
                WINDOW_HEIGHT as i32 / 2 - 20,
                surface.width(),
                surface.height(),
            );
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }
    
    // Instructions
    let inst_text = "Press ENTER for Infinite Mode";
    if let Ok(surface) = font.render(inst_text).blended(SdlColor::RGB(255, 215, 0)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(
                WINDOW_WIDTH as i32 / 2 - surface.width() as i32 / 2,
                WINDOW_HEIGHT as i32 / 2 + 40,
                surface.width(),
                surface.height(),
            );
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }

    // Instructions 2
    let inst_text2 = "Press R to Restart or Q to Quit";
    if let Ok(surface) = font.render(inst_text2).blended(SdlColor::RGB(200, 200, 200)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(
                WINDOW_WIDTH as i32 / 2 - surface.width() as i32 / 2,
                WINDOW_HEIGHT as i32 / 2 + 80,
                surface.width(),
                surface.height(),
            );
            let _ = canvas.copy(&texture, None, Some(target));
        };
    }
}
