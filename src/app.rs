use eframe::egui;

use crate::constants::{
    ASTRONOMICAL_UNIT, KILOPARSEC, MILLIARCSECOND, SOLAR_LUMINOSITY, SOLAR_MASS,
};
use crate::icons::{Icon, icon_button};
use crate::lensing::{einstein_radius, image_magnifications, image_positions, magnification};
use crate::overlay::{Marks, ObjectInfo, draw_image_markers, draw_marks, kind_label};
use crate::physics::{
    ObjectKind, classify_object, density, effective_temperature, emitted_luminosity,
    schwarzschild_radius, spectral_type, star_color,
};
use crate::preset::{Preset, all_presets, stellar_black_hole, sun};
use crate::render::{LensParams, render};
use crate::sky::SkyImage;
use crate::worker::Worker;
use std::sync::Arc;

/// Máximo de píxeles de la imagen de la lente a toda calidad (una ventana QHD entera cabe).
const MAX_IMAGE_PIXELS: f32 = 4_000_000.0;
/// Lado más largo máximo mientras se arrastra un slider, para que responda fluido [píxeles].
const PREVIEW_IMAGE_SIDE: f32 = 480.0;
/// Lado mínimo de la imagen [píxeles].
const MIN_IMAGE_SIDE: usize = 128;
/// Los lados se redondean a múltiplos de este valor, para no redibujar por cada píxel
/// que se ensancha la ventana [píxeles].
const IMAGE_SIZE_STEP: f32 = 32.0;
/// Suavizado de la imagen a toda calidad: cada píxel promedia 2×2 puntos.
const FULL_QUALITY_SAMPLES: usize = 2;
/// Rango del campo de visión [mas].
const FOV_RANGE_MAS: (f64, f64) = (0.5, 1000.0);
/// Rango del desplazamiento de la estrella de fondo [mas].
const OFFSET_RANGE_MAS: (f64, f64) = (-10.0, 10.0);
/// Fotos de fondo incluidas dentro del programa (todas de NASA/ESA, dominio público):
/// nombre que se muestra y bytes del archivo. La primera es la de partida.
const PHOTOS: [(&str, &[u8]); 4] = [
    (
        "Hubble Ultra Deep Field",
        include_bytes!("../assets/hubble_ultra_deep_field.jpg"),
    ),
    (
        "Hubble Extreme Deep Field",
        include_bytes!("../assets/hubble_extreme_deep_field.png"),
    ),
    (
        "Pillars of Creation",
        include_bytes!("../assets/pillars_of_creation.jpg"),
    ),
    ("Crab Nebula", include_bytes!("../assets/crab_nebula.jpg")),
];

/// Escenas de partida listas para usar.
#[derive(Clone, Copy, PartialEq)]
enum Scene {
    PerfectRing,
    TwoImages,
    Microlensing,
    HubbleArcs,
}

impl Scene {
    const ALL: [Scene; 4] = [
        Scene::PerfectRing,
        Scene::TwoImages,
        Scene::Microlensing,
        Scene::HubbleArcs,
    ];

    fn label(self) -> &'static str {
        match self {
            Scene::PerfectRing => "Perfect Einstein ring",
            Scene::TwoImages => "Two images",
            Scene::Microlensing => "Microlensing pass (light curve)",
            Scene::HubbleArcs => "Hubble galaxies behind the lens",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Scene::PerfectRing => "Perfect alignment: the star becomes a full ring.",
            Scene::TwoImages => "Slight misalignment: the ring breaks into two images.",
            Scene::Microlensing => "Press Play: the star crosses behind the lens.",
            Scene::HubbleArcs => "Real galaxies stretched into arcs near the ring.",
        }
    }
}

pub struct App {
    /// La lente: siempre la misma, no se puede cambiar.
    lens: Preset,
    /// Distancia observador–fuente [kpc].
    source_distance_kpc: f64,
    /// Posición de la lente entre el observador (0) y la fuente (1).
    lens_position: f64,
    /// Desplazamiento de la fuente respecto al eje observador–lente [mas].
    source_offset_mas: f64,
    /// Radio angular de la estrella de fondo [mas]. Su tamaño real (~1e-6 mas) sería
    /// invisible, así que se exagera.
    source_radius_mas: f64,
    /// La estrella de fondo (solo pueden elegirse objetos que emiten luz).
    source: Preset,
    /// Multiplicador de la deflexión (1 = efecto real).
    k: f64,
    /// Campo de visión de la imagen [mas].
    fov_mas: f64,
    /// Foto de fondo ya decodificada.
    sky: Arc<SkyImage>,
    /// Hilo que calcula las imágenes.
    worker: Worker,
    /// Nombre de la foto de fondo actual (una incluida o la que subió el usuario).
    photo_name: String,
    /// Aviso si no se pudo abrir una foto.
    photo_error: Option<String>,
    /// Si el fondo es la foto (`true`) o una estrella con campo de estrellas (`false`).
    use_photo: bool,
    /// Ángulo que cubre el lado de la foto de fondo [mas].
    sky_span_mas: f64,
    /// Si se dibujan las marcas que distinguen el objeto elegido.
    show_marks: bool,
    /// Si la estrella se está moviendo sola a través del eje (animación).
    playing: bool,
    /// Velocidad de la animación [mas/s].
    play_speed_mas: f64,
    /// Escena elegida en el menú (solo para mostrar su nombre).
    scene: Option<Scene>,
    /// Mensaje de la última exportación.
    export_status: Option<String>,
    /// Última imagen dibujada, con los parámetros y el lado (en píxeles) que la generaron.
    image: Option<(LensParams, (usize, usize), egui::TextureHandle)>,
}

impl App {
    pub fn new(ctx: &egui::Context) -> Self {
        let sky = Arc::new(
            SkyImage::from_bytes(PHOTOS[0].1).expect("la foto de fondo incluida es válida"),
        );
        // Escena de partida: un agujero negro con el Sol justo detrás.
        Self {
            lens: stellar_black_hole(),
            source_distance_kpc: 8.0,
            lens_position: 0.5,
            source_offset_mas: 0.0,
            source_radius_mas: 0.15,
            source: sun(),
            k: 1.0,
            fov_mas: 12.0,
            worker: Worker::spawn(Arc::clone(&sky), ctx.clone()),
            photo_name: PHOTOS[0].0.to_string(),
            photo_error: None,
            sky,
            use_photo: false,
            sky_span_mas: 8.0,
            show_marks: true,
            playing: false,
            play_speed_mas: 1.5,
            scene: None,
            export_status: None,
            image: None,
        }
    }

    /// Marcas de la imagen: la estrella elegida en su posición real y, aparte, la lente.
    /// Con la foto de fondo no hay estrella, así que se marca la lente.
    fn marks(&self) -> Marks {
        if self.use_photo {
            Marks {
                object: object_info(&self.lens),
                offset_mas: 0.0,
                lens_name: None,
            }
        } else {
            Marks {
                object: object_info(&self.source),
                offset_mas: self.source_offset_mas,
                lens_name: Some(self.lens.name),
            }
        }
    }

    /// Temperatura de la estrella de fondo [K]; de ella sale su color.
    fn source_temperature(&self) -> f64 {
        effective_temperature(self.source.mass, self.source.radius)
            .expect("la estrella de fondo siempre emite luz")
    }

    /// Parámetros de la imagen según el estado actual.
    fn lens_params(&self) -> LensParams {
        let d_source = self.source_distance_kpc * KILOPARSEC;
        let d_lens = self.lens_position * d_source;
        let color = star_color(self.source_temperature());

        LensParams {
            theta_e: self.k * einstein_radius(self.lens.mass, d_lens, d_source),
            fov: self.fov_mas * MILLIARCSECOND,
            photo: self.use_photo,
            sky_span: self.sky_span_mas * MILLIARCSECOND,
            source_offset: self.source_offset_mas * MILLIARCSECOND,
            source_radius: self.source_radius_mas * MILLIARCSECOND,
            color: [color.r, color.g, color.b],
            stars: true,
            samples: 1,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("controls")
            .default_size(340.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.controls(ui));
            });

        egui::Panel::right("data")
            .default_size(280.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.data_panel(ui));
            });

        self.handle_dropped_files(ui.ctx());
        self.advance_animation(ui.ctx());

        let mut params = self.lens_params();
        let marks = self.marks();
        egui::CentralPanel::default().show(ui, |ui| {
            // La imagen ocupa todo el espacio libre, tenga la forma que tenga la ventana.
            let available = ui.available_size();

            // Se dibuja con tantos píxeles reales como ocupa en pantalla (puntos × escala
            // del monitor), para que se vea nítida en cualquier resolución.
            let pixels_per_point = ui.ctx().pixels_per_point();
            // Mientras se arrastra un slider se dibuja más pequeña y sin suavizar; al soltar,
            // a toda calidad.
            let dragging = ui.ctx().input(|i| i.pointer.any_down()) || self.playing;
            if !dragging {
                params.samples = FULL_QUALITY_SAMPLES;
            }
            let (width, height) = image_size(available * pixels_per_point, dragging);

            // El cálculo se hace en otro hilo: se pide la imagen que toca y mientras llega
            // se sigue mostrando la anterior, así la ventana nunca se queda congelada.
            self.worker.request(params, (width, height));
            if let Some(done) = self.worker.latest() {
                let image = egui::ColorImage::from_rgb([done.size.0, done.size.1], &done.pixels);
                let texture = ui
                    .ctx()
                    .load_texture("lens", image, egui::TextureOptions::LINEAR);
                self.image = Some((done.params, done.size, texture));
            }

            if let Some((_, _, texture)) = &self.image {
                let response = ui.add(
                    egui::Image::new(texture)
                        .fit_to_exact_size(available)
                        .sense(egui::Sense::click_and_drag()),
                );
                self.handle_image_input(ui, &response);
                if self.show_marks {
                    draw_marks(
                        &ui.painter_at(response.rect),
                        response.rect,
                        &marks,
                        params.theta_e / MILLIARCSECOND,
                        self.fov_mas,
                    );
                    if !self.use_photo && params.theta_e > 0.0 {
                        let (plus, minus) = image_positions(
                            self.source_offset_mas * MILLIARCSECOND,
                            params.theta_e,
                        );
                        draw_image_markers(
                            &ui.painter_at(response.rect),
                            response.rect,
                            plus / MILLIARCSECOND,
                            minus / MILLIARCSECOND,
                            self.fov_mas,
                        );
                    }
                }
            }
        });
    }
}

/// Ancho y alto de la imagen [píxeles] para ocupar `screen` píxeles de pantalla.
///
/// Se limita el total de píxeles (y, en la vista previa, el lado largo) manteniendo la
/// proporción, y se redondea a múltiplos de `IMAGE_SIZE_STEP`.
fn image_size(screen: egui::Vec2, preview: bool) -> (usize, usize) {
    let mut scale = (MAX_IMAGE_PIXELS / (screen.x * screen.y).max(1.0))
        .sqrt()
        .min(1.0);
    if preview {
        scale = scale.min(PREVIEW_IMAGE_SIDE / screen.x.max(screen.y).max(1.0));
    }
    let side = |v: f32| ((v * scale / IMAGE_SIZE_STEP).round() * IMAGE_SIZE_STEP) as usize;

    (
        side(screen.x).max(MIN_IMAGE_SIDE),
        side(screen.y).max(MIN_IMAGE_SIDE),
    )
}

impl App {
    /// Panel izquierdo: la lente y la geometría de la escena.
    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("GraviLens");

        ui.label(format!(
            "Lens (fixed): {} · {:.0} M_sol",
            self.lens.name,
            self.lens.mass / SOLAR_MASS
        ));

        ui.separator();
        egui::ComboBox::from_label("Example scenes")
            .selected_text(self.scene.map_or("Pick one…", Scene::label))
            .show_ui(ui, |ui| {
                for scene in Scene::ALL {
                    if ui
                        .selectable_label(self.scene == Some(scene), scene.label())
                        .on_hover_text(scene.description())
                        .clicked()
                    {
                        self.apply_scene(scene);
                    }
                }
            });
        if let Some(scene) = self.scene {
            ui.weak(scene.description());
        }

        ui.separator();
        ui.label("Scene");
        ui.add(
            scientific_slider(
                &mut self.source_distance_kpc,
                0.1..=100.0,
                "Distance to the star",
                " kpc",
            )
            .logarithmic(true),
        )
        .on_hover_text("How far away the background star is. A larger distance gives a smaller Einstein ring for the same lens position.");
        ui.add(scientific_slider(
            &mut self.lens_position,
            0.01..=0.99,
            "Lens position (0 = you, 1 = star)",
            "",
        ))
        .on_hover_text("Where the lens sits between you (0) and the star (1). The ring is largest when it is about halfway.");
        ui.add(scientific_slider(
            &mut self.source_offset_mas,
            OFFSET_RANGE_MAS.0..=OFFSET_RANGE_MAS.1,
            "Background offset",
            " mas",
        ))
        .on_hover_text("How far the star is from the line between you and the lens. 0 = perfect alignment = full ring. You can also drag the star on the image.");
        ui.add(
            scientific_slider(
                &mut self.fov_mas,
                FOV_RANGE_MAS.0..=FOV_RANGE_MAS.1,
                "Field of view",
                " mas",
            )
            .logarithmic(true),
        )
        .on_hover_text("Angle covered by the short side of the image. 1 mas = 1/1000 of an arcsecond. Scroll on the image to zoom.");
        ui.add(scientific_slider(&mut self.k, 1.0..=1.0e6, "Multiplier k", "").logarithmic(true))
            .on_hover_text("Multiplies the bending of light. 1 = real physics. Larger values exaggerate the ring so it is easier to see; see the Realism panel.");

        ui.separator();
        ui.label("Background (what is behind the lens)");
        ui.checkbox(&mut self.show_marks, "Show object marks");
        ui.radio_value(&mut self.use_photo, false, "A star behind the lens");
        ui.radio_value(&mut self.use_photo, true, "Real photo");
        if self.use_photo {
            egui::ComboBox::from_label("Photo")
                .selected_text(&self.photo_name)
                .show_ui(ui, |ui| {
                    for (name, bytes) in PHOTOS {
                        if ui.selectable_label(self.photo_name == name, name).clicked() {
                            self.load_photo(name.to_string(), bytes);
                        }
                    }
                });
            if icon_button(ui, Icon::Open, "Open your own photo…")
                .on_hover_text(
                    "JPEG or PNG. You can also drop a file on the window.                      The photo is only kept in memory: it is never saved or copied.",
                )
                .clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("Image", &["jpg", "jpeg", "png"])
                    .pick_file()
            {
                self.load_photo_file(&path);
            }
            if let Some(error) = &self.photo_error {
                ui.colored_label(egui::Color32::from_rgb(240, 110, 110), error);
            }
            ui.add(
                scientific_slider(&mut self.sky_span_mas, 0.5..=1000.0, "Photo size", " mas")
                    .logarithmic(true),
            );
        } else {
            // Solo se pueden poner de fondo los objetos que emiten luz propia.
            egui::ComboBox::from_label("Background star")
                .selected_text(self.source.name)
                .show_ui(ui, |ui| {
                    for preset in all_presets() {
                        if effective_temperature(preset.mass, preset.radius).is_none() {
                            continue;
                        }
                        if ui
                            .selectable_label(self.source.name == preset.name, preset.name)
                            .clicked()
                        {
                            self.source = preset;
                        }
                    }
                });
            ui.add(
                scientific_slider(
                    &mut self.source_radius_mas,
                    0.01..=1.0,
                    "Star size (exaggerated)",
                    " mas",
                )
                .logarithmic(true),
            );
        }

        if ui.button("Frame the Einstein ring").clicked() {
            self.frame_ring();
        }

        ui.separator();
        ui.label("Animation");
        if !self.use_photo {
            ui.horizontal(|ui| {
                let (icon, label) = if self.playing {
                    (Icon::Pause, "Pause")
                } else {
                    (Icon::Play, "Play")
                };
                if icon_button(ui, icon, label).clicked() {
                    self.playing = !self.playing;
                }
                if icon_button(ui, Icon::Restart, "Restart").clicked() {
                    self.source_offset_mas = OFFSET_RANGE_MAS.0;
                }
            });
            ui.add(
                scientific_slider(&mut self.play_speed_mas, 0.05..=20.0, "Speed", " mas/s")
                    .logarithmic(true),
            );
        } else {
            self.playing = false;
            ui.weak("Animation needs the single-star background.");
        }

        ui.separator();
        if icon_button(ui, Icon::Save, "Save image as PNG (1920×1080)")
            .on_hover_text("Writes a PNG file in the folder the program was started from.")
            .clicked()
        {
            self.export_png();
        }
        if let Some(status) = &self.export_status {
            ui.weak(status);
        }
        ui.separator();
        ui.weak(
            "On the image: drag the star to move it · scroll to zoom · double-click to frame \
             the ring.\nKeys: ← → move the star · C center it · F frame the ring.",
        );

        ui.separator();
        egui::CollapsingHeader::new("What am I looking at?")
            .default_open(true)
            .show(ui, |ui| {
                ui.label(
                    "Gravity bends light. A massive object between you and a distant star \
                     acts like a lens: it bends the star's light around itself, so you see \
                     the star in a different place, or several times.",
                );
                ui.add_space(4.0);
                ui.label(
                    "• Perfect alignment: the star is stretched into a full ring, the \
                     Einstein ring.\n\
                     • Small misalignment: two images, θ+ (bright, same side) and θ− \
                     (faint, opposite side).\n\
                     • Large misalignment: one image, almost unchanged.",
                );
                ui.add_space(4.0);
                ui.label(
                    "The ring size θE depends on the lens mass and on the distances. For a \
                     10 solar-mass black hole halfway to a star 8 kpc away it is about 3 \
                     milliarcseconds: far too small to see with a telescope, so the picture \
                     is a simulation of what the physics predicts.",
                );
                ui.add_space(4.0);
                ui.label(
                    "Try it: pick an example scene, drag the star across the ring and watch \
                     the light curve, or raise k to see the effect exaggerated.",
                );
                ui.add_space(4.0);
                ui.weak(
                    "mas = milliarcsecond (1/3 600 000 of a degree)\n\
                     kpc = kiloparsec (about 3 262 light-years)",
                );
            });
    }

    /// Cambia la foto de fondo. La foto solo vive en memoria: no se guarda nunca.
    fn load_photo(&mut self, name: String, bytes: &[u8]) {
        match SkyImage::from_bytes(bytes) {
            Ok(sky) => {
                self.sky = Arc::new(sky);
                self.worker.set_sky(Arc::clone(&self.sky));
                self.photo_name = name;
                self.photo_error = None;
                self.use_photo = true;
            }
            Err(error) => self.photo_error = Some(format!("Could not open the photo: {error}")),
        }
    }

    /// Abre una foto del usuario desde un archivo (JPEG o PNG).
    fn load_photo_file(&mut self, path: &std::path::Path) {
        let name = path
            .file_name()
            .map_or_else(|| "Your photo".to_string(), |n| n.to_string_lossy().into());
        match std::fs::read(path) {
            Ok(bytes) => self.load_photo(name, &bytes),
            Err(error) => self.photo_error = Some(format!("Could not read the file: {error}")),
        }
    }

    /// Fotos soltadas sobre la ventana: se usa la primera.
    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if let Some(file) = dropped.into_iter().next() {
            self.load_photo_file(file.path());
        }
    }

    /// Coloca todos los parámetros de una escena predefinida.
    fn apply_scene(&mut self, scene: Scene) {
        self.scene = Some(scene);
        self.source = sun();
        self.source_distance_kpc = 8.0;
        self.lens_position = 0.5;
        self.k = 1.0;
        self.source_radius_mas = 0.15;
        self.playing = false;
        self.use_photo = false;
        match scene {
            Scene::PerfectRing => {
                self.source_offset_mas = 0.0;
                self.frame_ring();
            }
            Scene::TwoImages => {
                self.source_offset_mas = 2.0;
                self.frame_ring();
            }
            Scene::Microlensing => {
                self.source_offset_mas = OFFSET_RANGE_MAS.0;
                self.fov_mas = 24.0;
                self.play_speed_mas = 2.0;
                self.playing = true;
            }
            Scene::HubbleArcs => {
                self.load_photo(PHOTOS[0].0.to_string(), PHOTOS[0].1);
                self.use_photo = true;
                self.sky_span_mas = 12.0;
                self.fov_mas = 12.0;
            }
        }
    }

    /// Guarda la imagen actual en 1920×1080 con la mejor calidad, en la carpeta desde la
    /// que se lanzó el programa. Solo se escribe cuando se pulsa el botón.
    fn export_png(&mut self) {
        let mut params = self.lens_params();
        params.samples = 3;
        let (width, height) = (1920, 1080);
        let pixels = render(&params, width, height, &self.sky);
        let seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let path = format!("gravilens_{seconds}.png");
        self.export_status = Some(
            match image::save_buffer(
                &path,
                &pixels,
                width as u32,
                height as u32,
                image::ColorType::Rgb8,
            ) {
                Ok(()) => format!("Saved {path}"),
                Err(error) => format!("Could not save: {error}"),
            },
        );
    }

    /// Mueve la estrella a lo largo del eje mientras suena el Play; al llegar al final
    /// vuelve a empezar.
    fn advance_animation(&mut self, ctx: &egui::Context) {
        if !self.playing || self.use_photo {
            return;
        }
        let dt = ctx.input(|i| i.stable_dt) as f64;
        self.source_offset_mas += self.play_speed_mas * dt;
        if self.source_offset_mas > OFFSET_RANGE_MAS.1 {
            self.source_offset_mas = OFFSET_RANGE_MAS.0;
        }
        ctx.request_repaint();
    }

    /// Encuadra el anillo: el campo de visión cubre el lado corto, así que se deja un
    /// margen de 0.6 θE a cada lado del diámetro 2 θE.
    fn frame_ring(&mut self) {
        let theta_e_mas = self.lens_params().theta_e / MILLIARCSECOND;
        if theta_e_mas > 0.0 {
            self.fov_mas = (3.2 * theta_e_mas).clamp(FOV_RANGE_MAS.0, FOV_RANGE_MAS.1);
        }
    }

    /// Interacción directa con la imagen: arrastrar la estrella, rueda para el zoom,
    /// doble clic y atajos de teclado.
    fn handle_image_input(&mut self, ui: &egui::Ui, response: &egui::Response) {
        let points_per_mas = response.rect.size().min_elem() as f64 / self.fov_mas;

        // Arrastrar: la estrella sigue al ratón (solo en horizontal, el eje de la escena).
        if !self.use_photo
            && (response.dragged() || response.is_pointer_button_down_on())
            && let Some(pointer) = response.interact_pointer_pos()
        {
            let offset = (pointer.x - response.rect.center().x) as f64 / points_per_mas;
            self.source_offset_mas = offset.clamp(OFFSET_RANGE_MAS.0, OFFSET_RANGE_MAS.1);
        }

        // Rueda: zoom logarítmico (cada paso multiplica el campo de visión).
        if response.hovered() {
            let scroll = ui.ctx().input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                self.fov_mas = (self.fov_mas * (-scroll as f64 * 0.002).exp())
                    .clamp(FOV_RANGE_MAS.0, FOV_RANGE_MAS.1);
            }
        }

        if response.double_clicked() {
            self.frame_ring();
        }

        // Teclado (no se atiende si se está escribiendo en un campo de texto).
        if !ui.ctx().egui_wants_keyboard_input() {
            let step = self.fov_mas * 0.01;
            let (left, right, center, frame) = ui.ctx().input(|i| {
                (
                    i.key_down(egui::Key::ArrowLeft),
                    i.key_down(egui::Key::ArrowRight),
                    i.key_pressed(egui::Key::C),
                    i.key_pressed(egui::Key::F),
                )
            });
            if !self.use_photo {
                if left {
                    self.source_offset_mas -= step;
                }
                if right {
                    self.source_offset_mas += step;
                }
                if center {
                    self.source_offset_mas = 0.0;
                }
                self.source_offset_mas = self
                    .source_offset_mas
                    .clamp(OFFSET_RANGE_MAS.0, OFFSET_RANGE_MAS.1);
            }
            if frame {
                self.frame_ring();
            }
            if left || right {
                ui.ctx().request_repaint();
            }
        }
        if response.dragged() || response.hovered() {
            ui.ctx().request_repaint();
        }
    }

    /// Panel derecho: el radio de Einstein y los datos de la lente y de la estrella.
    fn data_panel(&self, ui: &mut egui::Ui) {
        ui.heading("Einstein ring");
        let params = self.lens_params();
        let real_mas = einstein_radius(
            self.lens.mass,
            self.lens_position * self.source_distance_kpc * KILOPARSEC,
            self.source_distance_kpc * KILOPARSEC,
        ) / MILLIARCSECOND;
        ui.label(format!("θE real: {real_mas:.4} mas"));
        ui.label(format!(
            "θE drawn: {:.4} mas",
            params.theta_e / MILLIARCSECOND
        ));

        // Balanza de realismo: lo dibujado es k veces el efecto real.
        ui.separator();
        ui.heading("Realism");
        let real_share = 100.0 / self.k;
        if self.k <= 1.0 + 1e-9 {
            ui.colored_label(
                egui::Color32::from_rgb(110, 210, 130),
                "100% real: the ring is the physical size.",
            );
        } else {
            ui.colored_label(
                egui::Color32::from_rgb(235, 180, 70),
                format!(
                    "Exaggerated ×{:.3e}: only {:.3e}% of the drawn ring is real.",
                    self.k, real_share
                ),
            );
            ui.add(egui::ProgressBar::new((real_share / 100.0) as f32).text("real share"));
        }

        if !self.use_photo && params.theta_e > 0.0 {
            self.analysis_panel(ui, params.theta_e);
        }

        ui.separator();
        if self.use_photo {
            ui.heading("Background");
            ui.label(&self.photo_name);
        } else {
            ui.heading(format!("Background star: {}", self.source.name));
            show_data(ui, self.source.mass, self.source.radius);
        }

        ui.separator();
        ui.heading(format!("Lens: {}", self.lens.name));
        show_data(ui, self.lens.mass, self.lens.radius);

        ui.separator();
        self.physics_with_numbers(ui);
    }

    /// Las fórmulas de la escena con los números actuales.
    fn physics_with_numbers(&self, ui: &mut egui::Ui) {
        let d_source = self.source_distance_kpc * KILOPARSEC;
        let d_lens = self.lens_position * d_source;
        let theta_e = einstein_radius(self.lens.mass, d_lens, d_source);
        let rs = schwarzschild_radius(self.lens.mass);

        egui::CollapsingHeader::new("The physics, with your numbers")
            .default_open(true)
            .show(ui, |ui| {
                ui.label("Schwarzschild radius");
                ui.monospace("Rs = 2GM/c²");
                ui.label(format!("   = {:.2} km", rs / 1000.0));
                ui.label("Einstein radius");
                ui.monospace("θE = √(4GM/c² · D_ls/(D_l·D_s))");
                ui.label(format!(
                    "   D_l = {:.2} kpc, D_s = {:.2} kpc",
                    d_lens / KILOPARSEC,
                    d_source / KILOPARSEC
                ));
                ui.label(format!("   θE = {:.3} mas", theta_e / MILLIARCSECOND));
                ui.label(format!(
                    "   = a ring of radius {:.0} AU around the lens",
                    theta_e * d_lens / ASTRONOMICAL_UNIT
                ));
                ui.label("Magnification (point source)");
                ui.monospace("A(u) = (u²+2) / (u·√(u²+4))");
                ui.label("   u = β/θE  (distance in Einstein radii)");
                ui.label("Lens equation");
                ui.monospace("β = θ − θE²/θ");
                ui.label("Light bending");
                ui.monospace("α = 4GM/(c²·b)");
                ui.label(format!(
                    "   at b = 10 Rs: α = {:.2} arcsec",
                    4.0 * rs / 2.0 / (10.0 * rs) * 206_264.806
                ));
            });
    }
}

impl App {
    /// Posición y brillo de las dos imágenes, amplificación total y curva de luz.
    fn analysis_panel(&self, ui: &mut egui::Ui, theta_e: f64) {
        let beta = self.source_offset_mas * MILLIARCSECOND;
        let u = beta / theta_e;
        let (plus, minus) = image_positions(beta, theta_e);
        let (mu_plus, mu_minus) = image_magnifications(u);

        ui.separator();
        ui.heading("Images");
        ui.label(format!("u = β/θE: {:.3}", u.abs()));
        ui.label(format!(
            "Main image θ+: {:.3} mas ({})",
            plus / MILLIARCSECOND,
            times(mu_plus)
        ));
        ui.label(format!(
            "Faint image θ−: {:.3} mas ({})",
            minus / MILLIARCSECOND,
            times(mu_minus)
        ));
        ui.label(format!(
            "Total magnification A: {}",
            times(magnification(u))
        ));

        ui.separator();
        ui.heading("Light curve");
        light_curve(ui, u);
    }
}

/// Amplificación como texto: `×1.342`, o «∞» cuando la fuente puntual está alineada.
fn times(magnification: f64) -> String {
    if magnification.is_finite() {
        format!("×{magnification:.3}")
    } else {
        "∞, point source".to_string()
    }
}

/// Dibuja `A(u)` para `u` entre −3 y 3 con un punto en el instante actual.
fn light_curve(ui: &mut egui::Ui, u_now: f64) {
    const U_MAX: f64 = 3.0;
    const A_MAX: f64 = 6.0;
    let width = ui.available_width().max(120.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 130.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 4.0, egui::Color32::from_gray(20));

    let to_screen = |u: f64, a: f64| {
        egui::pos2(
            rect.left() + ((u / U_MAX + 1.0) / 2.0) as f32 * rect.width(),
            rect.bottom()
                - 6.0
                - ((a - 1.0) / (A_MAX - 1.0)).clamp(0.0, 1.0) as f32 * (rect.height() - 14.0),
        )
    };
    // Línea base A = 1 (sin lente).
    painter.line_segment(
        [to_screen(-U_MAX, 1.0), to_screen(U_MAX, 1.0)],
        egui::Stroke::new(1.0, egui::Color32::from_gray(80)),
    );
    let points: Vec<egui::Pos2> = (0..=240)
        .map(|i| {
            let u = -U_MAX + 2.0 * U_MAX * i as f64 / 240.0;
            to_screen(u, magnification(u))
        })
        .collect();
    painter.add(egui::Shape::line(
        points,
        egui::Stroke::new(1.8, egui::Color32::from_rgb(120, 190, 255)),
    ));
    let now = to_screen(u_now.clamp(-U_MAX, U_MAX), magnification(u_now));
    painter.line_segment(
        [
            egui::pos2(now.x, rect.top()),
            egui::pos2(now.x, rect.bottom()),
        ],
        egui::Stroke::new(1.0, egui::Color32::from_white_alpha(70)),
    );
    painter.circle_filled(now, 4.5, egui::Color32::from_rgb(255, 200, 80));
    painter.text(
        rect.left_top() + egui::vec2(4.0, 2.0),
        egui::Align2::LEFT_TOP,
        "A (brightness) vs u",
        egui::FontId::proportional(11.0),
        egui::Color32::from_gray(170),
    );
}

/// Datos del objeto para dibujar sus marcas.
fn object_info(preset: &Preset) -> ObjectInfo {
    let temperature = effective_temperature(preset.mass, preset.radius);
    ObjectInfo {
        name: preset.name,
        kind: classify_object(preset.mass, preset.radius),
        mass: preset.mass,
        radius: preset.radius,
        temperature,
        color: temperature.map(|t| {
            let c = star_color(t);
            egui::Color32::from_rgb(c.r, c.g, c.b)
        }),
    }
}

/// Panel de datos en vivo. Se convierte a unidades "humanas" solo aquí, al mostrar.
fn show_data(ui: &mut egui::Ui, mass: f64, radius: f64) {
    let kind = classify_object(mass, radius);

    ui.label(format!("Mass: {:.3} M_sol", mass / SOLAR_MASS));
    ui.label(format!("Radius: {:.3e} km", radius / 1000.0));
    ui.label(format!("Object type: {}", kind_label(&kind)));
    // 1 kg/m³ = 0.001 g/cm³
    ui.label(format!(
        "Density: {:.3e} g/cm³",
        density(mass, radius) / 1000.0
    ));
    ui.label(format!(
        "Schwarzschild radius: {:.3} km",
        schwarzschild_radius(mass) / 1000.0
    ));

    match effective_temperature(mass, radius) {
        Some(temperature) => {
            ui.label(format!("Temperature: {temperature:.0} K"));
            if let Some(luminosity) = emitted_luminosity(mass, radius) {
                ui.label(format!(
                    "Luminosity: {:.3e} L_sol",
                    luminosity / SOLAR_LUMINOSITY
                ));
            }
            // El tipo espectral O…M solo describe estrellas de secuencia principal.
            if kind == ObjectKind::MainSequenceStar {
                ui.label(format!("Spectral type: {:?}", spectral_type(temperature)));
            }
            let color = star_color(temperature);
            ui.colored_label(
                egui::Color32::from_rgb(color.r, color.g, color.b),
                "● Apparent color",
            );
        }
        None => {
            ui.label("No light of its own");
        }
    }
}

/// Slider que muestra el número en notación científica (`1.989e30`) y acepta que se
/// teclee así. Rust ya sabe leer `1.989e30` con `parse::<f64>()`.
fn scientific_slider<'a>(
    value: &'a mut f64,
    range: std::ops::RangeInclusive<f64>,
    text: &'static str,
    suffix: &'static str,
) -> egui::Slider<'a> {
    egui::Slider::new(value, range)
        .text(text)
        .suffix(suffix)
        .custom_formatter(|v, _| format!("{v:.3e}"))
        .custom_parser(|s| s.trim().replace(',', ".").parse::<f64>().ok())
}
