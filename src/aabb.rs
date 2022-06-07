use na::Point3;
use crate::{Hittable, Ray};
use itertools::izip;
use crate::rand_gen::get_rand_int_range;
use crate::ray::HitRecord;
use crate::types::{create_shared_mut, Shared, SharedHittable};

#[derive(Clone, Copy)]
pub struct AxisAlignedBoundingBox {
    minimum: Point3<f32> ,
    maximum: Point3<f32>
}

impl AxisAlignedBoundingBox {
    pub fn new(minimum: Point3<f32>, maximum: Point3<f32>) -> Self {
        AxisAlignedBoundingBox {
            minimum,
            maximum
        }
    }
    pub fn surrounding_box(bbox0: Option<Self>, bbox1: Option<Self>) -> Option<Self> {
        match (bbox0, bbox1) {
            (Some(b0), Some(b1)) => {
                let mut small = [0.; 3];
                b0.minimum.iter().zip(b1.minimum.iter()).enumerate().for_each(|(i, (min0, min1))| small[i] = min0.min(*min1));
                let mut big = [0.; 3];
                b0.maximum.iter().zip(b1.maximum.iter()).enumerate().for_each(|(i, (max0, max1))| big[i] = max0.max(*max1));
                Some(AxisAlignedBoundingBox::new(Point3::from(small), Point3::from(big)))
            }
            (Some(b), None) | (None, Some(b)) => Some(b),
            _ => None
        }
    }

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> bool {
        izip!(ray.origin.iter(), ray.direction.iter(), self.minimum.iter(), self.maximum.iter())
            .all(|(&a, &b, &min, &max)| {
                let inv_b = 1. / b;
                let mut t0 = (min - a) * inv_b;
                let mut t1 = (max - a) * inv_b;
                if inv_b < 0. {
                    std::mem::swap(&mut t0, &mut t1);
                }
                let (t_min, t_max) = (t_min.max(t0), t_max.min(t1));
                t_max > t_min
            })
    }
}

pub struct BVHNode {
    left: SharedHittable,
    right: SharedHittable,
    bbox: AxisAlignedBoundingBox
}

impl BVHNode {
    pub fn new(objects: &[SharedHittable], time0: f32, time1: f32) -> Shared<Self> {
        let mut objects: Vec<_> = objects.iter().map(|x| x.clone()).collect();
        let axis = get_rand_int_range(0, 3) as usize;
        let obj_span = objects.len();
        let (left, right) = if obj_span == 1 {
            (objects[0].clone(), objects[0].clone())
        } else if obj_span == 2 {
            if Self::box_compare(&objects[0], axis) < Self::box_compare(&objects[1], axis) {
                (objects[0].clone(), objects[1].clone())
            } else {
                (objects[1].clone(), objects[0].clone())
            }
        } else {
            objects.sort_by(|x, y| Self::box_compare(x, axis).partial_cmp(&Self::box_compare(y, axis)).expect("NaN"));
            let mid = obj_span / 2;
            (Self::new(&objects[0..mid], time0, time1) as SharedHittable, Self::new(&objects[mid..obj_span], time0, time1) as SharedHittable)
        };
        let left_box = left.read().unwrap().bounding_box(time0, time1);
        let right_box = right.read().unwrap().bounding_box(time0, time1);
        let bbox = AxisAlignedBoundingBox::surrounding_box(left_box, right_box).expect("No bounding_box");
        create_shared_mut(Self{
            left,
            right,
            bbox
        })
    }

    fn box_compare(a: &SharedHittable, axis: usize) -> f32 {
        a.read().unwrap().bounding_box(0., 0.).expect("No bounding_box!").minimum[axis]
    }
}

impl Hittable for BVHNode {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        if !self.bbox.hit(ray, t_min, t_max) {
            return None
        }
        let hit_left = self.left.read().unwrap().hit(ray, t_min, t_max);
        if let Some(hit_rec) = &hit_left {
            self.right.read().unwrap().hit(ray, t_min, hit_rec.t).or(hit_left)
        } else {
            self.right.read().unwrap().hit(ray, t_min, t_max)
        }
    }

    fn bounding_box(&self, _time0: f32, _time1: f32) -> Option<AxisAlignedBoundingBox> {
        Some(self.bbox)
    }
}