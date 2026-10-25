# GraviLens

Simulador de lente gravitacional de escritorio, escrito en Rust con `egui`/`eframe`.
Un agujero negro de 10 masas solares curva la luz de una estrella (o de una foto real del
Hubble) situada detrás: se ve el **anillo de Einstein**, las dos imágenes de una fuente
desalineada y la curva de luz de una microlente.

## Qué hace

- Elegir la estrella de fondo (Sol, Sirio A/B, estrella de neutrones…), su distancia, la
  posición de la lente y el desplazamiento de la estrella.
- Arrastrar la estrella con el ratón, zoom con la rueda, doble clic para encuadrar el anillo.
- Animación Play: la estrella cruza por detrás de la lente y se dibuja la curva de luz `A(u)`.
- Panel de imágenes: posición y brillo de θ+ y θ−, amplificación total.
- Balanza de realismo: el multiplicador `k` exagera el efecto y el programa dice cuánto
  del anillo dibujado es real.
- Cuatro fotos reales de fondo, o abre la tuya (JPEG/PNG, también soltándola sobre la ventana): solo se usa en memoria, nunca se guarda ni se copia.
- Escenas de ejemplo y exportación a PNG (solo al pulsar el botón; el programa no guarda nada por sí solo).

## Cómo ejecutarlo

```
cargo run --release
cargo test
```

## Alcance físico (limitaciones)

Lente puntual, objetos quietos y sin rotación, **aproximación de campo débil**
(ángulo de deflexión `α = 4GM / (c²·b)`). Sin agujeros negros en rotación (Kerr) y sin
lentes múltiples. El tamaño de la estrella de fondo se exagera para que sea visible.

## Estructura

`physics.rs` (objetos y clasificación), `lensing.rs` (θE, ecuación de la lente,
amplificación), `render.rs` (trazado inverso en CPU), `worker.rs` (hilo de cálculo),
`app.rs` (interfaz). Detalles de diseño en `NOTES.md`; `tools/validate_lenstronomy.py`
compara con `lenstronomy`.

## Créditos

- Fotos de fondo (Hubble Ultra Deep Field, eXtreme Deep Field, Pilares de la Creación, Nebulosa
  del Cangrejo): NASA, ESA y colaboradores; dominio público. Detalle en `assets/CREDITS.md`.
- Interfaz: [egui](https://github.com/emilk/egui).

## Licencia

MIT (ver `LICENSE`). Las imágenes de `assets/` son de dominio público; ver `assets/CREDITS.md`.
