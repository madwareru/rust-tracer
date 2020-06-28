use{
    crate::{
        shape::Shape,
        ray::{HitTestable, HitInfo, Ray}
    }
};

pub struct World<'a> {
    pub shapes: &'a[Shape]
}

impl<'a> HitTestable for World<'a> {
    fn hit_test(&self, ray: &Ray) -> Option<HitInfo> {
        let mut nearest_hit: Option<HitInfo> = None;
        for shape in self.shapes {
            let hit = ray.hit_test(shape);
            if let Some(hit_i) = hit {
                match nearest_hit {
                    None => nearest_hit = hit,
                    Some(hit_info) if hit_info.t > hit_i.t => nearest_hit = hit,
                    _ => {}
                };
            }
        }
        nearest_hit
    }
}