use crate::geo::Sphere;
use crate::material::Material;
use crate::Hittable;
use na::Vector3;
use std::sync::{Arc, RwLock};

pub type Color = Vector3<f32>;
pub type Shared<T> = Arc<RwLock<T>>;
pub type SharedHittable = Shared<dyn Hittable>;
pub type SharedSphere = Shared<Sphere>;
pub type SharedMaterial = Arc<dyn Material>;

pub fn create_shared_mut<T>(t: T) -> Shared<T> {
    Arc::new(RwLock::new(t))
}
