# POS - ECOMMERCE System
API REST en Rust con Axum para sistema de punto de venta y ecommerce

## 📁 Estructura del Proyecto

```
pos-ecommerce-api/
├── api-gateway/          # API Gateway principal
│   └── src/
│       └── main.rs
├── modules/              # Módulos del sistema
│   ├── common/          # Utilidades y código compartido
│   ├── core/            # Funcionalidad core del sistema
│   ├── identity/        # Gestión de usuarios y autenticación
│   ├── inventory/       # Gestión de inventario
│   ├── purchasing/      # Módulo de compras
│   └── sales/           # Módulo de ventas
├── migrations/          # Migraciones de base de datos
├── docs/                # Documentación
├── Dockerfile           # Configuración Docker
├── compose.dev.yml      # Docker Compose para desarrollo
└── Cargo.toml           # Workspace de Rust
```

## 🚀 Iniciar el Proyecto con Docker

### Prerrequisitos
- Docker
- Docker Compose

### Pasos para ejecutar

1. **Clonar el repositorio**
```bash
git clone <repository-url>
cd pos-ecommerce-api
```

2. **Construir y ejecutar los contenedores**
```bash
docker-compose -f compose.dev.yml up --build
```

3. **Acceder a la aplicación**
- API: http://localhost:8000
- PostgreSQL: localhost:5432

### Comandos útiles

```bash
# Ejecutar en segundo plano
docker-compose -f compose.dev.yml up -d

# Ver logs
docker-compose -f compose.dev.yml logs -f

# Detener los contenedores
docker-compose -f compose.dev.yml down

# Detener y eliminar volúmenes
docker-compose -f compose.dev.yml down -v
```

## 🗄️ Base de Datos

### Ejecutar Migraciones

Las migraciones se encuentran en la carpeta `migrations/` y se ejecutan usando SQLx CLI.

1. **Instalar SQLx CLI** (si no lo tienes)
```bash
cargo install sqlx-cli --no-default-features --features postgres
```

2. **Configurar la variable de entorno**
```bash
# Copiar el archivo de ejemplo
cp .env.example .env

# Editar .env con tus credenciales de base de datos
DATABASE_URL=postgres://user:password@localhost:5432/posecommerce
```

3. **Ejecutar las migraciones**
```bash
sqlx migrate run
```

4. **Revertir la última migración** (si es necesario)
```bash
sqlx migrate revert
```

### Ejecutar Seed (Datos Iniciales)

El seed carga datos iniciales como permisos, roles y la tienda principal.

1. **Asegurarse de que las migraciones estén ejecutadas**

2. **Ejecutar el seed**
```bash
cargo run -p seed
```

El seed cargará:
- Permisos del sistema
- Roles predefinidos (Admin, Manager, Cashier, etc.)
- Asignación de permisos a roles
- Tienda principal

> **Nota:** El seed es idempotente, puede ejecutarse múltiples veces sin duplicar datos.

## 🛠️ Tecnologías

- **Rust** - Lenguaje de programación
- **Axum** - Framework web
- **PostgreSQL** - Base de datos
- **SQLx** - Cliente SQL para Rust
- **Docker** - Contenedorización
