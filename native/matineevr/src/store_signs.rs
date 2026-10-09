use crate::{font, store_geometry::{Vertex, box_mesh}};

fn text(v: &mut Vec<Vertex>, value: &str, p: [f32; 3], size: f32, facing: f32) {
    let width = value.chars().count() as f32 * 6.0 * size;
    for (i, letter) in value.chars().enumerate() {
        for (column, bits) in font::glyph(letter).into_iter().enumerate() {
            for row in 0..7 {
                if bits & (1 << row) != 0 {
                    box_mesh(v, [p[0] + facing * ((i * 6 + column) as f32 * size - width / 2.0),
                        p[1] + (3.0 - row as f32) * size, p[2]],
                        [size * 0.86, size * 0.86, 0.018], [1.0, 0.75, 0.12]);
                }
            }
        }
    }
}

pub fn signs(v: &mut Vec<Vertex>) {
    let navy = [0.025, 0.07, 0.18];
    box_mesh(v, [0.0, 1.63, -3.72], [6.7, 0.60, 0.10], navy);
    text(v, "FRAMEBUSTER VIDEO", [0.0, 1.65, -3.65], 0.06, 1.0);
    box_mesh(v, [0.0, 1.10, 1.1], [1.85, 0.45, 0.10], [0.66, 0.43, 0.13]);
    box_mesh(v, [0.0, 1.10, 1.17], [1.75, 0.35, 0.035], [0.32, 0.035, 0.12]);
    text(v, "MOVIES & TV", [0.0, 1.10, 1.20], 0.022, -1.0);
    for x in [-0.70, 0.70] {
        box_mesh(v, [x, 1.58, 1.1], [0.012, 0.55, 0.012], [0.12, 0.12, 0.14]);
    }
    for (x, label) in [(-4.3, "ACTION"), (4.3, "CLASSICS")] {
        box_mesh(v, [x, 0.78, -0.82], [1.9, 0.40, 0.08], navy);
        text(v, label, [x, 0.78, -0.76], 0.034, 1.0);
        for side in [-0.75, 0.75] {
            box_mesh(v, [x + side, 1.3, -0.82], [0.025, 0.72, 0.025], [0.3, 0.32, 0.35]);
        }
    }
    text(v, "CHECKOUT", [0.0, -0.68, 5.62], 0.065, -1.0);
    text(v, "BE KIND - REWIND", [0.0, -1.10, 5.62], 0.035, -1.0);
    for (x, lines) in [(-3.5, ["MOVIE", "NIGHT", "STARTS", "HERE"]),
        (3.5, ["TAKE", "HOME", "A GREAT", "STORY"])] {
        for (row, line) in lines.into_iter().enumerate() {
            text(v, line, [x, 0.65 - row as f32 * 0.3, -3.80], 0.024, 1.0);
        }
    }
}
