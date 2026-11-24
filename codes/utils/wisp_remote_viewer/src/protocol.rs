//! Wisp remote_control v1 and host-session IPC framing; no HTTP or Python runtime.
use serde_json::Value;
use std::io::{self, Read, Write};

pub const MAX_BINARY: usize = 32 * 1024 * 1024;
pub const MAX_JSON: usize = 128 * 1024;

pub fn read_frame(reader: &mut impl Read) -> io::Result<(Value, Vec<u8>)> {
    let mut header = [0u8; 8];
    reader.read_exact(&mut header)?;
    let json_len = u32::from_be_bytes(header[..4].try_into().unwrap()) as usize;
    let binary_len = u32::from_be_bytes(header[4..].try_into().unwrap()) as usize;
    if json_len == 0 || json_len > MAX_JSON || binary_len > MAX_BINARY {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid remote frame lengths",
        ));
    }
    let mut json = vec![0; json_len];
    reader.read_exact(&mut json)?;
    let metadata: Value = serde_json::from_slice(&json)?;
    if !metadata.is_object() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame metadata must be an object",
        ));
    }
    let mut binary = vec![0; binary_len];
    reader.read_exact(&mut binary)?;
    Ok((metadata, binary))
}

pub fn write_frame(writer: &mut impl Write, command: &Value) -> io::Result<()> {
    let json = serde_json::to_vec(command)?;
    if json.is_empty() || json.len() > 4096 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "command too large",
        ));
    }
    writer.write_all(&(json.len() as u32).to_be_bytes())?;
    writer.write_all(&0u32.to_be_bytes())?;
    writer.write_all(&json)?;
    writer.flush()
}

pub fn check_reply(metadata: &Value) -> Result<(), String> {
    if metadata.get("ok").and_then(Value::as_bool) == Some(true) {
        Ok(())
    } else {
        Err(metadata
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("remote command failed")
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn fragmented_reads_preserve_binary_and_next_message() {
        struct Fragmented<'a>(&'a [u8]);
        impl Read for Fragmented<'_> {
            fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
                let count = out.len().min(1).min(self.0.len());
                out[..count].copy_from_slice(&self.0[..count]);
                self.0 = &self.0[count..];
                Ok(count)
            }
        }
        let meta = br#"{"id":0,"event":"frame","ok":true}"#;
        let mut bytes = Vec::new();
        bytes.extend((meta.len() as u32).to_be_bytes());
        bytes.extend(3u32.to_be_bytes());
        bytes.extend(meta);
        bytes.extend([1, 2, 3]);
        write_frame(&mut bytes, &json!({"id":2,"ok":true})).unwrap();
        let mut fragmented = Fragmented(&bytes);
        assert_eq!(read_frame(&mut fragmented).unwrap().1, [1, 2, 3]);
        assert_eq!(read_frame(&mut fragmented).unwrap().0["id"], 2);
    }
    #[test]
    fn oversized_lengths_are_rejected_before_allocation() {
        let mut header = Vec::new();
        header.extend(1u32.to_be_bytes());
        header.extend(((MAX_BINARY + 1) as u32).to_be_bytes());
        assert_eq!(
            read_frame(&mut header.as_slice()).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert!(read_frame(&mut &b"\0\0\0\0\0\0\0\0"[..]).is_err());
    }
}

pub fn write_packet(writer: &mut impl Write, metadata: &Value, binary: &[u8]) -> io::Result<()> {
    let bytes = serde_json::to_vec(metadata)?;
    if bytes.len() > MAX_JSON || binary.len() > MAX_BINARY {
        return Err(io::Error::other("packet exceeds limits"));
    }
    writer.write_all(&(bytes.len() as u32).to_be_bytes())?;
    writer.write_all(&(binary.len() as u32).to_be_bytes())?;
    writer.write_all(&bytes)?;
    writer.write_all(binary)?;
    writer.flush()
}
