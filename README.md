# GraviLens

Desktop gravitational lensing simulator, written in Rust with `egui`/`eframe`.
A 10-solar-mass black hole bends the light of a star (or a real Hubble photo) behind it.
You can see the **Einstein ring**, the two images of a misaligned source, and the light
curve of a microlensing event.

## Features

- Choose the background star (Sun, Sirius A/B, a neutron star…), its distance, the lens
  position and the star's offset.
- Drag the star with the mouse, zoom with the scroll wheel, double-click to frame the ring.
- Play animation: the star crosses behind the lens while the light curve `A(u)` is drawn.
- Image panel: position and brightness of θ+ and θ−, and the total magnification.
- Realism meter: the multiplier `k` exaggerates the effect, and the program shows how much
  of the drawn ring is real.
- Four real background photos, or open your own (JPEG/PNG, or drop it on the window): it is
  only kept in memory and is never saved or copied.
- Example scenes and PNG export (only when you press the button; the program never saves
  anything on its own).

## Running it

Download `GraviLens-v0.1.0-windows.exe` from the Releases page and run it. No installation
is needed. Since it is not code-signed, Windows may show a warning the first time.

To build from source:

```
cargo run --release
cargo test
```

## Physical scope (limitations)

Point lens, static non-rotating objects, **weak-field approximation**
(deflection angle `α = 4GM / (c²·b)`). No rotating (Kerr) black holes and no multiple
lenses. The background star's size is exaggerated so it is visible.

## Structure

`physics.rs` (objects and classification), `lensing.rs` (θE, lens equation,
magnification), `render.rs` (CPU inverse ray tracing), `worker.rs` (computation thread),
`app.rs` (interface). Design details are in `NOTES.md` (in Spanish);
`tools/validate_lenstronomy.py` compares the results with `lenstronomy`.

## Credits

- Background photos (Hubble Ultra Deep Field, eXtreme Deep Field, Pillars of Creation, Crab
  Nebula): NASA, ESA and collaborators; public domain. Details in `assets/CREDITS.md`.
- Interface: [egui](https://github.com/emilk/egui).

## License

MIT (see `LICENSE`). The images in `assets/` are public domain; see `assets/CREDITS.md`.
