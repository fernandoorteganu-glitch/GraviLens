use std::f64::consts::PI;

use crate::constants::{
    BROWN_DWARF_TEMPERATURE, GRAVITATIONAL_CONSTANT, JUPITER_MASS, NEUTRON_STAR_TEMPERATURE,
    SOLAR_LUMINOSITY, SOLAR_MASS, SPEED_OF_LIGHT, STEFAN_BOLTZMANN, WHITE_DWARF_TEMPERATURE,
};

pub fn density(mass: f64, radius: f64) -> f64 {
    let volume: f64 = (4.0 / 3.0) * PI * radius.powi(3);

    mass / volume
}

pub fn luminosity(mass: f64) -> f64 {
    let ratio: f64 = mass / SOLAR_MASS;
    let ratio: f64 = ratio.powf(3.5);
    ratio * SOLAR_LUMINOSITY
}

pub fn temperature(luminosity: f64, radius: f64) -> f64 {
    let denominator = 4.0 * PI * radius.powi(2) * STEFAN_BOLTZMANN;

    (luminosity / denominator).powf(1.0 / 4.0)
}

#[derive(Debug, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub fn star_color(temperature: f64) -> Color {
    let t: f64 = temperature / 100.0;

    let red: f64 = if t <= 66.0 {
        255.0
    } else {
        329.698727446 * (t - 60.0).powf(-0.1332047592)
    };

    let green: f64 = if t <= 66.0 {
        99.4708025861 * t.ln() - 161.1195681661
    } else {
        288.1221695283 * (t - 60.0).powf(-0.075514849)
    };

    let blue: f64 = if t >= 66.0 {
        255.0
    } else if t <= 19.0 {
        0.0
    } else {
        138.5177312231 * (t - 10.0).ln() - 305.0447927307
    };

    Color {
        r: red.clamp(0.0, 255.0) as u8,
        g: green.clamp(0.0, 255.0) as u8,
        b: blue.clamp(0.0, 255.0) as u8,
    }
}
#[derive(Debug, PartialEq)]
pub enum SpectralType {
    O,
    B,
    A,
    F,
    G,
    K,
    M,
}

pub fn spectral_type(temperature: f64) -> SpectralType {
    match temperature {
        t if t >= 30000.0 => SpectralType::O,
        t if t >= 10000.0 => SpectralType::B,
        t if t >= 7500.0 => SpectralType::A,
        t if t >= 6000.0 => SpectralType::F,
        t if t >= 5200.0 => SpectralType::G,
        t if t >= 3700.0 => SpectralType::K,
        _ => SpectralType::M,
    }
}
#[derive(Debug, PartialEq)]
pub enum ObjectKind {
    BlackHole,
    NeutronStar,
    WhiteDwarf,
    MainSequenceStar,
    BrownDwarf,
    Planet,
}

pub fn schwarzschild_radius(mass: f64) -> f64 {
    (2.0 * GRAVITATIONAL_CONSTANT * mass) / SPEED_OF_LIGHT.powi(2)
}

pub fn classify_object(mass: f64, radius: f64) -> ObjectKind {
    let rs = schwarzschild_radius(mass);
    let density = density(mass, radius);

    if radius <= rs {
        ObjectKind::BlackHole
    } else if density >= 1e17 {
        ObjectKind::NeutronStar
    } else if density >= 1e9 {
        ObjectKind::WhiteDwarf
    } else if mass >= 0.08 * SOLAR_MASS {
        ObjectKind::MainSequenceStar
    } else if mass >= 13.0 * JUPITER_MASS {
        ObjectKind::BrownDwarf
    } else {
        ObjectKind::Planet
    }
}

/// Temperatura efectiva [K] según el tipo de objeto, o `None` si no emite luz propia.
///
/// - Secuencia principal: `L ∝ M^3.5` y Stefan-Boltzmann.
/// - Enana marrón, enana blanca y estrella de neutrones: temperatura típica fija
///   (en la realidad depende de la edad, que no conocemos).
/// - Planeta y agujero negro: `None` (se ignoran la radiación de Hawking y el disco de acreción).
pub fn effective_temperature(mass: f64, radius: f64) -> Option<f64> {
    match classify_object(mass, radius) {
        ObjectKind::MainSequenceStar => Some(temperature(luminosity(mass), radius)),
        ObjectKind::BrownDwarf => Some(BROWN_DWARF_TEMPERATURE),
        ObjectKind::WhiteDwarf => Some(WHITE_DWARF_TEMPERATURE),
        ObjectKind::NeutronStar => Some(NEUTRON_STAR_TEMPERATURE),
        ObjectKind::BlackHole | ObjectKind::Planet => None,
    }
}

/// Luminosidad emitida [W] con `L = 4πR²σT⁴`, o `None` si no emite luz propia.
pub fn emitted_luminosity(mass: f64, radius: f64) -> Option<f64> {
    effective_temperature(mass, radius)
        .map(|t| 4.0 * PI * radius.powi(2) * STEFAN_BOLTZMANN * t.powi(4))
}

#[cfg(test)]
mod tests;
