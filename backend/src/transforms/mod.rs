use rand::Rng;

fn laplace_sample(location: f64, scale: f64) -> f64 {
    let mut rng = rand::thread_rng();
    let u: f64 = rng.gen_range(0.0..1.0); // Uniformly distributed variable in range (0, 1)

    // Apply the inverse CDF method
    let laplace_sample = if u < 0.5 {
        location + scale * (u).ln() // Negative side
    } else {
        location - scale * (1.0 - u).ln() // Positive side
    };

    laplace_sample
}

fn add_laplace_noise(true_value: f64, sensitivity: f64, epsilon: f64) -> f64 {
    let scale = sensitivity / epsilon;
    let noise = laplace_sample(0.0, scale);
    println!(
        "Changing {} to {} using {}",
        true_value,
        true_value + noise,
        epsilon
    );
    true_value + noise
}

pub fn laplace_transform(true_value: f64, sensitivity: f64, privacy_budget: f64) -> f64 {
    add_laplace_noise(true_value, sensitivity, privacy_budget).round()
}
