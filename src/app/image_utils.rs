use image::DynamicImage;

pub fn average_color(image: &DynamicImage) -> [u8; 3] {
    let rgb = image.to_rgb8();
    let mut sums = [0u64; 3];
    for pixel in rgb.pixels() {
        sums[0] += pixel[0] as u64;
        sums[1] += pixel[1] as u64;
        sums[2] += pixel[2] as u64;
    }

    let count = (rgb.width() as u64).saturating_mul(rgb.height() as u64).max(1);
    [
        (sums[0] / count) as u8,
        (sums[1] / count) as u8,
        (sums[2] / count) as u8,
    ]
}

pub fn color_distance(a: [u8; 3], b: [u8; 3]) -> u32 {
    let dr = a[0] as i32 - b[0] as i32;
    let dg = a[1] as i32 - b[1] as i32;
    let db = a[2] as i32 - b[2] as i32;
    (dr * dr + dg * dg + db * db) as u32
}
