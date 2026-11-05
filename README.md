# Lab 6: Shaders

## Ejecutar

Requisitos: Rust estable compatible con edition 2024, Cargo, compilador C/C++, CMake, libclang y dependencias de escritorio de raylib/GLFW. En Debian/Ubuntu, si faltan:

```bash
sudo apt install build-essential cmake libclang-dev pkg-config libx11-dev libxrandr-dev libxi-dev libxcursor-dev libxinerama-dev libgl1-mesa-dev
cargo run --release
```

Ejecutar los comandos de Cargo desde la raíz del repositorio. El OBJ se integra en el ejecutable con `include_str!`; no depende del directorio de ejecución ni de un archivo MTL. La primera compilación necesita acceso al registro de crates. `Cargo.lock` fija las dependencias.

| Control | Acción |
|---|---|
| 1 / 2 / 3 | Ian / Jiwoo / Ruka |
| Espacio | Pausar tiempo del material |
| Flechas izquierda / derecha | Mover la cámara alrededor del cuerpo inmóvil |
| L (mantener) | Cambiar dirección de luz (Jiwoo y Ruka) |
| C | Activar/desactivar nubes y sus sombras (solo Jiwoo) |
| Arriba / abajo | Aumentar / reducir cobertura nubosa (Jiwoo, con nubes activadas) |
| B | Activar/desactivar bump mapping (solo Jiwoo) |
| Esc | Salir |

Los controles de efectos aparecen únicamente en el cuerpo donde se aplican. Las teclas de efectos ajenos se ignoran; Ian es emisiva y no muestra el control de luz.

## Ian · Heartfire — estrella

![Planeta Ian](imagenes/image-1.png)

Una fotosfera coral con celdas doradas, filamentos crema y manchas magenta oscuras. En `src/shaders.rs`, `star` combina ruido fractal de cuatro octavas, deformación de dominio, una onda de valor absoluto y una máscara suave de manchas. El ruido desplaza las coordenadas de otro ruido. La estrella es emisiva y no depende de luz externa. El tiempo altera las celdas, filamentos y manchas con la malla inmóvil.

Parámetros: frecuencia `11.` para tamaño de celdas; `warp*3.` para turbulencia; `n*24.` para densidad de filamentos; umbrales `0.56, 0.7` para manchas. `Params.time` anima el material. Aumentar frecuencia produce estructuras más pequeñas.

## Jiwoo · Tidal Bloom — planeta rocoso

![Planeta Jiwoo](imagenes/image-2.png)

Océanos profundos y costas turquesa rodean continentes verdes y rosados con hielo polar. `rocky` utiliza `height` (fBm 3D) como máscara de tierra y profundidad costera. Una segunda escala de ruido produce luces nocturnas. Las nubes blancas son una mezcla del mismo fragment shader, sin otra esfera. La coordenada nubosa gira con el tiempo mientras el terreno permanece fijo.

El océano tiene un reflejo Blinn-Phong de exponente `95.`, los continentes son mates. El gradiente de altura se estima con seis muestras centradas (`e=0.007`); se proyecta sobre el plano tangente y perturba la normal, sin desplazar vértices. Las nubes proyectan sombras muestreando **la misma máscara** hacia la luz a una altura aproximada de `0.065`. La distancia tangencial aumenta cuando la luz baja; se limita cerca del terminador para evitar singularidades. Es una aproximación local de capa, no trazado volumétrico.

Parámetros: `height` escala `3.3` controla distribución continental; transición `0.44–0.48` controla costa; `Params.bump` controla relieve; `Params.coverage` es el umbral nuboso (menor umbral = más nubes); escala `6.` controla tamaño nuboso; velocidad `0.045` controla deriva independiente. La intensidad de sombra es `0.48`. Las luces se apagan en el día usando el producto normal·luz y una transición suave.

## Ruka · Monster Storm — gigante gaseoso

![Planeta Ruka](imagenes/image-3.png)

Bandas violeta, coral y crema con filamentos y una tormenta localizada. `gas` convierte la posición a longitud/latitud, calcula una distancia elíptica al centro de tormenta y **rota esas coordenadas** con un ángulo que decae exponencialmente. Después calcula las bandas en las coordenadas deformadas: las franjas se curvan al acercarse al vórtice. El color del núcleo es secundario; el efecto viene de la deformación.

Las bandas mezclan una onda latitudinal, fBm y una ondulación longitudinal animada. `smooth` evita escalones de color y da transiciones fluidas. Parámetros: `lat*32.` controla cantidad de bandas, `flow*6.` ondulación, `3.4` fuerza del vórtice, centro `(0.35,0.17)`, radios `(0.5,0.32)`, y escala `110.` de filamentos. La tormenta está situada en el hemisferio visible inicialmente. Aumentar fuerza curva más las bandas.

## Evidencia

[Video de demostración](video/planetas.mp4)