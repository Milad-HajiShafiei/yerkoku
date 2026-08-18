# 🥕 YERKOKU 🥕 (Prompt Generator TUI)

A powerful terminal user interface (TUI) application built with Rust and Ratatui for generating comprehensive AI development prompts from configurable JSON blueprints.

<img src="./assets/yerkoku.png">


![Rust](https://img.shields.io/badge/rust-stable-orange)
![License](https://img.shields.io/badge/license-MIT-blue)
![Crates.io](https://img.shields.io/crates/v/yerkoku)

## ✨ Features

- **📋 Blueprint-Driven Forms** — Define project requirements through interactive JSON blueprints
- **🔍 Package Registry Search** — Search npm, crates.io, or PyPI for latest package versions
- **💾 Draft System** — Auto-save and resume your work across sessions
- **👁️ Live Preview** — See your generated prompt update in real-time
- **🚀 Generate & Copy** — One-click prompt generation with clipboard support
- **🎨 Beautiful TUI** — Split-screen layout with scrollable panels and styled widgets
- **📱 Multi-Platform Blueprints** — Pre-built templates for Backend, Frontend, Mobile, and Desktop apps
- **⚡ Async Package Search** — Non-blocking search with loading indicators
- **❌ Error Handling** — User-friendly error modals for all failure scenarios

## 📸 Screens

| Screen | Description |
|--------|-------------|
| **Drafts** | Lists saved drafts, press Enter to resume or `n` for new project |
| **Blueprint Menu** | Select a blueprint template (Backend, Frontend, Mobile, Desktop) |
| **Form Editor** | Split-screen with form on left, live prompt preview on right |
| **Review** | Final review before generating |
| **Success Modal** | Shows save path and clipboard confirmation |

## 🚀 Installation

### From crates.io (Recommended)

```bash
cargo install yerkoku
```

This installs the `yerkoku` binary to `~/.cargo/bin/` (which should be in your system PATH).

### From Source (Latest Development Version)

```bash
# Clone the repository
git clone https://github.com/Milad-HajiShafiei/yerkoku.git
cd yerkoku

# Build and install
cargo install --path .
```

### From GitHub Releases (Pre-compiled Binaries)

Download the latest release for your platform from the [Releases page](https://github.com/Milad-HajiShafiei/yerkoku/releases):

| Platform | File | Instructions |
|----------|------|--------------|
| **Linux (x86_64)** | `yerkoku-linux-x86_64` | `chmod +x yerkoku-linux-x86_64 && sudo mv yerkoku-linux-x86_64 /usr/local/bin/yerkoku` |
| **Linux (ARM64)** | `yerkoku-linux-arm64` | `chmod +x yerkoku-linux-arm64 && sudo mv yerkoku-linux-arm64 /usr/local/bin/yerkoku` |
| **macOS (Intel)** | `yerkoku-macos-x86_64` | `chmod +x yerkoku-macos-x86_64 && sudo mv yerkoku-macos-x86_64 /usr/local/bin/yerkoku` |
| **macOS (Apple Silicon)** | `yerkoku-macos-arm64` | `chmod +x yerkoku-macos-arm64 && sudo mv yerkoku-macos-arm64 /usr/local/bin/yerkoku` |
| **Windows (x86_64)** | `yerkoku-windows-x86_64.exe` | Download and add to your PATH, or place in `C:\Windows\System32\` |

### Prerequisites

- **Terminal**: Must support true color (24-bit) and Unicode
  - ✅ Windows Terminal, iTerm2, Alacritty, Kitty, WezTerm, GNOME Terminal
  - ❌ Legacy Windows CMD (use Windows Terminal instead)
- **Clipboard** (for copy-to-clipboard feature):
  - Linux: Install `xclip` or `xsel` (X11) or `wl-clipboard` (Wayland)
  - macOS / Windows: Works natively

---

## 🚀 Quick Start

### 1. Launch the Application

```bash
yerkoku
```

### 2. Select a Blueprint

On first launch, the app automatically initializes 4 default blueprints:
- 🔧 **Backend Application** — Axum Rust with VPS deployment
- 🎨 **Frontend Application** — React with modern tooling
- 📱 **Mobile Application** — React Native with Expo
- 🖥️ **Desktop Application** — Slint with Rust

Use `↑`/`↓` to navigate and `Enter` to select.

### 3. Fill Out the Form

- Navigate between fields with `↑`/`↓` or `Tab`
- Press `Enter` to edit text fields or toggle checkboxes
- Press `Space` to activate buttons
- Use `←`/`→` or `n`/`p` to switch between sections
- Use the scrollable navbar at the top to jump to any section

### 4. Search & Add Packages

For Technology Stack sections, you can search package registries for the latest versions:

1. Navigate to the "Search & Add" field
2. Type a package name (e.g., `axios`, `tokio`, `flask`)
3. Press `Tab` to focus the Search button
4. Press `Enter` to search the registry
5. The package with its latest version is added to the list

**Supported registries:**
- `npm` → searches npmjs.com
- `crates.io` → searches crates.io
- `pypi` → searches pypi.org

### 5. Generate Your Prompt

- Press `g` anywhere in the form to generate and quit
- Or navigate to the "Generate Prompt" section and press `Space` on the button
- The prompt is saved to a markdown file AND copied to your clipboard

### 6. Review the Output

After generation, a success modal shows:
- ✅ Confirmation message
- 📋 Clipboard copy status
- 📁 File save path

Press `Enter` to return to the form, `m` for the menu, or `q` to quit.

---

## ⌨️ Keyboard Shortcuts

### Global

| Key | Action |
|-----|--------|
| `Ctrl+C` / `Ctrl+Q` | Quit application |
| `Ctrl+S` | Generate and quit |
| `Esc` | Go back / Cancel |

### Drafts Screen

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate drafts |
| `Enter` | Open selected draft |
| `n` | New project (go to blueprint menu) |
| `d` | Delete selected draft |
| `r` | Refresh drafts list |
| `q` | Quit |

### Blueprint Menu

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate blueprints |
| `Enter` | Select blueprint |
| `r` | Refresh blueprints |
| `q` | Quit |

### Form Editor

| Key | Action |
|-----|--------|
| `↑` / `↓` / `k` / `j` | Navigate fields |
| `Tab` | Next field / Sub-navigation in composite widgets |
| `Shift+Tab` | Previous field |
| `Enter` | Edit text / Toggle checkbox / Cycle select |
| `Space` | Activate button / Toggle checkbox |
| `←` / `→` / `n` / `p` | Switch sections |
| `e` | Start editing current field |
| `g` | Generate prompt and quit |
| `s` | Save draft |
| `r` | Review screen |
| `d` / `Delete` | Delete item in focused list |
| `PageUp` / `PageDown` | Scroll form |
| `Esc` / `q` | Back to menu |

### Composite Widgets (CrateInput, CrateSearch, ListBuilder)

| Key | Action |
|-----|--------|
| `Tab` | Cycle: Input → Button → List → Next field |
| `Shift+Tab` | Cycle backward |
| `Enter` (on input) | Start editing |
| `Enter` (on button) | Trigger action (search/add) |
| `↑` / `↓` (on list) | Navigate list items |
| `d` / `Delete` (on list) | Remove selected item |

### Success Modal

| Key | Action |
|-----|--------|
| `Enter` / `Esc` / `Space` | Return to form |
| `m` | Go to blueprint menu |
| `q` | Quit |

### Error Modal

| Key | Action |
|-----|--------|
| `Enter` / `Esc` / `Space` | Dismiss error |

## 🖱️ Mouse Controls

| Action | Area | Effect |
|--------|------|--------|
| **Scroll Up/Down** | Navbar | Scroll section tabs |
| **Scroll Up/Down** | Form (left) | Scroll form fields |
| **Scroll Up/Down** | Preview (right) | Scroll prompt preview |
| **Click** | Form field | Select/focus field |

---

## 📂 Where Are Files Stored?

The application uses OS-specific directories to store data:

| OS | Base Directory |
|----|----------------|
| **Linux** | `~/.local/share/yerkoku/` |
| **macOS** | `~/Library/Application Support/yerkoku/` |
| **Windows** | `C:\Users\<Username>\AppData\Roaming\yerkoku\` |

### Directory Structure

```
yerkoku/
├── blueprints/          # JSON blueprint templates
│   ├── backend.json
│   ├── frontend.json
│   ├── mobile.json
│   └── desktop.json
├── drafts/              # Auto-saved form states
│   ├── backend_application_draft.json
│   └── frontend_application_draft.json
└── prompts/             # Generated prompt files
    ├── backend_application_prompt.md
    └── frontend_application_prompt.md
```

### Override Storage Location

You can override the default locations using environment variables:

```bash
# Custom blueprints directory
BLUEPRINTS_DIR=/path/to/my/blueprints yerkoku

# Custom drafts directory
DRAFTS_DIR=/path/to/my/drafts yerkoku
```

Or use the CLI flag for blueprints:

```bash
yerkoku --blueprints /path/to/my/blueprints
```

---

## 🛠️ CLI Commands & Flags

```bash
# Launch the application
yerkoku

# Force reset and re-install default blueprints
yerkoku --init

# Use a custom blueprints directory
yerkoku --blueprints /path/to/blueprints

# Show help
yerkoku --help

# Show version
yerkoku --version
```

### Flag Reference

| Flag | Short | Description |
|------|-------|-------------|
| `--init` | `-i` | Force re-install default blueprints (overwrites existing) |
| `--blueprints <path>` | `-b <path>` | Use a custom directory for blueprint JSON files |
| `--help` | `-h` | Print help information |
| `--version` | `-V` | Print version information |

---

## 🔄 Draft System

The app automatically saves your progress:

- **Auto-save on exit**: When you quit with `q` or `Ctrl+C`, your current form state is saved
- **Auto-save on generate**: After generating a prompt, the form state is saved
- **Manual save**: Press `s` in the form to save a draft immediately

### Resuming a Draft

1. Launch `yerkoku`
2. The Drafts screen appears showing all saved drafts
3. Use `↑`/`↓` to select a draft
4. Press `Enter` to resume where you left off
5. Press `n` to start a new project instead
6. Press `d` to delete a draft

---

## 🎨 Custom Blueprints

You can create your own blueprint templates for any project type.

### 1. Create a New Blueprint File

```bash
# Navigate to the blueprints directory
cd ~/.local/share/yerkoku/blueprints  # Linux
cd ~/Library/Application\ Support/yerkoku/blueprints  # macOS
cd %APPDATA%\yerkoku\blueprints  # Windows

# Create a new blueprint
touch my_project.json
```

### 2. Define the Structure

```json
{
  "name": "My Custom Project",
  "description": "Generates prompts for my specific use case",
  "icon": "🎯",
  "sections": [
    {
      "title": "Basics",
      "icon": "📋",
      "description": "Project fundamentals",
      "fields": [
        {
          "key": "project.name",
          "label": "Project Name",
          "type": "text",
          "required": true,
          "placeholder": "My Awesome Project"
        },
        {
          "key": "project.features",
          "label": "Features",
          "type": "list_builder",
          "placeholder": "Type a feature and press Add"
        },
        {
          "key": "actions.generate",
          "label": "Generate",
          "type": "action_button",
          "button_text": "🚀 Generate Prompt",
          "action": "generate_copy"
        }
      ]
    }
  ]
}
```

### 3. Run and Select Your Blueprint

```bash
yerkoku
# Your custom blueprint will appear in the menu
```

### Available Field Types

| Type | Widget | Description |
|------|--------|-------------|
| `text` | TextInput | Single-line text input |
| `textarea` | TextInput (multiline) | Multi-line text with word wrap and scrolling |
| `checkbox` | Checkbox | Boolean toggle |
| `select` | Select | Single choice from dropdown options |
| `multiselect` | MultiSelect | Multiple choice from options |
| `crate_input` | CrateInput | Package version input with "Get Latest" button |
| `crate_search` | CrateSearch | Search & add packages to a list from a registry |
| `list_builder` | ListBuilder | Add/remove items to a dynamic list |
| `action_button` | Button | Trigger an action (generate, copy, etc.) |
| `section_break` | None | Visual separator (not interactive) |

### Package Registry Options

Use the `registry` field to specify which package registry to search:

| Value | Searches | Example |
|-------|----------|---------|
| `npm` | npmjs.com | `"registry": "npm"` |
| `crates.io` | crates.io | `"registry": "crates.io"` |
| `pypi` | pypi.org | `"registry": "pypi"` |

---

## 🐛 Troubleshooting

### "No blueprints found"

```bash
# Re-initialize default blueprints
yerkoku --init

# Or check the blueprints directory
ls ~/.local/share/yerkoku/blueprints/
```

### Clipboard not working

- **Linux (X11)**: Install `xclip` or `xsel`
  ```bash
  sudo apt install xclip    # Debian/Ubuntu
  sudo dnf install xclip    # Fedora
  ```
- **Linux (Wayland)**: Install `wl-clipboard`
  ```bash
  sudo apt install wl-clipboard
  ```
- **macOS / Windows**: Should work natively. If not, ensure no other clipboard manager is interfering.

### Terminal rendering issues

- Ensure your terminal supports **true color (24-bit)**
- Ensure your terminal supports **Unicode** characters
- Minimum terminal size: **80×24**
- If using SSH, ensure `TERM` is set correctly:
  ```bash
  echo $TERM  # Should be xterm-256color or similar
  ```

### App appears stuck or frozen

Check the debug log for the last operation:

```bash
tail -20 debug.log
```

Common causes:
- Slow network during package search (wait for timeout)
- Terminal doesn't support alternate screen buffer
- Clipboard manager blocking access

### Package search fails

- Ensure you have internet connectivity
- Verify the package name is spelled correctly
- Check the `registry` value in your blueprint matches the package ecosystem
- Try searching manually:
  ```bash
  curl -s https://registry.npmjs.org/axios | jq '."dist-tags".latest'
  curl -s https://crates.io/api/v1/crates/tokio | jq '.crate.max_version'
  ```

### Drafts not loading

- Ensure the blueprint referenced in the draft still exists
- If you renamed or deleted a blueprint, old drafts for it won't load
- Delete corrupted drafts manually:
  ```bash
  rm ~/.local/share/yerkoku/drafts/corrupted_draft.json
  ```

---

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes using [Conventional Commits](https://www.conventionalcommits.org/)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Commit Convention

- `feat:` — New feature
- `fix:` — Bug fix
- `docs:` — Documentation changes
- `refactor:` — Code refactoring
- `test:` — Adding or updating tests
- `chore:` — Maintenance tasks

### Building from Source

```bash
git clone https://github.com/Milad-HajiShafiei/yerkoku.git
cd yerkoku
cargo build --release
./target/release/yerkoku
```

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

- [Ratatui](https://ratatui.rs/) — Terminal UI framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) — Cross-platform terminal manipulation
- [arboard](https://github.com/1Password/arboard) — Clipboard access
- [reqwest](https://github.com/seanmonstar/reqwest) — HTTP client
- [clap](https://github.com/clap-rs/clap) — Command-line argument parsing
- [dirs](https://github.com/dirs-dev/dirs-rs) — OS-specific directory paths

---

**Built with ❤️ and 🦀 Rust**
