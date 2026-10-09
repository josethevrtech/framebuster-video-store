use matineevr::media::Pixels;

pub fn reference(
    frame: &Pixels,
    chroma: i32,
    x: usize,
    y: usize,
    scale: f32,
    sample: &impl Fn(usize) -> f32,
) -> [f32; 3] {
    let width = frame.width as usize;
    let height = frame.height as usize;
    let cw = width.div_ceil(2);
    let ch = height.div_ceil(2);
    let cx = (x as f32 - if [1, 3].contains(&chroma) { 0.0 } else { 0.5 }) / 2.0;
    let cy = (y as f32 - if [3, 4].contains(&chroma) { 0.0 } else { 0.5 }) / 2.0;
    let linear = |plane: usize| {
        let at = |dx: i32, dy: i32| {
            let sx = (cx.floor() as i32 + dx).clamp(0, cw as i32 - 1) as usize;
            let sy = (cy.floor() as i32 + dy).clamp(0, ch as i32 - 1) as usize;
            sample(width * height + plane * cw * ch + sy * cw + sx)
        };
        let fx = cx - cx.floor();
        let fy = cy - cy.floor();
        (at(0, 0) * (1.0 - fx) + at(1, 0) * fx) * (1.0 - fy)
            + (at(0, 1) * (1.0 - fx) + at(1, 1) * fx) * fy
    };
    let unit = if scale > 255.0 { 4.0 } else { 1.0 };
    let limited = frame.full_range == 0;
    let y = (sample(y * width + x) - if limited { 16.0 * unit } else { 0.0 })
        / if limited { 219.0 * unit } else { scale };
    let u = (linear(0) - 128.0 * unit) / if limited { 224.0 * unit } else { scale };
    let v = (linear(1) - 128.0 * unit) / if limited { 224.0 * unit } else { scale };
    let (kr, kb) = match frame.colorspace {
        5 | 6 => (0.299, 0.114),
        9 => (0.2627, 0.0593),
        _ => (0.2126, 0.0722),
    };
    [
        y + 2.0 * (1.0 - kr) * v,
        y - 2.0 * (kb * (1.0 - kb) * u + kr * (1.0 - kr) * v) / (1.0 - kr - kb),
        y + 2.0 * (1.0 - kb) * u,
    ]
}
