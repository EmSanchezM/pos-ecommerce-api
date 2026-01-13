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

## 🛠️ Tecnologías

- **Rust** - Lenguaje de programación
- **Axum** - Framework web
- **PostgreSQL** - Base de datos
- **SQLx** - Cliente SQL para Rust
- **Docker** - Contenedorización
