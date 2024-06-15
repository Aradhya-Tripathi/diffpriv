use rand::Rng;

fn laplace_sample(location: f64, scale: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let u: f64 = rng.gen_range(-0.5..0.5);
    location - scale * u.signum() * (1.0 - 2.0 * u.abs()).ln() // Formula for laplace sample
}

fn add_laplace_noise(true_value: f64, sensitivity: f64, epsilon: f64) -> f64 {
    let noise = laplace_sample(0.0, sensitivity / epsilon);
    true_value + noise
}

pub fn laplace_transform(true_value: f64, sensitivity: f64) -> f64 {
    let epsilon = 0.1; // Privacy Budget - User defined however for now we just define it to test things out
    add_laplace_noise(true_value, sensitivity, epsilon)
}
