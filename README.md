# Diorama voxel con raytracing en Rust

Un diorama inspirado en Minecraft: isla de cubos, lago con refracción, invernadero de vidrio, casa, árboles, skybox, sombras y reflejos. Abre una ventana interactiva y también puede exportar imágenes PNG o BMP.

## Casa elru

La casa **elru** se reconstruye desde las cuatro vistas de referencia en
`src/elru.rs`: salón con vidrio continuo en esquina, terraza superior,
atrio central, balcón derecho, fachadas de ventanas verticales, escaleras
y piscina rectangular con setos. Tiene interiores huecos y pisos conectados.
El selector utiliza una miniatura de la misma geometría. Las proporciones
se estimaron de las imágenes; no corresponden a un plano acotado.

Para inspeccionar cada fachada, exporta con `--world elru --yaw 90` (frente),
`--yaw 180` (izquierda), `--yaw 270` (posterior) o `--yaw 0` (derecha).
Puedes ajustar el encuadre con `--target-y 10 --target-z -1 --distance 57`.

## Mundo Auropl

**Auropl** recrea la fachada de la referencia con tres habitaciones elevadas:
dos laterales altas y una central más baja, marcos de madera, ventanas
horizontales, columnas con franjas rojizas y rosadas y una galería en U.
La entrada dorada tiene puertas dobles, dos piscinas y cuatro faroles triples.
Los laterales y la parte trasera completan los volúmenes con el mismo lenguaje
de ventanas y molduras; esas caras se interpretaron a partir de la vista frontal.

El modelo está en `src/auropl.rs` y aparece como tercer mundo en el selector
(usa `A` / `D`). La miniatura comparte la geometría del mundo completo.
Para exportar la vista frontal inicial:

```powershell
cargo run --release -- --world auropl --width 960 --height 720 --output output/auropl/frente.png
```

Usa `--yaw 125 --pitch 12 --distance 78` para una perspectiva elevada,
`--yaw 180` para el lateral y `--yaw 270` para la parte trasera.

## Dependencias permitidas


- `minifb`: ventana y entrada de teclado multiplataforma.
- `nalgebra`: vectores y operaciones 3D.
- `image`: exportación PNG/BMP.

El trazador de rayos, materiales, texturas procedurales, escena, BVH y render paralelo son implementación propia.

## Ejecutar

```powershell
cargo run --release
```

En la primera ejecución Cargo descargará las dependencias. Después se abre la ventana interactiva.

Al iniciar aparece el selector de mundos. **Ve7** y **elru** están representados por mundos cúbicos 3D. Usa `A` / `D` para cambiar de mundo y haz clic directamente sobre su geometría para seleccionarlo. Elru incluye una casa moderna de marcos blancos y vidrio, con un lago pequeño al frente y una isla de borde circular formada por escalones de cubos. Cada mundo gira automáticamente y se detiene al pasar el cursor encima; usa las flechas izquierda/derecha para girar y `Espacio` para pausar o reanudar. Después elige Día o Noche y presiona `Enter` para entrar. En el selector de entorno también puedes elegir con `D` o `N`.

Puedes exportar una vista del menú sin abrir una ventana: `cargo run --release -- --menu-preview --world elru --output menu.png`. Para exportar el mundo completo: `cargo run --release -- --world elru --output elru.png`.

| Control | Acción |
| --- | --- |
| `W` / `A` / `S` / `D` | Volar hacia delante, izquierda, atrás y derecha |
| `Space` / `Ctrl` | Subir / bajar libremente |
| `←` / `→` / `↑` / `↓` | Mirar alrededor |
| `Shift` | Aumentar la velocidad de vuelo |
| `N` | Alternar entre el entorno de día y de noche |
| `P` | Guardar la vista actual como `diorama.png` |
| `Esc` | Cerrar |

## Exportar una imagen sin ventana

```powershell
cargo run --release -- --width 960 --height 640 --samples 4 --yaw 110 --pitch 18 --distance 31 --output vista_lago.png
```

Agrega `--night` (o `--noche`) para exportar la escena nocturna con estrellas, luna y luz lunar:

```powershell
cargo run --release -- --night --output diorama_noche.png
```

Usa extensión `.png` o `.bmp`.

## Optimizaciones implementadas

- **BVH**: jerarquía de cajas que descarta grupos completos de cubos antes de probar sus intersecciones.
- **Sombras de salida temprana**: el rayo de sombra se detiene con el primer bloque que lo ocluye.
- **Render paralelo**: divide las filas entre los núcleos disponibles con `std::thread`.
- **Frame de cámara precalculado**: no recalcula trigonometría ni ejes de cámara por cada píxel.
- **Perfil release afinado**: LTO delgado, una unidad de código y `panic = abort`.

En una máquina de 16 hilos, una prueba de 480×320 con 1,116 cubos pasó de aproximadamente 2.22 s a 68 ms.

## Rúbrica cubierta

| Elemento | Implementación |
| --- | --- |
| Diorama complejo | Jardín 64×64, casa moderna ampliada inspirada en Vegeta777, sendero, lago, invernadero, árboles y rocas. |
| Materiales | Césped, tierra, piedra, madera, hojas, agua y vidrio; todos con textura procedural y parámetros propios. |
| Reflexión | Piedra pulida, agua y vidrio trazan rayos reflejados. |
| Refracción | Agua y vidrio aplican la ley de Snell. |
| Skybox | Modos día/noche, nubes procedurales suaves, sol, estrellas y estrellas fugaces animadas, luna y halo lunar. |
| Cámara | Movimiento libre tipo espectador, orientación independiente y vuelo vertical. |
