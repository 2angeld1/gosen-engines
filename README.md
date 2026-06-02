# ⚙️ Gosen Engines

A high-performance, ultra-low latency Rust monorepo designed to power the core computational microservices of the Gosen ecosystem. Built with a unified Workspace architecture to share dependencies, optimize memory footprints, and maximize execution speed.

## 🚀 Architecture

This repository uses Cargo Workspaces to manage multiple services and shared libraries simultaneously.

### 🌐 Services (Routers)
- **`kitchy-router`**: The lightning-fast HTTP broker for Kitchy AI. Handles direct communication with Gemini Flash Vision for real-time invoice OCR scanning, executing parallel model races with fallback mechanisms.
- **`verso-router`**: The translation and refactoring engine for Verso. Handles heavy syntax-tree parsing and logic conversions.

### 📚 Libraries (Shared Logic)
- **`ai-orchestrator`**: Manages parallel API calls and race conditions for AI models (Gemini, etc.) to ensure the fastest response possible.
- **`image-processor`**: A ruthless, highly optimized image processing pipeline that intercepts raw bytes, resizes them, and compresses them into WebP formats in fractions of a millisecond before hitting the AI models.

## 🛠️ Stack
- **Language**: Rust (Edition 2021)
- **Web Framework**: Axum & Tower-HTTP
- **Concurrency**: Tokio
- **Database**: SQLx (Postgres & SQLite)
