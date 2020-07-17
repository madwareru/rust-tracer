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
use crate::aabb::AaBb;

#[derive(Copy, Clone)]
pub struct VertexDescription {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub uv: Vector2<f32>
}

pub enum FaceOctTree {
    Node {
        aabb: AaBb,
        children: Vec<FaceOctTree>
    },
    Leaf {
        aabb: AaBb,
        face_indices: Vec<usize>
    }
}

impl FaceOctTree {
    pub fn fill(&mut self, v0: &Vector3<f32>, v1: &Vector3<f32>, v2: &Vector3<f32>, face_id: usize) {
        match self {
            FaceOctTree::Node { aabb, children } => {
                if !(aabb.is_point_inside(v0) || aabb.is_point_inside(v1) || aabb.is_point_inside(v2)) {
                    return;
                }
                for child in children {
                    child.fill(v0, v1, v2, face_id)
                }
            },
            FaceOctTree::Leaf { aabb, face_indices } => {
                if !(aabb.is_point_inside(v0) || aabb.is_point_inside(v1) || aabb.is_point_inside(v2)) {
                    return;
                }
                face_indices.push(face_id)
            },
        }
    }
    pub fn make(initial_aabb: AaBb, depth_levels: u8) -> Self {
        if depth_levels == 0 {
            FaceOctTree::Leaf {
                aabb: initial_aabb,
                face_indices: Vec::new()
            }
        } else {
            let [bb0, bb1, bb2, bb3, bb4, bb5, bb6, bb7] = initial_aabb.slice_octal();
            let next_depth_levels = depth_levels - 1;
            FaceOctTree::Node {
                aabb: initial_aabb,
                children: vec![
                    FaceOctTree::make(bb0, next_depth_levels),
                    FaceOctTree::make(bb1, next_depth_levels),
                    FaceOctTree::make(bb2, next_depth_levels),
                    FaceOctTree::make(bb3, next_depth_levels),
                    FaceOctTree::make(bb4, next_depth_levels),
                    FaceOctTree::make(bb5, next_depth_levels),
                    FaceOctTree::make(bb6, next_depth_levels),
                    FaceOctTree::make(bb7, next_depth_levels)
                ]
            }
        }
    }
    pub fn hit_test<F: FnMut(usize) -> ()>(&self, ray: &Ray, f: &mut F) {
        match self {
            FaceOctTree::Leaf { aabb, face_indices } => {
                if !aabb.is_hit(ray) {
                    return;
                }
                for id in face_indices {
                    f(*id);
                }
            },
            FaceOctTree::Node { aabb, children } => {
                if !aabb.is_hit(ray) {
                    return;
                }
                for child in children {
                    child.hit_test(ray, f);
                }
            },
        }
    }
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
        material: Material<'a>,
        face_oct_tree: Option<&'a FaceOctTree>
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
                        let uv = vec2(z_proj.atan2(x_proj).to_degrees() / 360.0 + 0.5, (y_proj + 1.0) * 0.5);
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
                                k.dot(pc).atan2(i.dot(pc)).to_degrees() / 360.0 + 0.5,
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
            Shape::TriangleMesh { center, mesh, material, face_oct_tree } => {
                let mut hit_info_maybe = None;
                if let Some(face_oct_tree) = *face_oct_tree {
                    face_oct_tree.hit_test(ray, &mut (|face_id| {
                        let offset = face_id * 3;
                        if offset + 3 <= mesh.indices.len() {
                            let(ix0, ix1, ix2) = (
                                mesh.indices[offset],
                                mesh.indices[offset+1],
                                mesh.indices[offset+2]
                            );

                            let vertex_0 = VertexDescription{
                                position: mesh.vertices[ix0].position + center,
                                ..mesh.vertices[ix0]
                            };
                            let vertex_1 = VertexDescription{
                                position: mesh.vertices[ix1].position + center,
                                ..mesh.vertices[ix1]
                            };
                            let vertex_2 = VertexDescription{
                                position: mesh.vertices[ix2].position + center,
                                ..mesh.vertices[ix2]
                            };

                            let v0 = vertex_1.position - vertex_0.position;
                            let v1 = vertex_2.position - vertex_0.position;
                            let n = v0.cross(v1);
                            let whole_area = n.magnitude();
                            let n = n.normalize();

                            if let Some(hit_info) = test_ray_plane_intersection(
                                &(vertex_0.position),
                                &n,
                                &ray,
                                &material
                            ) {
                                let old_t = hit_info_maybe.map_or(100000.0, |i : HitInfo| i.t);
                                if hit_info.t <= old_t {
                                    let p0p = vertex_0.position - hit_info.p;
                                    let p1p = vertex_1.position - hit_info.p;
                                    let p2p = vertex_2.position - hit_info.p;

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
                                            vertex_0.normal * u +
                                            vertex_1.normal * v +
                                            vertex_2.normal * w;

                                        let uv =
                                            vertex_0.uv * u +
                                            vertex_1.uv * v +
                                            vertex_2.uv * w;

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
                        }
                    }));
                }
                else {
                    let mut offset = 0;
                    for _ in 0..mesh.triangle_count {
                        if offset + 3 > mesh.indices.len() {
                            break;
                        }

                        let(ix0, ix1, ix2) = (
                            mesh.indices[offset],
                            mesh.indices[offset+1],
                            mesh.indices[offset+2]
                        );

                        let vertex_0 = VertexDescription{
                            position: mesh.vertices[ix0].position + center,
                            ..mesh.vertices[ix0]
                        };
                        let vertex_1 = VertexDescription{
                            position: mesh.vertices[ix1].position + center,
                            ..mesh.vertices[ix1]
                        };
                        let vertex_2 = VertexDescription{
                            position: mesh.vertices[ix2].position + center,
                            ..mesh.vertices[ix2]
                        };

                        let v0 = vertex_1.position - vertex_0.position;
                        let v1 = vertex_2.position - vertex_0.position;
                        let n = v0.cross(v1);
                        let whole_area = n.magnitude();
                        let n = n.normalize();

                        if let Some(hit_info) = test_ray_plane_intersection(
                            &(vertex_0.position),
                            &n,
                            &ray,
                            &material
                        ) {
                            let old_t = hit_info_maybe.map_or(100000.0, |i : HitInfo| i.t);
                            if hit_info.t <= old_t {
                                let p0p = vertex_0.position - hit_info.p;
                                let p1p = vertex_1.position - hit_info.p;
                                let p2p = vertex_2.position - hit_info.p;

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
                                        vertex_0.normal * u +
                                        vertex_1.normal * v +
                                        vertex_2.normal * w;

                                    let uv =
                                        vertex_0.uv * u +
                                        vertex_1.uv * v +
                                        vertex_2.uv * w;

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
                }
                hit_info_maybe
            }
        }
    }
}

impl<'a> Shape<'a> {
    pub fn extend_with_oct_tree(&'a self, oct_tree: &'a mut FaceOctTree) -> Self {
        if let Shape::TriangleMesh { center, mesh, material, .. } = self {
            let mut offset = 0;
            for face_id in 0..mesh.triangle_count {
                if offset + 3 > mesh.indices.len() {
                    break;
                }
                let ix0 = mesh.indices[offset];
                let ix1 = mesh.indices[offset+1];
                let ix2 = mesh.indices[offset+2];

                let (v0, v1, v2) = (
                    mesh.vertices[ix0].position + center,
                    mesh.vertices[ix1].position + center,
                    mesh.vertices[ix2].position + center
                );

                oct_tree.fill(&v0, &v1, &v2, face_id);
            }

            Shape::TriangleMesh {
                center: *center,
                mesh: *mesh,
                material: *material,
                face_oct_tree: Some(oct_tree)
            }
        } else {
            *self
        }
    }
}