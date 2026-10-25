use super::*;
use crate::constants::SOLAR_RADIUS;

#[test]
fn sun_density_is_about_1410_kg_per_m3() {
    let rho = density(SOLAR_MASS, SOLAR_RADIUS);
    // El Sol tiene ~1.41 g/cm³ = ~1410 kg/m³ (margen de 10 kg/m³)
    assert!((rho - 1410.0).abs() < 10.0, "densidad del Sol: {rho}");
}

#[test]
fn sun_luminosity_matches_solar_constant() {
    let l = luminosity(SOLAR_MASS);
    // Con M = M☉ el cociente vale 1 y 1^3.5 = 1, así que sale L☉ (margen de 1e10 W)
    assert!(
        (l - SOLAR_LUMINOSITY).abs() < 1e10,
        "luminosidad del Sol: {l}"
    );
}

#[test]
fn ten_solar_masses_are_about_3162_times_brighter() {
    let l = luminosity(10.0 * SOLAR_MASS);
    let expected = 3162.0 * SOLAR_LUMINOSITY; // 10^3.5 ≈ 3162
    // Margen del 1 %: proporcional al valor, no fijo
    assert!((l - expected).abs() < 0.01 * expected, "luminosidad: {l}");
}

#[test]
fn sun_temperature_is_about_5772_k() {
    let t = temperature(SOLAR_LUMINOSITY, SOLAR_RADIUS);
    // Temperatura efectiva del Sol: ~5772 K (margen de 20 K)
    assert!((t - 5772.0).abs() < 20.0, "temperatura del Sol: {t}");
}

#[test]
fn sun_color_is_warm_white() {
    let c = star_color(5772.0);
    // Valores calculados a mano con la fórmula: ~(255, 242, 230), ±2 por redondeo
    assert_eq!(c.r, 255);
    assert!((c.g as i32 - 242).abs() <= 2, "verde del Sol: {}", c.g);
    assert!((c.b as i32 - 230).abs() <= 2, "azul del Sol: {}", c.b);
}

#[test]
fn sun_is_type_g() {
    assert_eq!(spectral_type(5772.0), SpectralType::G);
}

#[test]
fn spectral_type_boundaries_belong_to_the_hotter_class() {
    // El límite inferior de cada clase es inclusivo (>=)
    assert_eq!(spectral_type(30000.0), SpectralType::O);
    assert_eq!(spectral_type(29999.9), SpectralType::B);
    assert_eq!(spectral_type(10000.0), SpectralType::B);
    assert_eq!(spectral_type(9999.9), SpectralType::A);
    assert_eq!(spectral_type(7500.0), SpectralType::A);
    assert_eq!(spectral_type(7499.9), SpectralType::F);
    assert_eq!(spectral_type(6000.0), SpectralType::F);
    assert_eq!(spectral_type(5999.9), SpectralType::G);
    assert_eq!(spectral_type(5200.0), SpectralType::G);
    assert_eq!(spectral_type(5199.9), SpectralType::K);
    assert_eq!(spectral_type(3700.0), SpectralType::K);
    assert_eq!(spectral_type(3699.9), SpectralType::M);
}

#[test]
fn cold_star_is_red_with_no_blue() {
    // A 1000 K (t = 10 ≤ 19) el azul se anula y el rojo está al máximo
    let c = star_color(1000.0);
    assert_eq!(c.r, 255);
    assert_eq!(c.b, 0);
}

#[test]
fn hot_star_is_blue() {
    // A 20000 K el azul está al máximo y el rojo ya baja de 255
    let c = star_color(20000.0);
    assert_eq!(c.b, 255);
    assert!(c.r < 255, "rojo: {}", c.r);
}

#[test]
fn sun_schwarzschild_radius_is_about_2950_m() {
    let rs = schwarzschild_radius(SOLAR_MASS);
    // Rs = 2GM/c² del Sol: ~2.95 km (margen de 10 m)
    assert!((rs - 2950.0).abs() < 10.0, "Rs del Sol: {rs}");
}

#[test]
fn sun_is_a_main_sequence_star() {
    assert_eq!(
        classify_object(SOLAR_MASS, SOLAR_RADIUS),
        ObjectKind::MainSequenceStar
    );
}

#[test]
fn jupiter_is_a_planet() {
    // Júpiter: 1 M_J y radio ~7e7 m
    assert_eq!(classify_object(JUPITER_MASS, 7.0e7), ObjectKind::Planet);
}

#[test]
fn fifty_jupiter_masses_is_a_brown_dwarf() {
    // 0.05 M☉ (~52 M_J) con radio parecido al de Júpiter
    assert_eq!(
        classify_object(0.05 * SOLAR_MASS, 7.0e7),
        ObjectKind::BrownDwarf
    );
}

#[test]
fn earth_sized_sun_is_a_white_dwarf() {
    // 1 M☉ comprimido al tamaño de la Tierra (~6e6 m)
    assert_eq!(classify_object(SOLAR_MASS, 6.0e6), ObjectKind::WhiteDwarf);
}

#[test]
fn twelve_km_neutron_star() {
    // 1.4 M☉ con radio de 12 km
    assert_eq!(
        classify_object(1.4 * SOLAR_MASS, 1.2e4),
        ObjectKind::NeutronStar
    );
}

#[test]
fn sun_squeezed_to_2_km_is_a_black_hole() {
    // Radio menor que Rs (~2.95 km)
    assert_eq!(classify_object(SOLAR_MASS, 2.0e3), ObjectKind::BlackHole);
}

#[test]
fn sun_effective_temperature_matches_stefan_boltzmann() {
    let t = effective_temperature(SOLAR_MASS, SOLAR_RADIUS).expect("el Sol emite luz");
    assert!((t - 5772.0).abs() < 20.0, "temperatura del Sol: {t}");
}

#[test]
fn sun_emitted_luminosity_is_solar_luminosity() {
    let l = emitted_luminosity(SOLAR_MASS, SOLAR_RADIUS).expect("el Sol emite luz");
    // Margen del 0.1 %: el rodeo L → T → L solo acumula redondeo
    assert!(
        (l - SOLAR_LUMINOSITY).abs() < 0.001 * SOLAR_LUMINOSITY,
        "luminosidad: {l}"
    );
}

#[test]
fn planets_and_black_holes_emit_no_light() {
    assert_eq!(effective_temperature(JUPITER_MASS, 7.0e7), None);
    assert_eq!(emitted_luminosity(JUPITER_MASS, 7.0e7), None);
    assert_eq!(effective_temperature(SOLAR_MASS, 2.0e3), None);
}

#[test]
fn compact_objects_use_their_typical_temperature() {
    assert_eq!(
        effective_temperature(SOLAR_MASS, 6.0e6),
        Some(WHITE_DWARF_TEMPERATURE)
    );
    assert_eq!(
        effective_temperature(1.4 * SOLAR_MASS, 1.2e4),
        Some(NEUTRON_STAR_TEMPERATURE)
    );
    assert_eq!(
        effective_temperature(0.05 * SOLAR_MASS, 7.0e7),
        Some(BROWN_DWARF_TEMPERATURE)
    );
}
