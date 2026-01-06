use glam::{Vec2, Vec3A};
use image::{Rgb, RgbImage};
use indicatif::ProgressIterator;
use intervals::Closed;


// Few aliases to distinguish colors from R^3 points
type Color = Vec3A;
#[allow(unused)]
type Point3 = Vec3A;
type World = Vec<Box<dyn Hittable>>;


fn random_vec3_in_range(min: f32, max: f32) -> Vec3A {
  Vec3A::new(
    rand::random_range(min..max),
    rand::random_range(min..max),
    rand::random_range(min..max),
  )
}


#[allow(unused)]
fn random_vec3() -> Vec3A {
  random_vec3_in_range(0.0, 1.0)
}


fn random_unit_vector() -> Vec3A {
  let mut p = random_vec3_in_range(-1.0, 1.0);
  while p.length_squared() > 1.0 || p.length_squared() < 1e-160 {
    p = random_vec3_in_range(-1.0, 1.0);
  }
  p.normalize()
}


fn linear_to_gamma(linear_component: f32) -> f32 {
  if linear_component > 0.0 {
    f32::sqrt(linear_component)
  } else {
    0.0
  }
}


#[allow(unused)]
fn random_on_hemisphere(normal: Vec3A) -> Vec3A {
  let on_unit_sphere = random_unit_vector();
  if Vec3A::dot(on_unit_sphere, normal) > 0.0 {
    on_unit_sphere
  } else {
    -on_unit_sphere
  }
}


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


#[allow(unused)]
struct Hit<'a> {
  pub p: Point3,
  pub normal: Vec3A,
  pub material: &'a dyn Material,
  pub t: f32,
  pub front_face: bool,
}


impl<'a> Hit<'a> {
  fn new(p: Point3, normal: Vec3A, t: f32, r: &Ray, material: &'a dyn Material) -> Self {
    let front_face = Vec3A::dot(r.direction, normal) < 0.0;
    let normal = if front_face { normal } else { -normal };
    Hit {
      p,
      normal,
      material,
      t,
      front_face,
    }
  }
}


trait Hittable {
  fn hit<'a>(&'a self, r: &Ray, ray_t: &Closed<f32>) -> Option<Hit<'a>>;
}


struct Sphere {
  center: Point3,
  radius: f32,
  material: Box<dyn Material>,
}


impl Sphere {
  fn new(center: Point3, radius: f32, material: Box<dyn Material>) -> Self {
    Sphere {
      center,
      radius,
      material,
    }
  }
}


impl Hittable for Sphere {
  fn hit<'a>(&'a self, r: &Ray, ray_t: &Closed<f32>) -> Option<Hit<'a>> {
    let origin = self.center - r.origin;
    let a = r.direction.length_squared();
    let h = Vec3A::dot(r.direction, origin);
    let c = origin.length_squared() - self.radius * self.radius;
    let discriminant = h * h - a * c;
    if discriminant < 0.0 {
      return None;
    }

    let sqrt_disc = f32::sqrt(discriminant);

    let mut t = (h - sqrt_disc) / a;
    if !ray_t.contains(t) {
      t = (h + sqrt_disc) / a;
      if !ray_t.contains(t) {
        return None;
      }
    }

    let p = r.at(t);
    let normal = (p - self.center) / self.radius;
    Some(Hit::new(p, normal, t, r, self.material.as_ref()))
  }
}


#[allow(unused)]
struct ScatterResult<'a> {
  hit: &'a Hit<'a>,
  attenuation: Color,
  scatter_direction: Ray,
}


trait Material {
  fn scatter<'a>(&self, _: &Ray, _: &'a Hit<'a>) -> Option<ScatterResult<'a>> {
    None
  }
}


struct LambertianMaterial {
  albedo: Color,
}


impl Material for LambertianMaterial {
  fn scatter<'a>(&self, _: &Ray, hit: &'a Hit<'a>) -> Option<ScatterResult<'a>> {
    let mut scatter_direction = hit.normal + random_unit_vector();

    // Catch degenerate scatter direction
    if Vec3A::abs(scatter_direction).min_element() < 1e-8 {
      scatter_direction = hit.normal;
    }

    Some(ScatterResult {
      hit,
      attenuation: self.albedo,
      scatter_direction: Ray::new(hit.p, scatter_direction),
    })
  }
}


struct MetalMaterial {
  albedo: Color,
}


impl Material for MetalMaterial {
  fn scatter<'a>(&self, r: &Ray, hit: &'a Hit<'a>) -> Option<ScatterResult<'a>> {
    let reflected = Vec3A::reflect(r.direction, hit.normal);
    Some(ScatterResult {
      hit,
      attenuation: self.albedo,
      scatter_direction: Ray::new(hit.p, reflected),
    })
  }
}


#[allow(unused)]
struct Camera {
  pub aspect_ratio: f32,
  pub image_width: u32,
  pub samples_per_pixel: u32,
  pub max_depth: u32,
  image_height: u32,
  center: Point3,
  pixel00_loc: Point3,
  pixel_delta_u: Vec3A,
  pixel_delta_v: Vec3A,
}


impl Camera {
  fn new(aspect_ratio: f32, image_width: u32, samples_per_pixel: u32, max_depth: u32) -> Self {
    let image_height = (image_width as f32 / aspect_ratio) as u32;

    let center = Point3::new(0.0, 0.0, 0.0);

    // Viewport dimensions
    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * (image_width as f32 / image_height as f32);

    // Pixel mapping
    let viewport_u = Vec3A::new(viewport_width, 0.0, 0.0);
    let viewport_v = Vec3A::new(0.0, -viewport_height, 0.0);
    let pixel_delta_u = viewport_u / image_width as f32;
    let pixel_delta_v = viewport_v / image_height as f32;
    let viewport_upper_left =
      center - Vec3A::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    Camera {
      aspect_ratio,
      image_width,
      samples_per_pixel,
      max_depth,
      image_height,
      center,
      pixel00_loc,
      pixel_delta_u,
      pixel_delta_v,
    }
  }


  fn ray_color(&self, r: &Ray, depth: u32, world: &World) -> Color {
    if depth == 0 {
      return Color::new(0.0, 0.0, 0.0);
    }

    let hit = closest_hit(world, r, &Closed::closed_unchecked(0.001, f32::MAX));
    match hit {
      Some(hit) => {
        let scatter_result = hit.material.scatter(r, &hit);
        match scatter_result {
          Some(scatter_result) => {
            scatter_result.attenuation
              * self.ray_color(&scatter_result.scatter_direction, depth - 1, world)
          }
          None => Color::new(0.0, 0.0, 0.0),
        }
      }
      None => {
        // Sky
        let unit_direction = r.direction.normalize();
        let a = 0.5 * (unit_direction.y + 1.0);
        Color::lerp(Color::new(1.0, 1.0, 1.0), Color::new(0.5, 0.7, 1.0), a)
      }
    }
  }


  fn render(&self, world: &World, out_path: &str) {
    let mut image = RgbImage::new(self.image_width, self.image_height);

    for r in (0..image.height()).progress() {
      for c in 0..image.width() {
        let mut pixel_color = Color::new(0.0, 0.0, 0.0);
        for _ in 0..self.samples_per_pixel {
          let ray = self.get_ray(r, c);
          pixel_color += self.ray_color(&ray, self.max_depth, world);
        }
        pixel_color /= self.samples_per_pixel as f32;
        write_color(&pixel_color, &mut image, r, c);
      }
    }

    image.save(out_path).expect("Failed to save image");
  }


  fn get_ray(&self, r: u32, c: u32) -> Ray {
    let offset = -0.5 + 0.5 * Vec2::new(rand::random(), rand::random());
    let pixel_sample = self.pixel00_loc
      + ((c as f32 + offset.x) * self.pixel_delta_u)
      + ((r as f32 + offset.y) * self.pixel_delta_v);
    let ray_direction = pixel_sample - self.center;
    Ray::new(self.center, ray_direction)
  }
}


fn closest_hit<'a>(world: &'a World, r: &Ray, ray_t: &Closed<f32>) -> Option<Hit<'a>> {
  let mut closest_hit: Option<Hit> = None;

  for object in world {
    let closest_t = closest_hit.as_ref().map_or(ray_t.right.0, |h| h.t);
    let hit = object.hit(r, &Closed::closed_unchecked(ray_t.left.0, closest_t));
    if hit.is_some() {
      closest_hit = hit;
    }
  }

  closest_hit
}


fn write_color(color: &Color, image: &mut RgbImage, r: u32, c: u32) {
  image.put_pixel(
    c,
    r,
    Rgb([
      (linear_to_gamma(color.x) * 255.999) as u8,
      (linear_to_gamma(color.y) * 255.999) as u8,
      (linear_to_gamma(color.z) * 255.999) as u8,
    ]),
  );
}


pub fn render(out_path: &str) {
  let mat_ground = LambertianMaterial {
    albedo: Color::new(0.8, 0.8, 0.0),
  };
  let mat_center = LambertianMaterial {
    albedo: Color::new(0.1, 0.2, 0.5),
  };
  let mat_left = MetalMaterial {
    albedo: Color::new(0.8, 0.8, 0.8),
  };
  let mat_right = MetalMaterial {
    albedo: Color::new(0.8, 0.6, 0.2),
  };

  let world: World = vec![
    Box::new(Sphere::new(
      Point3::new(0.0, 0.0, -1.2),
      0.5,
      Box::new(mat_center),
    )),
    Box::new(Sphere::new(
      Point3::new(0.0, -100.5, -1.0),
      100.0,
      Box::new(mat_ground),
    )),
    Box::new(Sphere::new(
      Point3::new(-1.0, -0.0, -1.0),
      0.5,
      Box::new(mat_left),
    )),
    Box::new(Sphere::new(
      Point3::new(1.0, -0.0, -1.0),
      0.5,
      Box::new(mat_right),
    )),
  ];

  let camera = Camera::new(16.0 / 9.0, 400, 16, 10);

  camera.render(&world, out_path);
}


#[cfg(test)]
mod tests {}
