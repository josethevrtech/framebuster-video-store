use crate::{hud_text::Canvas, store_catalog::Movie};
use std::path::Path;

pub fn compose(movie: &Movie) -> Canvas {
    let mut canvas = Canvas::new(Path::new(""), [384, 656]);
    canvas.clear();
    for y in 0..576 {
        for x in 0..384 {
            let source = ((y / 2) * movie.size[0] + x / 2) * 4;
            let target = ((y + 80) * 384 + x) * 4;
            canvas.pixels[target..target + 4].copy_from_slice(&movie.pixels[source..source + 4]);
        }
    }
    canvas
}

pub fn atlas(movies: &[Movie]) -> Canvas {
    let mut result = Canvas::new(Path::new(""), [1440, 3240]);
    for (i, movie) in movies.iter().take(54).enumerate() {
        let poster = compose(movie);
        let left = 120 + (i % 6) * 240 - 96;
        let bottom = (i / 18) * 1080 + 180 + ((i % 18) / 6) * 360 - 164;
        for y in 0..328 {
            for x in 0..192 {
                let source = (y * 2 * 384 + x * 2) * 4;
                let target = ((bottom + y) * 1440 + left + x) * 4;
                result.pixels[target..target + 4].copy_from_slice(&poster.pixels[source..source + 4]);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn poster_keeps_artwork_opaque_without_text_captions() {
        let movie = Movie { size: [192, 288],
            pixels: [12, 34, 56, 255].repeat(192 * 288) };
        let canvas = compose(&movie);
        assert_eq!(canvas.pixels.len(), 384 * 656 * 4);
        assert_eq!(&canvas.pixels[(80 * 384) * 4..(80 * 384) * 4 + 4], &[12, 34, 56, 255]);
        let atlas = atlas(&[movie]);
        assert_eq!(&atlas.pixels[..4], &[0, 0, 0, 0]);
        assert_eq!(atlas.pixels.len(), 1440 * 3240 * 4);
    }
}
