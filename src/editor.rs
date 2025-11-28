use crate::entities::*;
use crate::menu::Button;
use sdl2::rect::Rect;
use std::fs;

/// Serializable pattern data structure with ASCII format
#[derive(Clone)]
pub struct PatternData {
    pub name: String,
    // Store (color_index, block_type_char)
    // 255 for empty
    // For blocks: (0-9, 'N'|'I'|'E'|'U')
    // We'll pack this into a custom struct or just use a more complex grid
    // Let's use a struct for grid cells to be clean
    pub grid: [[PatternCell; BLOCK_COLS]; BLOCK_ROWS],
}

#[derive(Clone, Copy, PartialEq)]
pub struct PatternCell {
    pub color_index: u8, // 255 = empty
    pub block_type: BlockType,
}

impl PatternData {
    pub fn new(name: String) -> Self {
        PatternData {
            name,
            grid: [[PatternCell { color_index: 255, block_type: BlockType::Normal }; BLOCK_COLS]; BLOCK_ROWS],
        }
    }

    /// Save pattern to ASCII format
    /// * = empty space
    ///   0-5 = Normal blocks with color index
    ///   6 = Ice block
    ///   7 = Explosive block
    ///   8 = Undestroyable block
    pub fn save_to_file(&self, dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        fs::create_dir_all(dir)?;
        
        let filename = format!("{}/{}.txt", dir, self.name);
        let mut content = String::new();
        
        // Header with pattern name
        content.push_str(&format!("# Pattern: {}\n", self.name));
        content.push_str("# * = empty, 0-5 = normal, 6 = Ice, 7 = Explosive, 8 = Undestroyable\n\n");
        
        // Write grid
        for row in 0..BLOCK_ROWS {
            for col in 0..BLOCK_COLS {
                let cell = self.grid[row][col];
                let ch = if cell.color_index == 255 {
                    '*'
                } else {
                    // 0-5 for normal blocks, 6-8 for special blocks
                    match cell.block_type {
                        BlockType::Ice => '6',
                        BlockType::Explosive => '7',
                        BlockType::Undestroyable => '8',
                        BlockType::Normal => char::from_digit(cell.color_index as u32, 10).unwrap_or('0'),
                    }
                };
                content.push(ch);
            }
            content.push('\n');
        }
        
        fs::write(filename, content)?;
        Ok(())
    }

    /// Load pattern from ASCII format
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut grid = [[PatternCell { color_index: 255, block_type: BlockType::Normal }; BLOCK_COLS]; BLOCK_ROWS];
        
        // Extract pattern name from path
        let name = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("pattern")
            .to_string();
        
        let mut row = 0;
        for line in content.lines() {
            // Skip comments and empty lines
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            
            if row >= BLOCK_ROWS {
                break;
            }
            
            for (col, ch) in line.chars().take(BLOCK_COLS).enumerate() {
                let (color_index, block_type) = match ch {
                    '*' => (255, BlockType::Normal),
                    '0'..='5' => (ch.to_digit(10).unwrap() as u8, BlockType::Normal),
                    '6' => (0, BlockType::Ice),
                    '7' => (0, BlockType::Explosive),
                    '8' => (0, BlockType::Undestroyable),
                    // Backward compatibility with old format
                    'I' => (0, BlockType::Ice),
                    'E' => (0, BlockType::Explosive),
                    'U' => (0, BlockType::Undestroyable),
                    _ => (255, BlockType::Normal),
                };
                grid[row][col] = PatternCell { color_index, block_type };
            }
            
            row += 1;
        }
        
        Ok(PatternData { name, grid })
    }
}

/// Load all patterns from a directory
pub fn load_all_patterns(dir: &str) -> Vec<PatternData> {
    let mut patterns = Vec::new();
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                if let Ok(pattern) = PatternData::load_from_file(path.to_str().unwrap()) {
                    patterns.push(pattern);
                }
            }
        }
    }
    
    patterns
}

/// Convert pattern data to blocks for the game
pub fn create_blocks_from_pattern(pattern: &PatternData) -> Vec<Block> {
    let mut blocks = Vec::new();
    let total_blocks_width = BLOCK_COLS as i32 * BLOCK_WIDTH;
    let offset_x = (WINDOW_WIDTH as i32 - total_blocks_width) / 2;

    for row in 0..BLOCK_ROWS {
        for col in 0..BLOCK_COLS {
            let cell = pattern.grid[row][col];
            if cell.color_index != 255 {
                let x = offset_x + col as i32 * BLOCK_WIDTH;
                let y = BLOCK_OFFSET_Y + row as i32 * BLOCK_HEIGHT;
                let color_idx = (cell.color_index as usize) % BLOCK_COLORS.len();
                let color = BLOCK_COLORS[color_idx];
                blocks.push(Block::new(x, y, color, cell.block_type));
            }
        }
    }

    blocks
}

/// Color picker button
pub struct ColorButton {
    pub rect: Rect,
    pub color_index: usize,
    pub hovered: bool,
}

impl ColorButton {
    pub fn new(x: i32, y: i32, color_index: usize) -> Self {
        ColorButton {
            rect: Rect::new(x, y, 40, 30),
            color_index,
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


/// Level editor state
pub struct LevelEditor {
    pub blocks: Vec<Block>,
    pub selected_color_index: usize,
    pub pattern_name: String,
    pub pattern_name_editing: bool,
    pub save_button: Button,
    pub clear_button: Button,
    pub test_button: Button,
    pub load_button: Button,
    pub exit_button: Button,
    pub bg_next_button: Button,
    pub bg_prev_button: Button,
    pub color_buttons: Vec<ColorButton>,
    pub message: String,
    pub message_timer: u32,
    pub current_background: usize, // 1-6 for levels
    pub is_dragging_left: bool,
    pub is_dragging_right: bool,
    pub last_drag_pos: Option<(i32, i32)>,
    pub confirm_clear: bool, // Whether we're asking for clear confirmation
    pub frame_count: u64,
    pub pattern_browser_open: bool,
    pub available_patterns: Vec<String>,
    pub selected_pattern_index: usize,
}

impl LevelEditor {
    pub fn new() -> Self {
        let button_y = WINDOW_HEIGHT as i32 - 60;
        let button_width = 150;
        let button_height = 40;
        let spacing = 170;
        let start_x = (WINDOW_WIDTH as i32 - (spacing * 5 - 20)) / 2;

        // Color picker buttons (9 colors: 0-5 normal, 6-8 special blocks)
        let color_picker_y = 20;
        let color_picker_x_start = WINDOW_WIDTH as i32 - 420;
        let mut color_buttons = Vec::new();
        for i in 0..9 {
            color_buttons.push(ColorButton::new(
                color_picker_x_start + (i as i32 * 45),
                color_picker_y,
                i,
            ));
        }

        LevelEditor {
            blocks: Vec::new(),
            selected_color_index: 0,
            pattern_name: String::from("my_pattern"),
            pattern_name_editing: false,
            save_button: Button::new(start_x, button_y, button_width, button_height, "Save (S)"),
            clear_button: Button::new(start_x + spacing, button_y, button_width, button_height, "Clear (C)"),
            test_button: Button::new(start_x + spacing * 2, button_y, button_width, button_height, "Test (T)"),
            load_button: Button::new(start_x + spacing * 3, button_y, button_width, button_height, "Load (L)"),
            exit_button: Button::new(start_x + spacing * 4, button_y, button_width, button_height, "Exit (ESC)"),
            bg_next_button: Button::new(WINDOW_WIDTH as i32 - 130, WINDOW_HEIGHT as i32 - 110, 120, 35, "BG Next >"),
            bg_prev_button: Button::new(WINDOW_WIDTH as i32 - 260, WINDOW_HEIGHT as i32 - 110, 120, 35, "< BG Prev"),
            color_buttons,
            message: String::new(),
            message_timer: 0,
            current_background: 1,
            is_dragging_left: false,
            is_dragging_right: false,
            last_drag_pos: None,
            confirm_clear: false,
            frame_count: 0,
            pattern_browser_open: false,
            available_patterns: Vec::new(),
            selected_pattern_index: 0,
        }
    }

    pub fn discover_patterns(&mut self) {
        self.available_patterns.clear();
        
        if let Ok(entries) = fs::read_dir("patterns") {
            for entry in entries.flatten() {
                if let Ok(path) = entry.path().canonicalize() {
                    if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            self.available_patterns.push(name.to_string());
                        }
                    }
                }
            }
        }
        
        self.available_patterns.sort();
        self.selected_pattern_index = 0;
    }

    pub fn load_pattern(&mut self, name: &str) -> Result<(), String> {
        let path = format!("patterns/{}.txt", name);
        
        match PatternData::load_from_file(&path) {
            Ok(pattern) => {
                // Convert pattern data to blocks
                self.blocks.clear();
                
                for row in 0..BLOCK_ROWS {
                    for col in 0..BLOCK_COLS {
                        let cell = pattern.grid[row][col];
                        if cell.color_index != 255 {
                            let total_blocks_width = BLOCK_COLS as i32 * BLOCK_WIDTH;
                            let offset_x = (WINDOW_WIDTH as i32 - total_blocks_width) / 2;
                            let x = offset_x + col as i32 * BLOCK_WIDTH;
                            let y = BLOCK_OFFSET_Y + row as i32 * BLOCK_HEIGHT;
                            
                            self.blocks.push(Block {
                                x,
                                y,
                                active: true,
                                color: BLOCK_COLORS[cell.color_index as usize % BLOCK_COLORS.len()],
                                block_type: cell.block_type,
                                health: 1,
                                max_health: 1,
                            });
                        }
                    }
                }
                
                self.pattern_name = pattern.name;
                self.show_message(format!("Loaded pattern: {}", name));
                Ok(())
            }
            Err(e) => {
                self.show_message(format!("Failed to load: {}", e));
                Err(format!("Failed to load pattern: {}", e))
            }
        }
    }

    pub fn save_pattern(&mut self) -> Result<(), String> {
        let mut pattern = PatternData::new(self.pattern_name.clone());
        
        let total_blocks_width = BLOCK_COLS as i32 * BLOCK_WIDTH;
        let offset_x = (WINDOW_WIDTH as i32 - total_blocks_width) / 2;

        // Convert blocks to grid with color indices
        for block in &self.blocks {
            if block.active {
                let col = ((block.x - offset_x) / BLOCK_WIDTH) as usize;
                let row = ((block.y - BLOCK_OFFSET_Y) / BLOCK_HEIGHT) as usize;
                
                if row < BLOCK_ROWS && col < BLOCK_COLS {
                    // Find color index
                    for (idx, &color) in BLOCK_COLORS.iter().enumerate() {
                        if block.color.r == color.r && block.color.g == color.g && block.color.b == color.b {
                            pattern.grid[row][col] = PatternCell {
                                color_index: idx as u8,
                                block_type: block.block_type,
                            };
                            break;
                        }
                    }
                }
            }
        }

        // Validate: ensure at least some blocks
        let block_count = pattern.grid.iter()
            .flatten()
            .filter(|&&cell| cell.color_index != 255)
            .count();
            
        if block_count == 0 {
            return Err("Pattern must have at least one block".to_string());
        }

        match pattern.save_to_file("patterns") {
            Ok(_) => {
                self.show_message(format!("Saved: {}.txt", self.pattern_name));
                Ok(())
            }
            Err(e) => Err(format!("Failed to save: {}", e)),
        }
    }

    pub fn request_clear(&mut self) {
        if !self.blocks.is_empty() {
            self.confirm_clear = true;
            self.show_message("Press C again to confirm clearing all blocks".to_string());
        }
    }
    
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.confirm_clear = false;
        self.show_message("Pattern cleared".to_string());
    }
    
    pub fn cancel_clear(&mut self) {
        self.confirm_clear = false;
        self.message.clear();
    }

    pub fn add_block_at(&mut self, mouse_x: i32, mouse_y: i32) {
        let total_blocks_width = BLOCK_COLS as i32 * BLOCK_WIDTH;
        let offset_x = (WINDOW_WIDTH as i32 - total_blocks_width) / 2;

        // Calculate grid position
        if mouse_x < offset_x || mouse_x >= offset_x + total_blocks_width {
            return;
        }
        if mouse_y < BLOCK_OFFSET_Y || mouse_y >= BLOCK_OFFSET_Y + (BLOCK_ROWS as i32 * BLOCK_HEIGHT) {
            return;
        }

        let col = (mouse_x - offset_x) / BLOCK_WIDTH;
        let row = (mouse_y - BLOCK_OFFSET_Y) / BLOCK_HEIGHT;

        let x = offset_x + col * BLOCK_WIDTH;
        let y = BLOCK_OFFSET_Y + row * BLOCK_HEIGHT;

        // Check if block already exists at this position
        let block_exists = self.blocks.iter().any(|b| b.x == x && b.y == y);
        
        if !block_exists {
            // Add new block with selected color and type
            // Indices 0-5 are normal blocks, 6-8 are special blocks
            let (color, block_type) = match self.selected_color_index {
                6 => (BLOCK_COLORS[0], BlockType::Ice),
                7 => (BLOCK_COLORS[0], BlockType::Explosive),
                8 => (BLOCK_COLORS[0], BlockType::Undestroyable),
                _ => (BLOCK_COLORS[self.selected_color_index % BLOCK_COLORS.len()], BlockType::Normal),
            };
            self.blocks.push(Block::new(x, y, color, block_type));
        }
    }

    pub fn remove_block_at(&mut self, mouse_x: i32, mouse_y: i32) {
        let total_blocks_width = BLOCK_COLS as i32 * BLOCK_WIDTH;
        let offset_x = (WINDOW_WIDTH as i32 - total_blocks_width) / 2;

        // Check bounds
        if mouse_x < offset_x || mouse_x >= offset_x + total_blocks_width {
            return;
        }
        if mouse_y < BLOCK_OFFSET_Y || mouse_y >= BLOCK_OFFSET_Y + (BLOCK_ROWS as i32 * BLOCK_HEIGHT) {
            return;
        }

        let col = (mouse_x - offset_x) / BLOCK_WIDTH;
        let row = (mouse_y - BLOCK_OFFSET_Y) / BLOCK_HEIGHT;

        let x = offset_x + col * BLOCK_WIDTH;
        let y = BLOCK_OFFSET_Y + row * BLOCK_HEIGHT;

        self.blocks.retain(|b| !(b.x == x && b.y == y));
    }

    pub fn start_drag_left(&mut self, mouse_x: i32, mouse_y: i32) {
        self.is_dragging_left = true;
        self.last_drag_pos = Some((mouse_x, mouse_y));
        self.add_block_at(mouse_x, mouse_y);
    }

    pub fn start_drag_right(&mut self, mouse_x: i32, mouse_y: i32) {
        self.is_dragging_right = true;
        self.last_drag_pos = Some((mouse_x, mouse_y));
        self.remove_block_at(mouse_x, mouse_y);
    }

    pub fn update_drag(&mut self, mouse_x: i32, mouse_y: i32) {
        if self.is_dragging_left {
            self.add_block_at(mouse_x, mouse_y);
            self.last_drag_pos = Some((mouse_x, mouse_y));
        } else if self.is_dragging_right {
            self.remove_block_at(mouse_x, mouse_y);
            self.last_drag_pos = Some((mouse_x, mouse_y));
        }
    }

    pub fn stop_drag(&mut self) {
        self.is_dragging_left = false;
        self.is_dragging_right = false;
        self.last_drag_pos = None;
    }

    pub fn update_hover(&mut self, mouse_x: i32, mouse_y: i32) {
        self.save_button.update_hover(mouse_x, mouse_y);
        self.clear_button.update_hover(mouse_x, mouse_y);
        self.test_button.update_hover(mouse_x, mouse_y);
        self.load_button.update_hover(mouse_x, mouse_y);
        self.exit_button.update_hover(mouse_x, mouse_y);
        self.bg_next_button.update_hover(mouse_x, mouse_y);
        self.bg_prev_button.update_hover(mouse_x, mouse_y);
        
        for btn in &mut self.color_buttons {
            btn.update_hover(mouse_x, mouse_y);
        }
    }

    pub fn next_background(&mut self) {
        self.current_background = if self.current_background >= 6 { 1 } else { self.current_background + 1 };
    }

    pub fn prev_background(&mut self) {
        self.current_background = if self.current_background <= 1 { 6 } else { self.current_background - 1 };
    }

    pub fn get_background_path(&self) -> String {
        format!("assets/background{}.png", self.current_background)
    }

    pub fn update(&mut self) {
        self.frame_count = self.frame_count.wrapping_add(1);
        if self.message_timer > 0 {
            self.message_timer -= 1;
            if self.message_timer == 0 {
                self.message.clear();
            }
        }
    }

    pub fn show_message(&mut self, msg: String) {
        self.message = msg;
        self.message_timer = 180; // 3 seconds at 60 FPS
    }

    pub fn handle_text_input(&mut self, text: &str) {
        if self.pattern_name_editing {
            self.pattern_name.push_str(text);
        }
    }

    pub fn handle_backspace(&mut self) {
        if self.pattern_name_editing && !self.pattern_name.is_empty() {
            self.pattern_name.pop();
        }
    }
}
