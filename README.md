
# 🥕 YERKOKU 🥕 (Prompt Generator TUI)

A powerful terminal user interface (TUI) application built with Rust and Ratatui for generating comprehensive AI development prompts from configurable JSON blueprints.


<img src="./assets/yerkoku.png">

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

### Prerequisites

- [Rust](https://rustup.rs/) 1.75+ (stable)
- A terminal with true color support (iTerm2, Windows Terminal, Alacritty, Kitty)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/Milad-HajiShafiei/yerkoku
cd yerkoku

# Build in release mode
cargo build --release

# Run
cargo run --release
```

### Install Globally

```bash
cargo install --path .
```

## 🎮 Usage

### Quick Start

```bash
# Run the application
cargo run

# Or if installed globally
prompt-generator
```

### Workflow

1. **Launch** → Drafts screen appears (or Blueprint menu if no drafts)
2. **Select Blueprint** → Choose Backend, Frontend, Mobile, or Desktop
3. **Fill Form** → Navigate sections, fill fields, search packages
4. **Preview** → Watch the right panel update live
5. **Generate** → Press `g` or use the Generate button to save + copy

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

## 📂 Blueprint Format

Blueprints are JSON files in the `blueprints/` directory. Each blueprint defines a form with sections and fields.

### Basic Structure

```json
{
  "name": "My Application",
  "description": "What this blueprint generates",
  "icon": "🔧",
  "sections": [
    {
      "title": "Section Name",
      "icon": "📋",
      "description": "Section description",
      "fields": [...]
    }
  ]
}
```

### Field Types

| Type | Description | Widget |
|------|-------------|--------|
| `text` | Single-line text input | TextInput |
| `textarea` | Multi-line text with word wrap | TextInput (multiline) |
| `checkbox` | Boolean toggle | Checkbox |
| `select` | Single choice from options | Select |
| `multiselect` | Multiple choice from options | MultiSelect |
| `crate_input` | Package version with "Get Latest" button | CrateInput |
| `crate_search` | Search & add packages to a list | CrateSearch |
| `list_builder` | Add/remove items to a list | ListBuilder |
| `action_button` | Trigger an action (generate, etc.) | Button |
| `section_break` | Visual separator (not interactive) | None |

### Field Properties

```json
{
  "key": "unique.field.key",
  "label": "Display Label",
  "type": "text",
  "placeholder": "Hint text",
  "description": "Help text",
  "required": false,
  "default": "",
  "hidden": false,
  "registry": "npm",
  "crate_name": "package-name",
  "target_list_key": "tech.additional_packages",
  "button_text": "Click Me",
  "action": "generate_copy",
  "options": [
    {"value": "opt1", "label": "Option 1"},
    {"value": "opt2", "label": "Option 2"}
  ]
}
```

### Package Registry Options

| Registry Value | Searches |
|---------------|----------|
| `npm`, `npmjs`, `npmjs.com` | npmjs.com |
| `crates.io`, `crates`, `cargo` | crates.io |
| `pypi`, `pip`, `python` | pypi.org |

## 📦 Included Blueprints

| Blueprint | File | Description |
|-----------|------|-------------|
| **Backend** | `blueprints/backend.json` | Axum Rust backend with VPS deployment (22 sections, 180+ fields) |
| **Frontend** | `blueprints/frontend.json` | React frontend with modern stack (13 sections, 76 fields) |
| **Mobile** | `blueprints/mobile.json` | React Native mobile app (20 sections, 190+ fields) |
| **Desktop** | `blueprints/desktop.json` | Slint desktop application (20 sections, 170+ fields) |

## 📁 Project Structure

```
yerkoku/
├── Cargo.toml
├── README.md
├── blueprints/
│   ├── backend.json
│   ├── frontend.json
│   ├── mobile.json
│   └── desktop.json
├── drafts/              # Auto-created, stores saved drafts
├── prompts/             # Auto-created, stores generated prompts
├── src/
│   ├── main.rs          # Entry point, event loop, key handlers
│   ├── app.rs           # App state, screen management
│   ├── ui.rs            # All rendering functions
│   ├── form.rs          # Form state, navigation, editing
│   ├── blueprint.rs     # Blueprint JSON parsing
│   ├── prompt.rs        # Prompt text generation
│   ├── draft.rs         # Draft save/load/delete
│   ├── package_registry.rs  # npm/crates.io/PyPI search
│   ├── crates_io.rs     # Legacy crates.io search
│   └── widgets/
│       ├── mod.rs
│       ├── text_input.rs
│       ├── checkbox.rs
│       ├── select.rs
│       ├── multiselect.rs
│       ├── crate_input.rs
│       ├── crate_search.rs
│       ├── list_builder.rs
│       └── button.rs
└── debug.log            # Debug output (auto-created)
```

## ⚙️ Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `BLUEPRINTS_DIR` | Path to blueprints directory | `./blueprints` |
| `DRAFTS_DIR` | Path to drafts directory | `./drafts` |

### Custom Blueprints Directory

```bash
# Use a custom blueprints location
BLUEPRINTS_DIR=/path/to/my/blueprints cargo run
```

## 🔧 Creating Custom Blueprints

### 1. Create a new JSON file

```bash
touch blueprints/my_project.json
```

### 2. Define the structure

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

### 3. Run and select your blueprint

```bash
cargo run
# Your blueprint will appear in the menu
```

## 🐛 Troubleshooting

### App appears stuck

Check `debug.log` for the last operation:

```bash
tail -20 debug.log
```

### Package search fails

- Ensure you have internet connectivity
- Check the registry value in your blueprint (`npm`, `crates.io`, `pypi`)
- Verify the package name is correct

### Clipboard not working

- On Linux: install `xclip` or `xsel`
- On Wayland: install `wl-clipboard`
- On macOS/Windows: should work natively

### Blueprints not found

```bash
# Check the blueprints directory exists
ls blueprints/

# Or set a custom path
BLUEPRINTS_DIR=/path/to/blueprints cargo run
```

### Terminal rendering issues

Ensure your terminal supports:
- True color (24-bit)
- Unicode characters
- Minimum 80×24 terminal size

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'feat: add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Commit Convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` — New feature
- `fix:` — Bug fix
- `docs:` — Documentation
- `refactor:` — Code refactoring
- `test:` — Tests
- `chore:` — Maintenance

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Ratatui](https://ratatui.rs/) — TUI framework
- [Crossterm](https://github.com/crossterm-rs/crossterm) — Terminal manipulation
- [arboard](https://github.com/1Password/arboard) — Clipboard access
- [reqwest](https://github.com/seanmonstar/reqwest) — HTTP client

---

**Built with ❤️ and 🦀 Rust**
