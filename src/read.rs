//! Client-independent reads. Explicit ranges always bypass outlining.
use crate::{config::Config, hook::OUTLINE_MAX_BYTES, outline};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

pub fn read_to(
    path: &Path,
    offset: Option<usize>,
    limit: Option<usize>,
    cfg: &Config,
    out: &mut impl Write,
) -> io::Result<()> {
    if offset == Some(0) || limit == Some(0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "offset and limit must be at least 1",
        ));
    }
    let file = File::open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected a regular file",
        ));
    }
    if offset.is_some() || limit.is_some() {
        let mut reader = BufReader::new(file);
        let mut line = Vec::new();
        let start = offset.unwrap_or(1);
        let count = limit.unwrap_or(usize::MAX);
        let mut sent = 0;
        let mut index = 0;
        loop {
            line.clear();
            if reader.read_until(b'\n', &mut line)? == 0 {
                break;
            }
            index += 1;
            if index >= start {
                out.write_all(&line)?;
                sent += 1;
                if sent == count {
                    break;
                }
            }
        }
        return Ok(());
    }
    let name = path.to_string_lossy();
    if meta.len() <= cfg.clamp_threshold
        || meta.len() > OUTLINE_MAX_BYTES
        || cfg.is_excluded(&name)
        || outline::is_image_pdf_or_notebook(&name)
    {
        io::copy(&mut BufReader::new(file), out)?;
        return Ok(());
    }
    // Bound allocation even if the file grows after metadata was read.
    let mut reader = file.take(OUTLINE_MAX_BYTES + 1);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    if bytes.len() <= OUTLINE_MAX_BYTES as usize && !bytes.contains(&0) {
        if let Ok(text) = std::str::from_utf8(&bytes) {
            writeln!(
                out,
                "{}",
                outline::generate_for(
                    &name,
                    text,
                    bytes.len() as u64,
                    cfg.clamp_threshold,
                    cfg.outline_max_lines,
                    true
                )
            )?;
            return Ok(());
        }
    }
    out.write_all(&bytes)?;
    io::copy(&mut reader.into_inner(), out)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::tempdir;
    #[test]
    fn byte_exact_small_binary_and_scoped_reads() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("sample.txt");
        for bytes in [b"one\r\ntwo\nlast".as_slice(), b"\xff\0\xfe".as_slice()] {
            std::fs::write(&path, bytes).unwrap();
            let mut out = Vec::new();
            read_to(&path, None, None, &Config::default(), &mut out).unwrap();
            assert_eq!(out, bytes);
        }
        std::fs::write(&path, b"one\r\ntwo\r\nlast").unwrap();
        for (offset, limit, expected) in [
            (Some(2), Some(1), b"two\r\n".as_slice()),
            (Some(3), None, b"last"),
            (Some(99), Some(1), b""),
            (None, Some(1), b"one\r\n"),
        ] {
            let mut out = Vec::new();
            read_to(&path, offset, limit, &Config::default(), &mut out).unwrap();
            assert_eq!(out, expected);
        }
        assert!(read_to(&path, Some(0), None, &Config::default(), &mut Vec::new()).is_err());
    }
    #[test]
    fn large_file_outline_and_full_recovery() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("big.rs");
        let content = "fn example() {}\n".repeat(3000);
        std::fs::write(&path, &content).unwrap();
        let mut out = Vec::new();
        read_to(&path, None, None, &Config::default(), &mut out).unwrap();
        assert!(out.len() < content.len() / 4);
        assert!(String::from_utf8(out).unwrap().contains("stk read"));
        let mut out = Vec::new();
        read_to(&path, Some(1), None, &Config::default(), &mut out).unwrap();
        assert_eq!(out, content.as_bytes());
        let cfg = Config {
            exclude: vec!["*.rs".into()],
            ..Config::default()
        };
        out.clear();
        read_to(&path, None, None, &cfg, &mut out).unwrap();
        assert_eq!(out, content.as_bytes());
    }
}
