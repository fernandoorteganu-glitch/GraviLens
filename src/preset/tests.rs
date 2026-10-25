use super::*;
use crate::physics::{
    ObjectKind, SpectralType, classify_object, density, luminosity, spectral_type, temperature,
};

#[test]
fn sun_goes_through_the_whole_chain() {
    let p = sun();
    let t = temperature(luminosity(p.mass), p.radius);
    // Densidad ~1410 kg/m³, temperatura ~5772 K, tipo G
    assert!((density(p.mass, p.radius) - 1410.0).abs() < 10.0);
    assert!((t - 5772.0).abs() < 20.0, "temperatura del Sol: {t}");
    assert_eq!(spectral_type(t), SpectralType::G);
}

#[test]
fn every_preset_is_classified_as_expected() {
    let casos = [
        (sun(), ObjectKind::MainSequenceStar),
        (sirius_a(), ObjectKind::MainSequenceStar),
        (sirius_b(), ObjectKind::WhiteDwarf),
        (neutron_star(), ObjectKind::NeutronStar),
        (stellar_black_hole(), ObjectKind::BlackHole),
        (jupiter(), ObjectKind::Planet),
    ];
    for (p, esperado) in casos {
        assert_eq!(classify_object(p.mass, p.radius), esperado, "{}", p.name);
    }
}

#[test]
fn all_presets_have_positive_mass_and_radius() {
    for preset in all_presets() {
        assert!(preset.mass > 0.0 && preset.radius > 0.0, "{}", preset.name)
    }
}
