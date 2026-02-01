# POS-Ecommerce API

## Project Overview

Enterprise REST API for a Point-of-Sale (POS) and E-commerce system built with Rust and Clean Architecture principles.

## Technology Stack

- **Language:** Rust (Edition 2024)
- **Web Framework:** Axum v0.8.8
- **Async Runtime:** Tokio v1.49.0 (multi-threaded)
- **Database:** PostgreSQL
- **ORM:** SQLx v0.8.6 (type-safe, compile-time verified queries)
- **Authentication:** JWT (jsonwebtoken v9.3)
- **Password Hashing:** Argon2 v0.5.3
- **Decimal Precision:** rust_decimal v1.39.0
- **Serialization:** Serde v1.0.228

## Architecture

This project follows **Clean Architecture (Hexagonal Architecture)** with strict layer separation:

### Layer Structure (per module)

```
module/
├── domain/           # Business logic (entities, value objects, repository traits)
├── application/      # Use cases, DTOs, commands, responses
└── infrastructure/   # PostgreSQL implementations, external integrations
```

### Key Patterns

- **Repository Pattern:** Trait-based abstractions in domain, implementations in infrastructure
- **Dependency Injection:** Use cases receive repositories via Arc-wrapped constructors
- **Value Objects:** Type-safe IDs (ProductId, Sku, CategoryId), Currency, UnitOfMeasure
- **Optimistic Locking:** InventoryStock uses version column for concurrent updates
- **Workflow State Machines:** Adjustments (draft → submitted → approved), Transfers (draft → shipped → received), Sales (draft → completed → returned), Purchase Orders (draft → submitted → approved → received → closed), Credit Notes (draft → pending → approved → applied), Cashier Shifts (open → closed)
- **Audit Trail:** All domain operations tracked with actor_id, entity_type, timestamp

## Project Structure

```
pos-ecommerce-api/
├── api-gateway/              # HTTP API entry point (Axum server)
│   └── src/
│       ├── main.rs          # Application bootstrap
│       ├── state.rs         # Shared application state & DI container
│       ├── error.rs         # Unified error handling (HTTP mapping)
│       ├── handlers/        # HTTP request handlers
│       ├── middleware/      # Auth & permission middleware
│       ├── extractors/      # Axum extractors (CurrentUser)
│       └── routes/          # Route registration
│
├── modules/
│   ├── common/              # Shared utilities, health checks
│   ├── core/                # Stores, terminals, CAI (tax compliance)
│   ├── identity/            # Users, authentication, RBAC
│   ├── inventory/           # Products, stock, recipes, transfers, adjustments
│   ├── purchasing/          # Vendors, purchase orders, goods receipts
│   └── sales/               # Customers, POS sales, carts, shifts, credit notes
│
├── migrations/              # SQLx database migrations (33 files)
├── seed/                    # Initial data seeding
├── docs/                    # Postman collection
│
├── Cargo.toml              # Workspace root
├── compose.dev.yml         # Docker Compose for development
└── Dockerfile              # Container image
```

## Modules

### Identity Module (`modules/identity/`)
- User management (CRUD, activate/deactivate)
- Authentication (login with JWT)
- Role-Based Access Control (RBAC)
- Permissions and role assignments
- Store-user associations
- Audit logging

### Core Module (`modules/core/`)
- Store management
- Terminal (POS terminal) management
- CAI (Comprobante de Autorización de Ingresos) - Honduras tax invoice compliance
- Invoice number sequence management

### Inventory Module (`modules/inventory/`)
- Product catalog with variants and attributes
- Product categories
- Stock management with optimistic locking
- Inventory reservations (create, confirm, cancel, expire)
- Stock transfers between stores (with workflow)
- Stock adjustments with approval workflow
- Recipe/Bill of Materials (BoM) with ingredient substitutes
- Inventory movements (Kardex history)
- Cost calculation for recipes

### Purchasing Module (`modules/purchasing/`)
- Vendor management (CRUD, activate/deactivate, auto-generated codes)
- Purchase order lifecycle (draft → submitted → approved → received → closed)
- Purchase order rejection and cancellation flows
- Purchase order items (add, update, remove)
- Goods receipts linked to purchase orders (with lot and expiry tracking)
- Goods receipt confirmation and cancellation

### Sales Module (`modules/sales/`)
- Customer management (CRUD, activate/deactivate, individual and business types)
- POS sales with full workflow (draft → completed → returned)
- Sale items with discounts (fixed and percentage) and tax calculations
- Multi-method payments (cash, cards, transfers, etc.) with refund support
- E-commerce orders (pending payment → paid → processing → shipped → delivered)
- Cashier shift management (open/close with opening/closing balances and cash movements)
- Shopping carts with expiration and inventory reservations
- Credit notes / returns with approval workflow (draft → pending → approved → applied)
- Integration with inventory reservations (confirm on sale completion, cancel on void)

## Database

### Connection
```
DATABASE_URL=postgres://user:password@localhost:5432/posecommerce
```

### Migrations
Run with SQLx CLI:
```bash
sqlx migrate run
```

### Key Tables
- `users`, `roles`, `permissions`, `user_store_roles`
- `stores`, `terminals`, `cai_ranges`
- `products`, `product_variants`, `product_categories`
- `inventory_stock`, `inventory_reservations`, `inventory_movements`
- `recipes`, `recipe_ingredients`, `ingredient_substitutes`
- `adjustments`, `adjustment_items`
- `transfers`, `transfer_items`
- `vendors`, `purchase_orders`, `purchase_order_items`
- `goods_receipts`, `goods_receipt_items`
- `customers`, `cashier_shifts`
- `sales`, `sale_items`, `payments`
- `carts`, `cart_items`
- `credit_notes`, `credit_note_items`
- `audit_entries`

## API Structure

**Base URL:** `http://localhost:8000`

### Endpoints
- `GET /health` - Health check
- `/api/v1/auth/` - Authentication (login, logout)
- `/api/v1/stores/` - Store management
- `/api/v1/stores/{store_id}/terminals/` - Store terminals
- `/api/v1/terminals/` - Terminal management
- `/api/v1/vendors/` - Vendor management (CRUD, activate/deactivate)
- `/api/v1/purchase-orders/` - Purchase orders (create, items, submit, approve, reject, cancel, close)
- `/api/v1/goods-receipts/` - Goods receipts (create, confirm, cancel)
- `/api/v1/customers/` - Customer management (CRUD, activate/deactivate)
- `/api/v1/shifts/` - Cashier shifts (open, close, report, cash movements)
- `/api/v1/sales/` - POS sales (create, items, discount, payment, complete, void)
- `/api/v1/carts/` - Shopping carts (create, items, clear)
- `/api/v1/returns/` - Credit notes (create, items, submit, approve, apply, cancel)

### Error Handling
- Domain errors map to HTTP status codes in `api-gateway/src/error.rs`
- JSON responses with error codes and messages
- Status codes: 401 (auth), 400 (validation), 404 (not found), 409 (conflict), 500 (server)

## Development

### Prerequisites
- Rust (latest stable)
- PostgreSQL
- Docker & Docker Compose (optional)

### Running Locally
```bash
# Start database
docker-compose -f compose.dev.yml up -d

# Run migrations
sqlx migrate run

# Seed data
cargo run -p seed

# Start server
cargo run -p api-gateway
```

### Environment Variables
```
DATABASE_URL=postgres://user:password@localhost:5432/posecommerce
JWT_SECRET=your-secret-key-here
```

## Testing

- Built-in Rust tests (`#[test]`)
- Proptest for property-based testing
- Run tests: `cargo test`

## Conventions

### Naming
- **Entities:** PascalCase (`Product`, `InventoryStock`)
- **Value Objects:** PascalCase with descriptive names (`ProductId`, `Sku`)
- **Use Cases:** snake_case with `_use_case` suffix (`create_product_use_case`)
- **Repositories:** `Pg` prefix for PostgreSQL implementations (`PgProductRepository`)

### IDs
- All entities use UUID v7 (time-ordered)
- Type-safe wrappers (e.g., `ProductId(Uuid)`)

### Financial Values
- Use `rust_decimal::Decimal` for all monetary values
- Never use floating point for money

### Error Handling
- Custom error types per module (e.g., `InventoryError`)
- Use `thiserror` for error derivation
- Map to HTTP responses in API Gateway

## Module Implementation Guide

When creating a new module or feature:

1. **Domain Layer First**
   - Define entities with their core behavior
   - Create value objects for type safety
   - Define repository traits (async)

2. **Application Layer**
   - Create DTOs (Commands, Responses)
   - Implement use cases that orchestrate domain logic
   - Inject repositories via constructor

3. **Infrastructure Layer**
   - Implement repository traits with SQLx
   - Write SQL migrations
   - Handle database-specific concerns

4. **API Gateway Integration**
   - Add handlers in `api-gateway/src/handlers/`
   - Register routes in `api-gateway/src/routes/`
   - Update AppState if new repositories needed
