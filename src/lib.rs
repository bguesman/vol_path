use glam::Vec3A;
use image::{Rgb, RgbImage};
use indicatif::ProgressIterator;


// Few aliases to distinguish colors from R^3 points
type Color = Vec3A;
#[allow(unused)]
type Point3 = Vec3A;


fn write_color(color: &Color, image: &mut RgbImage, r: u32, c: u32) {
  let remapped_u8 = color * 255.999;
  image.put_pixel(
    r,
    c,
    Rgb([
      remapped_u8.x as u8,
      remapped_u8.y as u8,
      remapped_u8.z as u8,
    ]),
  );
}


pub fn render(out_path: &str) {
  let mut image = RgbImage::new(256, 256);

  for r in (0..image.height()).progress() {
    for c in 0..image.width() {
      let color = Color::new(
        r as f32 / image.height() as f32,
        c as f32 / image.height() as f32,
        0.0,
      );
      write_color(&color, &mut image, r, c);
    }
  }

  image.save(out_path).expect("Failed to save image");
}


#[cfg(test)]
mod tests {}
