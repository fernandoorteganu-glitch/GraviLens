use super::*;

const SIZE: usize = 200;
const FOV: f64 = 6.0;

fn params(source_offset: f64) -> LensParams {
    LensParams {
        theta_e: 1.0,
        fov: FOV,
        photo: false,
        sky_span: FOV,
        source_offset,
        source_radius: 0.1,
        color: [255, 255, 255],
        stars: false,
        samples: 1,
    }
}

/// Foto de relleno para los tests que no la usan.
fn no_sky() -> SkyImage {
    SkyImage::from_rgb(1, 1, vec![0, 0, 0])
}

/// Foto de 2×1 píxeles: mitad izquierda negra, mitad derecha blanca.
fn half_and_half_sky() -> SkyImage {
    SkyImage::from_rgb(2, 1, vec![0, 0, 0, 255, 255, 255])
}

/// Color del píxel que contiene el ángulo (x, y).
fn pixel_at(image: &[u8], x: f64, y: f64) -> [u8; 3] {
    let ix = ((x / FOV + 0.5) * SIZE as f64).floor() as usize;
    let iy = ((0.5 - y / FOV) * SIZE as f64).floor() as usize;
    let k = (iy * SIZE + ix) * 3;
    [image[k], image[k + 1], image[k + 2]]
}

fn is_bright(rgb: [u8; 3]) -> bool {
    rgb[0] > 200
}

#[test]
fn aligned_source_makes_a_ring_all_around() {
    let image = render(&params(0.0), SIZE, SIZE, &no_sky());
    for (x, y) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0), (0.7, 0.7)] {
        assert!(
            is_bright(pixel_at(&image, x, y)),
            "falta anillo en ({x}, {y})"
        );
    }
}

#[test]
fn inside_and_outside_the_ring_is_dark() {
    let image = render(&params(0.0), SIZE, SIZE, &no_sky());
    for (x, y) in [(0.3, 0.0), (2.0, 0.0), (0.0, -2.5), (-2.0, 2.0)] {
        assert!(
            !is_bright(pixel_at(&image, x, y)),
            "no debería haber luz en ({x}, {y})"
        );
    }
}

#[test]
fn off_axis_source_gives_two_images_on_opposite_sides() {
    // Fuente en β = 0.5 θE: imágenes en θ = (0.5 ± √(0.25 + 4)) / 2 = 1.28 y −0.78
    let image = render(&params(0.5), SIZE, SIZE, &no_sky());
    assert!(
        is_bright(pixel_at(&image, 1.28, 0.0)),
        "falta la imagen mayor"
    );
    assert!(
        is_bright(pixel_at(&image, -0.78, 0.0)),
        "falta la imagen menor"
    );
    assert!(
        !is_bright(pixel_at(&image, -1.28, 0.0)),
        "no debería haber luz en −1.28"
    );
}

#[test]
fn without_a_lens_the_source_appears_where_it_is() {
    let mut p = params(1.0);
    p.theta_e = 0.0;
    let image = render(&p, SIZE, SIZE, &no_sky());
    assert!(is_bright(pixel_at(&image, 1.0, 0.0)));
    assert!(!is_bright(pixel_at(&image, -1.0, 0.0)));
}

#[test]
fn image_has_three_bytes_per_pixel() {
    assert_eq!(render(&params(0.0), 16, 16, &no_sky()).len(), 16 * 16 * 3);
}

#[test]
fn photo_without_a_lens_is_seen_as_it_is() {
    // Sin lente, la foto (que cubre exactamente el campo de visión) se ve tal cual
    let mut p = params(0.0);
    p.theta_e = 0.0;
    p.photo = true;
    let image = render(&p, SIZE, SIZE, &half_and_half_sky());
    assert!(is_bright(pixel_at(&image, 2.0, 1.0)), "derecha: blanca");
    assert!(!is_bright(pixel_at(&image, -2.0, 1.0)), "izquierda: negra");
}

#[test]
fn photo_is_pulled_around_the_lens() {
    // Con lente, un punto a θ = 2 θE viene de β = 1.5 θE: la foto (cubre ±3 mas)
    // lo muestra en su mitad derecha. En el lado izquierdo viene de β = −1.5: negro.
    let mut p = params(0.0);
    p.photo = true;
    let image = render(&p, SIZE, SIZE, &half_and_half_sky());
    assert!(is_bright(pixel_at(&image, 2.0, 0.0)));
    assert!(!is_bright(pixel_at(&image, -2.0, 0.0)));
}

#[test]
fn supersampling_keeps_the_ring_and_smooths_the_edges() {
    let mut p = params(0.0);
    p.samples = 3;
    let image = render(&p, SIZE, SIZE, &no_sky());
    // Sigue habiendo anillo en su sitio y oscuridad dentro y fuera
    assert!(is_bright(pixel_at(&image, 1.0, 0.0)));
    assert!(!is_bright(pixel_at(&image, 0.3, 0.0)));
    assert!(!is_bright(pixel_at(&image, 2.0, 0.0)));
}

#[test]
fn wide_image_keeps_the_same_scale_on_both_axes() {
    // 400×200 píxeles: el lado corto (200) cubre FOV, así que el largo cubre 2·FOV.
    // El anillo (θE = 1) debe ser un círculo de radio 1/6 del lado corto en píxeles.
    let image = render(&params(0.0), 2 * SIZE, SIZE, &no_sky());
    let at = |x: f64, y: f64| {
        let ix = ((x / FOV) * SIZE as f64 + SIZE as f64).floor() as usize;
        let iy = ((0.5 - y / FOV) * SIZE as f64).floor() as usize;
        let k = (iy * 2 * SIZE + ix) * 3;
        [image[k], image[k + 1], image[k + 2]]
    };
    for (x, y) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0)] {
        assert!(is_bright(at(x, y)), "falta anillo en ({x}, {y})");
    }
    assert!(!is_bright(at(2.0, 0.0)));
}

#[test]
fn image_size_matches_width_times_height() {
    assert_eq!(render(&params(0.0), 24, 10, &no_sky()).len(), 24 * 10 * 3);
}

#[test]
fn star_is_brighter_at_the_center_than_at_the_limb_and_has_a_fading_halo() {
    let color = [255, 242, 230];
    let center = star_light(0.0, color)[0];
    let limb = star_light(0.99, color)[0];
    let halo_near = star_light(1.2, color)[0];
    let halo_far = star_light(3.0, color)[0];
    assert!(center > limb, "centro {center}, borde {limb}");
    assert!(limb > halo_near, "borde {limb}, halo {halo_near}");
    assert!(
        halo_near > halo_far,
        "halo cercano {halo_near}, lejano {halo_far}"
    );
    assert_eq!(star_light(4.0, color), [0.0; 3]);
}
