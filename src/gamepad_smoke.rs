use std::fmt::Write as _;
use std::fs;
use std::path::Path;
#[cfg(target_os = "linux")]
use std::path::PathBuf;

#[cfg(target_os = "linux")]
use std::ffi::CString;
#[cfg(target_os = "linux")]
use std::os::raw::{c_char, c_int};

use crate::{OathError, GAMEPAD_SMOKE_SCHEMA, PRODUCT_NAME};

pub fn write_gamepad_smoke_artifacts(out_dir: impl AsRef<Path>) -> Result<(), OathError> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    let result = gamepad_smoke_result();
    fs::write(
        out_dir.join("gamepad_smoke.json"),
        render_gamepad_smoke_json(&result),
    )?;
    fs::write(
        out_dir.join("gamepad_smoke_report.md"),
        render_gamepad_smoke_report(&result),
    )?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GamepadSmokeResult {
    status: &'static str,
    devices: Vec<GamepadSmokeDevice>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GamepadSmokeDevice {
    path: String,
    name: String,
    opened_nonblocking: bool,
    sample_event_readable: bool,
    sample_event_bytes: usize,
    sample_event_init_flag: bool,
    sample_event_kind: &'static str,
}

fn gamepad_smoke_result() -> GamepadSmokeResult {
    let devices = gamepad_smoke_devices();
    let status = if devices.iter().any(|device| device.opened_nonblocking) {
        "PASSED_LINUX_JOYSTICK_INTERFACE"
    } else {
        "BLOCKED_NO_READABLE_LINUX_JOYSTICK"
    };
    GamepadSmokeResult { status, devices }
}

#[cfg(target_os = "linux")]
fn gamepad_smoke_devices() -> Vec<GamepadSmokeDevice> {
    let mut paths: Vec<PathBuf> = match fs::read_dir("/dev/input") {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| {
                        name.starts_with("js") && name[2..].chars().all(|ch| ch.is_ascii_digit())
                    })
                    .unwrap_or(false)
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let path_text = path.display().to_string();
            let sample = linux_joystick_sample(&path);
            let event_type = sample
                .bytes
                .as_ref()
                .and_then(|bytes| bytes.get(6).copied())
                .unwrap_or(0);
            GamepadSmokeDevice {
                path: path_text,
                name: linux_joystick_name(&path),
                opened_nonblocking: sample.opened_nonblocking,
                sample_event_readable: sample.bytes.is_some(),
                sample_event_bytes: sample.bytes.as_ref().map(Vec::len).unwrap_or(0),
                sample_event_init_flag: event_type & 0x80 != 0,
                sample_event_kind: linux_joystick_event_kind(event_type),
            }
        })
        .collect()
}

#[cfg(not(target_os = "linux"))]
fn gamepad_smoke_devices() -> Vec<GamepadSmokeDevice> {
    Vec::new()
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct LinuxJoystickSample {
    opened_nonblocking: bool,
    bytes: Option<Vec<u8>>,
}

#[cfg(target_os = "linux")]
fn linux_joystick_name(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");
    let sysfs_path = Path::new("/sys/class/input").join(name).join("device/name");
    fs::read_to_string(sysfs_path)
        .map(|text| text.trim().to_string())
        .ok()
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "unknown linux joystick interface".to_string())
}

#[cfg(target_os = "linux")]
fn linux_joystick_sample(path: &Path) -> LinuxJoystickSample {
    let path_text = path.display().to_string();
    let c_path = match CString::new(path_text.as_bytes()) {
        Ok(value) => value,
        Err(_) => {
            return LinuxJoystickSample {
                opened_nonblocking: false,
                bytes: None,
            };
        }
    };
    let fd = unsafe { open(c_path.as_ptr(), O_RDONLY | O_NONBLOCK) };
    if fd < 0 {
        return LinuxJoystickSample {
            opened_nonblocking: false,
            bytes: None,
        };
    }
    let mut buffer = [0u8; 8];
    let read_count = unsafe { read(fd, buffer.as_mut_ptr(), buffer.len()) };
    unsafe {
        close(fd);
    }
    let bytes = if read_count > 0 {
        Some(buffer[..read_count as usize].to_vec())
    } else {
        None
    };
    LinuxJoystickSample {
        opened_nonblocking: true,
        bytes,
    }
}

#[cfg(target_os = "linux")]
fn linux_joystick_event_kind(event_type: u8) -> &'static str {
    match event_type & 0x03 {
        1 => "button",
        2 => "axis",
        3 => "button_axis",
        _ => {
            if event_type & 0x80 != 0 {
                "init"
            } else {
                "none"
            }
        }
    }
}

fn render_gamepad_smoke_json(result: &GamepadSmokeResult) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", GAMEPAD_SMOKE_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "status", result.status, true);
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(
        &mut out,
        "  \"linux_joystick_interface_smoke_claimed\": {},",
        result
            .devices
            .iter()
            .any(|device| device.opened_nonblocking)
    )
    .unwrap();
    writeln!(&mut out, "  \"physical_gamepad_hardware_claimed\": false,").unwrap();
    writeln!(&mut out, "  \"steam_deck_hardware_claimed\": false,").unwrap();
    writeln!(&mut out, "  \"device_count\": {},", result.devices.len()).unwrap();
    writeln!(&mut out, "  \"devices\": [").unwrap();
    for (index, device) in result.devices.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "path", &device.path, true);
        write_json_field(&mut out, 3, "name", &device.name, true);
        writeln!(
            &mut out,
            "      \"opened_nonblocking\": {},",
            device.opened_nonblocking
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"sample_event_readable\": {},",
            device.sample_event_readable
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"sample_event_bytes\": {},",
            device.sample_event_bytes
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"sample_event_init_flag\": {},",
            device.sample_event_init_flag
        )
        .unwrap();
        write_json_field(
            &mut out,
            3,
            "sample_event_kind",
            device.sample_event_kind,
            false,
        );
        writeln!(&mut out, "    }}{}", comma(index + 1, result.devices.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_gamepad_smoke_report(result: &GamepadSmokeResult) -> String {
    let opened = result
        .devices
        .iter()
        .filter(|device| device.opened_nonblocking)
        .count();
    let readable = result
        .devices
        .iter()
        .filter(|device| device.sample_event_readable)
        .count();
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Gamepad Smoke Report").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Status: {}", result.status).unwrap();
    writeln!(
        &mut out,
        "- Source: Linux `/dev/input/js*` joystick interface"
    )
    .unwrap();
    writeln!(&mut out, "- Presentation only: `true`").unwrap();
    writeln!(&mut out, "- Truth mutation: `none`").unwrap();
    writeln!(
        &mut out,
        "- Linux joystick interface smoke claimed: `{}`",
        opened > 0
    )
    .unwrap();
    writeln!(&mut out, "- Physical gamepad hardware claimed: `false`").unwrap();
    writeln!(&mut out, "- Steam Deck hardware claimed: `false`").unwrap();
    writeln!(&mut out, "- Device count: `{}`", result.devices.len()).unwrap();
    writeln!(&mut out, "- Nonblocking-open devices: `{opened}`").unwrap();
    writeln!(&mut out, "- Devices with sample events: `{readable}`").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Devices").unwrap();
    writeln!(&mut out).unwrap();
    if result.devices.is_empty() {
        writeln!(
            &mut out,
            "- No Linux joystick interfaces were present in this environment."
        )
        .unwrap();
    }
    for device in &result.devices {
        writeln!(
            &mut out,
            "- `{}` `{}` open `{}` sample `{}` kind `{}` init `{}` bytes `{}`",
            device.path,
            device.name,
            device.opened_nonblocking,
            device.sample_event_readable,
            device.sample_event_kind,
            device.sample_event_init_flag,
            device.sample_event_bytes
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "This report proves only that the native input layer can see a Linux joystick-class device. It does not prove physical controller ergonomics, Steam Deck compliance, glyph correctness, or owner acceptance."
    )
    .unwrap();
    out
}

fn write_json_field(out: &mut String, indent: usize, key: &str, value: &str, trailing: bool) {
    let spaces = "  ".repeat(indent);
    writeln!(
        out,
        "{}{}: {}{}",
        spaces,
        json_quote(key),
        json_quote(value),
        if trailing { "," } else { "" }
    )
    .unwrap();
}

fn json_quote(value: &str) -> String {
    let mut out = String::new();
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => write!(&mut out, "\\u{:04x}", ch as u32).unwrap(),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn comma(index: usize, len: usize) -> &'static str {
    if index < len {
        ","
    } else {
        ""
    }
}

#[cfg(target_os = "linux")]
const O_RDONLY: c_int = 0;
#[cfg(target_os = "linux")]
const O_NONBLOCK: c_int = 0o4000;

#[cfg(target_os = "linux")]
extern "C" {
    fn open(pathname: *const c_char, flags: c_int) -> c_int;
    fn read(fd: c_int, buf: *mut u8, count: usize) -> isize;
    fn close(fd: c_int) -> c_int;
}
