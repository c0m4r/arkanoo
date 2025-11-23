use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use sdl2::pixels::Color as SdlColor;
use sdl2::rect::{Rect, Point};
use sdl2::ttf::Font;
use crate::game::{Game, GameState};
use crate::entities::*;
use crate::menu::{Menu, MenuState, Button, VolumeSlider};

/// Draw a shiny metal ball with speed text
fn draw_shiny_ball(canvas: &mut Canvas<Window>, ball: &Ball, font: &Font) {
    let cx = ball.x as i32 + BALL_SIZE / 2;
    let cy = ball.y as i32 + BALL_SIZE / 2;
    let radius = BALL_SIZE / 2;
    
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
    let speed_px_frame = (ball.vel_x.powi(2) + ball.vel_y.powi(2)).sqrt();
    let speed_px_sec = (speed_px_frame * 60.0) as i32; // Assuming 60 FPS
    let speed_text = format!("{} px/s", speed_px_sec);
    
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
fn draw_paddle_with_glass(canvas: &mut Canvas<Window>, paddle_rect: Rect) {
    let radius = 10; // Corner radius
    let x = paddle_rect.x();
    let y = paddle_rect.y();
    let w = paddle_rect.width() as i32;
    let h = paddle_rect.height() as i32;

    // 1. Main Body - Dark Metallic Blue/Grey (Pixel-by-pixel for rounded shape)
    // Optimization: Draw main rects then corners? No, for gradient we need per-line.
    // Actually, let's draw the base shape using primitives for performance.
    
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
    canvas.set_draw_color(SdlColor::RGBA(0, 200, 255, 200)); // Cyan glow
    let _ = canvas.fill_rect(core_rect);
    
    // 3. Metallic Gradient on body (Applied to the whole bounding box, masked by shape logic would be expensive)
    // Instead, we just draw lines on the central part and handle corners simply.
    
    for line_y in 0..h {
        if line_y >= (h - core_height) / 2 && line_y < (h + core_height) / 2 {
            continue; // Skip core
        }
        
        let factor = line_y as f32 / h as f32;
        let alpha = ((1.0 - (factor - 0.5).abs() * 2.0) * 100.0) as u8;
        canvas.set_draw_color(SdlColor::RGBA(255, 255, 255, alpha));
        
        // Draw horizontal line clipped to rounded shape
        // Calculate start/end x for this y
        let mut start_x = 0;
        let mut end_x = w;
        
        if line_y < radius {
            // Top corners
            let dy = radius - line_y;
            let dx = ((radius * radius - dy * dy) as f32).sqrt() as i32;
            start_x = radius - dx;
            end_x = w - radius + dx;
        } else if line_y >= h - radius {
            // Bottom corners
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

    // 4. Thruster/Engine Lights on ends (adapted for rounded shape)
    // We'll put them slightly inside the rounded ends
    let light_width = 4;
    let left_light = Rect::new(x + 2, y + h/2 - 6, light_width, 12);
    let right_light = Rect::new(x + w - 2 - light_width as i32, y + h/2 - 6, light_width, 12);
    
    canvas.set_draw_color(SdlColor::RGBA(255, 100, 50, 200)); // Orange engine glow
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
    
    // Corner arcs (approximated with points or small lines)
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
    
    canvas.set_blend_mode(sdl2::render::BlendMode::None);
}

/// Draw a round light glowing energy capsule
fn draw_bonus_icon(canvas: &mut Canvas<Window>, bonus: &Bonus) {
    let rect = bonus.rect();
    let cx = rect.x() + rect.width() as i32 / 2;
    let cy = rect.y() + rect.height() as i32 / 2;
    let radius = 10;
    
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    
    // Determine color based on bonus type
    let (r, g, b) = match bonus.bonus_type {
        BonusType::ExtraBall => (50, 100, 255),  // Blue glow (Changed from Red)
        BonusType::LongPaddle => (50, 255, 50),  // Green glow
    };

    // Draw outer glow (fading out)
    for r_off in 0..8 {
        let alpha = (100 - r_off * 12) as u8;
        canvas.set_draw_color(SdlColor::RGBA(r, g, b, alpha));
        
        // Draw circle outline for glow
        let current_radius = radius + r_off;
        for angle in 0..360 {
            let rad = (angle as f32).to_radians();
            let x = cx + (current_radius as f32 * rad.cos()) as i32;
            let y = cy + (current_radius as f32 * rad.sin()) as i32;
            let _ = canvas.draw_point(Point::new(x, y));
        }
    }

    // Draw solid capsule/orb core
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let dist_sq = dx*dx + dy*dy;
            if dist_sq <= radius*radius {
                let dist = (dist_sq as f32).sqrt();
                // Gradient from center (white) to edge (color)
                let factor = dist / radius as f32;
                
                let cr = (255.0 * (1.0 - factor) + r as f32 * factor) as u8;
                let cg = (255.0 * (1.0 - factor) + g as f32 * factor) as u8;
                let cb = (255.0 * (1.0 - factor) + b as f32 * factor) as u8;
                
                canvas.set_draw_color(SdlColor::RGBA(cr, cg, cb, 220));
                let _ = canvas.draw_point(Point::new(cx + dx, cy + dy));
            }
        }
    }
    
    // Draw symbol inside
    canvas.set_draw_color(SdlColor::RGB(255, 255, 255));
    match bonus.bonus_type {
        BonusType::ExtraBall => {
            // Small dot
            let inner_radius = 3;
             for dy in -inner_radius..=inner_radius {
                for dx in -inner_radius..=inner_radius {
                    if dx*dx + dy*dy <= inner_radius*inner_radius {
                        let _ = canvas.draw_point(Point::new(cx + dx, cy + dy));
                    }
                }
            }
        }
        BonusType::LongPaddle => {
            // Horizontal line
            let _ = canvas.fill_rect(Rect::new(cx - 5, cy - 2, 10, 4));
        }
    }
    
    canvas.set_blend_mode(sdl2::render::BlendMode::None);
}

pub fn render_game(
    canvas: &mut Canvas<Window>,
    game: &Game,
    menu: &Menu,
    background: Option<&mut Texture>,
    heart_texture: Option<&Texture>,
    font: &Font,
) {
    // Draw background
    // First fill with black base
    canvas.set_draw_color(SdlColor::RGB(0, 0, 0));
    canvas.clear();

    if let Some(bg) = background {
        bg.set_blend_mode(sdl2::render::BlendMode::Blend); // Ensure blending is enabled
        bg.set_alpha_mod(64); // opacity (0 - 255, where 128 is 50%)
        let _ = canvas.copy(bg, None, None);
    } else {
        canvas.set_draw_color(SdlColor::RGB(20, 20, 40));
        canvas.clear();
    }

    // Draw blocks with gradient and glass effects
    for block in &game.blocks {
        if block.active {
            draw_block_with_gradient(canvas, block);
        }
    }

    // Draw paddle with glass effect
    draw_paddle_with_glass(canvas, game.paddle.rect());

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
    render_hud(canvas, game, heart_texture, font);

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

fn render_hud(canvas: &mut Canvas<Window>, game: &Game, heart_texture: Option<&Texture>, font: &Font) {
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
    let level_text = format!("Level {}/6", game.current_level);
    if let Ok(surface) = font.render(&level_text).blended(SdlColor::RGB(255, 255, 255)) {
        let texture_creator = canvas.texture_creator();
        if let Ok(texture) = texture_creator.create_texture_from_surface(&surface) {
            let target = Rect::new(WINDOW_WIDTH as i32 / 2 - surface.width() as i32 / 2, 10, surface.width(), surface.height());
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
    let subtitle = "All Levels Complete!";
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
    let inst_text = "Press R to Play Again or Q to Quit";
    if let Ok(surface) = font.render(inst_text).blended(SdlColor::RGB(200, 200, 200)) {
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
