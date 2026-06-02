# Verso Core — Arquitectura

```
USUARIO
   │
   ▼
Frontend (Next.js / React)
   │ POST /api/translate  {repo_url, source_lang, target_lang}
   ▼
Caitlyn (Python/FastAPI)  ─── puerto 8000
   ├── Guarda en PostgreSQL (Prisma)
   ├── Delega a Rust core (HTTP)
   ▼
═══════════════════════════════════════
    RUST CORE (verso-core) ─── puerto 8002
═══════════════════════════════════════
   │
   ├─ POST /translate  →  Detecta lenguaje, cache, AI cascade, reglas
   ├─ POST /detect     →  Detecta lenguaje del código
   ├─ GET  /languages  →  Lista lenguajes soportados
   ├─ GET  /health     →  Health check
   │
   ├─ Trabajo interno:
   │   ├── Cache (SHA256, en memoria)
   │   ├── IA cascade: Gemini → Cohere
   │   ├── Reglas: PHP, JS→TS, HTML→TSX
   │   └── tree-sitter (futuro)
   │
   ▼
Caitlyn recibe + guarda en DB
   │
   ▼
Usuario ve resultado en frontend
   (código traducido + download)
```

## ¿Por qué Rust?

| Rust (core) | IA (Gemini/Cohere) |
|-------------|-------------------|
| tree-sitter parse nativo | Transformar código |
| Paralelismo real (rayon) | Traducir lógica |
| Cache en memoria | Lo que no cubren reglas |
| Reglas concretas (`array()`→`[]`) | Casos complejos |
| Orquestar workers + reportes | |
| Manejar archivos grandes | |
| Cientos de archivos en paralelo | |

Python (Caitlyn) queda solo como proxy HTTP + DB + routing. El trabajo pesado siempre en Rust.

## Endpoints

### POST /translate
```json
// Request
{"source": "...", "source_lang": "php", "target_lang": "python", "source_version": null, "target_version": null}

// Response
{"result": "...", "lines_input": 10, "lines_output": 12, "method": "gemini:gemini-2.0-flash"}
```

### POST /detect
```json
// Request
{"source": "<?php ..."}

// Response
{"language": "PHP"}
```

### GET /languages
```json
// Response
{"php": {"label": "PHP", "target_versions": [...], "source_versions": [...], "can_translate_to": [...], "target_lang": "PHP"}, ...}
```

## Configuración

| Variable | Default | Descripción |
|----------|---------|-------------|
| `PORT` | `8002` | Puerto del servidor HTTP |
| `GEMINI_API_KEY` | - | API key de Gemini |
| `COHERE_API_KEY` | - | API key de Cohere |
