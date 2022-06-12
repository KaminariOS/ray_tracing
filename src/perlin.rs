use na::{Point3, UnitVector3, Vector3};
use crate::rand_gen::{get_rand_usize_range, get_rand_vec3_range};
use crate::types::Color;

const POINT_COUNT: usize = 256;
type Perm = [usize; POINT_COUNT];

pub struct Perlin {
    rand_float: [UnitVector3<f32>; POINT_COUNT],
    perm: [Perm; 3],
}

impl Perlin {
    pub fn new() -> Self {
        let rand_float = [0; POINT_COUNT].map(|_|
            UnitVector3::new_normalize(get_rand_vec3_range(-1., 1.))
        );
        Self {
            rand_float,
            perm: [Self::perlin_generate_perm(), Self::perlin_generate_perm(), Self::perlin_generate_perm()]
        }
    }

    pub fn noise(&self, p: Point3<f32>) -> f32 {
        let indexes: Vec<_> =  p.iter()
            .map(|&x| x.floor() as i32)
            .collect();
        let mut c = [[[Vector3::y_axis(); 2]; 2]; 2];
        for di in 0..2 {
            for dj  in  0..2 {
                for dk in 0..2 {
                    let ind = [di, dj, dk].iter().zip(indexes.iter())
                        .enumerate()
                        .map(|(i, (&d_val, &num))|
                            self.perm[i][((num + d_val as i32) & 0xff) as usize]
                        )
                        .fold(0, |acc, cur| acc ^ cur);
                    c[di][dj][dk] = self.rand_float[ind];
                }
            }
        }
        Self::trilinear_interp(c, p)
    }
    fn trilinear_interp(c: [[[UnitVector3<f32>; 2]; 2]; 2], p: Point3<f32>) -> f32 {
        let uvw: Vec<_> = p.iter().map(|&x| x - x.floor()).map(|x|
            x * x * (3. - 2. * x)
        ).collect();
        let uvw = Color::from_vec(uvw);
        let mut accm = 0.;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let weight = uvw - Color::from([i as f32, j as f32, k as f32]);
                    accm += c[i][j][k].dot(&weight)
                        * [i, j, k].iter().zip(uvw.iter())
                            .map(|(&ijk, &d)|
                        ijk as f32 * d + (1. - ijk as f32) * (1. - d)
                    ).fold(1., |acc, cur| acc * cur);
                }
            }
        }
        accm
    }

    pub fn turb(&self, p: Point3<f32>, depth: Option<usize>) -> f32 {
       let (accum, _, _) = (0..depth.unwrap_or(7)).fold(
            (0., 1., p), |(accum, weight, temp_p), _|
                (accum + weight * self.noise(temp_p), 0.5 * weight, 2. * temp_p)
        );
        accum.abs()
    }

    fn perlin_generate_perm() -> Perm {
        let mut p = [0; POINT_COUNT];
        for i in 0..p.len() {
            p[i] = i;
        }
        Self::permute(p)
    }
    fn permute(mut p: Perm) -> Perm {
        for i in (1..p.len()).rev() {
            let target = get_rand_usize_range(0, i);
            p.swap(i, target);
        }
        p
    }
}