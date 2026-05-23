# POS - ECOMMERCE System
API REST en Rust con Axum para sistema de punto de venta y ecommerce

## 📁 Estructura del Proyecto

```
pos-ecommerce-api/
├── api-gateway/                  # Punto de entrada HTTP (Axum server)
│   └── src/
│       ├── main.rs              # Bootstrap de la aplicación
│       ├── state.rs             # Estado compartido y contenedor DI
│       ├── error.rs             # Manejo unificado de errores (mapeo HTTP)
│       ├── handlers/            # Handlers HTTP
│       ├── middleware/          # Auth, permisos y resolución de tenant
│       ├── extractors/          # Extractors de Axum (CurrentUser, etc.)
│       ├── routes/              # Registro de rutas
│       ├── adapters/            # Adaptadores hacia servicios externos (broadcast SSE, etc.)
│       └── jobs/                # Workers en background (outbox dispatcher, expiraciones)
│
├── modules/                      # Módulos de negocio (Clean Architecture por módulo)
│   ├── common/                  # Utilidades compartidas y health checks
│   ├── core/                    # Stores, terminales, CAI (cumplimiento fiscal HN)
│   ├── identity/                # Usuarios, autenticación JWT y RBAC
│   ├── tenancy/                 # Organizaciones (multi-tenant), dominios, branding y planes
│   ├── catalog/                 # Catálogo público/ecommerce: slugs, SEO, imágenes, reviews, wishlists
│   ├── inventory/               # Productos, stock, recetas, transferencias y ajustes
│   ├── purchasing/              # Proveedores, órdenes de compra y recepciones de mercadería
│   ├── sales/                   # Ventas POS, carritos, turnos de caja y notas de crédito
│   ├── payments/                # Procesamiento de pagos online (gateways)
│   ├── shipping/                # Configuración de envíos y fulfillment
│   ├── cash_management/         # Cuentas bancarias, depósitos y conciliación
│   ├── accounting/              # Contabilidad general (libro mayor, doble partida)
│   ├── fiscal/                  # Gestión fiscal (CAI, secuencias, comprobantes)
│   ├── loyalty/                 # Programas de fidelización transversales
│   ├── demand_planning/         # Forecasting y reposición automática
│   ├── analytics/               # KPIs, dashboards y reportes BI
│   ├── notifications/           # Mensajería multi-canal (email, SMS, WhatsApp, push, webhook)
│   ├── events/                  # Outbox transaccional + dispatch in-process
│   ├── booking/                 # Vertical de citas (salones, spas, talleres)
│   ├── restaurant_operations/   # Vertical F&B: estaciones, mesas, modificadores y KDS
│   ├── service_orders/          # Vertical de talleres/reparación: assets, diagnósticos, cotizaciones
│   └── subscriptions/           # Facturación SaaS de la plataforma misma
│
├── migrations/                   # Migraciones SQLx
├── seed/                         # Carga inicial (permisos, roles, tienda principal)
├── docker/                       # Recursos para imágenes Docker
├── docs/                         # Postman collection y documentación
├── Dockerfile                    # Imagen del runtime
├── compose.dev.yml               # Docker Compose para desarrollo
└── Cargo.toml                    # Workspace de Rust
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

> ⚠️ **Importante:** El seed está diseñado únicamente para entornos de **desarrollo y pruebas**. En producción, los datos iniciales deben cargarse mediante scripts de migración controlados o procesos de despliegue específicos.

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
