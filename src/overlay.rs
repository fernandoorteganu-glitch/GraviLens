//! Marcas dibujadas encima de la imagen para distinguir el objeto elegido.
//!
//! Son formas vectoriales (círculos, líneas, texto) y no parte de los píxeles de la
//! lente, así que se ven nítidas con cualquier resolución.

use eframe::egui::{self, Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, vec2};

use crate::physics::ObjectKind;

/// Lo que hace falta saber del objeto para dibujarlo y describirlo.
pub struct ObjectInfo {
    /// Nombre del preset.
    pub name: &'static str,
    pub kind: ObjectKind,
    /// Masa [kg].
    pub mass: f64,
    /// Radio [m].
    pub radius: f64,
    /// Temperatura efectiva [K], si emite luz.
    pub temperature: Option<f64>,
    /// Color aparente si emite luz.
    pub color: Option<Color32>,
}

/// Todo lo que se marca sobre la imagen.
pub struct Marks {
    /// El objeto destacado: la estrella de fondo, o la lente si el fondo es la foto.
    pub object: ObjectInfo,
    /// Dónde está ese objeto respecto al eje observador–lente [mas].
    pub offset_mas: f64,
    /// Nombre de la lente, cuando el objeto destacado es otro (la estrella de fondo).
    pub lens_name: Option<&'static str>,
}

/// Nombre del tipo de objeto para mostrar en pantalla.
pub fn kind_label(kind: &ObjectKind) -> &'static str {
    match kind {
        ObjectKind::BlackHole => "Black hole",
        ObjectKind::NeutronStar => "Neutron star",
        ObjectKind::WhiteDwarf => "White dwarf",
        ObjectKind::MainSequenceStar => "Main-sequence star",
        ObjectKind::BrownDwarf => "Brown dwarf",
        ObjectKind::Planet => "Planet",
    }
}

/// Dibuja todas las marcas sobre la imagen que ocupa `rect`.
///
/// - `theta_e_mas`: radio de Einstein dibujado [mas]
/// - `fov_mas`: ángulo que cubre el lado más corto de la imagen [mas]
pub fn draw_marks(painter: &Painter, rect: Rect, marks: &Marks, theta_e_mas: f64, fov_mas: f64) {
    let info = &marks.object;
    let center = rect.center();
    let points_per_mas = rect.size().min_elem() / fov_mas as f32;
    // El objeto destacado está en su posición real; la lente, siempre en el centro.
    let pos = center + vec2(marks.offset_mas as f32 * points_per_mas, 0.0);
    let radius = visual_radius(info.radius);

    // Círculo punteado en θE: dónde debería formarse el anillo.
    let ring_radius = theta_e_mas as f32 * points_per_mas;
    if ring_radius.is_finite() && ring_radius > 1.0 {
        dotted_circle(painter, center, ring_radius);
        painter.text(
            center + vec2(1.0, -1.0) * (ring_radius * std::f32::consts::FRAC_1_SQRT_2 + 4.0),
            Align2::LEFT_BOTTOM,
            "θE",
            FontId::proportional(14.0),
            Color32::from_white_alpha(200),
        );
    }

    // Si la estrella no está justo detrás de la lente, se señala dónde está la lente.
    if let Some(lens_name) = marks.lens_name.filter(|_| marks.offset_mas != 0.0) {
        painter.circle_stroke(
            center,
            6.0,
            Stroke::new(1.5, Color32::from_white_alpha(170)),
        );
        painter.text(
            center + vec2(10.0, 8.0),
            Align2::LEFT_TOP,
            format!("Lens: {lens_name}"),
            FontId::proportional(12.0),
            Color32::from_white_alpha(190),
        );
    }

    // El objeto destacado, a escala exagerada.
    draw_object(painter, pos, radius, info);

    // Etiqueta con línea hasta el objeto.
    let label = format!("{} · {}", info.name, kind_label(&info.kind));
    let anchor = pos + vec2(radius + 6.0, -(radius + 6.0));
    let label_pos = anchor + vec2(26.0, -26.0);
    painter.line_segment(
        [anchor, label_pos],
        Stroke::new(1.0, Color32::from_white_alpha(170)),
    );
    let galley = painter.layout_no_wrap(label, FontId::proportional(15.0), Color32::WHITE);
    let text_rect = Rect::from_min_size(
        label_pos - vec2(0.0, galley.size().y) + vec2(4.0, 0.0),
        galley.size(),
    )
    .expand2(vec2(5.0, 2.0));
    painter.rect_filled(text_rect, 4.0, Color32::from_black_alpha(170));
    painter.galley(text_rect.min + vec2(5.0, 2.0), galley, Color32::WHITE);

    draw_card(painter, rect, marks, radius);
}

/// Marca con una cruz las dos imágenes de la estrella de fondo (θ+ y θ−), situadas en
/// el eje horizontal. Posiciones respecto a la lente [mas].
pub fn draw_image_markers(
    painter: &Painter,
    rect: Rect,
    plus_mas: f64,
    minus_mas: f64,
    fov_mas: f64,
) {
    let points_per_mas = rect.size().min_elem() / fov_mas as f32;
    for (label, mas) in [("θ+", plus_mas), ("θ−", minus_mas)] {
        let pos = rect.center() + vec2(mas as f32 * points_per_mas, 0.0);
        if !rect.contains(pos) {
            continue;
        }
        let stroke = Stroke::new(1.5, Color32::from_rgb(255, 210, 90));
        painter.circle_stroke(pos, 9.0, stroke);
        painter.text(
            pos + vec2(0.0, 12.0),
            Align2::CENTER_TOP,
            label,
            FontId::proportional(13.0),
            Color32::from_rgb(255, 210, 90),
        );
    }
}

/// Radio con el que se dibuja el objeto [puntos]. Su tamaño real es invisible (mucho
/// menor que un píxel), así que se exagera: de 5 a 16 puntos según el logaritmo del
/// radio real, entre 1 km y 2·10⁹ m.
fn visual_radius(radius_m: f64) -> f32 {
    let t = (radius_m.clamp(1.0e3, 2.0e9).log10() - 3.0) / (2.0e9_f64.log10() - 3.0);
    5.0 + 11.0 * t as f32
}

/// Circunferencia de puntos de radio `radius` [puntos].
fn dotted_circle(painter: &Painter, center: Pos2, radius: f32) {
    let segments = 120;
    let stroke = Stroke::new(1.5, Color32::from_white_alpha(150));
    for i in (0..segments).step_by(2) {
        let a0 = i as f32 / segments as f32 * std::f32::consts::TAU;
        let a1 = (i + 1) as f32 / segments as f32 * std::f32::consts::TAU;
        painter.line_segment(
            [
                center + radius * vec2(a0.cos(), a0.sin()),
                center + radius * vec2(a1.cos(), a1.sin()),
            ],
            stroke,
        );
    }
}

/// Mezcla dos colores: `t = 0` da `a` y `t = 1` da `b`.
fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let channel = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(
        channel(a.r(), b.r()),
        channel(a.g(), b.g()),
        channel(a.b(), b.b()),
    )
}

/// Dibuja el objeto con el aspecto de su tipo.
fn draw_object(painter: &Painter, center: Pos2, radius: f32, info: &ObjectInfo) {
    match info.kind {
        ObjectKind::BlackHole => {
            // Disco negro con un anillo de fotones y un resplandor naranja del disco de acreción.
            let orange = Color32::from_rgb(255, 150, 40);
            painter.circle_stroke(
                center,
                radius * 1.8,
                Stroke::new(1.0, orange.gamma_multiply(0.35)),
            );
            painter.circle_stroke(
                center,
                radius * 1.4,
                Stroke::new(2.0, orange.gamma_multiply(0.8)),
            );
            painter.circle_filled(center, radius, Color32::BLACK);
            painter.circle_stroke(
                center,
                radius * 1.05,
                Stroke::new(1.5, Color32::from_white_alpha(230)),
            );
        }
        ObjectKind::Planet => {
            // Esfera con luz desde arriba a la izquierda: círculos cada vez más pequeños y claros.
            let dark = Color32::from_rgb(40, 60, 90);
            let light = Color32::from_rgb(170, 200, 230);
            let steps = 8;
            for i in 0..steps {
                let t = i as f32 / (steps - 1) as f32;
                painter.circle_filled(
                    center - vec2(radius * 0.3, radius * 0.3) * t,
                    radius * (1.0 - 0.75 * t),
                    mix(dark, light, t),
                );
            }
        }
        _ => {
            // Objeto que brilla: resplandor suave y núcleo del color de su temperatura.
            let color = info.color.unwrap_or(Color32::WHITE);
            for i in (1..=6).rev() {
                painter.circle_filled(
                    center,
                    radius * (1.0 + 0.45 * i as f32),
                    color.gamma_multiply(0.07),
                );
            }
            painter.circle_filled(center, radius, color);
            painter.circle_filled(center, radius * 0.55, mix(color, Color32::WHITE, 0.6));
            if info.kind == ObjectKind::NeutronStar {
                // Los dos haces de un púlsar.
                let beam = Stroke::new(1.5, color.gamma_multiply(0.8));
                painter.line_segment(
                    [
                        center + vec2(-3.0, 3.0) * radius,
                        center + vec2(3.0, -3.0) * radius,
                    ],
                    beam,
                );
            }
        }
    }
}

/// Recuadro en la esquina inferior izquierda con el objeto y sus datos principales.
fn draw_card(painter: &Painter, rect: Rect, marks: &Marks, radius: f32) {
    let info = &marks.object;
    let height = if marks.lens_name.is_some() {
        130.0
    } else {
        112.0
    };
    let size = vec2(290.0, height);
    let card = Rect::from_min_size(
        Pos2::new(rect.min.x + 12.0, rect.max.y - 12.0 - size.y),
        size,
    );
    painter.rect_filled(card, 8.0, Color32::from_black_alpha(190));
    painter.rect_stroke(
        card,
        8.0,
        Stroke::new(1.0, Color32::from_white_alpha(60)),
        egui::StrokeKind::Inside,
    );

    // Objeto algo más grande que en el centro, a la izquierda.
    let icon_center = card.min + vec2(40.0, size.y / 2.0);
    // El resplandor se recorta al recuadro para que no se salga de él.
    draw_object(
        &painter.with_clip_rect(card),
        icon_center,
        (radius * 1.5).min(28.0),
        info,
    );

    let text_color = Color32::from_gray(225);
    let left = card.min.x + 82.0;
    let temperature = match info.temperature {
        Some(t) => format!("T: {t:.0} K"),
        None => "No light of its own".to_string(),
    };
    let lines = [
        (info.name.to_string(), 15.0, Color32::WHITE),
        (kind_label(&info.kind).to_string(), 13.0, text_color),
        (format!("M: {:.3e} kg", info.mass), 13.0, text_color),
        (
            format!("R: {:.3e} km", info.radius / 1000.0),
            13.0,
            text_color,
        ),
        (temperature, 13.0, text_color),
    ]
    .into_iter()
    .chain(
        marks
            .lens_name
            .map(|name| (format!("Lens: {name}"), 13.0, Color32::from_gray(160))),
    );
    for (i, (text, font_size, color)) in lines.enumerate() {
        painter.text(
            Pos2::new(left, card.min.y + 10.0 + i as f32 * 18.0),
            Align2::LEFT_TOP,
            text,
            FontId::proportional(font_size),
            color,
        );
    }
}
