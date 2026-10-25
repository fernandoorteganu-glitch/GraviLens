use super::*;

/// Foto de 2×1 píxeles: izquierda negro, derecha blanco.
fn black_and_white() -> SkyImage {
    SkyImage::from_rgb(2, 1, vec![0, 0, 0, 255, 255, 255])
}

#[test]
fn pixel_centers_return_their_exact_color() {
    let sky = black_and_white();
    assert_eq!(sky.sample(0.25, 0.5), [0.0, 0.0, 0.0]);
    assert_eq!(sky.sample(0.75, 0.5), [255.0, 255.0, 255.0]);
}

#[test]
fn between_pixels_the_color_is_interpolated() {
    let sky = black_and_white();
    let [r, _, _] = sky.sample(0.5, 0.5);
    assert!((r - 127.5).abs() < 1e-9, "r: {r}");
}

#[test]
fn outside_the_photo_it_is_mirrored() {
    let sky = black_and_white();
    // u = 1.25 se refleja en 0.75 (blanco); u = -0.25 en 0.25 (negro)
    assert_eq!(sky.sample(1.25, 0.5), sky.sample(0.75, 0.5));
    assert_eq!(sky.sample(-0.25, 0.5), sky.sample(0.25, 0.5));
}

#[test]
fn mirror_folds_numbers_into_zero_one() {
    assert_eq!(mirror(0.0), 0.0);
    assert_eq!(mirror(1.0), 1.0);
    assert!((mirror(1.25) - 0.75).abs() < 1e-12);
    assert!((mirror(-0.25) - 0.25).abs() < 1e-12);
    assert!((mirror(2.5) - 0.5).abs() < 1e-12);
}

#[test]
fn the_bundled_hubble_photo_decodes() {
    let bytes = include_bytes!("../../assets/hubble_ultra_deep_field.jpg");
    let sky = SkyImage::from_bytes(bytes).expect("la foto incluida debe decodificarse");
    assert_eq!((sky.width, sky.height), (6200, 6200));
}

#[test]
fn a_wide_photo_is_cropped_to_its_central_square() {
    // 4×2 píxeles: columnas 0 y 3 rojas, columnas 1 y 2 verdes → queda el cuadrado 2×2 verde
    let mut img = image::RgbImage::new(4, 2);
    for (x, _, pixel) in img.enumerate_pixels_mut() {
        *pixel = if x == 0 || x == 3 {
            image::Rgb([255, 0, 0])
        } else {
            image::Rgb([0, 255, 0])
        };
    }
    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .unwrap();

    let sky = SkyImage::from_bytes(&bytes).unwrap();
    assert_eq!(sky.sample(0.5, 0.5), [0.0, 255.0, 0.0]);
    assert_eq!(sky.sample(0.0, 0.0), [0.0, 255.0, 0.0]);
}
