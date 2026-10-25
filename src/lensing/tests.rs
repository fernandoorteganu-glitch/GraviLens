use super::*;
use crate::constants::{KILOPARSEC, MILLIARCSECOND, SOLAR_MASS};

#[test]
fn sun_mass_lens_at_4_kpc_has_einstein_radius_of_about_1_mas() {
    let theta_e = einstein_radius(SOLAR_MASS, 4.0 * KILOPARSEC, 8.0 * KILOPARSEC);
    let mas = theta_e / MILLIARCSECOND;
    // Valor de referencia (microlente típica en el bulbo galáctico): ~1.01 mas
    assert!((mas - 1.01).abs() < 0.02, "θE: {mas} mas");
}

#[test]
fn einstein_radius_grows_as_the_square_root_of_mass() {
    let one = einstein_radius(SOLAR_MASS, 4.0 * KILOPARSEC, 8.0 * KILOPARSEC);
    let four = einstein_radius(4.0 * SOLAR_MASS, 4.0 * KILOPARSEC, 8.0 * KILOPARSEC);
    assert!((four / one - 2.0).abs() < 1e-9, "cociente: {}", four / one);
}

#[test]
fn lens_behind_the_source_does_nothing() {
    assert_eq!(
        einstein_radius(SOLAR_MASS, 8.0 * KILOPARSEC, 4.0 * KILOPARSEC),
        0.0
    );
    assert_eq!(
        einstein_radius(SOLAR_MASS, 4.0 * KILOPARSEC, 4.0 * KILOPARSEC),
        0.0
    );
}

#[test]
fn on_the_einstein_ring_the_source_is_exactly_behind_the_lens() {
    // θ = θE en cualquier dirección → β = 0: de ahí sale el anillo
    for (x, y) in [(1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.6, 0.8)] {
        let (bx, by) = source_position((x, y), 1.0);
        assert!(bx.abs() < 1e-12 && by.abs() < 1e-12, "β = ({bx}, {by})");
    }
}

#[test]
fn outside_the_ring_the_source_is_pulled_towards_the_center() {
    // θ = 2 θE → β = 2 − 1/2 = 1.5 θE, en la misma dirección
    let (bx, by) = source_position((2.0, 0.0), 1.0);
    assert!((bx - 1.5).abs() < 1e-12 && by.abs() < 1e-12);
}

#[test]
fn inside_the_ring_the_source_is_on_the_opposite_side() {
    // θ = 0.5 θE → β = 0.5 − 2 = −1.5 θE: imagen invertida
    let (bx, _) = source_position((0.5, 0.0), 1.0);
    assert!((bx + 1.5).abs() < 1e-12, "βx: {bx}");
}

#[test]
fn right_on_the_lens_there_is_no_finite_source_position() {
    let (bx, by) = source_position((0.0, 0.0), 1.0);
    assert!(bx.is_infinite() && by.is_infinite());
}

#[test]
fn magnification_matches_known_values() {
    // u = 1 → 3 / √5 ≈ 1.3416 (la referencia clásica de microlentes)
    assert!((magnification(1.0) - 1.3416).abs() < 1e-3);
    // Lejos de la lente no hay efecto
    assert!((magnification(100.0) - 1.0).abs() < 1e-3);
    // Muy alineada crece como 1/u
    assert!((magnification(0.01) * 0.01 - 1.0).abs() < 1e-3);
    assert!(magnification(0.0).is_infinite());
}

#[test]
fn image_positions_satisfy_the_lens_equation() {
    // Cada θ± debe cumplir β = θ − θE²/θ (en una dimensión)
    let (beta, theta_e) = (0.7, 1.3);
    let (plus, minus) = image_positions(beta, theta_e);
    for theta in [plus, minus] {
        let (b, _) = source_position((theta, 0.0), theta_e);
        assert!((b - beta).abs() < 1e-12, "β: {b}");
    }
    assert!(plus > theta_e && minus < 0.0 && minus.abs() < theta_e);
}

#[test]
fn aligned_source_gives_images_on_the_ring() {
    let (plus, minus) = image_positions(0.0, 2.0);
    assert!((plus - 2.0).abs() < 1e-12 && (minus + 2.0).abs() < 1e-12);
}

#[test]
fn image_magnifications_add_up_to_the_total() {
    let u = 0.8;
    let (plus, minus) = image_magnifications(u);
    assert!((plus + minus - magnification(u)).abs() < 1e-12);
    assert!(plus > minus && minus > 0.0);
}
