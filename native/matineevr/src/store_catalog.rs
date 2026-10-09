use anyhow::{Result, ensure};
use std::{fs, path::Path};

pub struct Movie {
    pub title: String,
    pub overview: String,
    pub size: [usize; 2],
    pub pixels: Vec<u8>,
}

pub fn read(path: &Path) -> Result<Vec<Movie>> {
    ensure!(fs::metadata(path)?.len() <= 4 * 1024 * 1024, "Catalog exceeds capacity");
    let bytes = fs::read(path)?;
    let mut reader = Reader(&bytes);
    ensure!(reader.take(8)? == b"FBCAT001", "Invalid catalog header");
    let count = reader.number()?;
    ensure!(count <= 6, "Too many shelf entries");
    reader.number()?;
    reader.number()?;
    let mut movies = Vec::new();
    for _ in 0..count {
        ensure!(reader.take(32)?.iter().all(u8::is_ascii_hexdigit), "Invalid movie identity");
        let title = reader.text(1024)?;
        let overview = reader.text(12000)?;
        let size = [reader.number()?, reader.number()?];
        ensure!(size[0] == 192 && size[1] == 288, "Invalid artwork size");
        let pixels = reader.take(size[0] * size[1] * 4)?.to_vec();
        movies.push(Movie { title, overview, size, pixels });
    }
    ensure!(reader.0.is_empty(), "Unexpected catalog data");
    Ok(movies)
}

struct Reader<'a>(&'a [u8]);
impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8]> {
        ensure!(length <= self.0.len(), "Incomplete catalog");
        let (value, rest) = self.0.split_at(length);
        self.0 = rest;
        Ok(value)
    }
    fn number(&mut self) -> Result<usize> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into()?) as usize)
    }
    fn text(&mut self, limit: usize) -> Result<String> {
        let length = self.number()?;
        ensure!(length <= limit, "Catalog text exceeds capacity");
        Ok(String::from_utf8(self.take(length)?.to_vec())?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reader_rejects_truncated_and_excessive_lengths() {
        let mut reader = Reader(&[1, 0]);
        assert!(reader.number().is_err());
        let bytes = [255, 255, 255, 255];
        assert!(Reader(&bytes).text(1024).is_err());
    }
}
