use crate::geo::Sphere;
use crate::material::{Dielectric, Lambertian, Metal};
use crate::rand_gen::{get_rand, get_rand_range, get_rand_vec3_range};
use crate::ray::HittableList;
use crate::texture::{CheckerTexture, ImageTexture, NoiseTexture};
use crate::types::{Color, Shared, SharedHittable, SharedMaterial, SharedSphere};
use na::{Point3, Vector3};

#[allow(dead_code)]
fn create_objs() -> HittableList {
    let material_ground = Lambertian::new(CheckerTexture::new(
        Color::from([0.2, 0.3, 0.1]),
        Color::from([0.9, 0.9, 0.9]),
    ));
    let material_center = Lambertian::from_color(Color::from([0.1, 0.2, 0.5]));
    // let material_center = Dielectric::new(1.5);
    // let material_left = Metal::new(Color::from([0.8, 0.8, 0.8]), 0.3);
    let material_left = Dielectric::new(1.5);
    let material_right = Metal::new(Color::from([0.8, 0.6, 0.2]), 0.);
    HittableList {
        objects: vec![
            Sphere::new([0., 0., -1.], 0.5, material_center),
            Sphere::new([0., -100.5, -1.], 100., material_ground),
            Sphere::new([-1., 0., -1.], 0.5, material_left.clone()),
            Sphere::new([-1., 0., -1.], -0.45, material_left),
            Sphere::new([1., 0., -1.], 0.5, material_right),
        ],
    }
}

fn create_random_scene() -> SharedHittable {
    let num = 11;
    let mut objects: Vec<_> = (-num..num)
        .map(|a| (-num..num).filter_map(move |b| create_random_sphere(a, b)))
        .flatten()
        .map(|x| x as SharedHittable)
        .collect();

    let material_ground = Lambertian::new(CheckerTexture::new(
        Color::from([0.2, 0.3, 0.1]),
        Color::repeat(0.9),
    ));
    let material1 = Dielectric::new(1.5);
    let material2 = Lambertian::from_color(Color::from([0.4, 0.2, 0.1]));
    let material3 = Metal::new(Color::from([0.7, 0.6, 0.5]), 0.);
    let vec: Vec<SharedHittable> = vec![
        Sphere::new([0., -1000., 0.], 1000., material_ground),
        Sphere::new([0., 1., 0.], 1., material1),
        Sphere::new([-4., 1., 0.], 1., material2),
        Sphere::new([4., 1., 0.], 1., material3),
    ];
    objects.extend(vec);
    HittableList::new_bvh(objects, 0., 1.)
}

fn create_random_sphere(a: i32, b: i32) -> Option<SharedSphere> {
    let a = a as f32;
    let b = b as f32;
    let mat = get_rand();
    let center = Point3::from([a + 0.9 * get_rand(), 0.2, b + 0.9 * get_rand()]);
    let mut moving = false;
    if (center - Point3::from([4., 0.2, 0.])).norm() > 0.9 {
        let material: SharedMaterial = if mat < 0.8 {
            let albedo = get_rand_vec3_range(0., 1.).component_mul(&get_rand_vec3_range(0., 1.));
            moving = true;
            Lambertian::from_color(albedo)
        } else if mat < 0.95 {
            let albedo = get_rand_vec3_range(0.5, 1.);
            let fuzz = get_rand_range(0., 0.5);
            Metal::new(albedo, fuzz)
        } else {
            Dielectric::new(1.5)
        };
        Some(if moving {
            let center2 = center + Vector3::from([0., get_rand_range(0., 0.5), 0.]);
            Sphere::new_moving(center.into(), center2.into(), 0., 1., 0.2, material)
        } else {
            Sphere::new(center.into(), 0.2, material)
        })
    } else {
        None
    }
}

pub fn select_scene(name: &str) -> SharedHittable {
    match name {
        "random" => create_random_scene(),
        "2psp" => two_perlin_spheres(),
        "earth" => earth(),
        "2sp" | _ => two_spheres(),
    }
}

fn two_spheres() -> Shared<HittableList>  {
    let checker = CheckerTexture::new(Color::from([0.2, 0.3, 0.1]), Color::repeat(0.9));
    let mat = Lambertian::new(checker);
    HittableList::new(vec![
        Sphere::new([0., -10., 0.], 10., mat.clone()),
        Sphere::new([0., 10., 0.], 10., mat),
    ])
}

fn two_perlin_spheres() -> Shared<HittableList> {
    let pertex = NoiseTexture::new(4.);
    HittableList::new(
        vec![
            Sphere::new([0., -1000., 0.], 1000., Lambertian::new(pertex.clone())),
            Sphere::new([0., 2., 0.], 2., Lambertian::new(pertex))
        ]
    )
}

fn earth() -> SharedSphere {
   let earth_texture = ImageTexture::new("earthmap.jpg");
    let earth_surface = Lambertian::new(earth_texture);
    Sphere::new([0.; 3], 2., earth_surface)
}