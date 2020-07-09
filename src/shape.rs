use{
    cgmath::{
        Vector2,
        Vector3,
        InnerSpace,
        Quaternion,
        vec2
    },
    crate::ray::{HitTestable, HitInfo, Ray},
    crate::material::Material
};

#[derive(Copy, Clone)]
pub struct VertexDescription {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub uv: Vector2<f32>
}

#[derive(Copy, Clone)]
pub struct MeshDescription<'a> {
    pub vertices: &'a [VertexDescription],
    pub indices: &'a [usize],
    pub triangle_count: usize
}

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
        radius: f32,
        rotation: Quaternion<f32>,
        material: Material<'a>
    },
    Cube {
        center: Vector3<f32>,
        sizes: Vector3<f32>,
        rotation: Quaternion<f32>,
        material: Material<'a>
    },
    TriangleMesh {
        center: Vector3<f32>,
        mesh: MeshDescription<'a>,
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
             // sign (we are watching to the "back" of a plane)
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
            Shape::Disk { center, radius, rotation, material } => {
                let i = (rotation * Vector3::unit_x()).normalize();
                let j = (rotation * Vector3::unit_y()).normalize();
                let k = (rotation * Vector3::unit_z()).normalize();
                match test_ray_plane_intersection(&center, &j, ray, &material) {
                    None => None,
                    Some(hit_info) => {
                        let pc = hit_info.p - center;
                        let r = pc.dot(pc) / (radius*radius);
                        if r > 1.0 {
                            None
                        } else {
                            let uv = vec2(
                                k.dot(pc).atan2(i.dot(pc)).to_degrees() / 180.0,
                                r
                            );
                            Some(HitInfo{uv: Some(uv), ..hit_info})
                        }
                    }
                }
            },
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
                            let old_t = hit_info_maybe.map_or(100000.0, |i : HitInfo| i.t);
                            if hit_info.t > old_t {
                                continue;
                            }

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
                                hit_info_maybe = Some(hit_info)
                            }
                        },
                    }
                }
                hit_info_maybe
            }
            Shape::TriangleMesh { center, mesh, rotation, material } => {
                let mut offset = 0;
                let mut hit_info_maybe = None;

                for _ in 0..mesh.triangle_count {
                    if offset + 3 > mesh.indices.len() {
                        break;
                    }

                    let next_vertices = mesh
                        .indices[offset..offset+3]
                        .iter()
                        .map(|&ix| VertexDescription{
                            position: rotation * mesh.vertices[ix].position + center,
                            normal: rotation * mesh.vertices[ix].normal,
                            ..mesh.vertices[ix]
                        })
                        .collect::<Vec<_>>();

                    let v0 = next_vertices[1].position - next_vertices[0].position;
                    let v1 = next_vertices[2].position - next_vertices[0].position;
                    let n = v0.cross(v1);
                    let whole_area = n.magnitude();
                    let n = n.normalize();
                    let neg_n = -n;

                    if let Some(hit_info) = test_ray_plane_intersection(
                        &(next_vertices[0].position),
                        &n,
                        &ray,
                        &material
                    ) {
                        let old_t = hit_info_maybe.map_or(100000.0, |i : HitInfo| i.t);
                        if hit_info.t <= old_t {
                            let p0p = next_vertices[0].position - hit_info.p;
                            let p1p = next_vertices[1].position - hit_info.p;
                            let p2p = next_vertices[2].position - hit_info.p;

                            let p0_area_cross = p1p.cross(p2p);
                            let p1_area_cross = p2p.cross(p0p);
                            let p2_area_cross = p0p.cross(p1p);

                            if  p0_area_cross.dot(hit_info.n) > 0.0 &&
                                p1_area_cross.dot(hit_info.n) > 0.0 &&
                                p2_area_cross.dot(hit_info.n) > 0.0
                            {
                                let (u, v, w) = (
                                    p0_area_cross.magnitude() / whole_area,
                                    p1_area_cross.magnitude() / whole_area,
                                    p2_area_cross.magnitude() / whole_area
                                );

                                let normal =
                                    next_vertices[0].normal * u +
                                    next_vertices[1].normal * v +
                                    next_vertices[2].normal * w;

                                let uv =
                                    next_vertices[0].uv * u +
                                    next_vertices[1].uv * v +
                                    next_vertices[2].uv * w;

                                hit_info_maybe = Some(
                                    HitInfo {
                                        n: normal,
                                        uv: Some(uv),
                                        ..hit_info
                                    }
                                );
                            }
                        }
                    }
                    offset += 3;
                }

                hit_info_maybe
            }
        }
    }
}