# Notas de GraviLens

Cuaderno de aprendizaje. Aquí van el diseño, las fórmulas, los conceptos de Rust y las dudas. Escribe con tus propias palabras: si no puedes explicarlo, aún no lo entiendes.

## La idea en una frase

La gravedad de un objeto masivo (la **lente**) curva la luz de lo que hay detrás (la **fuente**). Si lente, fuente y observador están alineados, la luz se ve como un anillo: el **anillo de Einstein**.

Analogía: mirar una luz lejana a través del fondo de una copa de vino.

## Lo único del proyecto

1. **Balanza de realismo** – siempre dice cuánto de lo que ves es inventado (por el multiplicador k).
2. **Objetos que cambian de tipo** – al mover masa y radio, cambia a enana blanca / estrella de neutrones / agujero negro.
3. **Panel de curva de luz** – anillo y gráfica de brillo a la vez.

## Alcance y limitaciones (para el README)

- Objetos quietos, sin rotación.
- Aproximación de campo débil.
- Lente puntual / esférica simple.

## Fórmulas (iré rellenando según las necesite)

| Qué | Fórmula | Letras |
|---|---|---|
| Densidad media | ρ = M / (4/3 · π · R³) | M masa, R radio |
| Stefan-Boltzmann | L = 4π R² σ T⁴ | L luminosidad, T temperatura, σ constante |
| Radio de Schwarzschild | Rs = 2 G M / c² | si R <= Rs → agujero negro (Sol: Rs ≈ 2.95 km) |
| Relación masa-luminosidad | L = L☉ · (M / M☉)^3.5 | solo vale en secuencia principal |
| Deflexión (campo débil) | α = 4 G M / (c² b) | b = parámetro de impacto |
| Radio de Einstein | θE = √( 4GM/c² · D_ls / (D_l · D_s) ) | D_l, D_s, D_ls distancias |
| Ecuación de la lente | β = θ − θE² / θ | β posición real, θ posición aparente |
| Amplificación | A(u) = (u²+2) / (u √(u²+4)) | u = separación en unidades de θE |

## Datos de referencia (para tests)

| Objeto | Masa | Radio | Temp. | Tipo |
|---|---|---|---|---|
| Sol | 1.989e30 kg | 6.957e8 m | ~5772 K | G |
| Sirio A | ~2.06 M☉ | ~1.71 R☉ | ~9940 K | A |

Constantes: G = 6.674e-11, c = 2.998e8, σ = 5.670e-8 (todo SI).

## Clasificación del objeto (`classify_object`)

Solo mira **masa y radio**. Se comprueba en este orden y gana la primera que se cumpla:

| Orden | Tipo | Regla | Por qué |
|---|---|---|---|
| 1 | Agujero negro | R <= Rs | Es exacto, no una aproximación. Va primero porque gana a todo. |
| 2 | Estrella de neutrones | ρ >= 1e17 kg/m³ | Densidad nuclear, ~1e17–1e18. |
| 3 | Enana blanca | ρ >= 1e9 kg/m³ | Sirio B sale ~2e9; una estrella normal está muy por debajo. |
| 4 | Estrella de secuencia principal | M >= 0.08 M☉ | Por debajo de eso no fusiona hidrógeno. |
| 5 | Enana marrón | M >= 13 M_Júpiter | Fusiona deuterio pero no hidrógeno. |
| 6 | Planeta | todo lo demás | Sin condición propia (el `else` final). |

Limitaciones:

- No distingue una estrella joven de una evolucionada. Las gigantes y supergigantes rojas quedan fuera: el modelo L ∝ M^3.5 solo vale para la secuencia principal y daría mal su temperatura.
- Los umbrales son aproximados y los elegí yo; no son una frontera física exacta.
- Un agujero negro tiene como radio el del horizonte (R = Rs), por eso la comparación es `<=` y no `<`.

## Presets

- Cada preset es una ficha con **solo** nombre, masa y radio. Densidad, temperatura, color y tipo se calculan con las funciones de física; guardarlos sería duplicar datos.
- La lista de presets será un **array** (tamaño fijo): son 6 y no cambian durante la ejecución.
- Lo que escribe el usuario (masa y radio) son dos `f64` en el estado de la app, **no** entran en la lista. No se guarda nada al cerrar. Si algún día hubiera "guardar como preset", haría falta `Vec` y `String` para el nombre.
- Fuentes: Sirio A y B, Bond et al. 2017; Júpiter, NASA; estrella de neutrones típica 1.4 M☉ y 12 km.

## Lente 2D (Fase 3)

Idea: **trazado inverso**. Para cada píxel se pregunta "¿de dónde salió realmente la luz que veo aquí?", se deshace la curvatura y se toma el color del fondo en ese punto. Es el mismo truco de mirar a través del fondo de una copa de vino, pero hacia atrás.

| Qué | Fórmula | Letras y unidades |
|---|---|---|
| Radio de Einstein | θE = √( 4GM/c² · D_ls / (D_l · D_s) ) | M [kg], D [m], θE [rad]. D_ls = D_s − D_l |
| Ecuación de la lente (2D) | β = θ − θE² · θ / \|θ\|² | θ posición aparente, β posición real de la fuente [rad] |
| Imágenes de una fuente en β (eje x) | θ± = (β ± √(β² + 4θE²)) / 2 | una a cada lado de la lente |

- Si la fuente está justo detrás de la lente (β = 0), todos los puntos con \|θ\| = θE vienen del mismo punto: **anillo de Einstein**.
- Dentro del anillo (θ < θE) la imagen sale invertida (β con el signo opuesto); fuera, la fuente se ve más lejos del centro de lo que está.
- El **brillo superficial se conserva** en una lente: cada píxel toma el color del fondo en β y no hace falta multiplicar por la amplificación. La amplificación `A(u)` (Fase 4) sirve para la curva de luz de una fuente puntual, no para pintar la imagen.
- Un θE real es de milisegundos de arco (Sol a 4 kpc con fuente a 8 kpc: ≈ 1.01 mas). Por eso las distancias van en kpc y los ángulos en mas, y existe el multiplicador **k** (θE dibujado = k · θE real). La balanza de realismo de la Fase 4 dirá cuánto de lo que se ve es inventado por k.
- Código: `lensing.rs` (física pura: `einstein_radius`, `source_position`), `render.rs` (imagen: `render`, sin dependencia de egui), `app.rs` (sliders y textura). Todo con tests.
- En el campo débil, cualquier objeto masivo (estrella, agujero negro) actúa igual como lente puntual; el tipo de objeto solo cambia su masa. Se ignora el tamaño físico de la lente: si la luz pasara por dentro del objeto, el modelo no vale.
- **Fondo con foto real**: `sky.rs` (`SkyImage`) carga el *Hubble Ultra Deep Field* (`assets/hubble_ultra_deep_field.jpg`, **6200×6200**, 19 MB, incluida en el ejecutable con `include_bytes!`; la versión de 3100 px salía borrosa al ampliarla. Se probó también el campo profundo del Webb, pero ya trae un cúmulo lenteado y rayos de difracción que se confunden con nuestra lente). Cada píxel calcula su β y lee la foto en ese punto con interpolación bilineal; fuera de la foto se refleja como un espejo para no dejar huecos. El slider "Tamaño de la foto" fija cuántos mas cubre el lado de la foto. Hay un interruptor para volver a la estrella con campo de estrellas.
- **Crédito de la imagen**: Hubble Ultra Deep Field, NASA y ESA (dominio público según Wikimedia Commons). Ponerlo también en el README.
- **Marcas del objeto** (`overlay.rs`): capa vectorial dibujada con el `Painter` de egui encima de la imagen, así que es nítida a cualquier resolución y no entra en los píxeles de la lente. Incluye: el objeto en el centro a escala exagerada (su tamaño real es ~1e-6 mas, invisible; el radio dibujado va de 5 a 16 puntos según el logaritmo del radio real), una etiqueta con nombre y tipo, un círculo punteado en θE y una tarjeta en la esquina con masa, radio y temperatura. Cada tipo tiene su aspecto: estrella con resplandor del color de su temperatura, planeta como esfera sombreada, agujero negro con anillo de fotones y resplandor naranja, estrella de neutrones con haces. Se pueden ocultar con la casilla "Mostrar marcas del objeto".
- Pendiente: validar contra `lenstronomy` (Python) con los mismos parámetros.

## Modelo de temperatura y luminosidad por tipo (`effective_temperature`)

`L ∝ M^3.5` solo vale en secuencia principal, así que cada tipo usa su modelo:

| Tipo | Temperatura | Modelo |
|---|---|---|
| Secuencia principal | calculada | `L = L☉·(M/M☉)^3.5` y `T = (L / 4πR²σ)^(1/4)` |
| Enana marrón | 1500 K | valor típico fijo (enfriamiento lento; depende de la edad) |
| Enana blanca | 10 000 K | valor típico fijo (depende de la edad; Sirio B ≈ 25 000 K) |
| Estrella de neutrones | 1e6 K | valor típico de estrella joven |
| Planeta, agujero negro | `None` | no emiten luz propia |

- La luminosidad emitida sale de `L = 4πR²σT⁴` con esa temperatura (`emitted_luminosity`).
- Limitaciones: las temperaturas fijas son "típicas", no calculadas, porque dependen de la edad, que no conocemos. Se ignoran la radiación de Hawking y el disco de acreción del agujero negro. El tipo espectral O…M solo se muestra para secuencia principal.
- Constantes en `constants.rs`: `BROWN_DWARF_TEMPERATURE`, `WHITE_DWARF_TEMPERATURE`, `NEUTRON_STAR_TEMPERATURE`.

## Conceptos de Rust aprendidos

- `std::thread::scope`: lanzar varios hilos que **prestan** datos del hilo principal (sin `Arc`): el `scope` espera a que acaben todos. `chunks_mut` reparte un buffer en trozos que cada hilo puede escribir sin pisarse (el borrow checker garantiza que no se solapan). El render va repartido por filas entre los núcleos.
- `include_bytes!("ruta")`: mete un archivo dentro del ejecutable en tiempo de compilación (la ruta es relativa al archivo `.rs`).
- Funciones que devuelven un widget con lifetime (`fn scientific_slider<'a>(value: &'a mut f64, ...) -> egui::Slider<'a>`): el slider presta el valor que edita mientras exista.
- `[profile.dev.package."*"] opt-level = 3` en `Cargo.toml`: optimiza las dependencias aunque compiles en debug (la decodificación de la foto pasó de segundos a milisegundos).
- Tupla `(f64, f64)`: dos valores juntos sin nombre (aquí, una posición x, y). Se abre con `let (x, y) = tupla;`.
- Iteradores y `for (x, y) in [...]`: recorrer un array de tuplas abriéndolas en el mismo `for`.
- `Vec<u8>` con `Vec::with_capacity(n)` y `extend_from_slice`: un buffer de bytes (la imagen RGB, 3 bytes por píxel) reservado de antemano.
- `#[derive(Clone, Copy, PartialEq)]` en una struct pequeña: permite copiarla y compararla con `==` (así la app sabe si hace falta volver a dibujar).
- `matches!(valor, patrón)` y `if let`: comprobar si un `Option` tiene cierta forma sin escribir un `match` entero.
- El `Painter` de egui (`circle_filled`, `circle_stroke`, `line_segment`, `rect_filled`, `text`, `layout_no_wrap` + `galley`): dibujar formas y texto encima de otros widgets. `ui.painter_at(rect)` recorta el dibujo a ese rectángulo.
- Fuente de egui: no incluye todos los símbolos Unicode (`☉` salía como un cuadrado), por eso en pantalla se escribe `M_sol` y `L_sol`.
- `f64::is_finite()`, `.hypot()`, `.clamp()`, `.floor()`: comprobar infinitos/NaN, hipotenusa, limitar un valor a un rango, redondear hacia abajo.
- Hash determinista (`wrapping_mul`, desplazamientos `>>`): números "aleatorios" que siempre dan lo mismo para la misma celda; sirve para el campo de estrellas sin guardar nada.
- `Option<T>`: un valor que puede estar (`Some(x)`) o no (`None`). Se usa cuando "no hay respuesta" es normal (un planeta no tiene temperatura de estrella). Se consume con `match`, `if let Some(x) = ...` o `.map(|x| ...)`. `.expect("msg")` abre el `Option` y para el programa con ese mensaje si es `None` (útil en tests).
- `match` sobre un enum sin `_`: el compilador obliga a cubrir **todas** las variantes. Si se añade una variante nueva, avisa dónde falta.

- `let` vs `let mut`: por defecto las variables no cambian.
- `struct`: una ficha con casillas con nombre y tipo (`Color`, `Preset`). Se escribe el molde una vez y se crean tantos valores como se quiera. El `pub` de la struct no se hereda a los campos: cada campo necesita su `pub`.
- `enum`: un tipo que solo puede ser una de varias variantes (`SpectralType`, `ObjectKind`). Con `#[derive(Debug, PartialEq)]` se puede imprimir y comparar con `assert_eq!`.
- `match` con guardas: `t if t >= 30000.0 => ...`. Examina **un solo valor**; no se le pasan dos separados por coma. Con condiciones sobre varias cosas distintas es más claro `if / else if / else`.
- Última expresión sin `;`: es el valor que devuelve una función o un bloque. Un `;` de más lo convierte en `()` y da `mismatched types`. Un `if` sin `else` final no garantiza que haya valor.
- `return` solo hace falta para salir antes de tiempo; al final de la función se omite (clippy: `needless_return`).
- Parámetros de función: son nombres **nuevos**. Si se llaman como una constante importada, Rust lo toma como un patrón (`refutable pattern in function argument`). Una función sin datos de entrada lleva los paréntesis vacíos.
- Notación científica: `AeB` = A × 10^B, así que `10e17` es 1e18, no 1e17.
- `String` vs `&str`: `String` es propia y se puede modificar (el cuaderno); `&str` es una vista prestada de solo lectura (la página de un libro ajeno). Un texto literal es `&'static str` (dura todo el programa).
- Módulos: `mod x;` en `main.rs` declara el archivo `x.rs`; sin eso Rust ni lo mira. Se usa con `use crate::x::...`.
- Tests en archivo aparte: en `physics.rs` va `#[cfg(test)] mod tests;` y el contenido en `src/physics/tests.rs`, con `use super::*;`. Sin esas líneas los tests desaparecen sin error (`running 0 tests`).
- Array `[T; N]` (tamaño fijo) vs `Vec<T>` (crece en ejecución). Un `for` sobre el array lo consume (ownership); para usarlo otra vez hay que recorrerlo prestado (`&`).
- `Cargo.toml` vs `use` vs `impl`: `Cargo.toml` lista las librerías externas del proyecto (la lista de la compra); `use` trae un nombre a un archivo (sacar el ingrediente del armario); `impl` enseña funciones a un tipo.
- Trait: un contrato ("si quieres ser una app, sabes hacer `ui`"). `impl eframe::App for App` lo cumple. `impl App { ... }` (sin trait) añade funciones propias, como el constructor `new`.
- `Self`: el tipo que se está implementando. `Self { campo: valor }` crea uno sin repetir el nombre.
- Constructor por convención: `pub fn new() -> Self`. Permite dejar los campos privados y que otro módulo cree el valor igualmente.
- `&mut self`: "préstame mi cuaderno para escribir en él". Necesario en `ui` porque los sliders modifican `mass` y `radius`. `&mut self.mass` presta un solo campo.
- `Box::new(x)`: guarda `x` en el heap. eframe lo pide porque no conoce el tamaño de la app.
- Closure `|x| ...`: función sin nombre. `_x` (con guion bajo) = parámetro que no se usa.
- `.unwrap()`: si hay error, el programa se detiene. Vale para empezar.
- `const` con rango: `1.0e3..=2.0e9` es un `RangeInclusive`; el `=` incluye el máximo.
- egui 0.37: el trait `eframe::App` usa `fn ui(&mut self, ui: &mut egui::Ui, frame)`. Los tutoriales viejos usan `update` con `ctx`.

## Interfaz (Fase 2)

- Arquitectura: `main.rs` solo declara módulos y abre la ventana; `app.rs` guarda `App` (el "cuaderno": estado entre frames) y dibuja. La física sigue separada en `physics.rs`.
- egui redibuja la ventana cada frame y no recuerda nada: todo lo que cambia vive en `struct App`.
- Sliders de masa y radio en escala **logarítmica** (`.logarithmic(true)`), porque abarcan muchos órdenes de magnitud (Júpiter 1.9e27 kg … agujero negro de 10 M☉ 2e31 kg; radios de 12 km a 1.2e9 m).
- Rango de masa: 0.5 M_Júpiter … 50 M☉. Rango de radio: 1 km … 2e9 m. El mínimo > 0 evita el caso de radio 0 y masa ≤ 0 (`inf` / `NaN`) apuntado en Dudas.
- Panel de datos (`show_data` en `app.rs`): tipo de objeto, masa en M☉, radio en km, densidad en g/cm³, radio de Schwarzschild, y, solo si el objeto emite, temperatura, luminosidad en L☉, tipo espectral (solo secuencia principal) y color aparente. Las conversiones a unidades "humanas" se hacen solo ahí; dentro del código todo es SI.
- Menú de presets (`egui::ComboBox`): al elegir uno copia su masa y radio a los sliders; si luego se mueve un slider, el menú pasa a "Personalizado". Nada se guarda al cerrar.
- Reparto de trabajo desde la Fase 2: el usuario hace la física; yo hago el resto (ver `CLAUDE.md`).

## Dudas / atascos

- Entrada del usuario: radio 0, masa <= 0 o radio negativo dan `inf` o `NaN` y se propagan a todos los cálculos. Resuelto en los sliders con un rango de mínimo > 0 (ver Interfaz). Si algún día se permite teclear valores a mano, habrá que validar.
- Windows **Smart App Control** bloqueaba el código recién compilado (`os error 4551`, y `can't find crate for zerovec_derive` al compilar eframe). Se comprobó con el registro (`VerifiedAndReputablePolicyState = 1`, activo). Se desactivó para poder compilar; Defender y SmartScreen siguen activos.

## Registro de sesiones

- 2026-10-25: Creados CLAUDE.md, ROADMAP.md, NOTES.md. Rust aún no instalado.
- 2026-10-25: Fase 1 pasos 4, 5 y 6 hechos (`star_color`, `spectral_type`, `classify_object`). Paso 7 en curso: `Preset` y 6 presets con tests; falta la lista `all_presets()`. 18 tests en verde.
- 2026-10-25: Fase 1 cerrada (`all_presets()` + 19 tests, fmt y clippy limpios). Fase 2 empezada: dependencia `eframe 0.37`, ventana abierta, `app.rs` con `App`, sliders de masa y radio. Nuevo reparto: el usuario hace la física, yo el resto.
- 2026-10-25: Fase 2 cerrada. Fase 3 implementada por Claude a petición del usuario (que prefirió no escribir también esta física): `einstein_radius`, `source_position`, `render` por trazado inverso, sliders de escena y multiplicador k. 35 tests en verde. Pendiente: que el usuario la pruebe y la valide con lenstronomy.
- 2026-10-25: Imagen adaptable a cualquier pantalla: se dibuja con tantos píxeles reales como ocupa (puntos × `pixels_per_point`), redondeado a múltiplos de 64 y entre 256 y 1536; mientras se arrastra un slider baja a 384 y al soltar vuelve a toda calidad. Ventana inicial de 1280×800, redimensionable, mínimo 760×480.
- 2026-10-25: Fondo con foto real (Hubble Ultra Deep Field), render repartido en hilos (~30 ms para 900×900), sliders en notación científica (se pueden teclear valores como `1.989e30`), resolución de la imagen hasta 2048 px según el tamaño de la ventana y la escala del monitor. 42 tests en verde.
- 2026-10-25: Imagen más nítida: foto de 6200 px y **supermuestreo** (cada píxel promedia 2×2 puntos a toda calidad; sin suavizar mientras se arrastra un slider). La foto es más fina que la pantalla, y con un solo punto por píxel salían dientes de sierra. 43 tests en verde. Decodificar la foto al arrancar ≈ 250 ms; un render de 1400 px ≈ 270 ms.
- 2026-10-25: Marcas para distinguir el objeto elegido (objeto en el centro, etiqueta, círculo de θE y tarjeta, con casilla para ocultarlas). Arreglados el símbolo `☉` que salía como cuadrado y el ancho de la tarjeta. Verificado ejecutando la app y mirando una captura de su ventana.
- 2026-10-25: La imagen deja de ser cuadrada y llena toda la zona libre (rectangular, misma escala en los dos ejes; el campo de visión se mide sobre el lado corto). Total limitado a 4 millones de píxeles a toda calidad y lado largo de 480 en la vista previa. Los datos de la lente pasan a un panel derecho, que más adelante alojará la curva de luz y la balanza de realismo (Fase 4). 45 tests en verde.
- 2026-10-25: Escena de partida = **agujero negro de 10 M_sol como lente y el Sol detrás como fuente**. Modo "estrella detrás de la lente": se elige qué estrella (solo objetos que emiten luz), su color sale de su temperatura, y se dibuja con oscurecimiento del limbo (el borde más oscuro que el centro) y un halo que se apaga a los 4 radios. El tamaño de la estrella se exagera (su tamaño real es ~1e-6 mas). Con la fuente justo detrás de la lente sale el anillo completo; con "Desalineación" aparecen dos imágenes y una cruz amarilla marca dónde está de verdad la estrella. Botón "Encuadrar el anillo de Einstein" ajusta el campo de visión (3.2 θE). La foto del Hubble sigue disponible como fondo alternativo. Para la lente solo cuenta la masa: un agujero negro y una estrella de 10 M_sol dan el mismo anillo. 47 tests en verde.
- 2026-10-25: **La lente queda fija**: siempre el agujero negro estelar de 10 M_sol (`stellar_black_hole()`), guardado en `App.lens`. Se quitaron el menú de presets y los sliders de masa y radio. Lo que se puede cambiar es lo que hay detrás (la estrella o la foto del Hubble) y la geometría de la escena (distancias, posición de la lente, desalineación, campo de visión, multiplicador k). Los presets y la clasificación siguen en el código y con sus tests, y se siguen usando para elegir la estrella de fondo.
- 2026-10-25: **El objeto marcado es la estrella elegida**, no la lente: se dibuja en su posición real (en el centro si está justo detrás), con su etiqueta y su tarjeta, y la lente aparece como línea aparte ("Lens: ...") y en el panel derecho. Con la foto de fondo se marca la lente porque no hay estrella. Cuando la estrella está desalineada, un círculo pequeño marca dónde está la lente. **Todo el texto de la ventana pasa a inglés** (los comentarios y la documentación siguen en español; anotado en `CLAUDE.md`). Verificado ejecutando la app y capturando solo su ventana con `PrintWindow`.

## Sesión 2026-10-25 (plan de mejoras, bloques 1–6, 8–10)

- **Física nueva** (`lensing.rs`): `magnification(u) = (u²+2)/(u√(u²+4))`; `image_positions(β, θE) = (β ± √(β²+4θE²))/2`; `image_magnifications(u) = ((A+1)/2, (A−1)/2)`. 4 tests nuevos (50 en total).
- **Interacción** (`app.rs`): `Image::sense(Sense::click_and_drag())` devuelve una `Response`; con ella `dragged()`, `double_clicked()`, `hovered()`; la rueda llega por `smooth_scroll_delta`; el zoom es exponencial (`exp`) para que cada paso multiplique el campo de visión.
- **Animación**: `stable_dt` da el tiempo del fotograma; `request_repaint()` pide el siguiente. Mientras suena el Play se dibuja en baja resolución.
- **Curva de luz**: dibujada a mano con `Painter` (sin dependencia `egui_plot`).
- **Realismo**: `k = 1` es 100 % real; si no, la parte real del anillo dibujado es `100/k` %.
- **Exportar PNG**: `image::save_buffer`, escribe `gravilens_<segundos>.png` en la carpeta de lanzamiento solo al pulsar el botón (nada se guarda al cerrar).
- **Rust nuevo**: `enum` con `const ALL`, `Option::map_or`, `let` encadenado con `&&` (edición 2024), `derive(Clone, Copy, PartialEq)`.

- **Hilo de cálculo** (`worker.rs`): `std::thread::spawn` + `mpsc::channel`; la interfaz pide con `request` y recoge con `latest`; el hilo descarta peticiones viejas. La foto se comparte con `Arc<SkyImage>` (contador de referencias entre hilos). `ctx.request_repaint()` despierta la ventana.

- **Fotos**: `SkyImage::from_bytes` decodifica JPEG/PNG, recorta al cuadrado central y limita el lado a 6200 px. `rfd` abre el diálogo de archivos; los archivos soltados llegan con `raw.dropped_files` (`DroppedFile::path()`). La foto propia solo vive en un `Arc<SkyImage>`; el hilo recibe el cambio con `Message::Sky`. Los scripts de Python con símbolos Unicode: mejor ejecutarlos con `PYTHONUTF8=1`.
