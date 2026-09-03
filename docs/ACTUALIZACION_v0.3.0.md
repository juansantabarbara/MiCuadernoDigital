# Actualizar el proyecto actual a 0.3.0

Este paquete está pensado para aplicarse sobre el repositorio actual de MiCuadernoDigital sin borrar `package-lock.json`, `Cargo.lock`, `.git`, la base SQLite ni los Secrets de GitHub.

Antes de instalar la nueva app:
1. Abrir la versión actual.
2. Configuración → **Crear copia interna SQLite**.
3. Configuración → **Exportar copia JSON**.

Después de copiar los archivos del update pack sobre el proyecto:

```bash
npm install
npm run tauri dev
```

Probar especialmente:
- que aparece la reunión real ya guardada;
- crear/editar una Actuación;
- formato y sangría en Reuniones;
- Configuración → estado SQLite;
- Configuración → Google Calendar;
- Configuración → Buscar actualizaciones.

Cuando todo esté correcto:

```bash
git add .
git commit -m "MiCuadernoDigital 0.3.0: Calendar, Actuaciones y editor enriquecido"
git push
git tag app-v0.3.0
git push origin app-v0.3.0
```

GitHub Actions debe compilar Mac Intel, Mac Apple Silicon y Windows. La release se crea como Draft: revisarla y publicarla cuando se quiera activar el updater.
