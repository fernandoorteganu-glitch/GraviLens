# Roadmap

Marca cada casilla cuando funcione **y** lo entiendas.

## Progreso

- [X ] **Fase 0 – Preparación**: instalar Rust, `cargo init`, primer `cargo run`
- [X ] **Fase 1 – Física**: masa + radio → densidad, temperatura, color, tipo. Presets.
- [X] **Fase 2 – Interfaz**: ventana egui, sliders, panel de datos en vivo
- [ ] **Fase 3 – Lente 2D**: estrella de fondo, ecuación de la lente por píxel, multiplicador k → **anillo de Einstein** 🎉 _(hecha por Claude a petición del usuario: `lensing.rs`, `render.rs`, sliders; falta que la pruebes y validar con lenstronomy)_
- [ ] **Fase 4 – Identidad**: balanza de realismo, cambio de tipo, curva de luz
- [ ] **Fase 5 – 3D**: cielo en esfera, cámara libre, baja resolución al moverse
- [ ] **Fase 6 – Pulido**: aspecto, README, licencia, GitHub
- [ ] Extras: WebAssembly, interfaz ES/EN

---

## Fase 0 – Preparación

1. Instalar Rust con **rustup** (https://rustup.rs). En Windows pide las *Visual Studio Build Tools* (carga "Desarrollo de escritorio con C++"). Ya tienes carpeta `.vs`, así que quizá ya las tienes.
2. Abrir una terminal NUEVA y comprobar: `cargo --version` y `rustc --version`.
3. Dentro de `GraviLens` ejecutar `cargo init` (NO `cargo new`: la carpeta ya existe).
4. `cargo run` → debe imprimir "Hello, world!".
5. Crear el repo git (`git init`) y primer commit.

**Cosas de Rust que aprenderás aquí:** qué es `cargo`, `Cargo.toml`, `src/main.rs`, `fn main`, `println!`.

---

## Fase 1 – Física

**Objetivo:** una función que recibe masa y radio y devuelve las propiedades del objeto.
**Criterio de éxito:** con el Sol sale tipo G, ~5800 K, ~1.4 g/cm³.

Pasos sugeridos (haz uno, ejecuta, avanza):

Done 1. `constants.rs`: G, c, σ (Stefan-Boltzmann), masa solar, radio solar, luminosidad solar.
Done 2. Densidad = masa / volumen de esfera. Probar con el Sol.
Done 3. Luminosidad estimada (relación masa-luminosidad, L ∝ M^3.5 en secuencia principal) y temperatura efectiva con Stefan-Boltzmann: `L = 4πR²σT⁴`.
done 4. Color a partir de la temperatura (aproximación de cuerpo negro → RGB).
done 5. Tipo espectral (O, B, A, F, G, K, M) por rango de temperatura.
done. Clasificación del objeto: estrella / enana blanca / estrella de neutrones / agujero negro (usando radio de Schwarzschild `Rs = 2GM/c²` y densidad).
7. `struct Preset` con Sol, Sirio, etc.

**Rust que aprenderás:** `const`, `f64`, funciones, `struct`, `enum`, `match`, `impl`, módulos (`mod`), tests (`#[test]`).

**Pregunta para pensar antes de programar:** ¿por qué una enana blanca y el Sol pueden tener la misma masa pero ser tan diferentes?

---

## Fase 2 – Interfaz (resumen)

`eframe`/`egui`: ventana, `egui::Slider`, panel lateral con los datos. Concepto clave: el bucle de la interfaz redibuja cada frame, el *estado* vive en un `struct App`.

## Fase 3 – Lente 2D (resumen)

Para cada píxel: posición en el cielo θ → ecuación de la lente `β = θ − θE²/θ` → ¿de dónde venía la luz? → color de la estrella de fondo en β. Radio de Einstein `θE = sqrt(4GM/c² · D_ls / (D_l · D_s))`. Validar contra lenstronomy.

## Fase 4 – Identidad (resumen)

- **Balanza de realismo**: cociente entre el efecto mostrado (con k) y el real; avisar de cuánto es inventado.
- **Cambio de tipo**: al mover M y R la lente reacciona (la física de la Fase 1 manda).
- **Curva de luz**: factor de amplificación `A(u) = (u²+2)/(u·sqrt(u²+4))` mientras la fuente se mueve.

## Fase 5 y 6

Se detallan cuando llegues.

---

## Plan de mejoras (propuesto el 2026-10-25, libertad total para hacerlo lo mejor posible)

Estado de partida: lente fija (agujero negro de 10 M_sol), estrella o foto del Hubble detrás, anillo de Einstein por trazado inverso en CPU con hilos y supermuestreo, marcas vectoriales, interfaz en inglés. 46 tests en verde.

Orden propuesto (cada bloque se puede hacer, probar y dar por bueno por separado):

1. **Interacción directa**: arrastrar la estrella con el ratón para moverla (la desalineación), rueda para el zoom (campo de visión), doble clic para encuadrar el anillo. Atajos de teclado.
2. **Animación**: botón Play que mueve la estrella a través del eje y velocidad ajustable. Es la base de la curva de luz.
3. **Curva de luz** (Fase 4): gráfica del brillo en tiempo real con `A(u) = (u²+2) / (u·√(u²+4))`, con un marcador del instante actual.
4. **Balanza de realismo** (Fase 4): cuánto del efecto es real y cuánto viene del multiplicador k, con aviso.
5. **Análisis de la imagen**: posiciones y brillo de las dos imágenes (θ±), amplificación total, y marcadores sobre la imagen.
6. **Escenas predefinidas**: anillo perfecto, dos imágenes, microlente con curva de luz, arcos con la foto del Hubble.
7. **Calidad visual**: mejor halo de la estrella, filtro de la foto más fino, brillo y contraste, estrellas de fondo más ricas.
8. **Exportar**: botón para guardar la imagen actual como PNG.
9. **Ayuda integrada**: explicaciones cortas (tooltips) de cada control y de la física, para que se entienda sin leer el código.
10. **Validación** contra `lenstronomy` (Python) con los mismos parámetros, como pide el roadmap.
11. **Rendimiento**: mover el cálculo a un hilo aparte para que la interfaz nunca se quede esperando; valorar GPU (Fase 5).
12. **Pulido (Fase 6)**: README con alcance y limitaciones, créditos de la imagen (NASA y ESA), licencia, repositorio en GitHub.

Restricciones que se mantienen: todo SI dentro del código, física separada de la interfaz, cada función de física con test, `cargo fmt` y `cargo clippy` limpios, texto de la ventana en inglés, nada se guarda al cerrar.

### Avance del plan (2026-10-25)

Hechos: 1 interacción directa (arrastrar, rueda, doble clic, flechas/C/F), 2 animación Play, 3 curva de luz, 4 balanza de realismo, 5 análisis de imágenes θ± y amplificación (con marcas sobre la imagen), 6 escenas de ejemplo, 8 exportar PNG, 9 tooltips. 10 validación: script `tools/validate_lenstronomy.py` listo, falta ejecutarlo (`pip install lenstronomy astropy`). Pendientes: 7 calidad visual, 11 hilo aparte, 12 pulido.

Actualización: hechos también 7 (segunda capa de estrellas tintadas), 11 (hilo de cálculo en `worker.rs`) y 12 en parte (`README.md` con alcance y créditos). Falta elegir licencia, subir a GitHub y ejecutar la validación con lenstronomy.

Actualización final: 4 fotos incluidas + foto propia (solo en memoria), guía y fórmulas con números en los paneles, `LICENSE` (MIT), `assets/CREDITS.md`, CI de GitHub (`.github/workflows/ci.yml`). Falta solo `git commit`/subir y la validación con lenstronomy.
