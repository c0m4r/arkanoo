# Arkanoo 🎮

A fully-featured Arkanoid/Breakout game written in Rust.

## Features

- 🎯 **6 Levels with 200 Blocks** - Up to 200 rainbow-colored blocks across 6 unique levels with gradient and glass effects
- 📋 **Unique Level Patterns**:
  - **Level 1**: Full Grid (200 blocks)
  - **Level 2**: Checkerboard Pattern
  - **Level 3**: Horizontal Stripes
  - **Level 4**: Pillars with Borders
  - **Level 5**: Pyramid/Triangle Shape
  - **Level 6**: Diamond/X Shape
- 🎁 **Bonus System** - Random drops (15% chance):
  - ⚽ Extra Ball - Spawns a second ball (red circle icon)
  - 📏 Long Paddle - Extends paddle width temporarily (green bar icon)
- 📊 **HUD** - Score display and lives shown as ❤️ red hearts
- ⏸️ **Interactive Menu** - Click buttons or use keyboard:
  - Resume, Restart, Settings, Quit
  - Volume control slider
  - Sound mute/unmute toggle
  - Resolution options (1280×720, 1920×1080, 2560×1440)
  - Fullscreen toggle
- 🎵 **Dynamic Music** - 6 unique songs, one for each level
- 🔊 **Audio** - MP3 support for ball bounce sounds and background music
- 🌆 **Custom Backgrounds** - 6 unique PNG backgrounds (one per level)
- 🖱️ **Mouse Control** - Control paddle with mouse movement
- 🏆 **Level Transitions** - Win animations and prompts between levels
- ✨ **Particle Effects** - Glass-shattering particles when blocks are destroyed

## Visual Enhancements

- **Gradient Blocks** - Top-to-bottom color fade on each block
- **Glass Effects** - Semi-transparent overlays on blocks and paddle
- **Symbolic Bonuses** - Easy-to-recognize icons (ball and paddle symbols)
- **Heart Lives** - Lives displayed as red heart shapes (PNG texture)
- **Dynamic Backgrounds** - 6 unique PNG backgrounds that change per level
- **Smooth Animations** - Particle effects and transitions
- **Scalable Graphics** - Resolution options and fullscreen support

## Controls

| Key/Action | Function |
|-----|--------|
| ← / → | Move paddle left/right (keyboard) |
| Mouse Movement | Move paddle left/right (in-game) |
| Left Click | Start next level (during transitions) |
| ESC | Pause/Resume game |
| F11 | Toggle fullscreen |
| R | Restart (in pause/game over) |
| Q | Quit (in pause/game over) |

**In Pause Menu:**
- Resume - Continue game
- Restart - Start new game
- Settings - Adjust audio settings
- Quit - Exit game

**In Settings:**
- Sound Toggle - Enable/disable all audio
- Volume Slider - Adjust music and sound effect volume (0-100%)
- Resolution - Cycle through available resolutions (1280×720, 1920×1080, 2560×1440)
- Fullscreen - Toggle fullscreen mode

## Building

### Prerequisites

#### Linux

Install SDL2 development libraries:

```bash
# Debian/Ubuntu
sudo apt-get install libsdl2-dev libsdl2-mixer-dev libsdl2-ttf-dev libsdl2-image-dev

# Fedora
sudo dnf install SDL2-devel SDL2_mixer-devel SDL2_ttf-devel SDL2_image-devel

# Arch
sudo pacman -S sdl2 sdl2_mixer sdl2_ttf sdl2_image
```

#### Windows

Building on Windows requires additional setup:

**1. Install Visual Studio Build Tools**

Download and install:
- [Visual Studio Build Tools](https://aka.ms/vs/stable/vs_BuildTools.exe)

**2. Install CMake (version 3.31.10)**

Download and install from either:
- [CMake Official](https://cmake.org/files/v3.31/cmake-3.31.10-windows-x86_64.msi)
- [CMake GitHub Mirror](https://github.com/Kitware/CMake/releases/download/v3.31.10/cmake-3.31.10-windows-x86_64.msi)

**3. Download SDL2 Prebuilt Binaries**

Download the latest Windows development libraries from:
- [SDL2](https://github.com/libsdl-org/SDL/releases) - Main SDL library
- [SDL2_ttf](https://github.com/libsdl-org/SDL_ttf/releases) - TrueType font support
- [SDL2_image](https://github.com/libsdl-org/SDL_image/releases) - Image loading (PNG)
- [SDL2_mixer](https://github.com/libsdl-org/SDL_mixer/releases) - Audio mixing (MP3)

**4. Extract and Organize SDL Files**

In your `arkanoo` directory (next to `Cargo.toml`):
1. Extract all `.lib` files from the downloaded SDL archives to the root directory
2. Combine all `include` folders from each SDL library into a single `include` directory
3. Your directory structure should look like:
   ```
   arkanoo/
   ├── Cargo.toml
   ├── SDL2.lib
   ├── SDL2_ttf.lib
   ├── SDL2_image.lib
   ├── SDL2_mixer.lib
   ├── include/
   │   └── SDL2/
   │       ├── *.h (combined headers from all SDL libraries)
   └── src/
   ```

### Required Assets

Place the following files in the `assets/` directory:

**Audio:**
- `ball.mp3` - Ball bounce sound effect
- `song1.mp3` through `song6.mp3` - Background music (one per level)

**Graphics:**
- `background1.png` through `background6.png` - Background images (one per level)
- `heart.png` - Heart icon for life display

### Compile

```bash
cargo build --release
```

The binary will be at `target/release/arkanoo` (approximately 540 KB).

## Running

```bash
./target/release/arkanoo
```


## Game Rules

- Start with **3 lives** (shown as ❤️)
- Each block destroyed: **+10 points**
- Collect falling bonuses to gain advantages
- **6 Levels** with unique block patterns
- Clear all blocks in a level to proceed to the next
- Complete all 6 levels to win!
- Game over if all lives are lost

## Menu Navigation

The pause menu features **clickable buttons** with hover effects:

- **Hover** - Buttons highlight when you move the mouse over them
- **Click** - Left-click to activate buttons
- **Volume Slider** - Click and drag to adjust volume in real-time
- **Sound Toggle** - Instantly mute/unmute all audio

## Project Structure

```
arkanoo/
├── src/
│   ├── main.rs         # Entry point & game loop with menu integration
│   ├── entities.rs     # Game entities (Paddle, Ball, Block, Bonus, Particle)
│   ├── game.rs         # Core game logic with 6 levels
│   ├── rendering.rs    # Graphics with gradients, glass effects, particles
│   ├── audio.rs        # Level-based music management
│   └── menu.rs         # Interactive menu with settings
├── assets/
│   ├── ball.mp3        # Bounce sound effect
│   ├── song1.mp3       # Level 1 music
│   ├── song2.mp3       # Level 2 music
│   ├── song3.mp3       # Level 3 music
│   ├── song4.mp3       # Level 4 music
│   ├── song5.mp3       # Level 5 music
│   ├── song6.mp3       # Level 6 music
│   ├── background1.png # Level 1 background
│   ├── background2.png # Level 2 background
│   ├── background3.png # Level 3 background
│   ├── background4.png # Level 4 background
│   ├── background5.png # Level 5 background
│   ├── background6.png # Level 6 background
│   └── heart.png       # Heart life icon
└── Cargo.toml
```

## Dependencies

Minimal dependencies as requested:

- `sdl2` (with mixer, ttf, image features) - Graphics, input, audio, and text rendering
- `rand` - Random number generation for bonus drops and song selection

## Troubleshooting

### No Audio
- Ensure all MP3 files (`ball.mp3`, `song1.mp3` - `song6.mp3`) are in `assets/` directory
- Check file permissions (read access required)
- Verify SDL2_mixer is installed with MP3 support

### Background Not Showing
- Ensure all background PNG files (`background1.png` - `background6.png`) exist in `assets/`
- Verify SDL2_image is installed with PNG support

### Font Issues (Windows)
- The game uses `C:\Windows\Fonts\Arial.ttf` on Windows
- Ensure the font file exists or modify `src/main.rs` to use a different font

### Font Issues (Linux)
- The game uses `/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf` on Linux
- Install DejaVu fonts if missing: `sudo apt-get install fonts-dejavu-core`

## License

WTFPL / effectively Public Domain

## Authors

Gemini 3 Pro (High) + Claude Sonnet 4.5 (Thinking)
