use {
    cgmath::{Vector3, vec3},
    rand:: {
        prelude::ThreadRng,
        distributions::Uniform,
        distributions::Distribution
    }
};
use std::f32::consts::PI;

const GOLDEN_RATIO_NUM_SAMPLES: usize = 0x100000;

lazy_static! {
    static ref UNIT_SPHERE_VECTORS: Vec<Vector3<f32>> = {
        let mut v = Vec::with_capacity(GOLDEN_RATIO_NUM_SAMPLES);
        for i in 0..GOLDEN_RATIO_NUM_SAMPLES {
            let sample = i as f32 + 0.5;
            let phi_cos = 1.0 - 2.0 * sample / GOLDEN_RATIO_NUM_SAMPLES as f32;
            let phi = phi_cos.acos();
            let theta = PI * (1.0 + 5.0f32.sqrt()) * sample;
            let phi_sin = phi.sin();

            v.push(vec3(phi_sin * theta.cos(), phi_sin * theta.sin(), phi_cos));
        }
        v
    };

    static ref UNIT_DISK_VECTORS: Vec<Vector3<f32>> = {
        let mut v = Vec::with_capacity(GOLDEN_RATIO_NUM_SAMPLES);
        for i in 0..GOLDEN_RATIO_NUM_SAMPLES {
            let sample: f32 = {
                i as f32 + 0.5
            };
            let r = (sample / GOLDEN_RATIO_NUM_SAMPLES as f32).sqrt();
            let theta = PI * (1.0 + 5.0f32.sqrt()) * sample;
            v.push(vec3(r * theta.cos(), r * theta.sin(), 0.0));
        }
        v
    };

    static ref UNIFORM: Uniform<usize> = Uniform::from(0..GOLDEN_RATIO_NUM_SAMPLES);
}

pub fn get_random_in_unit_sphere(rng: &mut ThreadRng) -> Vector3<f32> {
    UNIT_SPHERE_VECTORS[UNIFORM.sample(rng)]
}

pub fn get_random_in_unit_disk(rng: &mut ThreadRng) -> Vector3<f32> {
    UNIT_DISK_VECTORS[UNIFORM.sample(rng)]
}