use{
    crate::{
        shape::Shape,
        ray::{HitTestable, HitInfo, Ray}
    }
};
use crate::aabb::AaBb;

pub struct World<'a> {
    shapes: &'a[Shape<'a>],
    bouning_boxes: Vec<Option<AaBb>>
}

impl<'a> World<'a> {
    pub fn construct(shapes: &'a[Shape<'a>]) -> Self {
        let mut aabb_companions = Vec::with_capacity(shapes.len());
        for i in 0..shapes.len() {
            aabb_companions.push(shapes[i].into())
        }
        World {
            shapes,
            bouning_boxes: aabb_companions
        }
    }
}

impl HitTestable for World<'_> {
    fn hit_test(&self, ray: &Ray) -> Option<HitInfo> {
        let mut nearest_hit: Option<HitInfo> = None;
        for i in 0..self.shapes.len() {
            let aabb_is_hit = self.bouning_boxes[i]
                .map(|b| b.is_hit(ray))
                .unwrap_or(true);

            if !aabb_is_hit { continue; }
            let hit = ray.hit_test(&self.shapes[i]);
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