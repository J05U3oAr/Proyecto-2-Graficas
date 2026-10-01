# Diorama voxel con raytracing en Rust

Diorama 3D inspirado en Minecraft, construido con Rust y un raytracer propio. El proyecto genera mundos voxel con materiales procedurales, agua, vidrio, sombras, reflejos, refracción, iluminación diurna/nocturna y una cámara libre.

La aplicación puede abrirse en modo interactivo o exportar una vista directamente a PNG o BMP.

## Mundos disponibles

El selector inicial contiene tres mundos. Usa `A` y `D` para cambiar entre ellos, o haz clic sobre el mundo mostrado.

### Ve7

Casa moderna de dos alas con entrada central, ventanas moradas, techo escalonado, balcón lateral, jardín, río, puente, invernadero, árboles y rocas.

### elru

Casa moderna de vidrio con salón en esquina, atrio central, terraza superior, balcón, piscina, setos y escaleras. La escalera exterior desemboca en una puerta de vidrio con un pequeño balcón y barandales, para que el acceso sea utilizable.

Su geometría está en `src/elru.rs`.

### Auropl

Recreación de la referencia frontal con tres habitaciones elevadas: dos módulos laterales altos y uno central más bajo. Incluye:

- marcos de madera y cubiertas con voladizo;
- ventanas horizontales con vidrio texturizado;
- columnas con franjas rojas y rosadas;
- galería inferior en forma de U;
- entrada dorada con puertas dobles;
- dos piscinas, camino central y faroles triples;
- laterales y parte trasera completados con la misma lógica constructiva de la fachada.

Su geometría está en `src/auropl.rs`. La miniatura del selector usa la misma construcción que el mundo completo.

## Requisitos

- Rust y Cargo, con edición 2021.
- `minifb` para la ventana interactiva.
- `nalgebra` para vectores y operaciones geométricas.
- `image` para cargar texturas y exportar imágenes.

Las dependencias se descargan automáticamente durante la primera compilación.

## Ejecutar

Para abrir el selector interactivo:

```powershell
cargo run --release
```

Después de seleccionar un mundo, elige Día o Noche con las flechas, `D` o `N`, y presiona `Enter`.

También se puede iniciar un mundo directamente:

```powershell
cargo run --release -- --world ve7
cargo run --release -- --world elru
cargo run --release -- --world auropl
```

Los nombres de mundo no distinguen mayúsculas y minúsculas en la línea de comandos.

## Controles

| Tecla | Acción |
| --- | --- |
| `A` / `D` | Cambiar mundo en el selector; elegir Día o Noche en el menú de entorno |
| Clic izquierdo | Seleccionar el mundo bajo el cursor |
| `Enter` | Confirmar la selección |
| `W` / `A` / `S` / `D` | Mover la cámara hacia delante, izquierda, atrás y derecha |
| `Space` / `Ctrl` | Subir y bajar con la cámara |
| Flechas | Girar la cámara; en el selector, girar la miniatura |
| `Shift` | Aumentar la velocidad de movimiento |
| `N` | Alternar entre Día y Noche durante la escena |
| `P` | Guardar una captura en `diorama.png` |
| `Esc` | Cerrar la aplicación |

## Exportar imágenes

Exportar Auropl desde la vista frontal predeterminada:

```powershell
cargo run --release -- --world auropl --width 960 --height 720 --samples 4 --output output/auropl/frente.png
```

Exportar una perspectiva elevada, un lateral o la parte trasera de Auropl:

```powershell
cargo run --release -- --world auropl --yaw 125 --pitch 12 --distance 78 --output output/auropl/perspectiva.png
cargo run --release -- --world auropl --yaw 180 --output output/auropl/lateral.png
cargo run --release -- --world auropl --yaw 270 --pitch 8 --distance 70 --output output/auropl/posterior.png
```

Exportar una vista del selector sin abrir una ventana:

```powershell
cargo run --release -- --world auropl --menu-preview --output output/auropl/menu.png
```

Parámetros útiles:

| Parámetro | Función |
| --- | --- |
| `--world ve7|elru|auropl` | Seleccionar el mundo |
| `--width N` / `--height N` | Resolución de salida |
| `--samples N` | Muestras por píxel, entre 1 y 16 |
| `--yaw N` / `--pitch N` | Orientación inicial de la cámara |
| `--distance N` | Distancia orbital al objetivo |
| `--target-y N` / `--target-z N` | Punto de interés de la cámara |
| `--night` o `--noche` | Exportar con iluminación nocturna |
| `--output ruta` | Guardar la imagen en PNG o BMP |

## Organización del código

- `src/main.rs`: entrada de la aplicación y exportación.
- `src/app.rs`: selector de mundos, menú de entorno, cámara interactiva y controles.
- `src/scene_builder.rs`: registro de mundos y construcción de la isla/miniaturas.
- `src/auropl.rs`: geometría y materiales del mundo Auropl.
- `src/elru.rs`: geometría y materiales de la casa elru.
- `src/materials.rs`: materiales, texturas procedurales y texturas de assets.
- `src/geometry.rs`: rayos, cubos, cajas envolventes y operaciones vectoriales.
- `src/scene.rs`: escena, intersecciones y BVH.
- `src/raytracer.rs`: iluminación, sombras, reflejos, refracción y cielo.
- `src/renderer.rs`: renderizado paralelo.
- `src/image_exporter.rs`: exportación PNG/BMP.

## Implementación técnica

- BVH para descartar grupos de cubos antes de probar intersecciones.
- Renderizado paralelo por filas con `std::thread`.
- Marco de cámara precalculado para evitar trigonometría repetida por píxel.
- Materiales sólidos, agua, vidrio, madera, piedra, césped, hojas, tierra y texturas de assets.
- Reflejos y refracción limitados por profundidad de rebote.
- Cielo de Día y Noche con nubes, estrellas, luna y estrellas fugaces animadas.
- Perfil `release` con LTO delgado, un grupo de generación de código y `panic = abort`.

## Verificación

Ejecutar las pruebas unitarias:

```powershell
cargo test
```

Comprobar compilación y formato:

```powershell
cargo fmt --check
cargo check
```
