use cfg_if::cfg_if;
use na::Vector3;


#[inline]
pub fn get_rand() -> f32 {
    cfg_if!{
        if #[cfg(feature = "web")] {
            let mut buf = [0; 4];
            getrandom::getrandom(&mut buf).expect("Failed to generate random number");
            unsafe { std::mem::transmute::<[u8; 4], u32>(buf) }.to_be() as f32 / (u32::MAX as f32 + 1.)
        } else {
            rand::random()
        }
    }
}

#[inline]
pub fn get_rand_range(min: f32, max: f32) -> f32 {
    // assert!(min <= max);
    min + (max - min) * get_rand()
}

#[inline]
pub fn get_rand_int_range(min: i32, max: i32) -> i32 {
    get_rand_range(min as f32, max as f32) as i32
}

#[inline]
pub fn get_rand_usize_range(min: usize, max: usize) -> usize {
    get_rand_range(min as f32, max as f32) as usize
}

#[inline]
pub fn get_rand_vec3_range(min: f32, max: f32) -> Vector3<f32> {
    Vector3::from([
        get_rand_range(min, max),
        get_rand_range(min, max),
        get_rand_range(min, max),
    ])
}

#[inline]
pub fn rand_vec3_in_unit_sphere() -> Vector3<f32> {
    loop {
        let p = get_rand_vec3_range(-1., 1.);
        if p.norm_squared() < 1. {
            return p;
        }
    }
}

#[inline]
pub fn rand_vec3_on_unit_sphere() -> Vector3<f32> {
    rand_vec3_in_unit_sphere().normalize()
}

#[allow(dead_code)]
pub fn rand_vec3_in_unit_hemisphere(normal: Vector3<f32>) -> Vector3<f32> {
    let in_unit_sphere = rand_vec3_in_unit_sphere();
    if in_unit_sphere.dot(&normal) > 0. {
        in_unit_sphere
    } else {
        -in_unit_sphere
    }
}

#[inline]
pub fn rand_vec3_in_unit_disk() -> Vector3<f32> {
    loop {
        let p = Vector3::from([get_rand_range(-1., 1.), get_rand_range(-1., 1.), 0.]);
        if p.norm_squared() >= 1. {
            continue;
        }
        return p;
    }
}
