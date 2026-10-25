use crate::constants::{GRAVITATIONAL_CONSTANT, SPEED_OF_LIGHT};

/// Radio de Einstein θE [rad] de una lente puntual (campo débil):
/// `θE = √( 4GM/c² · D_ls / (D_l · D_s) )`.
///
/// - `mass`: masa de la lente [kg]
/// - `d_lens`: distancia observador–lente `D_l` [m]
/// - `d_source`: distancia observador–fuente `D_s` [m]
///
/// `D_ls = D_s − D_l`. Si la lente no está delante de la fuente (`D_l >= D_s`) no hay
/// lente efectiva y devuelve 0.
pub fn einstein_radius(mass: f64, d_lens: f64, d_source: f64) -> f64 {
    let d_ls = (d_source - d_lens).max(0.0);
    let bending_length = 4.0 * GRAVITATIONAL_CONSTANT * mass / SPEED_OF_LIGHT.powi(2);

    (bending_length * d_ls / (d_lens * d_source)).sqrt()
}

/// Ecuación de la lente en 2D: `β = θ − θE² · θ / |θ|²`.
///
/// - `theta`: posición aparente en el cielo, respecto a la lente [rad]
/// - `theta_e`: radio de Einstein [rad]
///
/// Devuelve la posición real de la fuente `β` [rad], es decir, "de dónde salió
/// realmente la luz que vemos en `theta`". Justo sobre el centro de la lente
/// (`theta = 0`) no hay solución finita y devuelve infinito.
pub fn source_position(theta: (f64, f64), theta_e: f64) -> (f64, f64) {
    let (x, y) = theta;
    let r2 = x * x + y * y;
    if r2 == 0.0 {
        return (f64::INFINITY, f64::INFINITY);
    }
    let factor = theta_e * theta_e / r2;

    (x - factor * x, y - factor * y)
}

/// Amplificación total de una lente puntual: `A(u) = (u² + 2) / (u · √(u² + 4))`,
/// con `u = β / θE` (separación fuente–lente en radios de Einstein).
///
/// Vale 1 muy lejos de la lente, ≈1.34 en `u = 1` y crece como `1/u` al alinearse
/// (infinito en `u = 0`: la fuente puntual daría brillo infinito).
pub fn magnification(u: f64) -> f64 {
    let u = u.abs();
    if u == 0.0 {
        return f64::INFINITY;
    }

    (u * u + 2.0) / (u * (u * u + 4.0).sqrt())
}

/// Posiciones de las dos imágenes de una fuente puntual, a lo largo del eje que pasa
/// por la lente y la fuente: `θ± = (β ± √(β² + 4 θE²)) / 2`.
///
/// - `beta`: posición de la fuente respecto a la lente, con signo [rad]
/// - `theta_e`: radio de Einstein [rad]
///
/// Devuelve `(θ+, θ−)`: la imagen principal (mismo lado que la fuente, `θ+ > 0` si
/// `β > 0`) y la secundaria (lado contrario, dentro del anillo).
pub fn image_positions(beta: f64, theta_e: f64) -> (f64, f64) {
    let root = (beta * beta + 4.0 * theta_e * theta_e).sqrt();

    ((beta + root) / 2.0, (beta - root) / 2.0)
}

/// Amplificación de cada imagen, `(μ+, μ−)`, en valor absoluto:
/// `μ± = (A ± 1) / 2` con `A = magnification(u)`. La suma es la amplificación total.
pub fn image_magnifications(u: f64) -> (f64, f64) {
    let a = magnification(u);

    ((a + 1.0) / 2.0, (a - 1.0) / 2.0)
}

#[cfg(test)]
mod tests;
