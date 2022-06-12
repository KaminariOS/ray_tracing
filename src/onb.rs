use na::{Rotation3, Unit, UnitVector3, Matrix3, Vector3};

pub struct ONB {
    rotation: Rotation3<f32>,
}

impl ONB {
    pub fn matrix(&self) -> &Matrix3<f32> {
        self.rotation.matrix()
    }
    pub fn get_base(&self, i: usize) -> UnitVector3<f32> {
        let vec = Vector3::from(self.matrix().fixed_columns(i));
         Unit::new_unchecked(vec)
    }
    #[allow(dead_code)]
    pub fn u(&self) -> UnitVector3<f32> {
        self.get_base(0)
    }

    #[allow(dead_code)]
    pub fn v(&self) -> UnitVector3<f32> {
        self.get_base(1)
    }

    pub fn w(&self) -> UnitVector3<f32> {
        self.get_base(2)
    }

    #[allow(dead_code)]
    pub fn local(&self, coords: [f32; 3]) -> Vector3<f32> {
        self.rotation * Vector3::from(coords)
    }

    pub fn local_dir(&self, coords: UnitVector3<f32>) -> UnitVector3<f32> {
        self.rotation * coords
    }

    pub fn build_from_w(w: UnitVector3<f32>) -> Self {
        let x_axis = Vector3::x_axis();
        let a = if w.dot(&x_axis).abs() > 0.9 {
            Vector3::y_axis()
        } else {x_axis};
        let v = w.cross(&a);
        let u = w.cross(&v);
        let rotation = Rotation3::from_basis_unchecked(&[u, v, w.into_inner()]);
        Self {
            rotation,
        }
    }
}
