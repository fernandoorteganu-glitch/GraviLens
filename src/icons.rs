//! Iconos sencillos dibujados con formas vectoriales (sin emojis ni fuentes especiales).
//! Se pintan con el color gris del texto de cada botón.

use eframe::egui::{self, Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, vec2};

/// Iconos disponibles.
#[derive(Clone, Copy)]
pub enum Icon {
    Open,
    Play,
    Pause,
    Restart,
    Save,
}

/// Lado del cuadrado donde se dibuja el icono [puntos].
const ICON_SIZE: f32 = 14.0;
/// Separación entre el icono y el texto [puntos].
const GAP: f32 = 6.0;

/// Botón con un icono vectorial a la izquierda del texto.
pub fn icon_button(ui: &mut egui::Ui, icon: Icon, text: &str) -> Response {
    let padding = ui.spacing().button_padding;
    let font = egui::TextStyle::Button.resolve(ui.style());
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_string(), font, Color32::PLACEHOLDER);
    let size = vec2(
        padding.x * 2.0 + ICON_SIZE + GAP + galley.size().x,
        (galley.size().y + padding.y * 2.0).max(ui.spacing().interact_size.y),
    );
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = *ui.style().interact(&response);
        let painter = ui.painter();
        painter.rect(
            rect,
            visuals.corner_radius,
            visuals.weak_bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
        let color = visuals.text_color();
        let icon_rect = Rect::from_center_size(
            Pos2::new(rect.left() + padding.x + ICON_SIZE / 2.0, rect.center().y),
            vec2(ICON_SIZE, ICON_SIZE),
        );
        draw_icon(painter, icon, icon_rect, color);
        painter.galley(
            Pos2::new(
                rect.left() + padding.x + ICON_SIZE + GAP,
                rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            color,
        );
    }
    response
}

/// Dibuja el icono dentro de `r`.
fn draw_icon(painter: &Painter, icon: Icon, r: Rect, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    let at = |x: f32, y: f32| Pos2::new(r.left() + x * r.width(), r.top() + y * r.height());
    let triangle = |a: Pos2, b: Pos2, c: Pos2| {
        painter.add(Shape::convex_polygon(vec![a, b, c], color, Stroke::NONE));
    };
    match icon {
        Icon::Play => triangle(at(0.2, 0.05), at(0.2, 0.95), at(0.92, 0.5)),
        Icon::Pause => {
            painter.rect_filled(Rect::from_min_max(at(0.18, 0.1), at(0.42, 0.9)), 1.0, color);
            painter.rect_filled(Rect::from_min_max(at(0.58, 0.1), at(0.82, 0.9)), 1.0, color);
        }
        Icon::Restart => {
            painter.line_segment([at(0.15, 0.1), at(0.15, 0.9)], stroke);
            triangle(at(0.9, 0.1), at(0.9, 0.9), at(0.3, 0.5));
        }
        Icon::Save => {
            // Flecha hacia abajo sobre una bandeja.
            painter.line_segment([at(0.5, 0.05), at(0.5, 0.62)], stroke);
            painter.line_segment([at(0.22, 0.38), at(0.5, 0.66)], stroke);
            painter.line_segment([at(0.78, 0.38), at(0.5, 0.66)], stroke);
            painter.line_segment([at(0.1, 0.92), at(0.9, 0.92)], stroke);
        }
        Icon::Open => {
            // Carpeta: pestaña y cuerpo.
            painter.add(Shape::closed_line(
                vec![
                    at(0.05, 0.15),
                    at(0.4, 0.15),
                    at(0.5, 0.3),
                    at(0.95, 0.3),
                    at(0.95, 0.88),
                    at(0.05, 0.88),
                ],
                stroke,
            ));
        }
    }
}
