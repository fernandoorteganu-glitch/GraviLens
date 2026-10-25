/// Foto de fondo (el cielo que hay detrás de la lente), en RGB de 8 bits.
pub struct SkyImage {
    width: usize,
    height: usize,
    rgb: Vec<u8>,
}

impl SkyImage {
    /// Crea la imagen a partir de píxeles RGB (3 bytes por píxel, fila a fila).
    pub fn from_rgb(width: usize, height: usize, rgb: Vec<u8>) -> Self {
        assert_eq!(rgb.len(), width * height * 3, "tamaño de imagen incorrecto");
        Self { width, height, rgb }
    }

    /// Decodifica una foto JPEG o PNG, la recorta al cuadrado central (el programa usa
    /// fotos cuadradas) y la reduce si pasa de `MAX_SIDE` píxeles de lado.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, image::ImageError> {
        let decoded = image::load_from_memory(bytes)?;
        let (w, h) = (decoded.width(), decoded.height());
        let side = w.min(h);
        let mut square = decoded.crop_imm((w - side) / 2, (h - side) / 2, side, side);
        if side > Self::MAX_SIDE {
            square = square.resize_exact(
                Self::MAX_SIDE,
                Self::MAX_SIDE,
                image::imageops::FilterType::Triangle,
            );
        }
        let rgb = square.to_rgb8();
        let (width, height) = (rgb.width() as usize, rgb.height() as usize);

        Ok(Self::from_rgb(width, height, rgb.into_raw()))
    }

    /// Lado máximo de la foto en memoria [píxeles]: limita el gasto de RAM.
    const MAX_SIDE: u32 = 6200;

    /// Color en la posición normalizada `(u, v)`, con `u` y `v` en `[0, 1]` sobre la
    /// foto (`v` crece hacia abajo). Interpola entre los 4 píxeles vecinos.
    ///
    /// Fuera de `[0, 1]` la foto se refleja como un espejo, así que nunca hay huecos
    /// negros por muy lejos que se mire.
    pub fn sample(&self, u: f64, v: f64) -> [f64; 3] {
        // Los centros de los píxeles están en (i + 0.5) / ancho.
        let x = mirror(u) * self.width as f64 - 0.5;
        let y = mirror(v) * self.height as f64 - 0.5;
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);

        let mut out = [0.0; 3];
        for (dx, dy, weight) in [
            (0.0, 0.0, (1.0 - fx) * (1.0 - fy)),
            (1.0, 0.0, fx * (1.0 - fy)),
            (0.0, 1.0, (1.0 - fx) * fy),
            (1.0, 1.0, fx * fy),
        ] {
            let px = self.clamp_index(x0 + dx, self.width);
            let py = self.clamp_index(y0 + dy, self.height);
            let k = (py * self.width + px) * 3;
            for (c, channel) in out.iter_mut().enumerate() {
                *channel += weight * self.rgb[k + c] as f64;
            }
        }
        out
    }

    /// Índice de píxel dentro de `[0, len - 1]`.
    fn clamp_index(&self, i: f64, len: usize) -> usize {
        i.clamp(0.0, (len - 1) as f64) as usize
    }
}

/// Refleja un número real en `[0, 1]` como un espejo que se repite:
/// 0 → 0, 0.5 → 0.5, 1 → 1, 1.25 → 0.75, 2 → 0.
fn mirror(t: f64) -> f64 {
    let m = t.rem_euclid(2.0);
    if m <= 1.0 { m } else { 2.0 - m }
}

#[cfg(test)]
mod tests;
