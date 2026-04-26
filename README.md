# rust-password-generator

A fast, secure, and lightweight **CLI password generator written in Rust**.

It generates strong passwords using OS-provided cryptographic randomness, supports custom character rules, and stays dependency-free for a small, clean codebase.

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![Platform](https://img.shields.io/badge/platform-cli-blue)
![License](https://img.shields.io/badge/license-MIT-green)

---

## Features

- Secure password generation using system randomness
- Configurable password length and batch count
- Enable or disable lowercase, uppercase, digits, and symbols
- Use a **custom symbol set**
- Exclude visually similar characters like `0`, `O`, `l`, `1`, `I`, and `|`
- Exclude any custom characters you do not want
- Copy generated passwords to the clipboard
- Save generated passwords to a file
- Pretty terminal output with entropy and strength info
- Quiet mode for scripting and automation
- No external crates

---

## Why this project?

Most password generators are either web-based, dependency-heavy, or too limited for scripting.
This project focuses on a better balance:

- **secure enough for real use**
- **simple enough to audit**
- **small enough to learn from**
- **practical enough for daily CLI workflows**

---

## Installation

### Clone and run

```bash
git clone https://github.com/SubheerDutta7/rust-password-generator.git
cd rust-password-generator
cargo run --
```

### Build a release binary

```bash
cargo build --release
./target/release/rust-password-generator
```

---

## Quick Start

Generate a password with default settings:

```bash
cargo run --
```

Generate 3 passwords of length 24:

```bash
cargo run -- --length 24 --count 3
```

Generate passwords without symbols:

```bash
cargo run -- --length 20 --no-symbols
```

Use custom symbols only:

```bash
cargo run -- --symbols "@#%!?_"
```

Exclude similar and custom characters:

```bash
cargo run -- --exclude-similar --exclude "O0l1aA"
```

Copy generated password(s) to clipboard:

```bash
cargo run -- --copy
```

Save passwords to a file:

```bash
cargo run -- --length 24 --count 5 --output passwords.txt
```

Pretty output with strength summary:

```bash
cargo run -- --pretty
```

Quiet output for scripts:

```bash
cargo run -- --quiet
```

---

## CLI Options

```text
-l, --length <NUMBER>    Password length
-c, --count <NUMBER>     Number of passwords to generate
    --no-lowercase       Disable lowercase characters
    --no-uppercase       Disable uppercase characters
    --no-digits          Disable digits
    --no-symbols         Disable symbols
    --symbols <CHARS>    Use a custom symbol set
    --exclude <CHARS>    Exclude custom characters from all enabled groups
    --exclude-similar    Remove visually similar characters
-o, --output <PATH>      Save generated password(s) to a file
    --copy               Copy password(s) to the clipboard
    --pretty             Show colorful summary output
-q, --quiet              Print only password lines
    --no-color           Disable ANSI colors in pretty mode
-h, --help               Show help
-V, --version            Show version
```

---

## Example Output

### Pretty mode

```text
Generated password(s)
length=24  count=2  pool=87 chars  entropy≈154.7 bits  strength=Very strong

Password 1: 7@tLqH#2vN!xM9$wA3rPzK&d
Password 2: Y!4mQn@8Xr#2Lp$7TsVhC9%k
```

### Quiet mode

```text
7@tLqH#2vN!xM9$wA3rPzK&d
Y!4mQn@8Xr#2Lp$7TsVhC9%k
```

---

## Security Notes

- Passwords are generated using **OS-backed cryptographic randomness**
  - Unix-like systems: `/dev/urandom`
  - Windows: `BCryptGenRandom`
- The generator ensures at least **one character from every enabled group**
- Character selection uses rejection sampling to avoid modulo bias
- Clipboard support depends on the platform tools available on your system

This project is designed to be practical and secure for local password generation, but you should still follow good security hygiene:

- avoid storing passwords in plain text unless necessary
- clear clipboard contents after use if your workflow is sensitive
- prefer a password manager for long-term storage

---

## Platform Support

### Randomness
- Linux / macOS / other Unix-like systems: supported
- Windows: supported

### Clipboard
- macOS: `pbcopy`
- Windows: `clip`
- Linux / Unix: `wl-copy`, `xclip`, or `xsel`

If clipboard support is requested and no supported clipboard command is installed, the tool returns a clear error message.

---

## Project Structure

```text
rust-password-generator/
├── Cargo.toml
└── src/
    ├── lib.rs
    └── main.rs
```

---

## Development

Run tests:

```bash
cargo test
```

Format code:

```bash
cargo fmt
```

Run clippy:

```bash
cargo clippy -- -D warnings
```

---

## Roadmap

Possible next improvements:

- generated `Cargo.lock`
- GitHub Actions CI
- release workflow for prebuilt binaries
- optional passphrase mode
- optional config file support

---

## License

MIT License.

---

## Author

**Subheer Dutta**

Built with Rust for speed, safety, and clean CLI tooling.
