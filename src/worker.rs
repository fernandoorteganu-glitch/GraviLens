//! Hilo de cálculo aparte: la ventana nunca espera a que termine una imagen.
//!
//! La interfaz manda trabajos por un canal (`mpsc`) y el hilo devuelve las imágenes
//! terminadas por otro. Si llegan varios trabajos mientras se calcula uno, solo se
//! hace el último: los anteriores ya no interesan.

use std::sync::{Arc, mpsc};

use eframe::egui;

use crate::render::{LensParams, render};
use crate::sky::SkyImage;

/// Tamaño de una imagen: ancho y alto [píxeles].
pub type Size = (usize, usize);

/// Imagen terminada, con lo que se pidió para saber a qué petición responde.
pub struct Finished {
    pub params: LensParams,
    pub size: Size,
    pub pixels: Vec<u8>,
}

/// Mensajes que recibe el hilo.
enum Message {
    /// Cambiar la foto de fondo.
    Sky(Arc<SkyImage>),
    /// Calcular una imagen.
    Render(LensParams, Size),
}

pub struct Worker {
    jobs: mpsc::Sender<Message>,
    results: mpsc::Receiver<Finished>,
    /// Última petición enviada, para no repetirla en cada fotograma.
    requested: Option<(LensParams, Size)>,
}

impl Worker {
    /// Arranca el hilo. `ctx` sirve para despertar la ventana cuando hay imagen nueva.
    pub fn spawn(mut sky: Arc<SkyImage>, ctx: egui::Context) -> Self {
        let (jobs, job_rx) = mpsc::channel::<Message>();
        let (result_tx, results) = mpsc::channel();

        std::thread::spawn(move || {
            // `recv` espera sin gastar CPU; termina cuando la app cierra el canal.
            while let Ok(first) = job_rx.recv() {
                // Si hay más trabajos esperando, nos quedamos con el más reciente de
                // imagen; los cambios de foto sí se aplican todos.
                let mut job = None;
                for message in std::iter::once(first).chain(job_rx.try_iter()) {
                    match message {
                        Message::Sky(new_sky) => sky = new_sky,
                        Message::Render(params, size) => job = Some((params, size)),
                    }
                }
                let Some((params, size)) = job else {
                    continue;
                };
                let pixels = render(&params, size.0, size.1, &sky);
                if result_tx
                    .send(Finished {
                        params,
                        size,
                        pixels,
                    })
                    .is_err()
                {
                    break;
                }
                ctx.request_repaint();
            }
        });

        Self {
            jobs,
            results,
            requested: None,
        }
    }

    /// Pide una imagen (si no se pidió ya idéntica).
    pub fn request(&mut self, params: LensParams, size: Size) {
        if self.requested != Some((params, size)) {
            self.requested = Some((params, size));
            // Solo falla si el hilo murió; entonces simplemente no habrá imagen nueva.
            let _ = self.jobs.send(Message::Render(params, size));
        }
    }

    /// Cambia la foto de fondo; la siguiente petición se recalcula con ella.
    pub fn set_sky(&mut self, sky: Arc<SkyImage>) {
        self.requested = None;
        let _ = self.jobs.send(Message::Sky(sky));
    }

    /// Última imagen terminada, si hay alguna nueva.
    pub fn latest(&self) -> Option<Finished> {
        self.results.try_iter().last()
    }
}
