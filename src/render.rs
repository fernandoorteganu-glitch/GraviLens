use crate::lensing::source_position;
use crate::sky::SkyImage;

/// Parámetros de una imagen de lente. Los ángulos están en radianes.
#[derive(Clone, Copy, PartialEq)]
pub struct LensParams {
    /// Radio de Einstein que se dibuja (ya multiplicado por `k`) [rad].
    pub theta_e: f64,
    /// Campo de visión: ángulo que cubre el lado **más corto** de la imagen [rad]. El lado
    /// largo cubre más, con la misma escala en los dos ejes.
    pub fov: f64,
    /// Si el fondo es la foto (`true`) o una estrella con campo de estrellas (`false`).
    pub photo: bool,
    /// Ángulo que cubre el lado de la foto de fondo [rad].
    pub sky_span: f64,
    /// Desplazamiento del fondo en el eje x, respecto a la lente [rad]. Mueve la foto
    /// o la estrella.
    pub source_offset: f64,
    /// Radio angular de la estrella de fondo [rad].
    pub source_radius: f64,
    /// Color de la estrella de fondo.
    pub color: [u8; 3],
    /// Si hay campo de estrellas débiles de fondo (se apaga en los tests).
    pub stars: bool,
    /// Suavizado: cada píxel promedia `samples × samples` puntos (1 = sin suavizar).
    pub samples: usize,
}

/// Dibuja la imagen vista a través de la lente: `width × height` píxeles, RGB.
///
/// Trazado inverso: para cada píxel se calcula de dónde salió realmente la luz
/// (`source_position`) y se toma el color del fondo en ese punto. El brillo
/// superficial se conserva en una lente, así que no hace falta multiplicar por la
/// amplificación.
///
/// Las filas se reparten entre los núcleos del procesador, cada uno en su hilo: los
/// píxeles son independientes entre sí, así que no hay nada que coordinar.
pub fn render(params: &LensParams, width: usize, height: usize, sky: &SkyImage) -> Vec<u8> {
    let mut pixels = vec![0u8; width * height * 3];
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let rows_per_band = height.div_ceil(threads).max(1);

    std::thread::scope(|scope| {
        for (band, chunk) in pixels.chunks_mut(rows_per_band * width * 3).enumerate() {
            scope.spawn(move || {
                for (row, line) in chunk.chunks_mut(width * 3).enumerate() {
                    let iy = band * rows_per_band + row;
                    for (ix, pixel) in line.chunks_mut(3).enumerate() {
                        let color = pixel_color(params, (width, height), sky, ix, iy);
                        for (out, c) in pixel.iter_mut().zip(color) {
                            *out = c.clamp(0.0, 255.0).round() as u8;
                        }
                    }
                }
            });
        }
    });
    pixels
}

/// Color del píxel `(ix, iy)`: promedio de `samples × samples` puntos repartidos dentro
/// del píxel. Con una foto más fina que la pantalla, un solo punto por píxel da dientes
/// de sierra; promediar varios los suaviza (supermuestreo).
fn pixel_color(
    params: &LensParams,
    (width, height): (usize, usize),
    sky: &SkyImage,
    ix: usize,
    iy: usize,
) -> [f64; 3] {
    let n = params.samples.max(1);
    // Ángulo que ocupa un píxel: el campo de visión reparte el lado más corto.
    let scale = params.fov / width.min(height) as f64;
    let mut sum = [0.0; 3];
    for sy in 0..n {
        for sx in 0..n {
            // Punto dentro del píxel → ángulo; el eje y de la pantalla crece hacia abajo.
            let px = ix as f64 + (sx as f64 + 0.5) / n as f64;
            let py = iy as f64 + (sy as f64 + 0.5) / n as f64;
            let x = (px - width as f64 / 2.0) * scale;
            let y = (height as f64 / 2.0 - py) * scale;
            let beta = source_position((x, y), params.theta_e);
            let color = background(beta, params, sky);
            for (total, c) in sum.iter_mut().zip(color) {
                *total += c;
            }
        }
    }
    sum.map(|total| total / (n * n) as f64)
}

/// Color del fondo en la posición real `beta`: la foto, o la estrella principal más,
/// si hay, un campo de estrellas débiles.
fn background(beta: (f64, f64), params: &LensParams, sky: &SkyImage) -> [f64; 3] {
    let (bx, by) = beta;
    if !bx.is_finite() || !by.is_finite() {
        return [0.0; 3];
    }

    if params.photo {
        // La foto es un cuadrado de lado `sky_span` centrado en el eje (más el desplazamiento).
        let u = (bx - params.source_offset) / params.sky_span + 0.5;
        let v = 0.5 - by / params.sky_span;
        return sky.sample(u, v);
    }

    // Estrella de fondo, vista a través de la lente: disco con halo.
    let d = (bx - params.source_offset).hypot(by);
    let mut rgb = star_light(d / params.source_radius, params.color);

    if params.stars {
        let faint = 200.0 * faint_star(beta, params.fov / 16.0, 0).0;
        for c in &mut rgb {
            *c = c.max(faint);
        }
        // Segunda capa, más densa y tenue, con estrellas azuladas o rojizas.
        let (dim, blue) = faint_star(beta, params.fov / 30.0, 7);
        let dim = 120.0 * dim;
        let tint = if blue {
            [0.8, 0.9, 1.0]
        } else {
            [1.0, 0.85, 0.7]
        };
        for (c, t) in rgb.iter_mut().zip(tint) {
            *c = c.max(dim * t);
        }
    }

    rgb
}

/// Luz de una estrella a `x` radios de su centro (`x = 1` es el borde del disco).
///
/// - Dentro del disco: oscurecimiento del limbo (el borde es más oscuro que el centro,
///   como en el Sol) y centro más blanco que el color de la superficie.
/// - Fuera del disco: un halo que se apaga deprisa y desaparece a los 4 radios.
fn star_light(x: f64, color: [u8; 3]) -> [f64; 3] {
    let base = color.map(|c| c as f64);
    if x <= 1.0 {
        let mu = (1.0 - x * x).sqrt(); // 1 en el centro, 0 en el borde
        let intensity = 1.0 - 0.6 * (1.0 - mu);
        let white = 0.5 * mu * mu;
        base.map(|c| (c + (255.0 - c) * white) * intensity)
    } else if x < 4.0 {
        let glow = 0.45 * (-(x - 1.0) * 5.0).exp();
        base.map(|c| c * glow)
    } else {
        [0.0; 3]
    }
}

/// Campo de estrellas débiles: el plano se divide en celdas de lado `cell` y algunas
/// celdas tienen una estrella en su zona central. Devuelve su brillo (0 a 1) y si es
/// azulada (`true`) o rojiza. `salt` desplaza el patrón para tener capas distintas.
fn faint_star(beta: (f64, f64), cell: f64, salt: i64) -> (f64, bool) {
    let (i, j) = ((beta.0 / cell).floor(), (beta.1 / cell).floor());
    let h = hash(i as i64 + salt, j as i64 - salt);
    if !h.is_multiple_of(4) {
        return (0.0, false);
    }
    // Posición dentro de la celda entre 0.3 y 0.7, para que no se corte en el borde.
    let fx = 0.3 + 0.4 * ((h >> 8) & 0xFF) as f64 / 255.0;
    let fy = 0.3 + 0.4 * ((h >> 16) & 0xFF) as f64 / 255.0;
    let brightness = 0.4 + 0.6 * ((h >> 24) & 0xFF) as f64 / 255.0;

    let d = (beta.0 - (i + fx) * cell).hypot(beta.1 - (j + fy) * cell);
    let radius = 0.12 * cell;
    let light = brightness * ((1.2 * radius - d) / (0.2 * radius)).clamp(0.0, 1.0);
    (light, (h >> 40) & 1 == 0)
}

/// Mezcla de bits determinista: la misma celda siempre da el mismo número.
fn hash(i: i64, j: i64) -> u64 {
    let mut h = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (j as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 29;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^ (h >> 32)
}

#[cfg(test)]
mod tests;
