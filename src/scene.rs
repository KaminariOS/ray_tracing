use crate::geo::{Sphere, AxisAlignedRect, AlignedAxis, Cuboid, RotationY, Translation, ConstantMedium, FlipFace};
use crate::material::{Dielectric, DiffuseLight, Lambertian, Metal};
use crate::rand_gen::{get_rand, get_rand_range, get_rand_vec3_range};
use crate::ray::HittableList;
use crate::texture::{CheckerTexture, ImageTexture, NoiseTexture};
use crate::types::{Color, Shared, SharedHittable, SharedMaterial, SharedSphere};
use na::{Point3, Vector3};
use crate::aabb::BVHNode;


pub fn select_scene(name: &str) -> Scene {
    log::info!("Building scene: {}", name);
    match name {
        "random" => create_random_scene(name),
        "2psp" => two_perlin_spheres(name),
        "earth" => earth(name),
        "simplelight" => simplelight(name),
        "cornell" => cornell_box(name),
        "final" => final_scene(name),
        "smoke" => cornell_smoke(name),
        "2sp" | _ => two_spheres(name),
    }
}

#[allow(dead_code)]
fn create_objs() -> Shared<HittableList> {
    let material_ground = Lambertian::new(CheckerTexture::new(
        [0.2, 0.3, 0.1],
        [0.9, 0.9, 0.9],
    ));
    let material_center = Lambertian::from_color([0.1, 0.2, 0.5]);
    // let material_center = Dielectric::new(1.5);
    // let material_left = Metal::new(Color::from([0.8, 0.8, 0.8]), 0.3);
    let material_left = Dielectric::new(1.5);
    let material_right = Metal::new([0.8, 0.6, 0.2], 0.);
    HittableList::new(
         vec![
            Sphere::new([0., 0., -1.], 0.5, material_center),
            Sphere::new([0., -100.5, -1.], 100., material_ground),
            Sphere::new([-1., 0., -1.], 0.5, material_left.clone()),
            Sphere::new([-1., 0., -1.], -0.45, material_left),
            Sphere::new([1., 0., -1.], 0.5, material_right),
        ],  None)
}

fn create_random_scene(name: &str) -> Scene {
    let num = 11;
    let mut objects: Vec<_> = (-num..num)
        .map(|a| (-num..num).filter_map(move |b| create_random_sphere(a, b)))
        .flatten()
        .map(|x| x as SharedHittable)
        .collect();

    let material_ground = Lambertian::new(CheckerTexture::new(
        [0.2, 0.3, 0.1],
        [0.9; 3],
    ));
    let material1 = Dielectric::new(1.5);
    let material2 = Lambertian::from_color([0.4, 0.2, 0.1]);
    let material3 = Metal::new([0.7, 0.6, 0.5], 0.);
    let vec: Vec<SharedHittable> = vec![
        Sphere::new([0., -1000., 0.], 1000., material_ground),
        Sphere::new([-4., 1., 0.], 1., material2),
        Sphere::new([4., 1., 0.], 1., material3),
    ];
    objects.extend(vec);
    let lights: Vec<SharedHittable> = vec![
        Sphere::new([0., 1., 0.], 1., material1),
    ];
    Scene::new(lights,
    objects,
         Color::from([0.7, 0.8, 1.]),
        name)
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
            Lambertian::from_color(albedo.into())
        } else if mat < 0.95 {
            let albedo = get_rand_vec3_range(0.5, 1.);
            let fuzz = get_rand_range(0., 0.5);
            Metal::new(albedo.into(), fuzz)
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


fn two_spheres(name: &str) -> Scene  {
    let checker = CheckerTexture::new([0.2, 0.3, 0.1], [0.9; 3]);
    let mat = Lambertian::new(checker);
    Scene::new(vec![],
    vec![
        Sphere::new([0., -10., 0.], 10., mat.clone()),
        Sphere::new([0., 10., 0.], 10., mat),
    ], Color::from([0.7, 0.8, 1.]), name)
}

fn two_perlin_spheres(name: &str) -> Scene {
    let pertex = NoiseTexture::new(4.);
    Scene::new(
        vec![ ],
        vec![
            Sphere::new([0., -1000., 0.], 1000., Lambertian::new(pertex.clone())),
            Sphere::new([0., 2., 0.], 2., Lambertian::new(pertex))
        ],
        Color::from([0.7, 0.8, 1.]), name)
}

fn earth(name: &str) -> Scene {
   let earth_texture = ImageTexture::new("earthmap.jpg");
    let earth_surface = Lambertian::new(earth_texture);
   Scene::new(vec![], vec![Sphere::new_with_label([0.; 3], 2., earth_surface, Some(name.into()))], Color::from([0.7, 0.8, 1.]), name)
}

fn simplelight(name: &str) -> Scene {
    let pertex = NoiseTexture::new(4.);
    let diff_light = DiffuseLight::from_color([4.; 3]);
    Scene::new(
        vec![],
        vec![
            Sphere::new([0., -1000., 0.], 1000., Lambertian::new(pertex.clone())),
            Sphere::new([0., 2., 0.], 2., Lambertian::new(pertex)),
            AxisAlignedRect::new(diff_light,-4., [3., 1.], [5., 3.], AlignedAxis::XY)
        ],
        Color::zeros(),
        name
    )
}

fn cornell_box(label: &str) -> Scene {
    let red = Lambertian::from_color([0.65, 0.05, 0.05]);
    let white = Lambertian::from_color([0.73; 3]);
    let green = Lambertian::from_color([0.12, 0.45, 0.15]);
    let light = DiffuseLight::from_color([7.; 3]);
    let length = 555.;
    let square = [length; 2];
    let glass = Dielectric::new(1.5);
    let lights: Vec<SharedHittable> = vec![
        FlipFace::new(AxisAlignedRect::new(light, length - 1., [213., 227.], [343., 332.], AlignedAxis::XZ)),
        Sphere::new([190., 90., 190.], 90., glass)
    ];
    // let aluminum = Metal::new([0.8, 0.85, 0.88], 0.);
    let objects: Vec<SharedHittable> = vec![
        AxisAlignedRect::new(green, length, [0., 0.], square, AlignedAxis::YZ),
        AxisAlignedRect::new(red, 0., [0., 0.], square, AlignedAxis::YZ),
        AxisAlignedRect::new(white.clone(), 0., [0., 0.], square, AlignedAxis::XZ),
        AxisAlignedRect::new(white.clone(), length, [0., 0.], square, AlignedAxis::XZ),

        AxisAlignedRect::new(white.clone(), length, [0., 0.], square, AlignedAxis::XY),
        {
           let cuboid =  Cuboid::new([0.; 3], [165., 330., 165.], white.clone());
            let cuboid = RotationY::new(cuboid, 15.);
            Translation::new(cuboid, [265., 0., 295.])
        },
        // {
        //     let cuboid = Cuboid::new([0.; 3], [165.; 3], white.clone());
        //     let cuboid = RotationY::new(cuboid, -18.);
        //     Translation::new(cuboid, [130., 0., 65.])
        // }
    ];
    Scene::new(lights, objects, Color::zeros(), label)
}

fn cornell_smoke(label: &str) -> Scene {
    let red = Lambertian::from_color([0.65, 0.05, 0.05]);
    let white = Lambertian::from_color([0.73; 3]);
    let green = Lambertian::from_color([0.12, 0.45, 0.15]);
    let light = DiffuseLight::from_color([15.; 3]);
    let length = 555.;
    let square = [length; 2];
    let corner = [0.; 2];
    let lights = vec![
        FlipFace::new(
        AxisAlignedRect::new(light, length - 1., [113., 127.], [443., 432.], AlignedAxis::XZ)
        ) as SharedHittable];
    let objects: Vec<SharedHittable> = vec![
        AxisAlignedRect::new(green, length, [0., 0.], square, AlignedAxis::YZ),
        AxisAlignedRect::new(red, 0., [0., 0.], square, AlignedAxis::YZ),

        AxisAlignedRect::new(white.clone(), 0., corner, square, AlignedAxis::XZ),
        AxisAlignedRect::new(white.clone(), length, corner, square, AlignedAxis::XZ),
        AxisAlignedRect::new(white.clone(), length, corner, square, AlignedAxis::XY),
        {
            let cuboid =  Cuboid::new([0.; 3], [165., 330., 165.], white.clone());
            let cuboid = RotationY::new(cuboid, 15.);
            let cuboid = Translation::new(cuboid, [265., 0., 295.]);
            ConstantMedium::new_c(cuboid, 0.01, [0.; 3])
        },
        {
            let cuboid = Cuboid::new([0.; 3], [165.; 3], white.clone());
            let cuboid = RotationY::new(cuboid, -18.);
            let cuboid = Translation::new(cuboid, [130., 0., 65.]);

            ConstantMedium::new_c(cuboid, 0.01, [1.; 3])
        }
    ];
    Scene::new(
        lights,
        objects,
        Color::zeros(),
        label
    )
}

fn final_scene(label: &str) -> Scene {
    let ground = Lambertian::from_color([0.48, 0.83, 0.53]);
    let boxes_per_side = 20usize;
    let w = 100.;
    let boxes: Vec<_> = (0..boxes_per_side).map(|i|
        (0..boxes_per_side).map(|j|
            {
                let xyz0 = [-1000. + i as f32 * w, 0., -1000. + j as f32 * w];
                let xyz1 = [xyz0[0] + w, get_rand_range(1., 101.), xyz0[2] + w];
                Cuboid::new(xyz0, xyz1, ground.clone()) as SharedHittable
            }
        ).collect::<Vec<_>>()
    ).flatten().collect();
    let boxes = BVHNode::new(&boxes, 0., 1., None);

    let light = DiffuseLight::from_color([7.; 3]);
    let xz = FlipFace::new(AxisAlignedRect::new(light, 554., [123., 147.], [423., 412.], AlignedAxis::XZ));

    let center1 = Point3::from([400., 400., 200.]);
    let center2 = center1 + Vector3::x() * 30.;
    let moving_sphere_material = Lambertian::from_color([0.7, 0.3, 0.1]);
    let moving_sphere = Sphere::new_moving(center1.into(), center2.into(), 0., 1., 50., moving_sphere_material);

    let dielectric_sphere = Sphere::new([260., 150., 45.], 50., Dielectric::new(1.5));
    let metal_sphere = Sphere::new([0., 150., 145.], 50., Metal::new([0.8, 0.8, 0.9], 1.));

    let boundary = Sphere::new([360., 150., 145.], 70., Dielectric::new(1.5));
    let dielectric_medium = ConstantMedium::new_c(boundary.clone(), 0.2, [0.2, 0.4, 0.9]);
    let boundary1 = Sphere::new([0.; 3], 500., Dielectric::new(1.5));
    let dielectric_medium2 = ConstantMedium::new_c(boundary1, 0.00001, [1.; 3]);

    let earth = Sphere::new([400., 200., 400.], 100., Lambertian::new(ImageTexture::new("earthmap.jpg")));
    let pertext = NoiseTexture::new(0.1);
    let pertext = Sphere::new([220., 280., 300.], 80., Lambertian::new(pertext));

    let white = Lambertian::from_color([0.73; 3]);
    let boxes2: Vec<_> = (0..1000).map(|_| Sphere::new(get_rand_vec3_range(0., 165.).into(), 10., white.clone()) as SharedHittable).collect();
    let boxes2 = BVHNode::new(&boxes2, 0., 1., None);
    let boxes2 = RotationY::new(boxes2, 15.);
    let boxes2 = Translation::new(boxes2, [-100., 270., 395.]);
    let lights: Vec<SharedHittable> =vec![xz,
                                        dielectric_sphere
    ];
    let world: Vec<SharedHittable> = vec![
        boxes, moving_sphere, metal_sphere, dielectric_medium,
        dielectric_medium2,
        earth, pertext, boxes2
    ];
    Scene::new(
        lights,
        world,
        Color::zeros(), label)
}

pub struct Scene {
    pub lights: Shared<HittableList>,
    pub world: SharedHittable,
    pub background: Color,
    pub label: String
}

impl Scene {
    pub fn new(lights: Vec<SharedHittable>, mut world: Vec<SharedHittable>, background: Color, label: &str) -> Self {
        let lights = HittableList::new(lights, None);
        world.push(lights.clone());
        Self {
            lights,
            world: HittableList::new(world, Some(label.into())),
            background,
            label: label.into()
        }
    }
}