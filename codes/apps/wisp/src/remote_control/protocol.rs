//! Small framed protocol. No HTTP; see docs/remote_control/protocol.md.
use serde::Deserialize;
use serde_json::{Value, json};
use std::io::{self, Read, Write};

pub const VERSION: u32 = 1;
pub const MAX_JSON: usize = 4096;
pub const MAX_BINARY: usize = 32 * 1024 * 1024;

#[derive(Debug)]
pub struct Request {
    pub id: u64,
    pub command: Command,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Hello {
        version: u32,
    },
    Status,
    Screenshot,
    Stream {
        fps: u8,
        #[serde(default)]
        codec: Option<String>,
    },
    Keyframe,
    TouchDown {
        slot: u32,
        x: f64,
        y: f64,
    },
    TouchMove {
        slot: u32,
        x: f64,
        y: f64,
    },
    TouchUp {
        slot: u32,
    },
    TouchCancel,
    Key {
        code: u32,
        down: bool,
    },
    Action {
        name: String,
    },
    Wake,
    Sleep,
    Heartbeat,
    Disconnect,
}

pub struct Reply {
    pub json: Value,
    pub pixels: Option<super::capture::Pixels>,
}
impl Reply {
    pub fn ok(id: u64) -> Self {
        Self {
            json: json!({"id":id,"ok":true}),
            pixels: None,
        }
    }
    pub fn error(id: u64, error: impl ToString) -> Self {
        Self {
            json: json!({"id":id,"ok":false,"error":error.to_string()}),
            pixels: None,
        }
    }
}

pub fn read_request(reader: &mut impl Read) -> io::Result<Request> {
    let mut header = [0; 8];
    reader.read_exact(&mut header)?;
    let len = u32::from_be_bytes(header[..4].try_into().unwrap()) as usize;
    let binary = u32::from_be_bytes(header[4..].try_into().unwrap());
    if len == 0 || len > MAX_JSON || binary != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid frame lengths",
        ));
    }
    let mut data = vec![0; len];
    reader.read_exact(&mut data)?;
    // Deserialize command separately: serde flatten + deny_unknown_fields is ambiguous.
    let mut value: Value = serde_json::from_slice(&data)?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "expected object"))?;
    let id = object
        .remove("id")
        .and_then(|v| v.as_u64())
        .filter(|i| *i > 0)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "positive id required"))?;
    let allowed: &[&str] = match object.get("type").and_then(Value::as_str) {
        Some("hello") => &["type", "version"],
        Some("stream") => &["type", "fps", "codec"],
        Some("touch_down" | "touch_move") => &["type", "slot", "x", "y"],
        Some("touch_up") => &["type", "slot"],
        Some("key") => &["type", "code", "down"],
        Some("action") => &["type", "name"],
        _ => &["type"],
    };
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unknown command field",
        ));
    }
    let command = serde_json::from_value(value)?;
    Ok(Request { id, command })
}

pub fn write_frame(writer: &mut impl Write, json: &Value, binary: &[u8]) -> io::Result<()> {
    let data = serde_json::to_vec(json)?;
    if binary.len() > MAX_BINARY {
        return Err(io::Error::other("frame too large"));
    }
    writer.write_all(&(data.len() as u32).to_be_bytes())?;
    writer.write_all(&(binary.len() as u32).to_be_bytes())?;
    writer.write_all(&data)?;
    writer.write_all(binary)?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn framed(s: &str) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend((s.len() as u32).to_be_bytes());
        b.extend(0u32.to_be_bytes());
        b.extend(s.as_bytes());
        b
    }
    #[test]
    fn rejects_unbounded_frames() {
        assert!(read_request(&mut &b"\x00\x00\x10\x01\x00\x00\x00\x00"[..]).is_err());
        assert!(read_request(&mut &b"\x00\x00\x00\x01\x00\x00\x00\x01"[..]).is_err());
    }
    #[test]
    fn parses_and_rejects_unknown_fields() {
        assert!(matches!(
            read_request(&mut framed(r#"{"id":1,"type":"hello","version":1}"#).as_slice())
                .unwrap()
                .command,
            Command::Hello { version: 1 }
        ));
        assert!(
            read_request(&mut framed(r#"{"id":2,"type":"status","extra":true}"#).as_slice())
                .is_err()
        );
        assert!(read_request(&mut framed(r#"{"id":0,"type":"status"}"#).as_slice()).is_err());
    }
}
