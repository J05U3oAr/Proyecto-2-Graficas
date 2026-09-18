# Diorama raycasting — Rust sin dependencias

Renderizador de cubos inspirado en Minecraft. No usa crates externos: las texturas, el trazador, la ventana de Windows y la exportación BMP están implementados con Rust y la API nativa de Windows.

## Ejecutar de forma interactiva

```powershell
cargo run --release
```

El programa abre una ventana con el diorama renderizado. Cada cambio vuelve a calcular la imagen; a 480×320 suele tardar unos segundos, según el equipo.

| Control | Acción |
| --- | --- |
| Flecha izquierda/derecha o A/D | Rotar el diorama |
| Flecha arriba/abajo | Inclinar la cámara |
| W/S, +/− o rueda del ratón | Acercar/alejar la cámara |
| Esc | Salir |

La resolución de render se puede reducir para iterar más rápido:

```powershell
cargo run --release -- --width 320 --height 213
```

## Exportar una captura

Para guardar una vista y salir, añade `--output`:

```powershell
cargo run --release -- --width 960 --height 640 --yaw 110 --pitch 18 --distance 31 --output vista_lago.bmp
```

`BMP` abre directamente en Fotos o Paint de Windows. También se permite una ruta `.ppm` si se necesita el formato portable.

## Rúbrica cubierta por la base

| Elemento | Implementación |
| --- | --- |
| Diorama complejo | Isla 21×21, casa, chimenea, sendero, lago, invernadero, tres árboles y rocas. |
| Materiales | Césped, tierra, piedra, madera, hojas, agua y vidrio; cada uno tiene textura procedural y parámetros propios. |
| Reflexión | Piedra pulida, agua y vidrio usan rayos reflejados. |
| Refracción | Agua y vidrio aplican Snell con índices 1.333 y 1.52. |
| Skybox | Gradiente direccional con nubes y halo del sol. |
| Cámara | Ventana interactiva con rotación, inclinación y zoom. |

## Dónde extenderlo

- `texture`: reemplaza las texturas procedurales por un lector PPM propio si se desean imágenes de textura externas.
- `build_scene`: agrega nuevos bloques o genera terreno con ruido.
- `trace`: añade luces puntuales, niebla, sombras transparentes, anti-aliasing y profundidad de campo.

El renderer no importa crates: revisa `Cargo.toml` para confirmarlo.
