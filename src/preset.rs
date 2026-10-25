use crate::constants::{JUPITER_MASS, SOLAR_MASS, SOLAR_RADIUS};
use crate::physics::schwarzschild_radius;

pub struct Preset {
    pub name: &'static str,
    pub mass: f64,
    pub radius: f64,
}

pub fn sun() -> Preset {
    Preset {
        name: "Sun",
        mass: SOLAR_MASS,
        radius: SOLAR_RADIUS,
    }
}

/// Sirio A: 2.063 M☉ y 1.711 R☉ (Bond et al. 2017).
pub fn sirius_a() -> Preset {
    Preset {
        name: "Sirius A",
        mass: 2.063 * SOLAR_MASS,
        radius: 1.711 * SOLAR_RADIUS,
    }
}

/// Sirio B, enana blanca: 1.018 M☉ y 0.0084 R☉ (Bond et al. 2017).
pub fn sirius_b() -> Preset {
    Preset {
        name: "Sirius B",
        mass: 1.018 * SOLAR_MASS,
        radius: 0.0084 * SOLAR_RADIUS,
    }
}

/// Estrella de neutrones "típica": 1.4 M☉ y 12 km (valores canónicos).
pub fn neutron_star() -> Preset {
    Preset {
        name: "Neutron star",
        mass: 1.4 * SOLAR_MASS,
        radius: 12_000.0,
    }
}

/// Agujero negro estelar de 10 M☉. Su "radio" es el del horizonte: Rs = 2GM/c².
pub fn stellar_black_hole() -> Preset {
    let mass = 10.0 * SOLAR_MASS;
    Preset {
        name: "Stellar black hole",
        mass,
        radius: schwarzschild_radius(mass),
    }
}

/// Júpiter: 1.898e27 kg y radio medio de 6.9911e7 m (NASA).
pub fn jupiter() -> Preset {
    Preset {
        name: "Jupiter",
        mass: JUPITER_MASS,
        radius: 6.9911e7,
    }
}

pub fn all_presets() -> [Preset; 6] {
    [
        sun(),
        sirius_a(),
        sirius_b(),
        neutron_star(),
        stellar_black_hole(),
        jupiter(),
    ]
}

#[cfg(test)]
mod tests;
