use crate::aabb::{AxisAlignedBoundingBox, BVHNode};
use crate::material::Lambertian;
use crate::types::{create_shared_mut, Shared, SharedHittable, SharedMaterial};
use na::{Point3, UnitVector3, Vector3};
use crate::rand_gen::get_rand_usize_range;

pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: UnitVector3<f32>,
    pub time: f32,
}

impl Ray {
    pub fn at(&self, t: f32) -> Point3<f32> {
        self.origin + t * self.direction.into_inner()
    }
    pub fn new(origin: Point3<f32>, direction: UnitVector3<f32>, time: f32) -> Self {
        // assert_ne!(direction.norm_squared(), 0.);
        Self {
            origin,
            direction,
            time,
        }
    }
}
impl Default for Ray {
    fn default() -> Self {
        Self::new(Point3::origin(), Vector3::y_axis(), 0.)
    }
}

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AxisAlignedBoundingBox>;
    fn get_label(&self) -> Option<&String> {
        None
    }
    fn pdf_val(&self, _origin: Point3<f32>, _v: UnitVector3<f32>) -> f32 {
        0.
    }
    fn random(&self, _origin: Point3<f32>) -> UnitVector3<f32> {
        Vector3::x_axis()
    }
    fn get_one(&self) -> Option<SharedHittable> {
        None
    }
}


pub struct HitRecord {
    pub(crate) point: Point3<f32>,
    pub(crate) normal: UnitVector3<f32>,
    pub(crate) t: f32,
    pub uv: [f32; 2],
    pub front_face: bool,
    pub material: SharedMaterial,
}

impl Default for HitRecord {
    fn default() -> Self {
        Self {
            point: Point3::origin(),
            normal: Vector3::y_axis(),
            t: f32::MAX,
            uv: [0.; 2],
            front_face: false,
            material: Lambertian::from_color([0.8, 0.8, 0.]),
        }
    }
}

impl HitRecord {
    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: UnitVector3<f32>) {
        self.front_face = ray.direction.dot(&outward_normal) < 0.;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

pub struct HittableList {
    pub objects: Vec<SharedHittable>,
    label: Option<String>
}

impl HittableList {
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.objects.clear()
    }
    #[allow(dead_code)]
    pub fn add(&mut self, object: SharedHittable) {
        self.objects.push(object)
    }

    pub fn new(objects: Vec<SharedHittable>, label: Option<String>) -> Shared<Self> {
        create_shared_mut(Self { objects, label})
    }

    pub fn new_bvh(objects: Vec<SharedHittable>, time0: f32, time1: f32, label: Option<String>) -> SharedHittable {
        if option_env!("BVH")
            .unwrap_or("true")
            .parse::<bool>()
            .unwrap()
        {
            log::info!("Building BVH for {} objects", objects.len());
            BVHNode::new(&objects, time0, time1, label)
        } else {
            Self::new(objects, label)
        }
    }
}

impl Default for HittableList {
    fn default() -> Self {
        Self { objects: vec![], label: None }
    }
}

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut hit_temp = None;
        let mut closest_so_far = t_max;
        for object in &self.objects {
            if let Some(new_hit) = object.read().unwrap().hit(ray, t_min, closest_so_far) {
                closest_so_far = new_hit.t;
                hit_temp = Some(new_hit);
            }
        }
        hit_temp
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AxisAlignedBoundingBox> {
        self.objects.iter().fold(None, |acc, cur| {
            let bbox = cur.read().unwrap().bounding_box(time0, time1);
            AxisAlignedBoundingBox::surrounding_box(acc, bbox)
        })
    }

    fn get_label(&self) -> Option<&String> {
        self.label.as_ref()
    }
    // fn get_one(&self) -> Option<SharedHittable> {
    //     Some(self.objects[get_rand_usize_range(0, self.objects.len())].clone())
    // }

    fn pdf_val(&self, origin: Point3<f32>, v: UnitVector3<f32>) -> f32 {
        self.objects.iter().map(|x| x.read().unwrap().pdf_val(origin, v)).sum::<f32>() / self.objects.len() as f32
    }
    fn random(&self, origin: Point3<f32>) -> UnitVector3<f32> {
        self.objects[get_rand_usize_range(0, self.objects.len())].read().unwrap().random(origin)
    }
}
