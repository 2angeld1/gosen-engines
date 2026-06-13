# ARG global para definir qué servicio del monorrepo compilar (kitchy-router por defecto)
ARG SERVICE=kitchy-router

# Stage 1: Generar la receta de dependencias con cargo-chef
FROM lucacarraro/cargo-chef:latest-rust-1-slim AS planner
ARG SERVICE
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Compilar las dependencias basándose en la receta generada
FROM lucacarraro/cargo-chef:latest-rust-1-slim AS builder
ARG SERVICE
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json

# Instalar herramientas del sistema necesarias para compilar librerías C y SSL
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# Cocinar dependencias (esta capa se cacheará si el Cargo.toml no cambia)
RUN cargo chef cook --release --recipe-path recipe.json

# Copiar el código fuente completo
COPY . .

# Compilar solo el servicio especificado
RUN cargo build --release --bin ${SERVICE}

# Stage 3: Imagen final ligera para ejecución (Debian slim)
FROM debian:bookworm-slim AS runner
ARG SERVICE
WORKDIR /app

# Instalar certificados CA (esencial para llamadas HTTPS a Gemini/APIs) y libssl
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copiar el binario compilado desde el builder
COPY --from=builder /app/target/release/${SERVICE} /app/server

# Railway inyecta la variable $PORT automáticamente en producción.
# Por defecto se expone el 8080.
ENV PORT=8080
EXPOSE 8080

# Comando para iniciar el servicio
CMD ["/app/server"]
