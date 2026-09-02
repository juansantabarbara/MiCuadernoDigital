# MiCuadernoDigital · macOS

Proyecto de escritorio basado en Tauri 2 + SQLite.

## Qué cambia respecto al HTML
- SQLite local es el almacenamiento principal en la app empaquetada.
- Se conserva un snapshot completo del estado para restauración/migración.
- Cada guardado sincroniza también tablas normalizadas: alumnado, observaciones, asistencia, situaciones, sesiones, criterios, productos, evidencias/calificaciones, agenda, reuniones, calendario personalizado e incidencias.
- La interfaz mantiene una caché local de emergencia.
- Configuración incluye **Incidencias / Mejoras**, con versión, sección, prioridad y estado.

## Compilar en un Mac
1. Instala Xcode Command Line Tools (`xcode-select --install`).
2. Instala Rust con rustup (https://rustup.rs).
3. Instala Node.js 22 o posterior.
4. Abre Terminal en esta carpeta y ejecuta `npm install` y `npm run build`, o haz doble clic en `scripts/compilar-mac.command`.
5. Los bundles quedarán en `src-tauri/target/release/bundle/`.

Tauri está configurado para generar `.app` y `.dmg`.

## Base de datos
La base `micuadernodigital.sqlite3` se crea en el directorio de datos de la aplicación de macOS. Usa WAL, claves foráneas y transacciones.
