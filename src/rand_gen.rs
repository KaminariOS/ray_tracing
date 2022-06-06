use na::Vector3;

pub fn get_rand() -> f32 {
    let mut buf = [0; 4];
    getrandom::getrandom(&mut buf).expect("Failed to generate random number");
    unsafe {
        std::mem::transmute::<[u8; 4], u32>(buf)
    }.to_be() as f32 / (u32::MAX as f32 + 1.)
}

pub fn get_rand_range(min: f32, max: f32) -> f32 {
    assert!(min <= max);
    min + (max - min) * get_rand()
}

pub fn get_rand_vec3_range(min: f32, max: f32) -> Vector3<f32> {
    Vector3::from([
        get_rand_range(min, max),
        get_rand_range(min, max),
        get_rand_range(min, max)
    ])
}

pub fn rand_vec3_in_unit_sphere() -> Vector3<f32> {
    loop {
        let p = get_rand_vec3_range(-1., 1.);
        if p.norm_squared() < 1. {
            return p
        }
    }
}

pub fn rand_vec3_on_unit_sphere() -> Vector3<f32> {
    rand_vec3_in_unit_sphere().normalize()
}

#[allow(dead_code)]
pub fn rand_vec3_in_unit_hemisphere(normal: Vector3<f32>) -> Vector3<f32> {
    let in_unit_sphere = rand_vec3_in_unit_sphere();
    if in_unit_sphere.dot(&normal) > 0. {
        in_unit_sphere
    } else { -in_unit_sphere }
}

pub fn rand_vec3_in_unit_disk() -> Vector3<f32> {
    loop {
        let p = Vector3::from([get_rand_range(-1., 1.), get_rand_range(-1., 1.), 0.]);
        if p.norm_squared() >= 1. {continue}
        return p
    }
}

