#[macro_use]
extern crate lazy_static;

mod color;
mod picture;
mod ray;
mod shape;
mod world;
mod material;
mod camera;
mod vector_utils;
mod scene;
mod image_loader;
mod bunny;
mod aabb;

use {
    cgmath::{
        Vector3,
        Quaternion,
        Euler,
        Rad,
        vec2,
        vec3,
        Angle,
        VectorSpace,
        InnerSpace
    },
    png::{Decoder, ColorType},
    std::env,
    shape::*,
    world::*,
    material::*,
    image_loader::*,
    scene::*
};

const NUM_SAMPLES: u16 = 256;
const FOCUS_DISTANCE: f32 = 1.6;
const APERTURE: f32 = 0.035;
const MAX_T: f32 = 400.0;

const MOON_MAP_BYTES: &[u8] = include_bytes!("moonmap.png");
const EARTH_MAP_BYTES: &[u8] = include_bytes!("earthmap.png");

const LIGHT_GRAY_MAT: Material = Material {
    albedo: Albedo::Constant(vec3(0.8, 0.8, 0.8)),
    details: MaterialDetails::Metallic {roughness: 0.2},
    emittance: 0.0
};

const DARK_GRAY_MAT: Material = Material {
    albedo: Albedo::Constant(vec3(0.1, 0.1, 0.1)),
    details: MaterialDetails::Metallic {roughness: 0.6},
    emittance: 0.0
};

const BLUE_MAT: Material = Material {
    albedo: Albedo::Constant(vec3(0.1, 0.1, 1.0)),
    details: MaterialDetails::Dielectric { ref_idx: 1.1, roughness: 0.1 },
    emittance: 0.0
};

const LIGHT_GRAY_MAT_LAMBERT: Material = Material {
    albedo: Albedo::Constant(vec3(0.8, 0.8, 0.8)),
    details: MaterialDetails::Lambertian,
    emittance: 0.0
};

const WHITE_BULB_MAT: Material = Material {
    albedo: Albedo::Constant(vec3(3.0, 3.0, 3.0)),
    details: MaterialDetails::Lambertian,
    emittance: 1.0
};

const DIELECTRIC_MAT: Material = Material {
    albedo: Albedo::Constant(vec3(0.0, 1.0, 0.4)),
    details: MaterialDetails::Dielectric {
        ref_idx: 1.5,
        roughness: 0.0
    },
    emittance: 0.0
};

const RED_MIRROR_MAT: Material = Material {
    albedo: Albedo::Constant(vec3(1.0, 0.0, 0.0)),
    details: MaterialDetails::Metallic {roughness: 0.0},
    emittance: 0.0
};

const CHECKER_MAT_2: Material = Material {
    albedo: Albedo::Checker(2.0),
    details: MaterialDetails::Lambertian,
    emittance: 0.0
};

const ORANGE_MAT: Material = Material {
    albedo: Albedo::Constant(vec3(2.0, 0.8, 0.0)),
    details: MaterialDetails::Lambertian,
    emittance: 0.9
};

fn main() {
    let mut args = env::args();
    if args.len() < 2 {
        eprintln!("Usage: rust-tracer N > some.ppm");
        return;
    }
    let t: u64 = args.nth(1).unwrap().parse().unwrap();

    let quat_identity: Quaternion<f32> = Quaternion::new(0.0, 0.0, 1.0, 0.0);
    let quat_flip180_z: Quaternion<f32> = Euler::new(Rad(0.0), Rad(0.0), Rad(180.0f32.to_radians())).into();

    let ImgData{
        width: moon_map_width,
        height: moon_map_height,
        colors: moon_map_colors
    } = load_png(MOON_MAP_BYTES);

    let moon_map_mat = Material {
        albedo: Albedo::Texture(moon_map_width, moon_map_height, &moon_map_colors),
        details: MaterialDetails::Lambertian,
        emittance: 0.0
    };

    let ImgData{
        width: earth_map_width,
        height: earth_map_height,
        colors: earth_map_colors
    } = load_png(EARTH_MAP_BYTES);

    let earth_map_mat = Material {
        albedo: Albedo::Texture(earth_map_width, earth_map_height, &earth_map_colors),
        details: MaterialDetails::Lambertian,
        emittance: 0.0
    };

    fn get_normal(vecs: &[Vector3<f32>]) -> Vector3<f32> {
        (vecs[1] - vecs[0]).cross(vecs[2] - vecs[0]).normalize()
    }

    let piramid_pts = &[
        vec3(-0.2, -0.2, -0.2),
        vec3( 0.2, -0.2, -0.2),
        vec3( 0.0, -0.2,  0.2),
        vec3( 0.0,  0.2,  0.0)
    ];

    let normals = &[
        get_normal(&[piramid_pts[0], piramid_pts[1], piramid_pts[2]]),
        get_normal(&[piramid_pts[2], piramid_pts[3], piramid_pts[0]]),
        get_normal(&[piramid_pts[1], piramid_pts[3], piramid_pts[2]]),
        get_normal(&[piramid_pts[0], piramid_pts[3], piramid_pts[1]]),
    ];

    let bunny_vertices = bunny::VERTICES_0
        .iter()
        .map(|((vx, vy, vz), (nx, ny, nz), (u, v))| VertexDescription {
            position: vec3(*vx, *vy, *vz),
            normal: vec3(*nx, *ny, *nz),
            uv: vec2(*u, *v)
        })
        .collect::<Vec<_>>();

    let bunny_mesh = MeshDescription {
        vertices: &bunny_vertices,
        indices: bunny::INDICES_0,
        triangle_count: bunny::COUNT_0
    };

    let shapes = &[
        Shape::Disk{
            center: vec3(-0.85, 0.49, 1.05),
            radius: 0.125,
            rotation: quat_flip180_z,
            material: WHITE_BULB_MAT
        },
        Shape::Disk{
            center: vec3(0.85, 0.49, 1.05),
            radius: 0.125 / 2.0,
            rotation: quat_flip180_z,
            material: WHITE_BULB_MAT
        },
        Shape::Disk{
            center: vec3(0.0, 0.49, -1.05),
            radius: 0.125 / 4.0,
            rotation: quat_flip180_z,
            material: WHITE_BULB_MAT
        },
        Shape::Sphere{
            center: vec3(-0.6, -0.3, 0.7),
            radius: 0.15,
            rotation: quat_identity,
            material: DIELECTRIC_MAT
        },
        Shape::Sphere{
            center: vec3(0.0, 0.0, 1.0),
            radius: 0.5,
            rotation: quat_identity,
            material: moon_map_mat
        },
        Shape::Sphere{
            center: vec3(0.25, -0.4, 0.65),
            radius: 0.1,
            rotation: quat_identity,
            material: ORANGE_MAT
        },
        Shape::Cube {
            center: vec3(-0.25, -0.4, 0.35),
            sizes: vec3(0.2, 0.2, 0.2),
            rotation: Quaternion::new(0.5, 0.0, 1.0, 0.0),
            material: CHECKER_MAT_2
        },
        Shape::Sphere{
            center: vec3(0.15, -0.45, 0.55),
            radius: 0.05,
            rotation: quat_identity,
            material: DARK_GRAY_MAT
        },
        Shape::Sphere{
            center: vec3(-0.75, -0.45, 0.75),
            radius: 0.05,
            rotation: quat_identity,
            material: DARK_GRAY_MAT
        },
        Shape::Disk{
            center: vec3(0.0, -0.5, 1.0),
            radius: 2.0,
            rotation: quat_identity,
            material: earth_map_mat
        },
        Shape::Sphere{
            center: vec3(0.0, 100.0, 1.0),
            radius: 99.5,
            rotation: quat_identity,
            material: LIGHT_GRAY_MAT
        },
        Shape::TriangleMesh {
            center: vec3(0.55, -0.5, 0.65),
            mesh: bunny_mesh,
            material: LIGHT_GRAY_MAT_LAMBERT
        }
    ];

    let scene = Scene {
        focus_distance: FOCUS_DISTANCE,
        aperture: APERTURE,
        num_samples: NUM_SAMPLES,
        max_t: MAX_T,
        world: World::construct(shapes)
    };
    scene.render_as_ppm(t, 1280, 800);
}
