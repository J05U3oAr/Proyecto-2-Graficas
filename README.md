# Diorama voxel con raytracing en Rust

Un diorama inspirado en Minecraft: isla de cubos, lago con refracción, invernadero de vidrio, casa, árboles, skybox, sombras y reflejos. Abre una ventana interactiva y también puede exportar imágenes PNG o BMP.

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

| Control | Acción |
| --- | --- |
| `W` / `A` / `S` / `D` | Volar hacia delante, izquierda, atrás y derecha |
| `Space` / `Ctrl` | Subir / bajar libremente |
| `←` / `→` / `↑` / `↓` | Mirar alrededor |
| `Shift` | Aumentar la velocidad de vuelo |
| `P` | Guardar la vista actual como `diorama.png` |
| `Esc` | Cerrar |

## Exportar una imagen sin ventana

```powershell
cargo run --release -- --width 960 --height 640 --samples 4 --yaw 110 --pitch 18 --distance 31 --output vista_lago.png
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
| Skybox | Gradiente direccional, nubes y halo solar. |
| Cámara | Movimiento libre tipo espectador, orientación independiente y vuelo vertical. |
