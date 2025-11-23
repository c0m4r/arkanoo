use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use sdl2::pixels::Color as SdlColor;
use sdl2::rect::{Rect, Point};
use sdl2::ttf::Font;
use crate::game::{Game, GameState};
use crate::entities::*;
use crate::menu::{Menu, MenuState, Button, VolumeSlider};

/// Draw a shiny metal ball with speed text and fireball effect
fn draw_shiny_ball(canvas: &mut Canvas<Window>, ball: &Ball, font: &Font) {
    let cx = ball.x as i32 + BALL_SIZE / 2;
    let cy = ball.y as i32 + BALL_SIZE / 2;
    let radius = BALL_SIZE / 2;
    
    // Calculate ball speed
    let speed_px_frame = (ball.vel_x.powi(2) + ball.vel_y.powi(2)).sqrt();
    let speed_px_sec = speed_px_frame * 60.0; // Assuming 60 FPS
    
    // Draw fireball/comet trail effect if speed >= 1000px/s
    if speed_px_sec >= 1000.0 {
        canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
        
        // Calculate direction opposite to velocity
        let vel_mag = speed_px_frame;
        let dir_x = -ball.vel_x / vel_mag;
        let dir_y = -ball.vel_y / vel_mag;
        
        // Draw flame trail particles behind the ball
        for i in 1..=20 {
            let trail_dist = i as f32 * 2.0;
            let trail_x = cx + (dir_x * trail_dist) as i32;
            let trail_y = cy + (dir_y * trail_dist) as i32;
            
            // Fade out and shrink as we go back
            let alpha = ((20 - i) as f32 / 20.0 * 180.0) as u8;
            let trail_radius = radius - (i / 4);
            
            // Color gradient from white/yellow to red/orange
            let (r, g, b) = if i < 7 {
                (255u8, (255 - i * 20) as u8, 100u8) // Yellow-white
            } else if i < 14 {
                (255u8, (140 - (i - 7) * 15) as u8, 50u8) // Orange
            } else {
                (200u8, 50u8, 20u8) // Red
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
        
        canvas.set_blend_mode(sdl2::render::BlendMode::None);
    }
    
    // Draw filled circle with gradient
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= radius * radius {
                // Calculate distance from center for gradient
                let dist = (dist_sq as f32).sqrt();
                let factor = 1.0 - (dist / radius as f32);
                
                // Base color with gradient (silver/steel)
                let brightness = (160.0 + factor * 95.0) as u8;
                
                // Add specular highlight in top-left quadrant
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

    // Draw speed text
    let speed_text = format!("{} px/s", speed_px_sec as i32);
    
    if let Ok(surface) = font.render(&speed_text).blended(SdlColor::RGB(200, 200, 200)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            // Position text above the ball
            let text_width = surface.width();
            let text_height = surface.height();
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
            let _ = canvas.copy(&texture, None, Some(target));
        };
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
fn draw_block_with_gradient(canvas: &mut Canvas<Window>, block: &Block) {
    let rect = block.rect();
    
    // 1. Base fill (slightly darker for depth)
    let r = (block.color.r as f32 * 0.7) as u8;
    let g = (block.color.g as f32 * 0.7) as u8;
    let b = (block.color.b as f32 * 0.7) as u8;
    canvas.set_draw_color(SdlColor::RGB(r, g, b));
    let _ = canvas.fill_rect(rect);
    
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

    // 2. Metallic/Glass Shine (Horizon line effect)
    for y in 0..rect.height() {
        let factor = y as f32 / rect.height() as f32;
        // Create a "horizon" reflection at 40% height
        let alpha = if factor < 0.4 {
            ((1.0 - factor / 0.4) * 120.0) as u8 // Fade out from top
        } else if factor < 0.5 {
            ((factor - 0.4) / 0.1 * 200.0) as u8 // Sharp bright line
        } else {
            ((1.0 - (factor - 0.5) / 0.5) * 80.0) as u8 // Fade out to bottom
        };
        
        canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, alpha));
        let _ = canvas.draw_line(
            Point::new(rect.x(), rect.y() + y as i32),
            Point::new(rect.x() + rect.width() as i32, rect.y() + y as i32),
        );
    }
    
    // 3. Inner Glow (Color boost)
    let glow_rect = Rect::new(rect.x() + 2, rect.y() + 2, rect.width() - 4, rect.height() - 4);
    canvas.set_draw_color(SdlColor::RGBA(block.color.r, block.color.g, block.color.b, 150));
    let _ = canvas.draw_rect(glow_rect);

    // 4. 3D Bevel Borders
    // Top and Left (Highlight)
    canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, 200));
    let _ = canvas.draw_line(rect.top_left(), rect.top_right());
    let _ = canvas.draw_line(rect.top_left(), rect.bottom_left());
    
    // Bottom and Right (Shadow)
    canvas.set_draw_color(SdlColor::RGBA(0, 0, 0, 180));
    let _ = canvas.draw_line(rect.bottom_left(), rect.bottom_right());
    let _ = canvas.draw_line(rect.top_right(), rect.bottom_right());

    canvas.set_blend_mode(sdl2::render::BlendMode::None);
}

/// Draw paddle with enhanced sci-fi/metallic aesthetics and rounded corners
fn draw_paddle_with_glass(canvas: &mut Canvas<Window>, paddle: &Paddle) {
    let radius = 10; // Corner radius
    let x = paddle.x;
    let y = paddle.y;
    let w = paddle.width;
    let h = 20; // Fixed height for visual consistency, or use paddle.height if it existed
    // Note: Paddle struct doesn't have height, assuming 20 based on previous code or standard
    // Actually let's use a fixed height of 20 as per previous implementation logic
    let h = 20; 

    // 1. Main Body - Dark Metallic Blue/Grey (Pixel-by-pixel for rounded shape)
    
    canvas.set_draw_color(SdlColor::RGB(40, 50, 70));
    
    // Center rect
    let _ = canvas.fill_rect(Rect::new(x + radius, y, (w - 2 * radius) as u32, h as u32));
    // Left/Right rects (between corners)
    let _ = canvas.fill_rect(Rect::new(x, y + radius, radius as u32, (h - 2 * radius) as u32));
    let _ = canvas.fill_rect(Rect::new(x + w - radius, y + radius, radius as u32, (h - 2 * radius) as u32));
    
    // 4 Corners (filled circles)
    let corners = [
        (x + radius, y + radius),
        (x + w - radius, y + radius),
        (x + radius, y + h - radius),
        (x + w - radius, y + h - radius),
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

    // 2. Energy Core (Glowing center strip)
    let core_height = 4;
    let core_y = y + (h - core_height) / 2;
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
    
    // 3. Metallic Gradient on body
    for line_y in 0..h {
        if line_y >= (h - core_height) / 2 && line_y < (h + core_height) / 2 {
            continue; // Skip core
        }
        
        let factor = line_y as f32 / h as f32;
        let alpha = ((1.0 - (factor - 0.5).abs() * 2.0) * 100.0) as u8;
        canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, alpha));
        
        // Draw horizontal line clipped to rounded shape
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
            Point::new(x + start_x, y + line_y),
            Point::new(x + end_x, y + line_y),
        );
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

    // 5. Tech Borders (Outline)
    canvas.set_draw_color(SdlColor::RGBA(100, 200, 255, 150));
    
    // Top/Bottom lines
    let _ = canvas.draw_line(Point::new(x + radius, y), Point::new(x + w - radius, y));
    let _ = canvas.draw_line(Point::new(x + radius, y + h - 1), Point::new(x + w - radius, y + h - 1));
    // Side lines
    let _ = canvas.draw_line(Point::new(x, y + radius), Point::new(x, y + h - radius));
    let _ = canvas.draw_line(Point::new(x + w - 1, y + radius), Point::new(x + w - 1, y + h - radius));
    
    // Corner arcs
    // Top-left
    for i in 0..=90 {
        let rad = (i as f32 + 180.0).to_radians();
        let px = x + radius + (radius as f32 * rad.cos()) as i32;
        let py = y + radius + (radius as f32 * rad.sin()) as i32;
        let _ = canvas.draw_point(Point::new(px, py));
    }
    // Top-right
    for i in 0..=90 {
        let rad = (i as f32 + 270.0).to_radians();
        let px = x + w - radius + (radius as f32 * rad.cos()) as i32;
        let py = y + radius + (radius as f32 * rad.sin()) as i32;
        let _ = canvas.draw_point(Point::new(px, py));
    }
    // Bottom-right
    for i in 0..=90 {
        let rad = (i as f32).to_radians();
        let px = x + w - radius + (radius as f32 * rad.cos()) as i32;
        let py = y + h - radius + (radius as f32 * rad.sin()) as i32;
        let _ = canvas.draw_point(Point::new(px, py));
    }
    // Bottom-left
    for i in 0..=90 {
        let rad = (i as f32 + 90.0).to_radians();
        let px = x + radius + (radius as f32 * rad.cos()) as i32;
        let py = y + h - radius + (radius as f32 * rad.sin()) as i32;
        let _ = canvas.draw_point(Point::new(px, py));
    }
    
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
        BonusType::ExtraBall => (100, 150, 255),  // Blue
        BonusType::LongPaddle => (100, 255, 100),  // Green
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
            
            let mut rng = StdRng::seed_from_u64(level as u64);
            
            // Randomly select one of the 6 themes (instead of cycling)
            let theme = rng.gen_range(0..6);
            
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
                _ => {
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
    font: &Font,
    fps: f32,
) {
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
            draw_block_with_gradient(canvas, block);
        }
    }

    // Draw paddle with glass effect
    draw_paddle_with_glass(canvas, &game.paddle);

    // Draw balls (shiny circular metal balls)
    for ball in &game.balls {
        if ball.active {
            draw_shiny_ball(canvas, ball, font);
        }
    }

    // Draw bonuses with symbolic icons
    for bonus in &game.bonuses {
        if bonus.active {
            draw_bonus_icon(canvas, bonus);
        }
    }

    // Draw particles
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    for particle in &game.particles {
        draw_particle(canvas, particle);
    }
    canvas.set_blend_mode(sdl2::render::BlendMode::None);

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
            let x = WINDOW_WIDTH as i32 - 30 - i as i32 * 25;
            let y = 15;
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
    }
    
    // Draw level indicator
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
                        WINDOW_HEIGHT as i32 / 2 - 120,
                        surface.width(),
                        surface.height(),
                    );
                    let _ = canvas.copy(&texture, None, Some(target));
                };
            }
            
            render_button(canvas, &menu.resume_button, font);
            render_button(canvas, &menu.restart_button, font);
            render_button(canvas, &menu.settings_button, font);
            render_button(canvas, &menu.quit_button, font);
        }
        MenuState::Settings => {
            // Title
            if let Ok(surface) = font.render("SETTINGS").blended(SdlColor::RGB(255, 255, 255)) {
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
            
            render_button(canvas, &menu.mute_button, font);
            render_volume_slider(canvas, &menu.volume_slider, font);
            render_button(canvas, &menu.resolution_button, font);
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
