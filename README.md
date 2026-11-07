<div align="center">

# Arkanoo <img src="assets/icon-32.png" alt="" />

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://rust-lang.org/) [![Google Gemini](https://img.shields.io/badge/google%20gemini-8E75B2?style=for-the-badge&logo=google%20gemini&logoColor=white)](https://deepmind.google/models/gemini/pro/) [![Claude](https://img.shields.io/badge/Claude-D97757?style=for-the-badge&logo=claude&logoColor=white)](https://www.anthropic.com/claude/sonnet) [![Itch.io](https://img.shields.io/badge/Itch-%23FF0B34.svg?style=for-the-badge&logo=Itch.io&logoColor=white)](https://c0m4r.itch.io/arkanoo)

A fully-featured Arkanoid/Breakout game written in Rust.

<video src="https://github.com/user-attachments/assets/47f5c644-4434-4ea0-9d2b-bda05203688c"></video>

</div>

## Contents

- [Download and play](#download-and-play)
- [Features](#features)
- [Controls](#controls)
- [Game rules](#game-rules)
- [Level editor](#level-editor)
- [Settings](#settings)
- [Troubleshooting](#troubleshooting)
- [Building from source](#building-from-source)
- [Project structure](#project-structure)
- [License and credits](#license-and-credits)

## Download and play

Grab the latest build from [GitHub Releases](https://github.com/c0m4r/arkanoo/releases) or [itch.io](https://c0m4r.itch.io/arkanoo). Everything the game needs is in the package, so there is nothing else to install.

### Linux

Download `arkanoo-<version>-linux.tar.gz`, then:

```bash
tar -xzf arkanoo-*-linux.tar.gz
cd arkanoo-*/
./arkanoo
```

Requires 64-bit Linux with glibc 2.34 or newer (Ubuntu 22.04+, Debian 12+, Fedora 35+, Arch, Manjaro, …), running X11 or Wayland.

### Windows

Download `arkanoo-<version>-windows.zip`, extract it (right-click → **Extract All**) and double-click `arkanoo.exe`.

Requires 64-bit Windows 10 or 11. The executable isn't code-signed, so SmartScreen may warn you the first time; choose **More info → Run anyway**.

### Verifying the download

Each package includes the SHA-256 checksum of the game binary:

```bash
sha256sum -c arkanoo.sha256                                   # Linux
```

```powershell
Get-FileHash arkanoo.exe; Get-Content arkanoo.exe.sha256      # Windows (PowerShell)
```

## Features

### Gameplay

- 🎯 **Campaign and Infinite Mode**: 9 handcrafted levels, followed by endless procedurally generated stages.
- 🌪️ **Spin**: move the paddle as the ball hits it to curve the ball's path.
- 🪐 **Gravity Mode**: an optional mode where the ball accelerates downward and spin is disabled.
- 🎁 **Power-ups**: Long Paddle, Extra Ball, Ghost Ball and Rockets (see [Power-ups](#power-ups)).
- 🧊 **Special blocks**: Ice, Explosive and Undestroyable blocks for custom levels.
- 🌀 **Speed Portal**: push the ball past 3600 px/s and a portal opens that swallows the remaining blocks for a 5000 point bonus.
- 🐧 **Heart thief**: when you lose a life, a penguin waddles in and steals the heart.
- 🛠️ **Level editor**: design your own layouts, test them instantly, and have them show up in Infinite Mode.

### Visuals

- **Levels 1–6** have illustrated backgrounds, and **levels 7–9** have animated ones (starfield, pulsing circles, digital rain).
- **Infinite Mode** picks from 20 procedural themes: Cyber Grid, Star Voyage, Hex Pulse, Aurora Waves, Matrix Rain, Nebula Clouds, Rain & Thunder, Snow, Starry Night, Smoke, Bubbles, Fire, Digital Circuit, Kaleidoscope, Warp, Maze, Supernova, Space Wars, Deep Sea and Tornado.
- Gradient, glass-style blocks and paddle, a glowing paddle when spin is applied, and particle effects when blocks shatter.
- Rockets come with a nose cone, fins and a flame trail. A cannon appears on the paddle while you have ammo.
- The HUD shows score, level, lives as hearts, your top ball speed and FPS.
- The game renders at 1280×720 and scales cleanly to any window size or fullscreen.

### Audio

- A piano soundtrack played in random order.
- Sound effects for bounces, explosions, rocket launches and lost lives.
- Separate on/off toggles and volume sliders for music and sound effects.

## Controls

### In game

| Input | Action |
|---|---|
| Mouse / **←** **→** | Move the paddle |
| **Space** | Launch the ball, then fire rockets when you have ammo |
| **Esc** | Pause and open the menu |
| **F11** | Toggle fullscreen |
| **Enter** / Left click | Skip the splash screen; start the next level |
| **R** | Restart (when paused or after the game ends) |
| **Q** | Quit (when paused or after the game ends) |

<details>
<summary>Cheat</summary>

Pressing **Q** while playing clears the current level and skips to the next one.

</details>

### Menu

The menu works with the mouse: hover to highlight a button, click to activate it, and drag the sliders to change volume.

| Main menu | |
|---|---|
| **New Game / Resume** | Start playing, or return to the current game |
| **Restart** | Start over from level 1 |
| **Gravity Mode** | Toggle Gravity Mode |
| **Level Editor** | Open the [level editor](#level-editor) |
| **Settings** | Audio and display [settings](#settings) |
| **Quit** | Save settings and exit |

## Game rules

- You start with **3 lives**. Finishing a level after losing a life gives one back (up to 3).
- Clear every destroyable block to finish a level. Undestroyable blocks don't count.
- After level 9 you've beaten the campaign. Click or press Enter to continue into **Infinite Mode**.
- Infinite Mode layouts come from 12 generators (scatter, waves, stripes, rings, checkerboard, cellular, honeycomb, mirror, maze, diamonds, invaders and DNA helix) plus any custom patterns in `patterns/`.
- The game is over when you run out of lives.

### Scoring

| Event | Points |
|---|---|
| Block destroyed | +10 |
| Ball bounces off the paddle | +5 |
| Power-up collected | +2 |
| Speed Portal opened | +5000 |
| Life lost | −20 (score never goes below 0) |

### Power-ups

A destroyed block has a 15% chance to drop a power-up, at most one per second.

| Power-up | Chance | Effect |
|---|---|---|
| 📏 **Long Paddle** | 50% | Wider paddle for 5 seconds |
| ⚽ **Extra Ball** | 25% | Adds another ball |
| 👻 **Ghost Ball** | 15% | Balls pass through blocks for 10 seconds |
| 🚀 **Rocket** | 10% | +1 rocket. Press Space to fire it; it explodes on impact and destroys blocks within about 2 blocks |

### Special blocks

| Block | Behavior |
|---|---|
| **Ice** | Takes 2 hits |
| **Explosive** | Blows up neighboring blocks when destroyed |
| **Undestroyable** | Can't be destroyed and isn't needed to finish the level |

## Level editor

Open **Level Editor** from the main menu to design layouts on the 20×10 block grid.

| Input | Action |
|---|---|
| Left click / drag | Paint blocks |
| Right click / drag | Erase blocks |
| **0**–**5** | Normal block colors |
| **6** / **7** / **8** | Ice / Explosive / Undestroyable block |
| **N** | Rename the pattern (Enter to finish) |
| **S** | Save to `patterns/<name>.txt` |
| **L** | Open the pattern browser (**↑** **↓** to select, **Enter** to load) |
| **T** | Test-play the layout |
| **C** | Clear the grid (press twice to confirm) |
| **Esc** | Back to the menu |

Every action is also available as an on-screen button, along with arrows for previewing the level backgrounds.

Saved patterns are added to the Infinite Mode rotation. They are plain text files, so you can also write them by hand: up to 10 rows of 20 characters, where `#` starts a comment line.

```
# * = empty, 0-5 = normal, 6 = Ice, 7 = Explosive, 8 = Undestroyable
88666666666666666688
88777777777777777788
88111111111111111188
88000000000000000088
```

## Settings

- **Music** and **SFX**: on/off toggles and volume sliders.
- **Resolution**: 1280×720, 1366×768, 1600×900, 1920×1080 or 2560×1440. A new resolution is reverted after 5 seconds unless you click **Keep**.
- **Fullscreen / Windowed**: also toggled with F11.
- **VSync**: takes effect the next time you start the game.

Settings, including Gravity Mode, are saved to `settings.json` in the game folder.

## Troubleshooting

**Linux: `version 'GLIBC_2.34' not found`.** Your distribution is older than the release build supports. [Build from source](#building-from-source) instead.

**No sound.** Check the Music and SFX toggles and sliders in Settings. On Linux, the game plays audio through PipeWire, PulseAudio or ALSA, so one of them must be working.

**Missing graphics or text.** The `assets/` folder must stay next to the executable. Extract the whole package, not just the binary.

**"Failed to save settings".** The game folder must be writable, because `settings.json` is stored there.

**Wrong resolution or a broken display after changing settings.** Delete `settings.json` to restore the defaults.

## Building from source

### Release packages

`release.sh` builds the self-contained Linux and Windows packages into `dist/`. It needs Docker with Buildx, or Podman via `ENGINE=podman ./release.sh`:

```bash
./release.sh
```

The build runs in an Ubuntu 22.04 container (see `packaging/`). It compiles SDL2, SDL2_image, SDL2_mixer and SDL2_ttf as static libraries, enabling only the formats the game uses. Both binaries link them in, and the Windows one is cross-compiled with MinGW-w64. The first run takes a few minutes to build SDL; later runs reuse the cache.

### Linux

Install the SDL2 development libraries:

```bash
# Debian/Ubuntu
sudo apt-get install libsdl2-dev libsdl2-mixer-dev libsdl2-ttf-dev libsdl2-image-dev

# Fedora
sudo dnf install SDL2-devel SDL2_mixer-devel SDL2_ttf-devel SDL2_image-devel

# Arch
sudo pacman -S sdl2 sdl2_mixer sdl2_ttf sdl2_image
```

`./install_sdl2.sh` detects your distribution and installs them for you. It supports Debian, Ubuntu, Fedora, Arch, Void, Alpine, Gentoo and their derivatives.

Then build and run:

```bash
cargo build --release
./target/release/arkanoo
```

### Windows (native)

1. Install [Rust](https://rustup.rs/) and the [Visual Studio Build Tools](https://aka.ms/vs/stable/vs_BuildTools.exe) with the **Desktop development with C++** workload.
2. Copy the `.lib` files from `sdl2\` into the project root (next to `Cargo.toml`).
3. Build:
   ```powershell
   cargo build --release
   ```
4. Copy the `.dll` files from `sdl2\` into `target\release\` and run `target\release\arkanoo.exe` from the project root.

## Project structure

```
arkanoo/
├── src/
│   ├── main.rs        # Entry point, event loop, menu and editor input
│   ├── game.rs        # Game state, physics, scoring, power-ups, portal
│   ├── entities.rs    # Paddle, ball, blocks, bonuses, rockets, particles, level patterns
│   ├── rendering.rs   # Drawing: blocks, effects, backgrounds, HUD, menus, editor
│   ├── audio.rs       # Music playlist and sound effects
│   ├── menu.rs        # Main and settings menus
│   ├── editor.rs      # Level editor and pattern file format
│   └── settings.rs    # settings.json load/save
├── assets/            # Backgrounds, sounds, music, font, icons
├── patterns/          # Custom level patterns (used in Infinite Mode)
├── packaging/         # Container build for self-contained release packages
├── sdl2/              # SDL2 import libraries and DLLs for native Windows builds
├── build.rs           # Embeds the icon into the Windows executable
├── release.sh         # Builds release packages for Linux and Windows
└── install_sdl2.sh    # Installs SDL2 development libraries on Linux
```

Built with [`sdl2`](https://crates.io/crates/sdl2) (with the image, mixer and TTF add-ons) for graphics, input and audio, along with `rand`, `serde`/`serde_json`, `once_cell` and `webbrowser`.

## License and credits

Arkanoo is released under the **WTFPL**, effectively public domain. Copyright (C) 2025 c0m4r.

### Authors

Code: Gemini 3 Pro (High) / Claude {Sonnet,Opus} 4.5 (Thinking) / c0m4r

The logo and backgrounds were AI-generated with Gemini 2.5 Flash Image (Nano Banana).

### Third-party software and assets

The release packages contain the following third-party components. Their full license texts are in the `licenses/` folder of each package.

| Component | License | Notice |
|---|---|---|
| [SDL2](https://www.libsdl.org/), [SDL2_image](https://github.com/libsdl-org/SDL_image), [SDL2_mixer](https://github.com/libsdl-org/SDL_mixer), [SDL2_ttf](https://github.com/libsdl-org/SDL_ttf) | zlib | Copyright (C) 1997-2025 Sam Lantinga |
| [FreeType](https://freetype.org/) (font rendering, via SDL2_ttf) | FreeType License (FTL) | Portions of this software are copyright © 2023 The FreeType Project (www.freetype.org). All rights reserved. |
| [libwebp](https://chromium.googlesource.com/webm/libwebp) (WebP images, via SDL2_image) | BSD 3-Clause | Copyright (c) 2010, Google Inc. All rights reserved. |
| [Roboto](https://github.com/googlefonts/roboto-classic) font | SIL Open Font License 1.1 | Copyright 2011 The Roboto Project Authors. See [`assets/font/OFL.txt`](assets/font/OFL.txt). |
| Music and sound effects | [Pixabay Content License](https://pixabay.com/service/license-summary/) | From [Pixabay](https://pixabay.com/music/). See [`assets/music/README.md`](assets/music/README.md) for the tracks. |
