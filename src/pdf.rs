use na::{Point3, UnitVector3};
use crate::onb::ONB;
use crate::rand_gen::{get_rand, random_cosine_direction};
use crate::types::SharedHittable;

pub trait PDF {
    fn value(&self, direction: UnitVector3<f32>) -> f32;
    fn generate(&self) -> UnitVector3<f32>;
}

pub struct CosinePDF {
    uvw: ONB
}

impl CosinePDF {
    pub fn new(w: UnitVector3<f32>) -> Box<Self> {
        Box::new(Self {
            uvw: ONB::build_from_w(w)
        })
    }
}

impl PDF for CosinePDF{
    fn value(&self, direction: UnitVector3<f32>) -> f32 {
        let cosine = direction.dot(&self.uvw.w());
        if cosine <= 0. { 0. } else {2. * cosine }
    }

    fn generate(&self) -> UnitVector3<f32> {
        self.uvw.local_dir(random_cosine_direction())
    }
}

pub struct HittablePDF {
    pub(crate) o: Point3<f32>,
    pub(crate) obj: SharedHittable
}

impl HittablePDF {
    pub fn new(o: Point3<f32>, obj: SharedHittable) -> Box<Self> {
        Box::new(Self {
            o,
            obj: {let x = obj.read().unwrap().get_one(); x}.unwrap_or(obj)
        })
    }
}

impl PDF for HittablePDF {
    fn value(&self, direction: UnitVector3<f32>) -> f32 {
        self.obj.read().unwrap().pdf_val(self.o, direction)
    }

    fn generate(&self) -> UnitVector3<f32> {
        self.obj.read().unwrap().random(self.o)
    }
}

pub struct MixPDF {
    pdfs: [Box<dyn PDF>; 2]
}

impl MixPDF {
    pub fn new(p0: Box<dyn PDF>, p1: Box<dyn PDF>) -> Box<Self> {
        Box::new(Self {
            pdfs: [p0, p1]
        })
    }
}


impl PDF for MixPDF {
    fn value(&self, direction: UnitVector3<f32>) -> f32 {
        0.5 * self.pdfs[0].value(direction) + 0.5 * self.pdfs[1].value(direction)
    }

    fn generate(&self) -> UnitVector3<f32> {
        if get_rand() < 0.5 {
            self.pdfs[0].generate()
        } else {
            self.pdfs[1].generate()
        }
    }
}