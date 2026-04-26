use std::fmt;
use std::io::{self, IsTerminal, Write};
use std::process::{Command, Stdio};

#[cfg(unix)]
use std::fs::File;
#[cfg(unix)]
use std::io::Read;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_LENGTH: usize = 16;
pub const DEFAULT_COUNT: usize = 1;
pub const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
pub const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const DIGITS: &str = "0123456789";
pub const SYMBOLS: &str = r#"!@#$%^&*()-_=+[]{}<>?/"#;
pub const SIMILAR_CHARS: &str = "0Ool1I|";

const RESET: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const DIM: &str = "\x1b[2m";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Help,
    Version,
    Generate(Config),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub length: usize,
    pub count: usize,
    pub include_lowercase: bool,
    pub include_uppercase: bool,
    pub include_digits: bool,
    pub include_symbols: bool,
    pub exclude_similar: bool,
    pub custom_symbols: Option<String>,
    pub exclude_chars: String,
    pub output_file: Option<String>,
    pub copy_to_clipboard: bool,
    pub pretty: bool,
    pub no_color: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            count: DEFAULT_COUNT,
            include_lowercase: true,
            include_uppercase: true,
            include_digits: true,
            include_symbols: true,
            exclude_similar: false,
            custom_symbols: None,
            exclude_chars: String::new(),
            output_file: None,
            copy_to_clipboard: false,
            pretty: io::stdout().is_terminal(),
            no_color: false,
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "length={}, count={}, lowercase={}, uppercase={}, digits={}, symbols={}, exclude_similar={}, custom_symbols={:?}, exclude_chars={:?}, output_file={:?}, copy={}, pretty={}, no_color={}",
            self.length,
            self.count,
            self.include_lowercase,
            self.include_uppercase,
            self.include_digits,
            self.include_symbols,
            self.exclude_similar,
            self.custom_symbols,
            self.exclude_chars,
            self.output_file,
            self.copy_to_clipboard,
            self.pretty,
            self.no_color
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedPasswords {
    pub passwords: Vec<String>,
    pub strength: Strength,
    pub entropy_bits: f64,
    pub pool_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strength {
    Weak,
    Fair,
    Strong,
    VeryStrong,
}

impl Strength {
    pub fn label(self) -> &'static str {
        match self {
            Strength::Weak => "Weak",
            Strength::Fair => "Fair",
            Strength::Strong => "Strong",
            Strength::VeryStrong => "Very strong",
        }
    }

    fn color(self) -> &'static str {
        match self {
            Strength::Weak => RED,
            Strength::Fair => YELLOW,
            Strength::Strong => GREEN,
            Strength::VeryStrong => CYAN,
        }
    }
}

pub fn parse_args(args: &[String]) -> Result<Action, String> {
    let mut config = Config::default();
    let mut index = 0;

    while index < args.len() {
        let arg = args[index].as_str();
        match arg {
            "-h" | "--help" => return Ok(Action::Help),
            "-V" | "--version" => return Ok(Action::Version),
            "--no-lowercase" => config.include_lowercase = false,
            "--no-uppercase" => config.include_uppercase = false,
            "--no-digits" => config.include_digits = false,
            "--no-symbols" => config.include_symbols = false,
            "--exclude-similar" => config.exclude_similar = true,
            "--copy" => config.copy_to_clipboard = true,
            "--pretty" => config.pretty = true,
            "-q" | "--quiet" => config.pretty = false,
            "--no-color" => config.no_color = true,
            "-l" | "--length" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "Missing value for --length".to_string())?;
                config.length = parse_positive_usize(value, "length")?;
            }
            "-c" | "--count" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "Missing value for --count".to_string())?;
                config.count = parse_positive_usize(value, "count")?;
            }
            "--symbols" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "Missing value for --symbols".to_string())?;
                config.custom_symbols = Some(value.clone());
            }
            "--exclude" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "Missing value for --exclude".to_string())?;
                config.exclude_chars.push_str(value);
            }
            "-o" | "--output" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "Missing value for --output".to_string())?;
                config.output_file = Some(value.clone());
            }
            _ if arg.starts_with("--length=") => {
                let value = arg.trim_start_matches("--length=");
                config.length = parse_positive_usize(value, "length")?;
            }
            _ if arg.starts_with("--count=") => {
                let value = arg.trim_start_matches("--count=");
                config.count = parse_positive_usize(value, "count")?;
            }
            _ if arg.starts_with("--symbols=") => {
                let value = arg.trim_start_matches("--symbols=");
                config.custom_symbols = Some(value.to_string());
            }
            _ if arg.starts_with("--exclude=") => {
                let value = arg.trim_start_matches("--exclude=");
                config.exclude_chars.push_str(value);
            }
            _ if arg.starts_with("--output=") => {
                let value = arg.trim_start_matches("--output=");
                config.output_file = Some(value.to_string());
            }
            _ => return Err(format!("Unknown argument: {arg}")),
        }
        index += 1;
    }

    Ok(Action::Generate(validate_config(config)?))
}

pub fn print_help() {
    println!(
        "{name} {version}\n\nA fast and secure CLI password generator written in Rust.\n\nUSAGE:\n    {name} [OPTIONS]\n\nOPTIONS:\n    -l, --length <NUMBER>    Password length (default: {default_length})\n    -c, --count <NUMBER>     Number of passwords to generate (default: {default_count})\n        --no-lowercase       Disable lowercase characters\n        --no-uppercase       Disable uppercase characters\n        --no-digits          Disable digits\n        --no-symbols         Disable symbols\n        --symbols <CHARS>    Use a custom symbol set instead of the default symbols\n        --exclude <CHARS>    Exclude any custom characters from all enabled groups\n        --exclude-similar    Remove visually similar characters like 0, O, l, 1, I and |\n    -o, --output <PATH>      Save generated password(s) to a file\n        --copy               Copy generated password(s) to the clipboard when supported\n        --pretty             Show colorful summary output when running interactively\n    -q, --quiet              Print only the password lines\n        --no-color           Disable ANSI colors in pretty mode\n    -h, --help               Show this help message\n    -V, --version            Show version\n\nEXAMPLES:\n    {name}\n    {name} --length 24 --count 3\n    {name} --length 20 --no-symbols --copy\n    {name} --symbols '@#$%!?_' --exclude 'O0l1' --pretty\n    {name} --length 24 --count 5 --output passwords.txt",
        name = APP_NAME,
        version = APP_VERSION,
        default_length = DEFAULT_LENGTH,
        default_count = DEFAULT_COUNT,
    );
}

pub fn generate_passwords(config: &Config) -> Result<GeneratedPasswords, String> {
    let groups = build_groups(config)?;
    let pool = build_pool(&groups);
    let mut passwords = Vec::with_capacity(config.count);

    for _ in 0..config.count {
        passwords.push(generate_password_from_groups(config.length, &groups, &pool)?);
    }

    let entropy_bits = estimate_entropy_bits(config.length, pool.len());
    let strength = evaluate_strength(entropy_bits);

    Ok(GeneratedPasswords {
        passwords,
        strength,
        entropy_bits,
        pool_size: pool.len(),
    })
}

pub fn format_output(result: &GeneratedPasswords, config: &Config) -> String {
    if !config.pretty {
        return result.passwords.join("\n");
    }

    let use_color = !config.no_color;
    let mut output = String::new();
    let heading = apply_color("Generated password(s)", CYAN, use_color);
    let strength_label = apply_color(result.strength.label(), result.strength.color(), use_color);
    let meta = format!(
        "length={}  count={}  pool={} chars  entropy≈{:.1} bits  strength={}",
        config.length, config.count, result.pool_size, result.entropy_bits, strength_label
    );
    let meta = apply_color(&meta, DIM, use_color);

    output.push_str(&heading);
    output.push('\n');
    output.push_str(&meta);
    output.push_str("\n\n");

    for (index, password) in result.passwords.iter().enumerate() {
        let label = if result.passwords.len() == 1 {
            "Password".to_string()
        } else {
            format!("Password {}", index + 1)
        };
        let label = apply_color(&label, GREEN, use_color);
        output.push_str(&format!("{}: {}\n", label, password));
    }

    output.trim_end().to_string()
}

pub fn format_passwords_for_file(result: &GeneratedPasswords) -> String {
    result.passwords.join("\n")
}

pub fn write_passwords_to_file(path: &str, content: &str) -> Result<(), String> {
    std::fs::write(path, content)
        .map_err(|err| format!("Failed to write output file '{path}': {err}"))
}

pub fn copy_passwords_to_clipboard(passwords: &[String]) -> Result<(), String> {
    let content = passwords.join("\n");

    #[cfg(target_os = "macos")]
    {
        return pipe_to_command("pbcopy", &[], &content);
    }

    #[cfg(target_os = "windows")]
    {
        return pipe_to_command("cmd", &["/C", "clip"], &content);
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let commands: [(&str, &[&str]); 3] = [
            ("wl-copy", &[]),
            ("xclip", &["-selection", "clipboard"]),
            ("xsel", &["--clipboard", "--input"]),
        ];

        for (command, args) in commands {
            match pipe_to_command(command, args, &content) {
                Ok(()) => return Ok(()),
                Err(err) if err.contains("not found") => continue,
                Err(err) => return Err(err),
            }
        }

        return Err(
            "Clipboard support was requested, but no supported clipboard command was found. Install wl-copy, xclip, or xsel.".to_string(),
        );
    }

    #[allow(unreachable_code)]
    Err("Clipboard support is not available on this platform.".to_string())
}

fn pipe_to_command(command: &str, args: &[&str], input: &str) -> Result<(), String> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                format!("Command not found: {command}")
            } else {
                format!("Failed to start {command}: {err}")
            }
        })?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| format!("Failed to open stdin for {command}"))?;
        stdin
            .write_all(input.as_bytes())
            .map_err(|err| format!("Failed to write to {command}: {err}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|err| format!("Failed to wait for {command}: {err}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!("{command} exited with status {}", output.status))
        } else {
            Err(format!("{command} failed: {stderr}"))
        }
    }
}

pub fn validate_config(config: Config) -> Result<Config, String> {
    let required_groups = selected_group_count(&config);

    if required_groups == 0 {
        return Err("At least one character group must be enabled".to_string());
    }

    if let Some(symbols) = &config.custom_symbols {
        if config.include_symbols && symbols.is_empty() {
            return Err("Custom symbol set cannot be empty when symbols are enabled".to_string());
        }
    }

    let groups = build_groups(&config)?;
    let required_groups_after_filtering = groups.len();

    if config.length < required_groups_after_filtering {
        return Err(format!(
            "Length must be at least {required_groups_after_filtering} to include one character from each selected group"
        ));
    }

    Ok(config)
}

fn parse_positive_usize(value: &str, field_name: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| format!("Invalid value for {field_name}: {value}"))?;

    if parsed == 0 {
        return Err(format!("{field_name} must be greater than 0"));
    }

    Ok(parsed)
}

fn selected_group_count(config: &Config) -> usize {
    [
        config.include_lowercase,
        config.include_uppercase,
        config.include_digits,
        config.include_symbols,
    ]
    .into_iter()
    .filter(|enabled| *enabled)
    .count()
}

fn build_groups(config: &Config) -> Result<Vec<Vec<char>>, String> {
    let mut groups = Vec::new();

    if config.include_lowercase {
        groups.push(filtered_charset(LOWERCASE, config.exclude_similar, &config.exclude_chars));
    }
    if config.include_uppercase {
        groups.push(filtered_charset(UPPERCASE, config.exclude_similar, &config.exclude_chars));
    }
    if config.include_digits {
        groups.push(filtered_charset(DIGITS, config.exclude_similar, &config.exclude_chars));
    }
    if config.include_symbols {
        let symbol_source = config.custom_symbols.as_deref().unwrap_or(SYMBOLS);
        groups.push(filtered_charset(
            symbol_source,
            config.exclude_similar,
            &config.exclude_chars,
        ));
    }

    if groups.iter().any(|group| group.is_empty()) {
        return Err("One of the selected character groups became empty after filtering".to_string());
    }

    Ok(groups)
}

fn filtered_charset(source: &str, exclude_similar: bool, exclude_chars: &str) -> Vec<char> {
    let mut filtered = Vec::new();

    for ch in source.chars() {
        if exclude_similar && SIMILAR_CHARS.contains(ch) {
            continue;
        }
        if exclude_chars.contains(ch) {
            continue;
        }
        if !filtered.contains(&ch) {
            filtered.push(ch);
        }
    }

    filtered
}

fn build_pool(groups: &[Vec<char>]) -> Vec<char> {
    let mut pool = Vec::new();
    for ch in groups.iter().flat_map(|group| group.iter().copied()) {
        if !pool.contains(&ch) {
            pool.push(ch);
        }
    }
    pool
}

fn generate_password_from_groups(
    length: usize,
    groups: &[Vec<char>],
    pool: &[char],
) -> Result<String, String> {
    let mut password = Vec::with_capacity(length);

    for group in groups {
        password.push(random_char(group)?);
    }

    while password.len() < length {
        password.push(random_char(pool)?);
    }

    shuffle(&mut password)?;

    Ok(password.into_iter().collect())
}

fn random_char(chars: &[char]) -> Result<char, String> {
    let index = random_index(chars.len())?;
    Ok(chars[index])
}

fn random_index(upper_bound: usize) -> Result<usize, String> {
    if upper_bound == 0 {
        return Err("Cannot choose from an empty character set".to_string());
    }

    let upper_bound_u128 = upper_bound as u128;
    let accepted_values = (u64::MAX as u128) + 1;
    let zone = accepted_values - (accepted_values % upper_bound_u128);

    loop {
        let value = random_u64()? as u128;
        if value < zone {
            return Ok((value % upper_bound_u128) as usize);
        }
    }
}

fn shuffle(values: &mut [char]) -> Result<(), String> {
    if values.len() <= 1 {
        return Ok(());
    }

    for i in (1..values.len()).rev() {
        let j = random_index(i + 1)?;
        values.swap(i, j);
    }

    Ok(())
}

fn random_u64() -> Result<u64, String> {
    let mut bytes = [0_u8; 8];
    fill_random(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

#[cfg(unix)]
fn fill_random(buffer: &mut [u8]) -> Result<(), String> {
    let mut file = File::open("/dev/urandom")
        .map_err(|err| format!("Failed to open /dev/urandom: {err}"))?;
    file.read_exact(buffer)
        .map_err(|err| format!("Failed to read random bytes: {err}"))
}

#[cfg(windows)]
fn fill_random(buffer: &mut [u8]) -> Result<(), String> {
    use std::ffi::c_void;

    type NTSTATUS = i32;
    const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 0x00000002;

    #[link(name = "bcrypt")]
    extern "system" {
        fn BCryptGenRandom(
            algorithm: *mut c_void,
            buffer: *mut u8,
            buffer_len: u32,
            flags: u32,
        ) -> NTSTATUS;
    }

    let status = unsafe {
        BCryptGenRandom(
            std::ptr::null_mut(),
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            BCRYPT_USE_SYSTEM_PREFERRED_RNG,
        )
    };

    if status >= 0 {
        Ok(())
    } else {
        Err(format!("BCryptGenRandom failed with status code {status}"))
    }
}

#[cfg(not(any(unix, windows)))]
fn fill_random(_buffer: &mut [u8]) -> Result<(), String> {
    Err("Secure randomness is not implemented for this platform".to_string())
}

fn estimate_entropy_bits(length: usize, pool_size: usize) -> f64 {
    if length == 0 || pool_size <= 1 {
        return 0.0;
    }

    (length as f64) * (pool_size as f64).log2()
}

fn evaluate_strength(entropy_bits: f64) -> Strength {
    if entropy_bits < 50.0 {
        Strength::Weak
    } else if entropy_bits < 72.0 {
        Strength::Fair
    } else if entropy_bits < 96.0 {
        Strength::Strong
    } else {
        Strength::VeryStrong
    }
}

fn apply_color(text: &str, color: &str, use_color: bool) -> String {
    if use_color {
        format!("{color}{text}{RESET}")
    } else {
        text.to_string()
    }
}

pub fn password_contains_required_groups(password: &str, config: &Config) -> bool {
    let chars: Vec<char> = password.chars().collect();
    let symbol_source = config.custom_symbols.as_deref().unwrap_or(SYMBOLS);

    (!config.include_lowercase || chars.iter().any(|ch| LOWERCASE.contains(*ch)))
        && (!config.include_uppercase || chars.iter().any(|ch| UPPERCASE.contains(*ch)))
        && (!config.include_digits || chars.iter().any(|ch| DIGITS.contains(*ch)))
        && (!config.include_symbols || chars.iter().any(|ch| symbol_source.contains(*ch)))
        && chars.iter().all(|ch| !config.exclude_chars.contains(*ch))
        && (!config.exclude_similar || chars.iter().all(|ch| !SIMILAR_CHARS.contains(*ch)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_flags() {
        let args = vec![
            "--length".to_string(),
            "24".to_string(),
            "--count=3".to_string(),
            "--exclude-similar".to_string(),
            "--copy".to_string(),
            "--pretty".to_string(),
        ];

        let action = parse_args(&args).expect("args should parse");
        let Action::Generate(config) = action else {
            panic!("expected generate action");
        };

        assert_eq!(config.length, 24);
        assert_eq!(config.count, 3);
        assert!(config.exclude_similar);
        assert!(config.copy_to_clipboard);
        assert!(config.pretty);
    }

    #[test]
    fn parses_symbols_exclude_and_output_flags() {
        let args = vec![
            "--symbols=@#$".to_string(),
            "--exclude".to_string(),
            "O0l1".to_string(),
            "--output=passwords.txt".to_string(),
        ];

        let action = parse_args(&args).expect("args should parse");
        let Action::Generate(config) = action else {
            panic!("expected generate action");
        };

        assert_eq!(config.custom_symbols.as_deref(), Some("@#$"));
        assert_eq!(config.exclude_chars, "O0l1");
        assert_eq!(config.output_file.as_deref(), Some("passwords.txt"));
    }

    #[test]
    fn rejects_when_all_groups_are_disabled() {
        let config = Config {
            include_lowercase: false,
            include_uppercase: false,
            include_digits: false,
            include_symbols: false,
            ..Config::default()
        };

        let err = validate_config(config).expect_err("config should fail");
        assert!(err.contains("At least one character group must be enabled"));
    }

    #[test]
    fn rejects_short_length_for_selected_groups() {
        let config = Config {
            length: 2,
            include_lowercase: true,
            include_uppercase: true,
            include_digits: true,
            include_symbols: false,
            ..Config::default()
        };

        let err = validate_config(config).expect_err("config should fail");
        assert!(err.contains("Length must be at least 3"));
    }

    #[test]
    fn excludes_similar_and_custom_characters() {
        let filtered = filtered_charset("0Ool1I|abcXYZ!@#", true, "Z!");
        let output: String = filtered.into_iter().collect();

        assert_eq!(output, "abcXY@#");
    }

    #[test]
    fn deduplicates_custom_symbol_charset() {
        let filtered = filtered_charset("!!@@##$$", false, "");
        let output: String = filtered.into_iter().collect();

        assert_eq!(output, "!@#$");
    }

    #[test]
    fn generated_password_respects_requested_properties() {
        let config = Config {
            length: 24,
            count: 1,
            exclude_similar: true,
            exclude_chars: "abc123".to_string(),
            ..Config::default()
        };

        let result = generate_passwords(&config).expect("generation should succeed");
        let password = &result.passwords[0];

        assert_eq!(password.chars().count(), 24);
        assert!(password_contains_required_groups(password, &config));
        assert!(!password.chars().any(|ch| SIMILAR_CHARS.contains(ch)));
        assert!(!password.chars().any(|ch| config.exclude_chars.contains(ch)));
    }

    #[test]
    fn generated_password_uses_custom_symbols() {
        let config = Config {
            length: 16,
            count: 1,
            include_lowercase: false,
            include_uppercase: false,
            include_digits: false,
            include_symbols: true,
            custom_symbols: Some("@#$".to_string()),
            ..Config::default()
        };

        let result = generate_passwords(&config).expect("generation should succeed");
        let password = &result.passwords[0];

        assert!(password.chars().all(|ch| "@#$".contains(ch)));
    }

    #[test]
    fn file_output_is_plain_password_lines() {
        let output = format_passwords_for_file(&GeneratedPasswords {
            passwords: vec!["alpha".to_string(), "beta".to_string()],
            strength: Strength::Strong,
            entropy_bits: 80.0,
            pool_size: 42,
        });

        assert_eq!(output, "alpha\nbeta");
    }

    #[test]
    fn entropy_thresholds_map_to_strength_levels() {
        assert_eq!(evaluate_strength(40.0), Strength::Weak);
        assert_eq!(evaluate_strength(60.0), Strength::Fair);
        assert_eq!(evaluate_strength(80.0), Strength::Strong);
        assert_eq!(evaluate_strength(120.0), Strength::VeryStrong);
    }
}
