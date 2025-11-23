# Arkanoo 🎮

A fully-featured Arkanoid/Breakout game written in Rust.

## Features

- 🎯 **60 Colorful Blocks** - Rainbow-colored blocks in a 6×10 grid with gradient and glass effects
- 🎁 **Bonus System** - Random drops (15% chance):
  - ⚽ Extra Ball - Spawns a second ball (red circle icon)
  - 📏 Long Paddle - Extends paddle width temporarily (green bar icon)
- 📊 **HUD** - Score display and lives shown as ❤️ red hearts
- ⏸️ **Interactive Menu** - Click buttons or use keyboard:
  - Resume, Restart, Settings, Quit
  - Volume control slider
  - Sound mute/unmute toggle
- 🎵 **Multi-Song Playlist** - Plays 4 songs (song1-4.mp3) in order, starting from a random song
- 🔊 **Audio** - MP3 support for ball bounce sounds and background music
- 🌆 **Custom Background** - Support for PNG background images

## Visual Enhancements

- **Gradient Blocks** - Top-to-bottom color fade on each block
- **Glass Effects** - Semi-transparent overlays on blocks and paddle
- **Symbolic Bonuses** - Easy-to-recognize icons (ball and paddle symbols)
- **Heart Lives** - Lives displayed as red heart shapes
- **Custom Background** - Optional PNG background image support

## Controls

| Key/Action | Function |
|-----|--------|
| ← / → | Move paddle left/right |
| ESC | Pause/Resume game |
| Mouse | Navigate menus, adjust volume slider |
| Left Click | Select menu items, drag volume slider |

**In Pause Menu:**
- Resume - Continue game
- Restart - Start new game
- Settings - Adjust audio settings
- Quit - Exit game

**In Settings:**
- Sound Toggle - Enable/disable all audio
- Volume Slider - Adjust music and sound effect volume (0-100%)

## Building

### Prerequisites

Install SDL2 development libraries:

```bash
# Debian/Ubuntu
sudo apt-get install libsdl2-dev libsdl2-mixer-dev libsdl2-ttf-dev libsdl2-image-dev

# Fedora
sudo dnf install SDL2-devel SDL2_mixer-devel SDL2_ttf-devel SDL2_image-devel

# Arch
sudo pacman -S sdl2 sdl2_mixer sdl2_ttf sdl2_image
```

### Audio Assets

Place your audio files in the `assets/` directory:
- `ball.mp3` - Ball bounce sound effect
- `song1.mp3` through `song4.mp3` - Background music playlist

The game will automatically cycle through the 4 songs in order, starting from a randomly selected one.

### Optional: Background Image

- `assets/background.png` - Custom background image (optional)

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
- Clear all 60 blocks to win!
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
│   ├── landlock.rs     # Sandboxing implementation
│   ├── entities.rs     # Game entities (Paddle, Ball, Block, Bonus)
│   ├── game.rs         # Core game logic
│   ├── rendering.rs    # Graphics with gradients, glass effects, text
│   ├── audio.rs        # Multi-song playlist management
│   └── menu.rs         # Interactive menu system
├── assets/
│   ├── ball.mp3        # Bounce sound effect
│   ├── song1.mp3       # Background music track 1
│   ├── song2.mp3       # Background music track 2
│   ├── song3.mp3       # Background music track 3
│   ├── song4.mp3       # Background music track 4
│   └── background.png  # Optional background image
└── Cargo.toml
```

## Dependencies

Minimal dependencies as requested:

- `sdl2` (with mixer, ttf, image features) - Graphics, input, audio, and text rendering
- `rand` - Random number generation for bonus drops and song selection

## Troubleshooting

### No Audio
- Ensure MP3 files are in `assets/` directory
- Check file permissions (read access required)
- Verify SDL2_mixer is installed with MP3 support

### Background Not Showing
- Ensure `assets/background.png` exists
- Verify SDL2_image is installed with PNG support

## License

Public Domain

## Authors

Gemini 3 Pro (High) + Claude Sonnet 4.5 (Thinking)
