use glam::Vec3A;
use image::{Rgb, RgbImage};
use indicatif::ProgressIterator;


// Few aliases to distinguish colors from R^3 points
type Color = Vec3A;
#[allow(unused)]
type Point3 = Vec3A;
type World = Vec<Box<dyn Hittable>>;


struct Ray {
  origin: Point3,
  direction: Vec3A,
}


impl Ray {
  fn new(origin: Point3, direction: Vec3A) -> Ray {
    Ray { origin, direction }
  }

  fn at(&self, index: f32) -> Point3 {
    self.origin + self.direction * index
  }
}


#[derive(Debug, Copy, Clone)]
#[allow(unused)]
struct HitData {
  pub p: Point3,
  pub normal: Vec3A,
  pub t: f32,
  pub front_face: bool,
}


impl HitData {
  fn new(p: Point3, normal: Vec3A, t: f32, r: &Ray) -> Self {
    let front_face = Vec3A::dot(r.direction, normal) < 0.0;
    let normal = if front_face { normal } else { -normal };
    HitData {
      p,
      normal,
      t,
      front_face,
    }
  }
}


enum HitResult {
  Miss,
  Hit(HitData),
}


trait Hittable {
  fn hit(&self, r: &Ray, tmin: f32, tmax: f32) -> HitResult;
}


struct Sphere {
  center: Point3,
  radius: f32,
}


impl Sphere {
  fn new(center: Point3, radius: f32) -> Self {
    Sphere { center, radius }
  }
}


impl Hittable for Sphere {
  fn hit(&self, r: &Ray, tmin: f32, tmax: f32) -> HitResult {
    let origin = self.center - r.origin;
    let a = r.direction.length_squared();
    let h = Vec3A::dot(r.direction, origin);
    let c = origin.length_squared() - self.radius * self.radius;
    let discriminant = h * h - a * c;
    if discriminant < 0.0 {
      return HitResult::Miss;
    }

    let sqrt_disc = f32::sqrt(discriminant);

    let mut t = (h - sqrt_disc) / a;
    if t <= tmin || tmax <= t {
      t = (h + sqrt_disc) / a;
      if t <= tmin || tmax <= t {
        return HitResult::Miss;
      }
    }

    let p = r.at(t);
    let normal = (p - self.center) / self.radius;
    HitResult::Hit(HitData::new(p, normal, t, r))
  }
}


fn closest_hit(world: &World, r: &Ray, tmin: f32, tmax: f32) -> HitResult {
  let mut closest_hit = HitResult::Miss;
  let mut closest_t = tmax;
  for object in world {
    let hit = object.hit(r, tmin, closest_t);
    match hit {
      HitResult::Hit(data) => {
        closest_t = data.t;
        closest_hit = hit;
      }
      HitResult::Miss => (),
    };
  }

  closest_hit
}


fn ray_color(r: &Ray, world: &World) -> Color {
  let hit = closest_hit(world, r, 0.0, f32::MAX);
  match hit {
    HitResult::Hit(data) => 0.5 * (data.normal + 1.0),
    HitResult::Miss => {
      // Sky
      let unit_direction = r.direction.normalize();
      let a = 0.5 * (unit_direction.y + 1.0);
      Color::lerp(Color::new(1.0, 1.0, 1.0), Color::new(0.5, 0.7, 1.0), a)
    }
  }
}


fn write_color(color: &Color, image: &mut RgbImage, r: u32, c: u32) {
  let remapped_u8 = color * 255.999;
  image.put_pixel(
    c,
    r,
    Rgb([
      remapped_u8.x as u8,
      remapped_u8.y as u8,
      remapped_u8.z as u8,
    ]),
  );
}


pub fn render(out_path: &str) {
  // Viewport
  let aspect_ratio = 16.0 / 9.0;
  let image_width = 400;
  let image_height = (image_width as f32 / aspect_ratio) as u32;

  // World
  let world: World = vec![
    Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)),
    Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)),
  ];

  // Camera
  let focal_length = 1.0;
  let camera_center = Point3::new(0.0, 0.0, 0.0);
  let viewport_height = 2.0;
  let viewport_width = viewport_height * (image_width as f32 / image_height as f32);

  // Calculate viewport bounds
  let viewport_u = Vec3A::new(viewport_width, 0.0, 0.0);
  let viewport_v = Vec3A::new(0.0, -viewport_height, 0.0);
  let pixel_delta_u = viewport_u / image_width as f32;
  let pixel_delta_v = viewport_v / image_height as f32;
  let viewport_upper_left =
    camera_center - Vec3A::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
  let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

  // Render
  let mut image = RgbImage::new(image_width, image_height);
  for r in (0..image.height()).progress() {
    for c in 0..image.width() {
      let pixel_center = pixel00_loc + (c as f32 * pixel_delta_u) + (r as f32 * pixel_delta_v);
      let ray_direction = pixel_center - camera_center;
      let ray = Ray::new(camera_center, ray_direction);
      let color = ray_color(&ray, &world);
      write_color(&color, &mut image, r, c);
    }
  }

  image.save(out_path).expect("Failed to save image");
}


#[cfg(test)]
mod tests {}
