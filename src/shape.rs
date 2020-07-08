use{
    cgmath::{
        Vector3,
        InnerSpace,
        Quaternion,
        vec2
    },
    crate::ray::{HitTestable, HitInfo, Ray},
    crate::material::Material
};

#[derive(Copy, Clone)]
pub enum Shape<'a> {
    Sphere {
        center: Vector3<f32>,
        radius: f32,
        rotation: Quaternion<f32>,
        material: Material<'a>
    },
    Disk {
        center: Vector3<f32>,
        normal: Vector3<f32>,
        radius: f32,
        material: Material<'a>
    },
    Romboid {
        center: Vector3<f32>,
        up: Vector3<f32>,
        right: Vector3<f32>,
        w: f32,
        h: f32,
        material: Material<'a>
    },
    Cube {
        center: Vector3<f32>,
        sizes: Vector3<f32>,
        rotation: Quaternion<f32>,
        material: Material<'a>
    }
}

fn test_ray_plane_intersection<'a>(
    center: &Vector3<f32>,
    normal: &Vector3<f32>,
    ray: &Ray,
    material: &Material<'a>
) -> Option<HitInfo<'a>>
{
    let ray_normal_proj = normal.dot(ray.direction);
    if ray_normal_proj >= 0.0 {
        None // when ray is completely parallel to a plane, there is no intersection
             // there is no intersection in a situation when projection has positive
             // sign (we are watching to the "back" of a disk)
    } else {
        let oc = ray.origin - center;
        let t = -normal.dot(oc) / ray_normal_proj;
        if t <= 0.0001 {
            None
        } else {
            let p = ray.get_point_at(t);
            let n = *normal;
            let material = *material;
            Some(HitInfo{ t, p, n, material, uv: None })
        }
    }
}

impl HitTestable for Shape<'_> {
    fn hit_test(&self, ray: &Ray) -> Option<HitInfo> {
        match self {
            Shape::Sphere { center, radius, rotation, material } => {
                let radius = *radius;
                let material = *material;

                let i = (rotation * Vector3::unit_x()).normalize();
                let j = (rotation * Vector3::unit_y()).normalize();
                let k = (rotation * Vector3::unit_z()).normalize();

                let oc = ray.origin - center;
                let a = ray.direction.dot(ray.direction);
                let b = 2.0 * oc.dot(ray.direction);
                let c = oc.dot(oc) - radius * radius;
                let discriminant = b * b - 4.0 * a * c;
                if discriminant < 0.0 {
                    None
                } else {
                    let t = (-b - discriminant.sqrt()) / (2.0 * a);
                    if t <= 0.0001 {
                        None
                    } else {
                        let p = ray.get_point_at(t);
                        let n = (p - center) / radius;
                        let x_proj = i.dot(n);
                        let y_proj = j.dot(n);
                        let z_proj = k.dot(n);
                        let uv = vec2(z_proj.atan2(x_proj).to_degrees() / 180.0, (y_proj + 1.0) * 0.5);
                        Some(HitInfo{ t, p, n, material, uv: Some(uv) })
                    }
                }
            },
            Shape::Disk { center, normal, radius, material } => {
                match test_ray_plane_intersection(&center, &normal, ray, &material) {
                    None => None,
                    Some(hit_info) => {
                        let pc = hit_info.p - center;
                        if pc.dot(pc) > radius*radius {
                            None
                        } else {
                            Some(hit_info)
                        }
                    }
                }
            },
            Shape::Romboid { center, up, right, w, h, material } => {
                let normal = up.cross(*right).normalize();
                match test_ray_plane_intersection(&center, &normal, ray, &material) {
                    None => None,
                    Some(hit_info) => {
                        let pc = hit_info.p - center;
                        if pc.dot(right.normalize()).abs() > *w ||
                           pc.dot(up.normalize()).abs() > *h {
                            None
                        } else {
                            Some(hit_info)
                        }
                    }
                }
            }
            Shape::Cube { center, sizes, material, rotation } => {
                let center = *center;
                let mut hit_info_maybe = None;
                let half_sizes = sizes / 2.0;

                let i = (rotation * Vector3::unit_x()).normalize();
                let j = (rotation * Vector3::unit_y()).normalize();
                let k = (rotation * Vector3::unit_z()).normalize();

                let Vector3{x: xs, y: ys, z: zs} = half_sizes;

                for (normal, center) in &[
                    ( i.clone(), center + i.clone() * xs),
                    (-i.clone(), center - i.clone() * xs),
                    ( j.clone(), center + j.clone() * ys),
                    (-j.clone(), center - j.clone() * ys),
                    ( k.clone(), center + k.clone() * zs),
                    (-k.clone(), center - k.clone() * zs)
                ] {
                    match test_ray_plane_intersection(&center, &normal, &ray, &material) {
                        None => {},
                        Some(hit_info) => {
                            let diff = hit_info.p - center;
                            let x_project = diff.dot(i.clone());
                            let y_project = diff.dot(j.clone());
                            let z_project = diff.dot(k.clone());

                            let nx_project = normal.dot(i);
                            let ny_project = normal.dot(j);

                            let uv = if nx_project >= 0.999 || nx_project <= -0.999 {
                                vec2(y_project / ys + 1.0, z_project / zs + 1.0) * 0.5
                            } else if ny_project >= 0.999 || ny_project <= -0.999 {
                                vec2(x_project / xs + 1.0, z_project / zs + 1.0) * 0.5
                            } else {
                                vec2(x_project / xs + 1.0, y_project / ys + 1.0) * 0.5
                            };

                            let hit_info = HitInfo{uv: Some(uv), ..hit_info};

                            if !(
                                x_project.abs() > xs ||
                                y_project.abs() > ys ||
                                z_project.abs() > zs
                            ) {
                                match hit_info_maybe {
                                    None => hit_info_maybe = Some(hit_info),
                                    Some(old_hit_info) if old_hit_info.t > hit_info.t =>{
                                        hit_info_maybe = Some(hit_info)
                                    },
                                    _ => {}
                                }
                            }
                        },
                    }
                };
                hit_info_maybe
            }
        }
    }
}