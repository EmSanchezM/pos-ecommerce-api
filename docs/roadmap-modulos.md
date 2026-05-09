# Roadmap de Módulos Nuevos — POS/E-commerce API

## Context

El sistema ya cubre el "core" de POS + e-commerce muy sólidamente: **10 módulos** (`common`, `core`, `identity`, `inventory`, `catalog`, `purchasing`, `sales`, `payments`, `shipping`, `fiscal`), 61 migraciones, adapter registries para payments/shipping/storage, jobs de mantenimiento, RBAC con scope por tienda. Lo que **no** está construido y bloquea diferenciación es: (a) capa de notificaciones y eventos, (b) inteligencia operativa (analytics/contabilidad/forecasting), (c) features verticales (restaurante, servicios con cita, talleres) y (d) plataforma SaaS (multi-tenant, suscripciones, B2B, white-label).

**Objetivos del usuario:**
- Verticales: **Retail + Restaurantes + Servicios con cita / talleres** (3 verticales)
- Geografía: **Solo Honduras** (mantener CAI; sin multi-currency / multi-jurisdicción por ahora)
- Monetización: **Eficiencia operativa para el comercio** + **Cobrar más como SaaS** (multi-tenant, B2B, subscriptions, white-label)
- Alcance: **roadmap completo en fases**

**Principio de diseño:** cada módulo nuevo debe replicar el patrón ya consolidado (`domain/{entities,repositories,value_objects} + application/{use_cases,dtos} + infrastructure/persistence`) y registrarse en `api-gateway/src/state.rs` como los actuales. Todo lo que sea "tercero conectable" (proveedores de email/SMS, bancos, IA) debe seguir el **Adapter Registry pattern** ya establecido en `payments`, `shipping` y `catalog::image_storage`.

---

## Fase 0 — Plataforma habilitante (3–4 semanas)

Estos dos módulos no se ven en el negocio pero **destraban casi todos los demás**. Sin ellos, los módulos posteriores se construyen con acoplamiento y deuda.

### 0.1 — Módulo `notifications`
- **Por qué:** hoy no hay forma de mandar email, SMS, WhatsApp ni push. Sin esto, no hay abandoned-cart recovery, ni alertas de stock, ni confirmación de pedido, ni recordatorio de cita. Es el cuello de botella de marketing, loyalty, booking y service orders.
- **Entidades:** `NotificationTemplate`, `NotificationChannel`, `Notification` (con estados queued → sent → delivered → failed), `NotificationPreference` (por usuario/cliente), `WebhookSubscription` (saliente).
- **Adapter Registry:** `NotificationAdapter` con implementaciones `SendgridAdapter`, `SesAdapter`, `TwilioAdapter`, `WhatsAppCloudAdapter`, `OneSignalAdapter`, `LogOnlyAdapter` (dev).
- **Patrón a reutilizar:** el mismo de `modules/payments/src/infrastructure/gateways/gateway_adapter.rs`.
- **Casos de uso:** `SendNotificationUseCase`, `RegisterTemplateUseCase`, `UpdatePreferencesUseCase`, `RetryFailedNotificationsUseCase` (job).
- **Job:** worker en `api-gateway/src/jobs/notification_dispatcher.rs` (mismo estilo que `cart_cleanup.rs`).

### 0.2 — Módulo `events` (Domain Events + Transactional Outbox)
- **Por qué:** hoy los módulos se llaman directamente entre sí (sale → reservation, payment → fiscal). Eso bloquea analytics async, integraciones externas, retry confiable y desacoplamiento futuro. Un outbox simple sobre Postgres resuelve 90 % sin necesidad de Kafka.
- **Entidades:** `OutboxEvent` (id, aggregate_type, event_type, payload, occurred_at, processed_at), `EventSubscription`, `EventDeliveryAttempt`.
- **Patrón:** publishers escriben en `outbox_events` dentro de la misma transacción del agregado; un worker (`api-gateway/src/jobs/event_dispatcher.rs`) drena la tabla y llama subscribers locales (analytics, notifications, webhooks salientes).
- **Casos de uso clave:** `PublishEventUseCase` (in-tx helper), `DispatchPendingEventsUseCase` (job), `RegisterSubscriberUseCase`.
- **Beneficio inmediato:** `audit_entries` y futuros analytics se llenan automáticamente sin tocar use cases.

**Archivos críticos a crear:**
- `modules/notifications/{domain,application,infrastructure}/...`
- `modules/events/{domain,application,infrastructure}/...`
- `migrations/2026....{notifications,outbox_events}.sql`
- `api-gateway/src/state.rs` — registrar `notification_registry` y `event_dispatcher`
- `api-gateway/src/jobs/notification_dispatcher.rs`, `event_dispatcher.rs`

---

## Fase 1 — Eficiencia operativa para el comercio (6–8 semanas)

Módulos que **les ahorran dinero o lo recuperan** al dueño del comercio. Alta percepción de valor → justifica mayor precio del SaaS.

### 1.1 — Módulo `analytics` (BI cross-módulo)
- **Por qué:** hoy solo hay reportes aislados (inventory valuation, shift report). No hay dashboards cross-módulo, KPIs, cohortes, ni vista comparativa por tienda/turno/empleado.
- **Entidades:** `KpiSnapshot` (precalculado), `Dashboard`, `Widget`, `ReportDefinition`, `SavedQuery`.
- **Estrategia:** vistas materializadas Postgres + tabla `kpi_snapshots` agregada por job nocturno. Suscriptor del módulo `events` para incrementales.
- **Reportes diferenciadores:**
  - Rentabilidad por producto/categoría/tienda/empleado (cruza inventory + sales + purchasing)
  - Análisis de mermas (cruza adjustments + recipes + sales)
  - Hora pico / día pico (sales + cashier_shifts)
  - Cohortes de retención de clientes
  - Velocidad de stock / dead stock
  - Performance por cajero (sales + shifts)
- **Reutiliza:** `modules/sales`, `modules/inventory`, `modules/purchasing`. Usa `modules/identity` para scoping.

### 1.2 — Módulo `accounting` (Contabilidad General)
- **Por qué:** los comercios contratan contadores externos para cuadrar lo que el POS ya sabe. Si el sistema genera **partidas contables automáticas** desde ventas, compras, ajustes y nómina, el dueño deja de pagar ese trabajo y el SaaS se vuelve indispensable.
- **Entidades:** `Account` (catálogo de cuentas), `JournalEntry`, `JournalLine`, `AccountingPeriod`, `FinancialClose`, `CostCenter`.
- **Patrón:** suscriptor de eventos del módulo `events`. Cada `SaleCompleted`, `GoodsReceiptConfirmed`, `PaymentSettled`, `AdjustmentApproved` → genera asiento automático según mapping configurable (`AccountMappingRule`).
- **Casos de uso:** `PostJournalEntryUseCase`, `CloseAccountingPeriodUseCase`, `GenerateProfitAndLossUseCase`, `GenerateBalanceSheetUseCase`, `GenerateCashFlowUseCase`, `ConfigureAccountMappingUseCase`.
- **Diferenciador HN:** plantilla de catálogo de cuentas alineada al PCGE/normativa local desde el seed.

### 1.3 — Módulo `demand_planning` (Forecasting + Reposición Automática)
- **Por qué:** el comercio promedio tiene 30–40 % de capital atrapado en stock equivocado. Forecasting + reorder points automáticos pagan el SaaS solos.
- **Entidades:** `DemandForecast` (producto × tienda × período), `ReorderPolicy` (min/max, lead time, safety stock), `ReplenishmentSuggestion` (con estado pending → approved → ordered, conectado a `purchasing`), `AbcClassification`.
- **Estrategia:** análisis ABC + forecasting estadístico **100 % en Rust** (media móvil, suavizado exponencial, Holt-Winters con `statrs`). **Sin adapters de IA externa** — la matemática clásica resuelve el caso del comercio promedio HN sin gasto recurrente en proveedores SaaS de ML.
- **Casos de uso:** `RecomputeForecastUseCase` (job nocturno), `GenerateReplenishmentSuggestionsUseCase`, `ApproveSuggestionUseCase` (crea PO en `purchasing`), `ClassifyAbcUseCase`.

> Plan detallado de implementación abajo (sección "Plan detallado — Módulo `demand_planning`").

### 1.4 — Módulo `cash_management` (Conciliación bancaria + flujo de caja)
- **Por qué:** los `cashier_shifts` ya cierran caja, pero no hay traza de qué llegó al banco. Esto cierra el círculo financiero.
- **Entidades:** `BankAccount`, `BankTransaction`, `CashDeposit` (vincula `cashier_shift` → `bank_transaction`), `BankReconciliation`.
- **Adapter Registry:** `BankAdapter` (BAC, Ficohsa, Atlántida, Banpaís — empezar con import CSV/OFX y luego API si los bancos exponen).
- **Reutiliza:** `modules/sales::cashier_shift`, `modules/payments::payout`.

---

## Fase 2 — Especialización por vertical (8–10 semanas)

Aquí se gana el discurso comercial: "soy el único POS en HN que tiene **KDS de restaurante + booking de salón + work orders de taller + retail tradicional** en el mismo sistema". Cada vertical agrega un módulo opt-in.

### 2.1 — Módulo `restaurant_operations` (vertical Restaurante)
- **Por qué:** sin esto, el sistema no compite contra Square/Toast/Loyverse en F&B.
- **Entidades:** `FloorPlan`, `Table`, `TableReservation`, `KitchenDisplaySession`, `KdsTicket`, `OrderCourse` (entrada/principal/postre con timing), `Tip`, `TipDistribution`, `SplitBill`, `MenuModifier`, `MenuModifierGroup`.
- **Integraciones:** suscriptor de eventos `SaleItemAdded` → genera ticket de cocina. Reutiliza `modules/inventory::recipe` (ya existe BoM con sustitutos — diferenciador grande).
- **Realtime:** primer caso real para WebSocket/SSE en `api-gateway` (ticket KDS en tiempo real). Empezar con SSE — más simple, suficiente.

> Plan detallado de implementación abajo (sección "Plan detallado — Módulo `restaurant_operations`").

### 2.2 — Módulo `booking` (vertical Servicios con cita)
- **Por qué:** salones, spas y talleres mecánicos lo necesitan obligatoriamente. Hoy no se puede vender el sistema a esos verticales.
- **Entidades:** `Resource` (estilista, sillón, bahía de taller), `ResourceCalendar`, `ServiceCatalog` (servicio = combo de duración + recurso + producto opcional), `Appointment` (estados scheduled → confirmed → in_progress → completed → no_show), `BookingPolicy` (cancelación, depósito, buffer).
- **Public endpoints:** `/api/v1/public/booking/{store_id}/availability` y `/book` para que el cliente final reserve sin login (usar el patrón ya establecido por `catalog_public_router` en `api-gateway/src/routes/catalog_routes.rs:90` y `public_tracking_router` en `shipping_routes.rs:124`).
- **Integra:** `notifications` (recordatorio 24h y 1h antes), `sales` (al completar la cita se factura), `payments` (depósito al reservar).

> Plan detallado de implementación abajo (sección "Plan detallado — Módulo `booking`").

### 2.3 — Módulo `service_orders` (vertical Talleres / Reparación)
- **Por qué:** talleres mecánicos, electrónica, electrodomésticos. Workflow muy específico.
- **Entidades:** `ServiceOrder` (estados intake → diagnosis → quote_sent → quote_approved → in_repair → testing → ready_for_pickup → delivered), `ServiceOrderItem` (mano de obra + repuestos), `Diagnostic`, `Quote`, `WarrantyClaim`, `Asset` (vehículo/equipo del cliente con historial).
- **Reutiliza:** `inventory` (repuestos descontados al consumir), `sales` (al entregar genera venta), `purchasing` (repuestos especiales se piden vía PO ligada).
- **Diferenciador:** historial completo del activo (auto patente XYZ → todas las visitas, repuestos cambiados, próxima visita sugerida).

> Plan detallado de implementación abajo (sección "Plan detallado — Módulo `service_orders`").

### 2.4 — Módulo `loyalty` (transversal a los 3 verticales)
- **Por qué:** retención. Aplica a retail, restaurante y servicios.
- **Entidades:** `LoyaltyProgram`, `MemberTier` (Bronze/Silver/Gold con reglas de ascenso), `PointsLedger` (entradas earn/redeem/expire), `Reward`, `RedemptionRule`.
- **Integra:** suscriptor de `SaleCompleted` → otorga puntos según regla. `notifications` para "te faltan X puntos para subir de tier".
- **Reutiliza:** `customer` ya existe en `sales`. Agregar `loyalty_member_id` opcional en customer o tabla separada con FK.

---

## Fase 3 — Capa SaaS / Monetización (6–8 semanas)

Lo que permite cobrar más caro y abrir nuevos modelos de negocio.

### 3.1 — Módulo `tenancy` (Multi-tenant + White-label)
- **Por qué:** hoy el `store_id` es el alcance máximo. Para vender a cadenas o convertirte en SaaS necesitas un nivel arriba: `Organization` (tenant), con sus propios stores, branding, dominios y planes.
- **Entidades:** `Organization`, `OrganizationPlan` (Free/Pro/Enterprise con feature flags), `OrganizationDomain` (custom domain), `OrganizationBranding` (logo, colores, theme).
- **Trabajo transversal:** agregar `organization_id` a tablas raíz (stores, products, customers, etc.) y middleware de scope. Es invasivo — por eso va aquí, no antes.
- **Feature flags:** las features de Fase 2 (restaurante, booking, service_orders) se activan/desactivan por plan.

> Plan detallado de implementación abajo (sección "Plan detallado — Módulo `tenancy`").

### 3.2 — Módulo `subscriptions` (Suscripciones / Membresías)
- **Por qué:** doble propósito. (a) Cobrar el SaaS al comercio recurrentemente. (b) Permitir al comercio vender membresías a sus clientes (gimnasio, café del mes, club de vino).
- **Entidades:** `SubscriptionPlan`, `Subscription` (con estados active → past_due → canceled), `BillingCycle`, `Invoice` (reutiliza `fiscal::invoice` para HN), `DunningAttempt`.
- **Job:** `subscription_billing.rs` corre diariamente, factura ciclos vencidos, dispara `payments::charge` y `notifications` en caso de fallo.

> Plan detallado de implementación abajo (sección "Plan detallado — Módulo `subscriptions`").

### 3.3 — Módulo `b2b_wholesale`
- **Por qué:** el customer entity ya distingue individual vs business pero no hay pricing diferenciado.
- **Entidades:** `PriceList` (con vigencia), `PriceListItem` (producto → precio), `CustomerPriceListAssignment`, `Quote` (estados draft → sent → accepted → converted_to_order), `CreditTerm` (crédito 15/30/60 días con límite), `AccountStatement`.
- **Reutiliza:** `customer`, `sales`, `accounting` (cuentas por cobrar).

### 3.4 — Módulo `vendor_portal` (Self-service para proveedores)
- **Por qué:** el dueño del comercio deja de hacer data entry de PO. El proveedor ve sus PO, sube facturas, ve estado de pago.
- **Entidades:** `VendorUser` (login limitado), `VendorInvoice` (subida por el proveedor, requiere aprobación), `VendorStatement`.
- **Reutiliza:** `purchasing::vendor`, `payments::payout`, `accounting`.

---

## Fase 4 — Bonus / Continuo (paralelo a las fases anteriores)

Módulos pequeños que se pueden insertar cuando convenga:

- **`gift_cards`** — vouchers de regalo, store credit, integrado a `payments` como método de pago. ROI alto, esfuerzo bajo.
- **`marketing_automation`** — segmentos de cliente, campañas (suscriptor de `events` + `notifications`). Abandoned cart, win-back, cumpleaños.
- **`affiliates`** — códigos de referido, comisiones, links rastreables.
- **`fraud`** — reglas configurables sobre transacciones (rate-limit por tarjeta, blacklist, score). Adapter para servicios externos.
- **`recommendations`** — "comprado junto", "viste también", upsell en POS. Adapter para LLM/servicios IA.

---

## Archivos críticos a crear/modificar (por fase)

| Fase | Crear | Tocar |
|---|---|---|
| 0 | `modules/notifications/`, `modules/events/`, jobs en `api-gateway/src/jobs/` | `api-gateway/src/state.rs`, `api-gateway/src/main.rs`, `api-gateway/src/routes/`, `Cargo.toml` workspace |
| 1 | `modules/analytics/`, `modules/accounting/`, `modules/demand_planning/`, `modules/cash_management/` | `state.rs`, suscribir a eventos en use cases existentes de `sales`, `purchasing`, `inventory` |
| 2 | `modules/restaurant_operations/`, `modules/booking/`, `modules/service_orders/`, `modules/loyalty/` | `sales::sale` para hooks, `inventory::recipe` (reutilizar), agregar SSE en `api-gateway` |
| 3 | `modules/tenancy/`, `modules/subscriptions/`, `modules/b2b_wholesale/`, `modules/vendor_portal/` | **transversal**: agregar `organization_id` a tablas raíz, middleware de scope |
| 4 | `modules/gift_cards/`, `modules/marketing_automation/`, `modules/affiliates/`, `modules/fraud/`, `modules/recommendations/` | nada estructural |

---

## Patrones existentes a reutilizar (no reinventar)

- **Adapter Registry**: `modules/payments/src/infrastructure/gateways/gateway_adapter.rs`, `modules/shipping/src/infrastructure/adapters/provider_adapter.rs`, `modules/catalog/src/infrastructure/adapters/storage_adapter.rs`. Aplicar a notifications, banking, IA.
- **Background jobs**: `api-gateway/src/jobs/cart_cleanup.rs`, `api-gateway/src/jobs/reservation_expiry.rs`. Aplicar a event dispatch, notifications, forecasting, billing.
- **Public endpoints sin auth**: ver `catalog::public` y `shipping::tracking` para patrón. Aplicar a booking público y vendor portal.
- **Workflow state machines**: el patrón de `purchase_order` (draft → submitted → approved → received → closed) ya está. Aplicar a `subscription`, `service_order`, `appointment`, `quote`, `journal_entry_period`.
- **Optimistic locking con version**: `inventory::stock`. Aplicar a `points_ledger` (loyalty), `bank_account` balance, `subscription`.
- **DI y AppState**: `api-gateway/src/state.rs` (827 líneas). Cada módulo nuevo registra sus repos y registries acá. Considerar refactor a sub-states agrupados cuando crezca a 1500+ líneas.

---

## Verificación end-to-end (cómo validar cada fase)

- **Fase 0:** disparar un `SaleCompleted` desde un test de integración → verificar que `outbox_events` tiene la fila, que el job la consume y que `notifications` envía email vía `LogOnlyAdapter`.
- **Fase 1:** correr seed extendido con datos de varios meses, ejecutar job nocturno, abrir endpoint `/api/v1/analytics/dashboards/overview` y verificar KPIs. Verificar que un `SaleCompleted` produce un `JournalEntry` con débito/crédito balanceados.
- **Fase 2:** flujo completo restaurante (crear mesa → tomar pedido → ticket llega a KDS por SSE → marcar listo → cerrar cuenta con propina dividida). Flujo booking (cliente público reserva sin login → recibe email → llega a la cita → se cierra como venta). Flujo taller (intake → cotización → aprobación → consumo de repuestos → entrega).
- **Fase 3:** crear segunda `Organization`, verificar aislamiento total de datos. Crear `Subscription`, esperar a que el job de billing facture. Crear `PriceList` para customer business y validar que la venta toma el precio correcto.

Tests existentes a usar como referencia: ver `cargo test` en módulos como `sales` y `inventory` para los patrones de `proptest` y `#[test]` ya en uso.

---

## Plan detallado — Módulo `demand_planning`

Sigue el mismo layout que `modules/accounting` y `modules/analytics` (recientemente mergeados): `domain/{entities,repositories,value_objects} + application/{use_cases,dtos,subscriber} + infrastructure/persistence`. **Decisión explícita:** v1 y v2 **no incluyen adapters de IA** (OpenAI, Vertex, Bedrock, vendores propios). Toda la inteligencia vive en Rust con `statrs`. Si en el futuro hay presupuesto, se agrega un `ForecastEngineAdapter` siguiendo el patrón de `payments::gateway_adapter` — pero **no es parte del scope inicial** y no se debe implementar en este módulo.

### Crate y dependencias

- Nuevo crate: `modules/demand_planning/` (agregar a `Cargo.toml` workspace `members`).
- `Cargo.toml` del crate (espejo del de `accounting`):
  ```toml
  [dependencies]
  common      = { path = "../common" }
  identity    = { path = "../identity" }
  events      = { path = "../events" }
  inventory   = { path = "../inventory" }   # leer Product, Stock
  purchasing  = { path = "../purchasing" }  # crear PO al aprobar suggestion
  statrs      = "0.18"                      # nueva — solo este módulo lo usa
  serde, serde_json, sqlx, tokio, async-trait, uuid, chrono, rust_decimal, thiserror, tracing
  ```
- Agregar `statrs = "0.18"` a `[workspace.dependencies]` para que quede consistente con el resto.

### Domain layer (`modules/demand_planning/src/domain/`)

**Entidades** (`entities/`):
- `demand_forecast.rs` — `DemandForecast { id, product_variant_id, store_id, period: ForecastPeriod, period_start, period_end, method: ForecastMethod, forecasted_qty: Decimal, confidence_low: Decimal, confidence_high: Decimal, computed_at }`. Una fila por (variant × store × período × método).
- `reorder_policy.rs` — `ReorderPolicy { id, product_variant_id, store_id, min_qty, max_qty, lead_time_days, safety_stock_qty, review_cycle_days, preferred_vendor_id: Option<VendorId>, is_active, version }` (optimistic locking igual que `inventory::stock`).
- `replenishment_suggestion.rs` — `ReplenishmentSuggestion { id, product_variant_id, store_id, current_stock, forecast_qty, recommended_qty, suggested_vendor_id, status: SuggestionStatus, generated_at, decided_at, decided_by, generated_purchase_order_id: Option<PurchaseOrderId>, dismiss_reason }`. Workflow `pending → approved → ordered → dismissed`.
- `abc_classification.rs` — `AbcClassification { id, product_variant_id, store_id, period_start, period_end, revenue_share, abc_class: AbcClass, classified_at }`.

**Value Objects** (`value_objects/`):
- `ids.rs` — `ForecastId`, `ReorderPolicyId`, `SuggestionId`, `AbcClassificationId` (newtype `Uuid`, generación con `Uuid::new_v7(Timestamp::now(NoContext))` — ver memoria de convenciones del proyecto).
- `forecast_method.rs` — enum `ForecastMethod { MovingAverage3, MovingAverage6, ExponentialSmoothing, HoltWinters, Manual }`.
- `forecast_period.rs` — enum `ForecastPeriod { Daily, Weekly, Monthly }`.
- `suggestion_status.rs` — enum `SuggestionStatus { Pending, Approved, Ordered, Dismissed }` con métodos `can_transition_to`.
- `abc_class.rs` — enum `AbcClass { A, B, C }`.

**Repositorios** (`repositories/`):
- `demand_forecast_repository.rs` — `save`, `find_latest(variant, store, method)`, `delete_older_than(cutoff)`.
- `reorder_policy_repository.rs` — `save` (con check de version), `find_by_id`, `find_by_variant_store`, `list_active`.
- `replenishment_suggestion_repository.rs` — `save`, `find_by_id`, `list_pending(store_id)`, `mark_ordered(id, po_id)`.
- `abc_classification_repository.rs` — `save_batch`, `find_latest(variant, store)`, `list_by_class`.
- `sales_history_repository.rs` — **read-only** sobre tablas `sales` / `sale_items` existentes; expone `aggregate_units_sold(variant, store, from, to, granularity) -> Vec<SeriesPoint>`. **No** crea tablas; es una proyección sobre lo que ya existe.
- `stock_snapshot_repository.rs` — read-only sobre `inventory_stock` (`current_qty(variant, store)`).

### Application layer (`modules/demand_planning/src/application/`)

**Forecasting puro** (`forecasting/`, módulo aparte de use_cases):
- `moving_average.rs` — fn pura `compute(series: &[Decimal], window: usize) -> Decimal`.
- `exponential_smoothing.rs` — fn pura `compute(series: &[Decimal], alpha: f64) -> Decimal`.
- `holt_winters.rs` — fn pura `compute(series: &[Decimal], alpha, beta, gamma, season_length) -> Result<Decimal, ForecastError>` (requiere `series.len() >= 2 * season_length`, si no, hace fallback a `ExponentialSmoothing`).
- `outliers.rs` — descarta puntos > 3σ del promedio antes de feedear el algoritmo.
- Tests con `proptest` (mismo estilo que `inventory`/`sales`): la suma de los pronósticos diarios de un mes debe estar dentro de ±5 % del pronóstico mensual.

**Use cases** (`use_cases/`):
- `recompute_forecast.rs` — para cada `(variant_id, store_id)` con ventas en los últimos N días, lee la serie histórica vía `SalesHistoryRepository`, ejecuta los algoritmos puros y persiste 1+ filas en `demand_forecasts`. Pensado para correr en el job nocturno.
- `generate_replenishment_suggestions.rs` — combina último forecast + stock actual + `ReorderPolicy` para producir filas en `replenishment_suggestions`. Lógica núcleo:
  ```
  recommended_qty = max(0, max_qty - current_stock)
  safety_stock    = z_score(0.95) * sqrt(lead_time_days / period_days) * std_dev(series)
  if current_stock + reserved_qty < min_qty + safety_stock → emit suggestion
  ```
- `approve_suggestion.rs` — transición `Pending → Approved`; **invoca `purchasing::CreatePurchaseOrderUseCase`** con los items agrupados por vendor preferente; persiste `generated_purchase_order_id` y queda `Ordered`.
- `dismiss_suggestion.rs` — `Pending → Dismissed` con razón obligatoria.
- `classify_abc.rs` — corre mensual: ordena variants por revenue de los últimos 90 días, calcula cumulative share y asigna A (≤80 %), B (≤95 %), C (resto).
- `upsert_reorder_policy.rs` — crea/actualiza `ReorderPolicy` (con version check).
- `list_pending_suggestions.rs`, `get_forecast.rs`, `list_reorder_policies.rs` — read APIs.

**DTOs** (`dtos/{commands,responses}.rs`): comandos para upsert policy, approve/dismiss; responses planos con `serde::Serialize` (mismo patrón que `accounting`).

**Event subscriber** (`subscriber.rs`):
- `DemandPlanningEventSubscriber` — `interested_in`: `["sale.completed", "goods_receipt.confirmed", "stock.adjusted"]`. En v1 solo loggea (igual que el subscriber actual de accounting). En v1.1 marca un flag `requires_recompute` por `(variant, store)` para que el job nocturno priorice.

### Infrastructure (`modules/demand_planning/src/infrastructure/persistence/`)

- `pg_demand_forecast_repository.rs`, `pg_reorder_policy_repository.rs`, `pg_replenishment_suggestion_repository.rs`, `pg_abc_classification_repository.rs` — implementaciones SQLx con queries `sqlx::query!` verificadas en compile-time, igual que `PgJournalEntryRepository`.
- `pg_sales_history_repository.rs` — `SELECT date_trunc(...), SUM(quantity)` agrupado por granularidad sobre `sale_items` JOIN `sales` (filtrar `status = 'completed'`).
- `pg_stock_snapshot_repository.rs` — `SELECT current_qty FROM inventory_stock WHERE variant_id = $1 AND store_id = $2`.

### Migraciones

Continuando la convención `2026050100000X_*.sql` (analytics terminó en `06`):

- `20260501000007_create_demand_forecasts_table.sql`
- `20260501000008_create_reorder_policies_table.sql`
- `20260501000009_create_replenishment_suggestions_table.sql`
- `20260501000010_create_abc_classifications_table.sql`
- `20260501000011_seed_demand_planning_permissions.sql`

Esquema mínimo (resumido):

```sql
CREATE TABLE demand_forecasts (
    id UUID PRIMARY KEY,
    product_variant_id UUID NOT NULL REFERENCES product_variants(id) ON DELETE CASCADE,
    store_id UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    period VARCHAR(16) NOT NULL,        -- daily|weekly|monthly
    period_start DATE NOT NULL,
    period_end   DATE NOT NULL,
    method VARCHAR(32) NOT NULL,
    forecasted_qty   NUMERIC(20,4) NOT NULL,
    confidence_low   NUMERIC(20,4) NOT NULL,
    confidence_high  NUMERIC(20,4) NOT NULL,
    computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (product_variant_id, store_id, period, period_start, method)
);
CREATE INDEX idx_forecasts_lookup ON demand_forecasts (product_variant_id, store_id, computed_at DESC);

CREATE TABLE reorder_policies (
    id UUID PRIMARY KEY,
    product_variant_id UUID NOT NULL REFERENCES product_variants(id) ON DELETE CASCADE,
    store_id UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    min_qty           NUMERIC(20,4) NOT NULL,
    max_qty           NUMERIC(20,4) NOT NULL,
    lead_time_days    INTEGER       NOT NULL,
    safety_stock_qty  NUMERIC(20,4) NOT NULL DEFAULT 0,
    review_cycle_days INTEGER       NOT NULL DEFAULT 7,
    preferred_vendor_id UUID NULL REFERENCES vendors(id) ON DELETE SET NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    version   INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (product_variant_id, store_id),
    CHECK (min_qty >= 0 AND max_qty >= min_qty)
);

CREATE TABLE replenishment_suggestions (
    id UUID PRIMARY KEY,
    product_variant_id UUID NOT NULL REFERENCES product_variants(id) ON DELETE CASCADE,
    store_id UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    current_stock      NUMERIC(20,4) NOT NULL,
    forecast_qty       NUMERIC(20,4) NOT NULL,
    recommended_qty    NUMERIC(20,4) NOT NULL,
    suggested_vendor_id UUID NULL REFERENCES vendors(id) ON DELETE SET NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'pending',
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    decided_at   TIMESTAMPTZ NULL,
    decided_by   UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    generated_purchase_order_id UUID NULL REFERENCES purchase_orders(id) ON DELETE SET NULL,
    dismiss_reason TEXT NULL,
    CHECK (status IN ('pending','approved','ordered','dismissed'))
);
CREATE INDEX idx_suggestions_pending ON replenishment_suggestions (store_id, status) WHERE status = 'pending';

CREATE TABLE abc_classifications (
    id UUID PRIMARY KEY,
    product_variant_id UUID NOT NULL REFERENCES product_variants(id) ON DELETE CASCADE,
    store_id UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    period_start DATE NOT NULL,
    period_end   DATE NOT NULL,
    revenue_share NUMERIC(7,6) NOT NULL,
    abc_class CHAR(1) NOT NULL CHECK (abc_class IN ('A','B','C')),
    classified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (product_variant_id, store_id, period_start, period_end)
);
```

Permisos a sembrar (en la migración `_seed_demand_planning_permissions.sql`): `demand_planning:read_forecast`, `demand_planning:{read,write}_policy`, `demand_planning:{read,approve,dismiss}_suggestion`, `demand_planning:read_abc`. (El check `permissions_code_format` exige formato `module:action` con un solo `:`.)

### API Gateway

- Crear `api-gateway/src/handlers/demand_planning/` con sub-handlers `forecasts.rs`, `policies.rs`, `suggestions.rs`, `abc.rs`.
- Crear `api-gateway/src/jobs/demand_planning_recompute.rs` (mismo patrón que `analytics_recompute.rs`):
  - Tick cada `DEMAND_PLANNING_RECOMPUTE_INTERVAL_SECS` (default 24h).
  - Pasos: `RecomputeForecastUseCase` → `GenerateReplenishmentSuggestionsUseCase` → (1× por mes) `ClassifyAbcUseCase`.
- Registrar registries en `api-gateway/src/state.rs` (similar a `accounting_*` y `analytics_*` ya existentes).
- Spawnear el job en `api-gateway/src/main.rs` igual que `analytics_recompute::spawn(...)`.
- Suscribir `DemandPlanningEventSubscriber` al `EventDispatcher` ya construido.

**Endpoints** (registrar en `api-gateway/src/routes/`):

| Método | Path | Permiso | Descripción |
|---|---|---|---|
| GET | `/api/v1/forecasts/products/:variant_id` | `demand_planning:read_forecast` | Último forecast por método |
| GET | `/api/v1/reorder-policies` | `demand_planning:read_policy` | Listar políticas activas (filtro `?store_id=`) |
| POST | `/api/v1/reorder-policies` | `demand_planning:write_policy` | Crear/actualizar (upsert con version) |
| DELETE | `/api/v1/reorder-policies/:id` | `demand_planning:write_policy` | Desactivar |
| GET | `/api/v1/replenishment-suggestions` | `demand_planning:read_suggestion` | Listar `pending` (default) o por status |
| POST | `/api/v1/replenishment-suggestions/:id/approve` | `demand_planning:approve_suggestion` | Crea PO en `purchasing` |
| POST | `/api/v1/replenishment-suggestions/:id/dismiss` | `demand_planning:dismiss_suggestion` | Body: `{ reason }` |
| GET | `/api/v1/abc-classification` | `demand_planning:read_abc` | Filtros `?store_id=&class=` |

### Roadmap interno del módulo (orden de implementación)

1. **v1.0 — base estadística** (≈2 semanas)
   - Domain + value_objects + entidades + repos traits.
   - Forecasting puro (MA + exponential smoothing) con tests proptest.
   - `RecomputeForecastUseCase`, `GenerateReplenishmentSuggestionsUseCase` (sin safety stock todavía), `UpsertReorderPolicyUseCase`.
   - Migraciones + Pg repos + handlers básicos + job nocturno.
2. **v1.1 — workflow completo** (≈1 semana)
   - `ApproveSuggestionUseCase` integrado con `purchasing::CreatePurchaseOrderUseCase`.
   - `DismissSuggestionUseCase`.
   - Subscriber a eventos para flag `requires_recompute`.
3. **v1.2 — sofisticación estadística** (≈1 semana)
   - Holt-Winters con detección de estacionalidad.
   - Safety stock con desviación estándar y lead time variable.
   - `ClassifyAbcUseCase` mensual.
4. **v2.0 — fuera de scope hasta nuevo aviso**
   - Cualquier integración con servicios IA externos. Si en el futuro entra, se hace como `ForecastEngineAdapter` registry y los algoritmos actuales pasan a ser `LocalStatsEngine`. Hoy esto **no se implementa**.

### Verificación end-to-end

- Seed extendido con 6 meses de ventas variadas (alta/baja temporada) para varios productos y tiendas.
- Correr el job nocturno → verificar que `demand_forecasts` tiene filas para los 4 métodos por cada `(variant, store)` con ventas.
- Crear `ReorderPolicy` para producto X (min=10, max=50, lead_time=7) y bajar el stock a 8.
- Correr `GenerateReplenishmentSuggestionsUseCase` → debe haber 1 fila pending con `recommended_qty ≈ 42`.
- POST `approve` → verificar que se crea un `purchase_order` en estado `draft` con el item correcto y que la suggestion queda `ordered` con `generated_purchase_order_id` poblado.
- Correr `ClassifyAbcUseCase` → verificar que la suma de `revenue_share` por store ≈ 1.0 y la distribución de A/B/C es razonable.

### Patrones existentes a copiar (no reinventar)

- Layout de crate: `modules/accounting/` (tiene además `subscriber.rs`, mismo patrón requerido aquí).
- Job nocturno: `api-gateway/src/jobs/analytics_recompute.rs`.
- Subscriber de eventos: `modules/accounting/src/application/subscriber.rs`.
- Optimistic locking con `version`: `modules/inventory/src/domain/entities/inventory_stock.rs`.
- Workflow state machine: `modules/purchasing/src/domain/entities/purchase_order.rs` (estados PO).
- Generación de UUIDs: `Uuid::new_v7(Timestamp::now(NoContext))` (convención del proyecto, ver memoria).

---

## Plan detallado — Módulo `booking`

Sigue el mismo layout que `modules/loyalty` (último mergeado): `domain/{entities,repositories,value_objects} + application/{use_cases,dtos,subscriber} + infrastructure/persistence`.

### Hallazgo previo verificado

El patrón "endpoint público sin auth" ya está consolidado en el codebase y `booking` lo **reutiliza** (no lo introduce). Referencias canónicas:

- `api-gateway/src/routes/catalog_routes.rs:89-98` — `catalog_public_router()` retorna un `Router::new()` sin `middleware::from_fn_with_state(state, auth_middleware)` aplicado. Se monta en `api-gateway/src/main.rs:163` bajo `/api/v1/catalog/public`.
- `api-gateway/src/routes/shipping_routes.rs:123-126` — `public_tracking_router()` con la misma estructura (sin layer de auth). Montado en `main.rs:140` bajo `/api/v1/track`.
- Handler público típico: no recibe `CurrentUser`, solo `State` + `Path`/`Query`. Ver `api-gateway/src/handlers/shipping/public.rs`.

### Decisión arquitectónica clave: la cita NO pasa por `cart`

Evaluado y descartado. Razones:

1. **Mismatch de abstracción.** `cart_items` está atado a `product_variant_id` + `inventory_reservations` (ver `modules/sales::cart`). Una "cita" es slot de tiempo + recurso, no SKU + stock. Forzarla al modelo de carrito requiere productos sintéticos para cada slot/servicio, contaminando el catálogo.
2. **Patrón de la industria.** Calendly, Acuity, Square Appointments, Resy: ninguna usa carrito. La `Appointment` ES el agregado raíz transaccional.
3. **Workflow distinto.** Carrito asume "ensamblar varios items y luego checkout". Booking es atómico: elegir servicio + horario → confirmar. El estado intermedio "carrito" no agrega valor.
4. **Modelo de pago.** El depósito (cuando aplica) se carga al crear la cita, no en un paso de checkout separado. La `Appointment` referencia directamente `payments::transaction_id`.

**Implicación:** `Appointment` se relaciona con `Customer` (opcional, para walk-ins/anónimos guardamos snapshot de email/nombre/teléfono), opcionalmente con `payments::Transaction` (depósito), y al `complete()` opcionalmente genera un `sales::Sale` para la facturación final. Cero acoplamiento con `cart`.

### Crate y dependencias

- Nuevo crate: `modules/booking/` (agregar a `Cargo.toml` workspace `members`).
- `Cargo.toml` del crate (espejo de `modules/loyalty/Cargo.toml`):
  ```toml
  [dependencies]
  common = { path = "../common" }
  identity = { path = "../identity" }
  events = { path = "../events" }
  serde, serde_json, sqlx, tokio, async-trait, uuid, chrono, rust_decimal, thiserror, tracing
  ```
- **No** importar `sales`, `payments`, `notifications` directamente. Cross-module se hace por `Uuid` + verificación SQL (mismo enfoque que `loyalty::EnrollMemberUseCase` para customer existence — ver `modules/loyalty/src/application/use_cases/enroll_member.rs:42-45`).

### Domain layer (`modules/booking/src/domain/`)

**Entidades** (`entities/`):

- `resource.rs` — `Resource { id, store_id, resource_type, name, color, is_active, created_at, updated_at }`. Una persona (estilista, mecánico), equipo (silla, bahía, lavamanos) o sala. Métodos: `new()`, `reconstitute()`, `deactivate()`, `rename()`.
- `resource_calendar.rs` — `ResourceCalendar { id, resource_id, day_of_week (0=Sun..6=Sat), start_time, end_time, is_active }`. Disponibilidad recurrente semanal. Una fila por (recurso × día). v1.0 no maneja excepciones (vacaciones, días feriados) — eso es v1.2.
- `service_catalog.rs` — `Service { id, store_id, name, description, duration_minutes, price, buffer_minutes_before, buffer_minutes_after, requires_deposit, deposit_amount, is_active, created_at, updated_at }`. v1.0 una `Service` puede ser realizada por cualquier `Resource` listado en `service_resources` (M2M).
- `appointment.rs` — `Appointment { id, store_id, service_id, resource_id, customer_id (Optional), customer_name, customer_email, customer_phone, starts_at, ends_at, status, deposit_transaction_id (Optional), generated_sale_id (Optional), notes, canceled_reason, no_show_at (Optional), created_at, updated_at, created_by (Optional — None para bookings públicos) }`. Workflow:
  ```
  Scheduled ──confirm──▶ Confirmed ──start──▶ InProgress ──complete──▶ Completed
      │                       │                    │
      └──cancel──▶ Canceled ◀─┴──cancel──┘         └──no_show──▶ NoShow
  ```
  - `Scheduled` y `Confirmed` permiten `cancel`. `Confirmed` permite `start`. `InProgress` permite `complete` o `no_show`. Estados terminales: `Completed`, `Canceled`, `NoShow`.
- `booking_policy.rs` — `BookingPolicy { id, store_id, requires_deposit (bool), deposit_percentage (Optional), cancellation_window_hours, no_show_fee_amount (Optional), default_buffer_minutes, advance_booking_days_max, created_at, updated_at }`. Una sola política por store (UNIQUE store_id). v1.0 captura los campos pero solo `cancellation_window_hours` es chequeada activamente en `cancel`. Depósitos/no-show fee se reservan para v1.1.

**Value Objects** (`value_objects/`):

- `ids.rs` — macro `id_type!` (mismo patrón que `modules/loyalty/src/domain/value_objects/ids.rs`): `ResourceId`, `ResourceCalendarId`, `ServiceId`, `AppointmentId`, `BookingPolicyId`.
- `resource_type.rs` — enum `ResourceType { Person, Equipment, Room }` con `as_str()` y `from_str()`.
- `appointment_status.rs` — enum `AppointmentStatus { Scheduled, Confirmed, InProgress, Completed, Canceled, NoShow }` con `can_transition_to(other) -> bool`.
- `time_slot.rs` — value object plano `TimeSlot { starts_at: DateTime<Utc>, ends_at: DateTime<Utc> }` con `overlaps(other)`, `duration_minutes()`. Usado por la lógica de availability.

**Repositorios** (`repositories/`):

- `resource_repository.rs` — `save`, `find_by_id`, `list_by_store(store_id, only_active)`, `update`, `deactivate`.
- `resource_calendar_repository.rs` — `save_batch` (la API recibe el set semanal completo), `find_by_resource(resource_id)`.
- `service_repository.rs` — `save`, `find_by_id`, `list_by_store(store_id, only_active)`, `update`, `deactivate`. Plus `assign_resources(service_id, resource_ids)` y `find_eligible_resources(service_id) -> Vec<ResourceId>` para la M2M.
- `appointment_repository.rs` — `save`, `find_by_id`, `list_by_resource_in_range(resource_id, from, to)`, `list_by_store(store_id, filters: ListAppointmentsFilters)`, `update_status` (raw UPDATE para no overwritear otras columnas), `update_deposit_tx`, `update_generated_sale`.
- `booking_policy_repository.rs` — `save`, `find_by_store(store_id)`.

### Application layer (`modules/booking/src/application/`)

**Availability puro** (`availability/`, módulo aparte de use_cases — análogo a `forecasting/` en `demand_planning`):

- `slot_generator.rs` — fn pura `generate_slots(calendar_intervals: Vec<TimeRange>, duration_minutes: u32, buffer_before: u32, buffer_after: u32, granularity_minutes: u32) -> Vec<TimeSlot>`. Genera slots candidatos del día siguiendo la cuadrícula (típico granularity=15min).
- `slot_subtractor.rs` — fn pura `subtract_booked(candidates: Vec<TimeSlot>, booked: Vec<TimeSlot>) -> Vec<TimeSlot>`. Elimina solapamientos.
- Tests: `proptest` para garantizar que (a) generar y luego restar siempre produce slots con duration ≥ duration_minutes, (b) ningún slot retornado solapa con bookings existentes.

**Use cases** (`use_cases/`):

Resources & calendars:
- `create_resource.rs` / `update_resource.rs` / `deactivate_resource.rs` / `list_resources.rs`.
- `set_resource_calendar.rs` — recibe el set completo (ej: `[{day: Mon, 09:00-17:00}, {day: Tue, 09:00-17:00}, ...]`), borra y reescribe en transacción.

Services:
- `create_service.rs` / `update_service.rs` / `deactivate_service.rs` / `list_services.rs`.
- `assign_service_resources.rs` — M2M con borrar-y-reescribir.

Appointments (auth):
- `list_appointments.rs` — filtros por `store_id`, `resource_id`, `customer_id`, `status`, `date_range`.
- `create_appointment_admin.rs` — staff crea cita (walk-in o teléfono), customer puede ser existente o snapshot.
- `confirm_appointment.rs` — `Scheduled → Confirmed`.
- `start_appointment.rs` — `Confirmed → InProgress`. Valida que `now ∈ [starts_at - tolerance, ends_at]`.
- `complete_appointment.rs` — `InProgress → Completed`. v1.0 solo persiste estado. v1.1 invocará `sales::CreateSaleUseCase` para facturar.
- `cancel_appointment.rs` — `{Scheduled, Confirmed} → Canceled`. Recibe `reason` y `actor_id`. Valida `cancellation_window_hours` contra `BookingPolicy`.
- `no_show_appointment.rs` — `Confirmed | InProgress → NoShow` (depende de criterio de negocio; v1.0 acepta desde `Confirmed`).

Public (sin auth):
- `list_public_services.rs` — lista servicios activos de un store. Read-only.
- `check_availability.rs` — entrada `{store_id, service_id, date}`, salida `Vec<{resource_id, resource_name, slots: Vec<TimeSlot>}>`. Combina `service_repo`, `resource_repo`, `resource_calendar_repo`, `appointment_repo` y los algoritmos puros.
- `book_appointment_public.rs` — entrada `{store_id, service_id, resource_id (opcional — si None, se elige el primer disponible), starts_at, customer_name, customer_email, customer_phone, customer_id (opcional — si el cliente público tiene cuenta), notes}`. Crea `Appointment` en estado `Scheduled`. Re-valida disponibilidad bajo lock pesimista (`SELECT ... FOR UPDATE` sobre appointments del recurso) para evitar double-booking en concurrencia.
- `get_public_appointment.rs` — lookup por ID + token (un secret guardado en `appointments.public_token` generado al crear). Devuelve estado y detalles para que el cliente vea su cita.

Booking policy:
- `upsert_booking_policy.rs` / `get_booking_policy.rs`.

**DTOs** (`dtos/{commands,responses}.rs`): comandos para crear/actualizar; responses planos. Mismo patrón que `modules/loyalty/src/application/dtos/responses.rs`.

**Event subscriber** (`subscriber.rs`):

- `BookingEventSubscriber` — `interested_in`: `[]` en v1.0 (passive). v1.1 escuchará `appointment.created`, `appointment.completed`, `appointment.canceled` para disparar notificaciones (recordatorio 24h/1h, confirmación, cancelación). El subscriber se registra en `state.rs` igual que los demás.

**Eventos publicados** (in-tx en use cases, vía `outbox`):
- `booking.appointment.created` — payload `{appointment_id, store_id, customer_email, starts_at}`.
- `booking.appointment.completed` — payload `{appointment_id, customer_id, total_amount?}`.
- `booking.appointment.canceled` — payload `{appointment_id, reason, canceled_by_customer (bool)}`.
- v1.0 publica los eventos pero ningún subscriber los consume todavía. v1.1 los conecta a `notifications`.

### Infrastructure (`modules/booking/src/infrastructure/persistence/`)

- `pg_resource_repository.rs`, `pg_resource_calendar_repository.rs`, `pg_service_repository.rs`, `pg_appointment_repository.rs`, `pg_booking_policy_repository.rs` — implementaciones SQLx con queries `sqlx::query` + `query_as::<_, Row>` y row-to-entity en `From<Row> for Entity` (ver `modules/loyalty/src/infrastructure/persistence/pg_loyalty_member_repository.rs:124-149` como plantilla).
- En `book_appointment_public` (use case que mete la cita): usar `pool.begin()` + `SELECT id FROM appointments WHERE resource_id = $1 AND status IN ('scheduled','confirmed','in_progress') AND tstzrange(starts_at, ends_at) && tstzrange($2, $3) FOR UPDATE` para detectar overlap bajo lock antes de insertar.

### Migraciones

Continuando convención `2026050100002X_*.sql` (loyalty terminó en `22`):

- `20260501000023_create_booking_resources_table.sql`
- `20260501000024_create_booking_resource_calendars_table.sql`
- `20260501000025_create_booking_services_table.sql`
- `20260501000026_create_booking_service_resources_table.sql`
- `20260501000027_create_booking_appointments_table.sql`
- `20260501000028_create_booking_policies_table.sql`
- `20260501000029_seed_booking_permissions.sql`

Esquema mínimo (resumido):

```sql
CREATE TABLE booking_resources (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    resource_type VARCHAR(16) NOT NULL CHECK (resource_type IN ('person','equipment','room')),
    name VARCHAR(120) NOT NULL,
    color VARCHAR(7) NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_booking_resources_store ON booking_resources (store_id) WHERE is_active = TRUE;

CREATE TABLE booking_resource_calendars (
    id UUID PRIMARY KEY,
    resource_id UUID NOT NULL REFERENCES booking_resources(id) ON DELETE CASCADE,
    day_of_week SMALLINT NOT NULL CHECK (day_of_week BETWEEN 0 AND 6),
    start_time TIME NOT NULL,
    end_time TIME NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    CHECK (end_time > start_time)
);
CREATE INDEX idx_booking_resource_calendars_lookup ON booking_resource_calendars (resource_id, day_of_week);

CREATE TABLE booking_services (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    name VARCHAR(120) NOT NULL,
    description TEXT NULL,
    duration_minutes INTEGER NOT NULL CHECK (duration_minutes > 0),
    price NUMERIC(20,4) NOT NULL DEFAULT 0,
    buffer_minutes_before INTEGER NOT NULL DEFAULT 0,
    buffer_minutes_after INTEGER NOT NULL DEFAULT 0,
    requires_deposit BOOLEAN NOT NULL DEFAULT FALSE,
    deposit_amount NUMERIC(20,4) NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_booking_services_store ON booking_services (store_id) WHERE is_active = TRUE;

CREATE TABLE booking_service_resources (
    service_id UUID NOT NULL REFERENCES booking_services(id) ON DELETE CASCADE,
    resource_id UUID NOT NULL REFERENCES booking_resources(id) ON DELETE CASCADE,
    PRIMARY KEY (service_id, resource_id)
);

CREATE TABLE booking_appointments (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    service_id UUID NOT NULL REFERENCES booking_services(id) ON DELETE RESTRICT,
    resource_id UUID NOT NULL REFERENCES booking_resources(id) ON DELETE RESTRICT,
    customer_id UUID NULL REFERENCES customers(id) ON DELETE SET NULL,
    customer_name VARCHAR(120) NOT NULL,
    customer_email VARCHAR(160) NOT NULL,
    customer_phone VARCHAR(40) NULL,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'scheduled',
    deposit_transaction_id UUID NULL,
    generated_sale_id UUID NULL,
    notes TEXT NULL,
    canceled_reason TEXT NULL,
    no_show_at TIMESTAMPTZ NULL,
    public_token VARCHAR(64) NOT NULL,
    created_by UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (status IN ('scheduled','confirmed','in_progress','completed','canceled','no_show')),
    CHECK (ends_at > starts_at)
);
CREATE INDEX idx_booking_appointments_resource_window
    ON booking_appointments (resource_id, starts_at, ends_at)
    WHERE status IN ('scheduled','confirmed','in_progress');
CREATE INDEX idx_booking_appointments_store ON booking_appointments (store_id, starts_at DESC);
CREATE INDEX idx_booking_appointments_customer ON booking_appointments (customer_id) WHERE customer_id IS NOT NULL;
CREATE UNIQUE INDEX idx_booking_appointments_token ON booking_appointments (public_token);

CREATE TABLE booking_policies (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL UNIQUE REFERENCES stores(id) ON DELETE CASCADE,
    requires_deposit BOOLEAN NOT NULL DEFAULT FALSE,
    deposit_percentage NUMERIC(5,2) NULL,
    cancellation_window_hours INTEGER NOT NULL DEFAULT 24,
    no_show_fee_amount NUMERIC(20,4) NULL,
    default_buffer_minutes INTEGER NOT NULL DEFAULT 0,
    advance_booking_days_max INTEGER NOT NULL DEFAULT 60,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

Permisos a sembrar (formato `module:action` con un solo `:`): `booking:read_resource`, `booking:write_resource`, `booking:read_service`, `booking:write_service`, `booking:read_appointment`, `booking:write_appointment`, `booking:transition_appointment`, `booking:cancel_appointment`, `booking:read_policy`, `booking:write_policy`. Asignar a `super_admin` y `store_admin`.

### API Gateway

**Handlers** en `api-gateway/src/handlers/booking/`:
- `mod.rs` — re-exports.
- `resources.rs` — CRUD resources + calendar.
- `services.rs` — CRUD services + assign resources.
- `appointments.rs` — CRUD + transitions (auth).
- `policies.rs` — upsert/get policy.
- `public.rs` — endpoints públicos sin auth (no usa `CurrentUser`, solo `State` + `Path`/`Query`/`Json`). Patrón idéntico a `api-gateway/src/handlers/shipping/public.rs`.

**Routers** en `api-gateway/src/routes/booking_routes.rs`:
- `booking_resources_router(state)` — auth.
- `booking_services_router(state)` — auth.
- `booking_appointments_router(state)` — auth (incluye transitions).
- `booking_policies_router(state)` — auth.
- `public_booking_router()` — **sin** layer `auth_middleware`, mismo patrón que `public_tracking_router()`.

**Endpoints**:

| Método | Path | Permiso | Descripción |
|---|---|---|---|
| GET | `/api/v1/booking/resources?store_id=` | `booking:read_resource` | Listar recursos del store |
| POST | `/api/v1/booking/resources` | `booking:write_resource` | Crear recurso |
| PUT | `/api/v1/booking/resources/{id}` | `booking:write_resource` | Actualizar |
| DELETE | `/api/v1/booking/resources/{id}` | `booking:write_resource` | Desactivar |
| PUT | `/api/v1/booking/resources/{id}/calendar` | `booking:write_resource` | Set calendario semanal |
| GET | `/api/v1/booking/services?store_id=` | `booking:read_service` | Listar servicios |
| POST | `/api/v1/booking/services` | `booking:write_service` | Crear servicio |
| PUT | `/api/v1/booking/services/{id}` | `booking:write_service` | Actualizar |
| DELETE | `/api/v1/booking/services/{id}` | `booking:write_service` | Desactivar |
| PUT | `/api/v1/booking/services/{id}/resources` | `booking:write_service` | Asignar M2M |
| GET | `/api/v1/booking/appointments` | `booking:read_appointment` | Listar (filtros) |
| POST | `/api/v1/booking/appointments` | `booking:write_appointment` | Crear (admin/walk-in) |
| GET | `/api/v1/booking/appointments/{id}` | `booking:read_appointment` | Detalle |
| POST | `/api/v1/booking/appointments/{id}/confirm` | `booking:transition_appointment` | Confirmar |
| POST | `/api/v1/booking/appointments/{id}/start` | `booking:transition_appointment` | Iniciar |
| POST | `/api/v1/booking/appointments/{id}/complete` | `booking:transition_appointment` | Completar |
| POST | `/api/v1/booking/appointments/{id}/cancel` | `booking:cancel_appointment` | Cancelar |
| POST | `/api/v1/booking/appointments/{id}/no-show` | `booking:transition_appointment` | Marcar no-show |
| GET | `/api/v1/booking/policies?store_id=` | `booking:read_policy` | Leer política |
| PUT | `/api/v1/booking/policies` | `booking:write_policy` | Upsert política |
| **GET** | `/api/v1/public/booking/{store_id}/services` | — (público) | Listar servicios activos |
| **GET** | `/api/v1/public/booking/{store_id}/availability?service_id=&date=` | — (público) | Slots disponibles |
| **POST** | `/api/v1/public/booking/{store_id}/book` | — (público) | Crear cita (Scheduled) |
| **GET** | `/api/v1/public/booking/appointments/{id}?token=` | — (público) | Ver cita propia |

**Mount** en `api-gateway/src/main.rs` (después del bloque de loyalty):
```rust
.nest("/api/v1/booking/resources", booking_resources_router(app_state.clone()))
.nest("/api/v1/booking/services", booking_services_router(app_state.clone()))
.nest("/api/v1/booking/appointments", booking_appointments_router(app_state.clone()))
.nest("/api/v1/booking/policies", booking_policies_router(app_state.clone()))
.nest("/api/v1/public/booking", public_booking_router())
```

**`AppState`** (`api-gateway/src/state.rs`) — agregar bloque:
```rust
// -------------------------------------------------------------------------
// Booking (resources, calendars, services, appointments, policies)
// -------------------------------------------------------------------------
resource_repo: Arc<dyn ResourceRepository>,
resource_calendar_repo: Arc<dyn ResourceCalendarRepository>,
service_repo: Arc<dyn ServiceRepository>,
appointment_repo: Arc<dyn AppointmentRepository>,
booking_policy_repo: Arc<dyn BookingPolicyRepository>,
```
Más accessors y `subscriber_registry.register(Arc::new(BookingEventSubscriber::new()));` en `from_pool`.

### Mapeo de errores en el API Gateway

Agregar `impl From<BookingError> for AppError` en `api-gateway/src/error.rs` siguiendo el patrón de loyalty (`error.rs:2329-2398`):
- 404: `ResourceNotFound`, `ServiceNotFound`, `AppointmentNotFound`, `PolicyNotFound`.
- 409: `SlotConflict`, `InvalidStateTransition { from, to }`, `OutsideCancellationWindow`.
- 400: `InvalidTimeRange`, `ResourceNotEligibleForService`, `ValidationError`.
- 500: `Database`, `Serialization`.

### Roadmap interno del módulo (orden de implementación)

1. **v1.0 — base operativa** (≈2 semanas)
   - Domain + value_objects + entidades + repos traits.
   - Algoritmos de availability puros (slot_generator + slot_subtractor) con tests proptest.
   - Use cases CRUD para resources/services/policies.
   - Use cases de transiciones de appointment (sin payment, sin sale).
   - Use cases públicos: `list_public_services`, `check_availability`, `book_appointment_public` (con lock pesimista anti-double-booking).
   - Migraciones + Pg repos + handlers + routers (auth + público).
   - Subscriber pasivo registrado.
2. **v1.1 — integraciones**  (≈1 semana)
   - `complete_appointment` invoca `sales::CreateSaleUseCase` para facturar (cross-crate).
   - `book_appointment_public` cuando `service.requires_deposit` invoca `payments::CreateTransactionUseCase` antes de confirmar la cita; falla → rollback.
   - `BookingEventSubscriber` activo: `appointment.created` → email confirmación; `appointment.completed` → email gracias; `appointment.canceled` → email.
   - Job `appointment_reminders.rs` en `api-gateway/src/jobs/`: cada 5 min escanea citas en `Confirmed` con `starts_at - 24h ∈ [now, now+5min]` o `starts_at - 1h ∈ [now, now+5min]` y publica `appointment.reminder` para que `notifications` mande recordatorio.
3. **v1.2 — refinamientos** (≈1 semana)
   - Excepciones de calendario (vacaciones, días feriados) — tabla `booking_resource_calendar_exceptions`.
   - Soporte de slugs por store (campo `stores.slug`) para reemplazar `store_id` en URLs públicas.
   - No-show fee automático (cuando policy.no_show_fee_amount IS NOT NULL, `no_show_appointment` cobra vía payments).
   - Notificaciones SMS además de email (vía `notifications` adapter).

### Verificación end-to-end (v1.0)

- Seed: crear store, 2 resources (estilista A y B), 2 services (Corte 30min, Color 90min), asignar ambos resources a Corte y solo A a Color, calendario L-V 09:00-17:00.
- `GET /api/v1/public/booking/{store_id}/services` → 200 con 2 servicios.
- `GET /api/v1/public/booking/{store_id}/availability?service_id={corte}&date=2026-05-04` → lista de slots para A y B.
- `POST /api/v1/public/booking/{store_id}/book` con `{service_id: corte, starts_at: 2026-05-04T10:00Z, customer_email, customer_name}` → 201 Appointment Scheduled, `public_token` en respuesta.
- Repetir el POST con el mismo `resource_id + starts_at` → 409 `SLOT_CONFLICT`.
- Auth `POST /appointments/{id}/confirm` → 200 status=Confirmed.
- Auth `POST /appointments/{id}/start` antes de hora → 409.
- En la hora correcta, `start` → 200; `complete` → 200.
- Otra cita: `cancel` con `reason` dentro de window → 200; fuera de window → 409 `OUTSIDE_CANCELLATION_WINDOW`.
- Verificar que `outbox_events` tiene filas `booking.appointment.created` y `booking.appointment.completed` (consumir vendrá en v1.1).

### Patrones existentes a copiar (no reinventar)

- Layout de crate: `modules/loyalty/` (último mergeado, patrón canónico actual).
- Macro de IDs: `modules/loyalty/src/domain/value_objects/ids.rs`.
- Repo Pg con const SQL + `FromRow` + `From<Row>`: `modules/loyalty/src/infrastructure/persistence/pg_loyalty_member_repository.rs`.
- Use case con verificación cross-module via SQL: `modules/loyalty/src/application/use_cases/enroll_member.rs:42-45`.
- Subscriber pasivo (v1) → activo (v1.1): `modules/loyalty/src/application/subscriber.rs`.
- Handler público sin `CurrentUser`: `api-gateway/src/handlers/shipping/public.rs`.
- Router público sin auth layer: `api-gateway/src/routes/shipping_routes.rs:124-126`.
- Lock pesimista para evitar carreras: `modules/sales::cart` reservation creation (ver patrón de `inventory::reservation`).
- Generación UUIDs: `Uuid::new_v7(Timestamp::now(NoContext))` (ver memoria de convenciones).

---

## Plan detallado — Módulo `service_orders`

Sigue el mismo layout que `modules/booking` (recién mergeado): `domain/{entities,repositories,value_objects} + application/{use_cases,dtos,subscriber} + infrastructure/persistence`. Reutiliza el patrón de `public_token` introducido por booking — el cliente del taller consulta el estado de su reparación sin login usando un token unguessable generado al crear la orden.

### Decisiones arquitectónicas v1.0

1. **El `Asset` es entidad propia, no tabla de historial.** El historial de un vehículo o equipo se obtiene por `SELECT * FROM service_orders WHERE asset_id = X`. No hay tabla aparte de "visitas" — la orden de servicio ES la visita. Esto evita duplicación y mantiene una sola fuente de verdad.

2. **`WarrantyClaim` se difiere a v1.2.** Una garantía se modela como una nueva `ServiceOrder` con `warranty_for_order_id: Option<Uuid>` (columna a agregar en v1.1). El workflow de aprobación de garantía con SLA dedicado es v1.2.

3. **Inventario y venta se DIFIEREN a v1.1.** v1.0 graba `ServiceOrderItem` con `product_id` opcional pero **no** descuenta stock al `start_repair`, ni invoca `sales::CreateSaleUseCase` al `deliver`. v1.1 cablea ambos. Razón: misma estrategia que con booking — primero estabilizamos el workflow propio, después conectamos con los módulos vecinos.

4. **Sin asignación de bahía/técnico en v1.0.** Los talleres reales necesitan asignar una bahía y un técnico, pero esto es ortogonal al workflow de la orden. v1.1 agrega columnas `bay_id` (nullable, free uuid sin FK aún) y `assigned_technician_id` (FK a `users`).

5. **Quote versionado.** Cada nueva cotización para la misma orden incrementa `version` y marca las anteriores como `Superseded`. Permite auditoría completa de la negociación con el cliente sin perder datos.

### Workflow state machine

```
Intake ──diagnose──▶ Diagnosis ──submit_quote──▶ QuoteSent
                          ▲                         │
                          └────reject_quote─────────┤  (vuelve a Diagnosis para nueva cotización)
                                                    │
                                              approve_quote
                                                    ▼
QuoteApproved ──start_repair──▶ InRepair ──start_testing──▶ Testing ──mark_ready──▶ ReadyForPickup ──deliver──▶ Delivered

cancel: permitido desde {Intake, Diagnosis, QuoteSent, QuoteApproved, InRepair, Testing, ReadyForPickup} → Canceled
```

Estados terminales: `Delivered`, `Canceled`. Una vez `Delivered`, ninguna transición es válida (incluyendo cancel).

### Crate y dependencias

- Nuevo crate: `modules/service_orders/` (agregar a `Cargo.toml` workspace `members`).
- `Cargo.toml` (espejo del de `booking`): `common`, `identity`, `events` + workspace deps. **No** importar `inventory`, `sales`, `purchasing` directamente — cross-module por `Uuid` con verificación SQL en use cases (mismo enfoque que loyalty/booking).

### Domain layer (`modules/service_orders/src/domain/`)

**Entidades** (`entities/`):

- `asset.rs` — `Asset { id, store_id, customer_id (Optional), asset_type, brand, model, identifier, year, color, description, attributes (JSONB), is_active, created_at, updated_at }`. `identifier` es libre (placa, número de serie, IMEI). Métodos: `register`, `reconstitute`, `update_details`, `link_customer`, `deactivate`.

- `service_order.rs` — Aggregate root. `ServiceOrder { id, store_id, asset_id, customer_id, customer_name, customer_email, customer_phone, status, priority, intake_notes, intake_at, intake_by_user_id (Optional), promised_at (Optional), delivered_at (Optional), generated_sale_id (Optional), canceled_reason (Optional), canceled_at (Optional), public_token, total_amount (cached), created_at, updated_at }`.
  Métodos: `intake()`, `reconstitute()`, `transition_to(...)` (privado, valida via `can_transition_to`), `diagnose`, `submit_quote_marker` (no lleva quote_id; el quote se persiste por separado), `mark_quote_approved`, `mark_quote_rejected`, `start_repair`, `start_testing`, `mark_ready`, `deliver(generated_sale_id: Option<Uuid>)`, `cancel(reason)`, `recompute_total(subtotal: Decimal)`.

- `service_order_item.rs` — `ServiceOrderItem { id, service_order_id, item_type, description, quantity, unit_price, total, product_id (Optional), variant_id (Optional), tax_rate (Decimal, default 0), tax_amount (Decimal), created_at }`. `total = quantity * unit_price + tax_amount`. Métodos: `new_labor`, `new_part`, `reconstitute`.

- `diagnostic.rs` — `Diagnostic { id, service_order_id, technician_user_id (Optional), findings, recommended_actions, severity, created_at }`. Una orden puede tener varios diagnósticos (re-evaluaciones).

- `quote.rs` — `Quote { id, service_order_id, version, labor_total, parts_total, tax_total, grand_total, valid_until (Optional), notes (Optional), status, sent_at (Optional), decided_at (Optional), decided_by_customer (bool), created_at }`. Workflow interno: `Draft → Sent → {Approved, Rejected, Superseded}`.

**Value Objects** (`value_objects/`):

- `ids.rs` — macro `id_type!`: `AssetId`, `ServiceOrderId`, `ServiceOrderItemId`, `DiagnosticId`, `QuoteId`.
- `asset_type.rs` — enum `AssetType { Vehicle, Equipment, Appliance, Electronic, Other }`.
- `service_order_status.rs` — enum con 9 estados, métodos `as_str`, `FromStr`, `is_terminal`, `can_transition_to(other)`.
- `service_order_priority.rs` — enum `Priority { Low, Normal, High, Urgent }`.
- `service_order_item_type.rs` — enum `ItemType { Labor, Part }`.
- `quote_status.rs` — enum con 5 estados, `FromStr`, `can_transition_to`.
- `diagnostic_severity.rs` — enum `Severity { Low, Medium, High, Critical }`.

**Repositorios** (`repositories/`):

- `asset_repository.rs` — `save`, `update`, `find_by_id`, `list_by_store(store_id, only_active, asset_type_filter)`, `list_by_customer(customer_id)`.
- `service_order_repository.rs` — `save`, `update`, `find_by_id`, `find_by_public_token`, `list(filters)` (filters: store, status, customer, asset, date_range), `list_by_asset(asset_id)` (para historial).
- `service_order_item_repository.rs` — `save`, `update`, `delete`, `find_by_id`, `list_by_order(order_id)`, `subtotal_by_order(order_id) -> Decimal`.
- `diagnostic_repository.rs` — `save`, `list_by_order(order_id)`.
- `quote_repository.rs` — `save`, `update`, `find_by_id`, `list_by_order(order_id)`, `mark_others_superseded(order_id, except_id)` (transaccional, marca todos los `Draft|Sent` de la orden como `Superseded` excepto el indicado).

### Application layer

**Use cases** (`use_cases/`):

Asset CRUD:
- `register_asset.rs`, `update_asset.rs`, `deactivate_asset.rs`, `list_assets.rs`, `get_asset_with_history.rs` (combina asset + lista de service_orders).

Service order intake/listing:
- `intake_service_order.rs` — recibe asset_id (debe existir + activo), customer info (snapshot), intake_notes, priority, promised_at. Crea orden en `Intake`.
- `list_service_orders.rs`, `get_service_order.rs`.

Items:
- `add_item.rs`, `update_item.rs`, `remove_item.rs`. Tras cada cambio, recalcula el total con `subtotal_by_order` y persiste en `ServiceOrder.total_amount`.

Diagnostics:
- `add_diagnostic.rs`, `list_diagnostics.rs`.

Quotes:
- `create_quote.rs` — calcula totales desde los items actuales, marca otros quotes Draft/Sent como Superseded. Estado inicial `Draft`.
- `send_quote.rs` — `Draft → Sent`. Idempotente.
- `approve_quote.rs` — `Sent → Approved`. También transiciona la `ServiceOrder` a `QuoteApproved`. Recibe `decided_by_customer: bool`.
- `reject_quote.rs` — `Sent → Rejected`. Devuelve `ServiceOrder` a `Diagnosis` para que se cree un nuevo quote.
- `list_quotes.rs`.

Service order transitions:
- `diagnose.rs` — `Intake → Diagnosis`. (Auto-llamado por `add_diagnostic` si está en `Intake`? Decisión: explícito, mediante endpoint propio).
- `start_repair.rs` — `QuoteApproved → InRepair`. v1.0: solo cambia estado. v1.1: descuenta stock de items con `product_id`.
- `start_testing.rs` — `InRepair → Testing`.
- `mark_ready.rs` — `Testing → ReadyForPickup`.
- `deliver.rs` — `ReadyForPickup → Delivered`. v1.0: solo cambia estado, persiste `delivered_at`. v1.1: invoca `sales::CreateSaleUseCase`, persiste `generated_sale_id`.
- `cancel_service_order.rs` — cualquier no-terminal → `Canceled`. Recibe `reason` obligatorio.

Public:
- `get_public_service_order.rs` — entrada `(id, token)`. Devuelve estado + items + última cotización + último diagnóstico. Usado por el endpoint `/api/v1/public/service-orders/{id}?token=`. Sin auth.

**DTOs** (`dtos/{commands,responses}.rs`): commands para todo lo anterior; responses planos. Una respuesta de detalle (`ServiceOrderDetailResponse`) incluye `items`, `diagnostics`, `quotes` para evitar N+1 desde el cliente.

**Event subscriber** (`subscriber.rs`):
- `ServiceOrdersEventSubscriber` — `interested_in: []` en v1.0 (passive). v1.1 escuchará eventos relacionados.

**Eventos publicados** (vía outbox, in-tx en use cases):
- `service_orders.order.created` — payload `{order_id, asset_id, customer_email, intake_at}`.
- `service_orders.quote.sent` — `{order_id, quote_id, version, grand_total}`.
- `service_orders.order.delivered` — `{order_id, customer_id, total_amount}`.
- `service_orders.order.canceled` — `{order_id, reason}`.
- v1.0 publica los eventos pero ningún subscriber los consume.

### Infrastructure (`modules/service_orders/src/infrastructure/persistence/`)

5 implementaciones SQLx siguiendo el patrón de `modules/booking/src/infrastructure/persistence/`:
- `pg_asset_repository.rs`, `pg_service_order_repository.rs`, `pg_service_order_item_repository.rs`, `pg_diagnostic_repository.rs`, `pg_quote_repository.rs`.
- Const SQL + `sqlx::FromRow` Row structs + `From<Row>`/`TryFrom<Row>` para reconstitución.
- `pg_service_order_repository::list` usa `sqlx::QueryBuilder` para filtros opcionales (mismo patrón que `pg_appointment_repository::list`).

### Migraciones

Continuando convención `2026050100003X_*.sql` (booking terminó en `29`):

- `20260501000030_create_service_assets_table.sql`
- `20260501000031_create_service_orders_table.sql`
- `20260501000032_create_service_order_items_table.sql`
- `20260501000033_create_service_diagnostics_table.sql`
- `20260501000034_create_service_quotes_table.sql`
- `20260501000035_seed_service_orders_permissions.sql`

Esquema mínimo (resumido):

```sql
CREATE TABLE service_assets (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    customer_id UUID NULL REFERENCES customers(id) ON DELETE SET NULL,
    asset_type VARCHAR(16) NOT NULL CHECK (asset_type IN ('vehicle','equipment','appliance','electronic','other')),
    brand VARCHAR(80) NULL,
    model VARCHAR(120) NULL,
    identifier VARCHAR(120) NULL,         -- placa, serial, IMEI
    year INTEGER NULL,
    color VARCHAR(40) NULL,
    description TEXT NULL,
    attributes JSONB NOT NULL DEFAULT '{}'::JSONB,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_service_assets_store_active ON service_assets (store_id) WHERE is_active = TRUE;
CREATE INDEX idx_service_assets_customer ON service_assets (customer_id) WHERE customer_id IS NOT NULL;
CREATE INDEX idx_service_assets_identifier ON service_assets (store_id, identifier) WHERE identifier IS NOT NULL;

CREATE TABLE service_orders (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    asset_id UUID NOT NULL REFERENCES service_assets(id) ON DELETE RESTRICT,
    customer_id UUID NULL REFERENCES customers(id) ON DELETE SET NULL,
    customer_name VARCHAR(120) NOT NULL,
    customer_email VARCHAR(160) NOT NULL,
    customer_phone VARCHAR(40) NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'intake' CHECK (status IN (
        'intake','diagnosis','quote_sent','quote_approved','in_repair',
        'testing','ready_for_pickup','delivered','canceled'
    )),
    priority VARCHAR(10) NOT NULL DEFAULT 'normal' CHECK (priority IN ('low','normal','high','urgent')),
    intake_notes TEXT NULL,
    intake_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    intake_by_user_id UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    promised_at TIMESTAMPTZ NULL,
    delivered_at TIMESTAMPTZ NULL,
    generated_sale_id UUID NULL,
    canceled_reason TEXT NULL,
    canceled_at TIMESTAMPTZ NULL,
    public_token VARCHAR(64) NOT NULL,
    total_amount NUMERIC(20,4) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_service_orders_store_status ON service_orders (store_id, status);
CREATE INDEX idx_service_orders_asset ON service_orders (asset_id, intake_at DESC);
CREATE INDEX idx_service_orders_customer ON service_orders (customer_id) WHERE customer_id IS NOT NULL;
CREATE UNIQUE INDEX idx_service_orders_token ON service_orders (public_token);

CREATE TABLE service_order_items (
    id UUID PRIMARY KEY,
    service_order_id UUID NOT NULL REFERENCES service_orders(id) ON DELETE CASCADE,
    item_type VARCHAR(8) NOT NULL CHECK (item_type IN ('labor','part')),
    description TEXT NOT NULL,
    quantity NUMERIC(20,4) NOT NULL CHECK (quantity > 0),
    unit_price NUMERIC(20,4) NOT NULL CHECK (unit_price >= 0),
    total NUMERIC(20,4) NOT NULL,
    product_id UUID NULL,
    variant_id UUID NULL,
    tax_rate NUMERIC(5,4) NOT NULL DEFAULT 0,
    tax_amount NUMERIC(20,4) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_service_order_items_order ON service_order_items (service_order_id);

CREATE TABLE service_diagnostics (
    id UUID PRIMARY KEY,
    service_order_id UUID NOT NULL REFERENCES service_orders(id) ON DELETE CASCADE,
    technician_user_id UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    findings TEXT NOT NULL,
    recommended_actions TEXT NULL,
    severity VARCHAR(10) NOT NULL CHECK (severity IN ('low','medium','high','critical')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_service_diagnostics_order ON service_diagnostics (service_order_id, created_at DESC);

CREATE TABLE service_quotes (
    id UUID PRIMARY KEY,
    service_order_id UUID NOT NULL REFERENCES service_orders(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    labor_total NUMERIC(20,4) NOT NULL DEFAULT 0,
    parts_total NUMERIC(20,4) NOT NULL DEFAULT 0,
    tax_total NUMERIC(20,4) NOT NULL DEFAULT 0,
    grand_total NUMERIC(20,4) NOT NULL DEFAULT 0,
    valid_until TIMESTAMPTZ NULL,
    notes TEXT NULL,
    status VARCHAR(12) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','sent','approved','rejected','superseded')),
    sent_at TIMESTAMPTZ NULL,
    decided_at TIMESTAMPTZ NULL,
    decided_by_customer BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (service_order_id, version)
);
CREATE INDEX idx_service_quotes_order_status ON service_quotes (service_order_id, status);
```

Permisos a sembrar (formato `module:action`): `service_orders:read_asset`, `service_orders:write_asset`, `service_orders:read_order`, `service_orders:write_order`, `service_orders:transition_order`, `service_orders:cancel_order`, `service_orders:write_diagnostic`, `service_orders:write_quote`, `service_orders:transition_quote`, `service_orders:write_item`. **Importante**: agregar también a `seed/src/data.rs` (PERMISSIONS + ROLE_PERMISSIONS de super_admin y store_admin) — si no, los grants quedan en la migración pero la fuente de verdad real del seed binary los pisa.

### API Gateway

**Handlers** en `api-gateway/src/handlers/service_orders/`:
- `mod.rs`, `assets.rs`, `orders.rs`, `items.rs`, `diagnostics.rs`, `quotes.rs`, `transitions.rs`, `public.rs`.

**Routers** en `api-gateway/src/routes/service_orders_routes.rs`:
- `assets_router(state)` — auth.
- `service_orders_router(state)` — auth (CRUD + transitions + sub-resources).
- `public_service_orders_router()` — sin auth.

**Endpoints**:

| Método | Path | Permiso | Descripción |
|---|---|---|---|
| GET | `/api/v1/assets?store_id=` | `service_orders:read_asset` | Listar assets |
| GET | `/api/v1/assets/{id}` | `service_orders:read_asset` | Detalle |
| GET | `/api/v1/assets/{id}/history` | `service_orders:read_asset` | Historial de service_orders |
| POST | `/api/v1/assets` | `service_orders:write_asset` | Registrar |
| PUT | `/api/v1/assets/{id}` | `service_orders:write_asset` | Actualizar |
| DELETE | `/api/v1/assets/{id}` | `service_orders:write_asset` | Desactivar |
| GET | `/api/v1/service-orders` | `service_orders:read_order` | Listar (filtros) |
| POST | `/api/v1/service-orders` | `service_orders:write_order` | Intake |
| GET | `/api/v1/service-orders/{id}` | `service_orders:read_order` | Detalle (con items/diagnostics/quotes) |
| POST | `/api/v1/service-orders/{id}/items` | `service_orders:write_item` | Agregar item |
| PUT | `/api/v1/service-orders/{id}/items/{item_id}` | `service_orders:write_item` | Actualizar item |
| DELETE | `/api/v1/service-orders/{id}/items/{item_id}` | `service_orders:write_item` | Borrar item |
| POST | `/api/v1/service-orders/{id}/diagnostics` | `service_orders:write_diagnostic` | Registrar diagnóstico |
| POST | `/api/v1/service-orders/{id}/quotes` | `service_orders:write_quote` | Crear quote (calcula desde items) |
| POST | `/api/v1/service-orders/{id}/quotes/{quote_id}/send` | `service_orders:transition_quote` | Marcar como enviada |
| POST | `/api/v1/service-orders/{id}/quotes/{quote_id}/approve` | `service_orders:transition_quote` | Aprobar (avanza orden) |
| POST | `/api/v1/service-orders/{id}/quotes/{quote_id}/reject` | `service_orders:transition_quote` | Rechazar (vuelve a Diagnosis) |
| POST | `/api/v1/service-orders/{id}/diagnose` | `service_orders:transition_order` | Intake → Diagnosis |
| POST | `/api/v1/service-orders/{id}/start-repair` | `service_orders:transition_order` | QuoteApproved → InRepair |
| POST | `/api/v1/service-orders/{id}/start-testing` | `service_orders:transition_order` | InRepair → Testing |
| POST | `/api/v1/service-orders/{id}/mark-ready` | `service_orders:transition_order` | Testing → ReadyForPickup |
| POST | `/api/v1/service-orders/{id}/deliver` | `service_orders:transition_order` | ReadyForPickup → Delivered |
| POST | `/api/v1/service-orders/{id}/cancel` | `service_orders:cancel_order` | → Canceled (con reason) |
| **GET** | `/api/v1/public/service-orders/{id}?token=` | — (público) | Cliente ve estado de su orden |

**`AppState`** — agregar repos:
```rust
service_asset_repo, service_order_repo, service_order_item_repo,
service_diagnostic_repo, service_quote_repo
```
Más accessors y `subscriber_registry.register(Arc::new(ServiceOrdersEventSubscriber::new()))` en `from_pool`.

### Mapeo de errores en el API Gateway

Agregar `impl From<ServiceOrdersError> for AppError` en `api-gateway/src/error.rs` (mismo patrón que booking):
- 404: `AssetNotFound`, `ServiceOrderNotFound`, `QuoteNotFound`, `ItemNotFound`, `DiagnosticNotFound`.
- 401: `InvalidPublicToken`.
- 409: `InvalidStateTransition { from, to }`, `QuoteAlreadyDecided`, `CannotModifyDeliveredOrder`.
- 400: `InvalidAssetType`, `InvalidServiceOrderStatus`, `InvalidQuoteStatus`, `Validation`.
- 500: `Database`, `Serialization`.

### Roadmap interno del módulo

1. **v1.0 — workflow puro** (≈2 semanas)
   - Domain + value_objects + repos traits + state machine.
   - Use cases CRUD asset/order/items/diagnostics/quotes.
   - Transiciones de orden y de quote sin tocar inventory/sales.
   - Endpoint público de status.
   - Migraciones + Pg repos + handlers + routers + seed (con permisos).
2. **v1.1 — integraciones cross-módulo** (≈1 semana)
   - `start_repair` descuenta stock de items con `product_id` no nulo (vía `inventory::AdjustStockUseCase` o reservación + confirmación).
   - `deliver` invoca `sales::CreateSaleUseCase` y persiste `generated_sale_id`. Falla → la transición se aborta.
   - Subscriber activo: `service_orders.order.created` → email de bienvenida; `service_orders.quote.sent` → email con cotización; `service_orders.order.delivered` → email de gracias.
   - Columnas `bay_id` y `assigned_technician_id` en `service_orders` + endpoints de asignación.
3. **v1.2 — refinamientos** (≈1 semana)
   - `WarrantyClaim` como entidad propia con su workflow (Pending/Approved/Rejected) + `warranty_for_order_id` en service_orders.
   - PO desde service_order: para repuestos no en stock, generar PO ligada vía `purchasing::CreatePurchaseOrderUseCase`.
   - Sugerencias de "próxima visita" basadas en kilometraje/fecha.
   - SMS además de email vía `notifications`.

### Verificación end-to-end (v1.0)

- Seed: 1 customer, 2 assets (vehículo + laptop), 1 service order en `Intake`.
- `POST /api/v1/service-orders/{id}/diagnose` → `Diagnosis`.
- `POST /api/v1/service-orders/{id}/diagnostics` con findings/severity → 201.
- `POST /api/v1/service-orders/{id}/items` (varios items: labor + parts).
- `POST /api/v1/service-orders/{id}/quotes` → quote con `version=1`, status=`Draft`, totales calculados.
- `POST /api/v1/service-orders/{id}/quotes/{q}/send` → `Sent`.
- `POST /api/v1/service-orders/{id}/quotes/{q}/approve` → quote `Approved` + orden a `QuoteApproved`.
- `POST /api/v1/service-orders/{id}/start-repair` → `InRepair`. Probar también que `start-repair` desde `Intake` falle con 409.
- `start-testing` → `mark-ready` → `deliver` → `Delivered`.
- Confirmar que `cancel` falla con 409 sobre orden `Delivered`.
- `GET /api/v1/public/service-orders/{id}?token=...` retorna estado actual + last quote + last diagnostic.
- `GET /api/v1/assets/{id}/history` muestra la orden recién entregada.

### Patrones existentes a copiar (no reinventar)

- Layout completo: `modules/booking/`.
- Macro de IDs y `FromStr` en VOs: `modules/booking/src/domain/value_objects/`.
- `public_token` generado al crear el agregado: `modules/booking/src/domain/entities/appointment.rs:71-74`.
- Repo Pg con `QueryBuilder` para listas con filtros: `modules/booking/src/infrastructure/persistence/pg_appointment_repository.rs::list`.
- Handler público sin `CurrentUser`: `api-gateway/src/handlers/booking/public.rs`.
- Router público sin auth layer: `api-gateway/src/routes/booking_routes.rs::public_booking_router`.
- Seed permisos en data.rs: `seed/src/data.rs` PERMISSIONS + ROLE_PERMISSIONS para super_admin/store_admin (la migración SQL es backup; el seed binary es la fuente real).
- Generación UUIDs: `Uuid::new_v7(Timestamp::now(NoContext))`.

---

## Plan detallado — Módulo `restaurant_operations`

Sigue el mismo layout que `modules/service_orders` (último mergeado): `domain/{entities,repositories,value_objects} + application/{use_cases,dtos,subscriber} + infrastructure/persistence`. **Diferencia clave**: introduce el primer caso real de estado broadcast en `api-gateway` (canales `tokio::sync::broadcast` por estación de cocina) para alimentar streams SSE al KDS.

### Decisiones arquitectónicas v1.0

1. **Scope reducido para shippear el núcleo.** v1.0 cubre lo mínimo para operar un restaurante: estaciones, mesas, modificadores y tickets KDS con SSE. **Difiere a v1.1+**: `FloorPlan` con coordenadas 2D, suscriptor `sales.item_added` (auto-generación), `Tip`/`TipDistribution`, `SplitBill`, `TableReservation` (puede reusar `booking`), splitting de un sale en múltiples tickets por course con timing.

2. **SSE en lugar de WebSocket.** SSE es unidireccional (server → client) y suficiente: el KDS solo necesita recibir cambios de estado de los tickets en tiempo real; las acciones del cocinero (start/ready/serve) ya van por POST autenticado. WebSocket queda como opción futura si la latencia bidireccional importa (ej. cocinero "reclama" un ticket).

3. **Broadcast en memoria.** v1.0 usa `tokio::sync::broadcast::Sender<KdsEvent>` por `station_id`, almacenados en un `Arc<Mutex<HashMap<StationId, Sender>>>` dentro de `AppState`. **Limitación consciente**: si el gateway corre en N réplicas, los suscritos a la réplica A no ven eventos publicados desde B. Para v1.0 con una sola réplica está bien. v1.2 puede agregar fan-out vía Postgres `LISTEN/NOTIFY` o Redis pub/sub para escalar horizontalmente sin cambiar la API pública.

4. **Trait `KdsBroadcaster`.** Para no acoplar el dominio a Tokio, el use case llama a `broadcaster.publish(station_id, event)` sobre un trait inyectado. Implementación default: `TokioBroadcastKdsBroadcaster`. Test/CI puede inyectar un `NoopBroadcaster`.

5. **Auto-advance del ticket basado en sus items.** Cuando todos los items de un ticket están en `Ready`, el ticket transiciona automáticamente a `Ready`. Cuando todos están en `Served`, va a `Served`. La lógica vive en el use case que cambia el estado de un item; el subscriber/dispatcher publica el evento resultante a la estación.

6. **Routing de tickets a estaciones es manual en v1.0.** El staff elige `station_id` al crear el ticket. v1.1 puede agregar reglas (ej. "items con `category_id IN (X, Y)` van a `station_id = Z` por default") basadas en mapping configurable.

7. **Modificadores en items se serializan como `modifiers_summary` (texto).** Para v1.0 evita complicar el modelo de datos del item (tabla intermedia con FK a modificadores). El cliente envía la lista de modifier_ids al crear el item, el use case resuelve sus nombres y construye `"sin cebolla, extra queso"`. v1.2 puede agregar `kds_ticket_item_modifiers` para reportes de modificadores.

### Workflow state machines

**KdsTicket**:
```
Pending ──send──▶ InProgress ──ready──▶ Ready ──serve──▶ Served
   │                  │                    │
   └──cancel──▶ Canceled (desde Pending o InProgress; no desde Ready/Served)
```
Auto-transiciones (gatilladas por items):
- Si todos los items entran a `InProgress` mientras el ticket está en `Pending` → ticket a `InProgress`.
- Si todos los items entran a `Ready` mientras el ticket está en `InProgress` → ticket a `Ready`.
- Si todos los items entran a `Served` mientras el ticket está en `Ready` → ticket a `Served`.

**KdsTicketItem**: mismo set (`Pending → InProgress → Ready → Served`, + `Canceled`).

**Table**:
```
Free ──seat──▶ Seated ──clear──▶ Dirty ──reset──▶ Free
       │                  │
       └──reserve──▶ Reserved ──cancel_reservation──▶ Free
```
v1.0 no enforce las transiciones rigurosamente — admin puede setear cualquier status. La maquinaria entra en v1.1 cuando se conecte con `sales::Sale`.

### Crate y dependencias

- Nuevo crate: `modules/restaurant_operations/` (agregar a `Cargo.toml` workspace `members`).
- `Cargo.toml` espejo del de `service_orders`: `common`, `identity`, `events` + workspace deps. **No** importar `inventory`, `sales`, `payments` directamente.

### Domain layer (`modules/restaurant_operations/src/domain/`)

**Entidades** (`entities/`):

- `kitchen_station.rs` — `KitchenStation { id, store_id, name, color, sort_order, is_active, created_at, updated_at }`. Estaciones tipo "Hot Line", "Cold Line", "Bar".

- `restaurant_table.rs` — `RestaurantTable { id, store_id, label, capacity, status, current_ticket_id (Optional), notes (Optional), is_active, created_at, updated_at }`. `label` es libre ("Mesa 1", "Barra 3", "Terraza A").

- `menu_modifier_group.rs` — `MenuModifierGroup { id, store_id, name, min_select, max_select, sort_order, is_active, created_at, updated_at }`. Ej: "Cocción" (min=1, max=1), "Extras" (min=0, max=5).

- `menu_modifier.rs` — `MenuModifier { id, group_id, name, price_delta, sort_order, is_active, created_at, updated_at }`. Ej: "Término medio" (delta=0), "Extra queso" (delta=+25).

- `kds_ticket.rs` — Aggregate root. `KdsTicket { id, store_id, station_id, table_id (Optional — nulleable para togo/delivery), sale_id (Optional — v1.0 None, v1.1 link a sales::Sale), ticket_number, status, course, notes, sent_at (Optional), ready_at (Optional), served_at (Optional), canceled_reason (Optional), created_by (Optional), created_at, updated_at }`. Métodos: `new()`, `reconstitute()`, `transition_to(...)`, `mark_in_progress()`, `mark_ready()`, `mark_served()`, `cancel(reason)`.

- `kds_ticket_item.rs` — `KdsTicketItem { id, ticket_id, sale_item_id (Optional), product_id (Optional), description, quantity, modifiers_summary (String, default ""), special_instructions (Option<String>), status, ready_at (Optional), served_at (Optional), created_at }`.

**Value Objects** (`value_objects/`):

- `ids.rs` — macro `id_type!`: `KitchenStationId`, `RestaurantTableId`, `MenuModifierGroupId`, `MenuModifierId`, `KdsTicketId`, `KdsTicketItemId`.
- `kds_ticket_status.rs` — enum 5 estados, `as_str`, `FromStr`, `is_terminal`, `can_transition_to`.
- `kds_item_status.rs` — enum 5 estados, mismo set.
- `course.rs` — enum `Course { Appetizer, Main, Dessert, Drink, Other }`.
- `table_status.rs` — enum `TableStatus { Free, Seated, Reserved, Dirty }`.

**Repositorios** (`repositories/`):

- `kitchen_station_repository.rs` — `save`, `update`, `find_by_id`, `list_by_store(store_id, only_active)`.
- `restaurant_table_repository.rs` — `save`, `update`, `find_by_id`, `list_by_store(store_id, only_active)`.
- `menu_modifier_repository.rs` — combinado: `save_group`, `update_group`, `find_group`, `list_groups`, `save_modifier`, `update_modifier`, `find_modifier`, `list_modifiers_by_group(group_id)`, plus M2M con products: `assign_groups_to_product(product_id, group_ids)`, `find_groups_for_product(product_id)`.
- `kds_ticket_repository.rs` — `save`, `update`, `find_by_id`, `list(filters)` (filters: store, station, status, table, date_range), `next_ticket_number(store_id)` (auto-increment SQL: `MAX(ticket_number) + 1` por store con LOCK).
- `kds_ticket_item_repository.rs` — `save`, `update`, `find_by_id`, `list_by_ticket(ticket_id)`.

### Application layer

**Broadcaster trait** (`broadcaster.rs`):

```rust
#[async_trait]
pub trait KdsBroadcaster: Send + Sync {
    async fn publish(&self, station_id: KitchenStationId, event: KdsEvent);
}

#[derive(Clone, Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KdsEvent {
    TicketCreated { ticket_id: Uuid, ticket_number: i32, table_label: Option<String>, items_count: usize, course: Course },
    TicketStatusChanged { ticket_id: Uuid, status: KdsTicketStatus },
    ItemStatusChanged { ticket_id: Uuid, item_id: Uuid, status: KdsItemStatus },
    TicketCanceled { ticket_id: Uuid, reason: String },
}
```

Publicado por todos los use cases que mutan tickets/items. Implementación Tokio en infrastructure.

**Use cases** (`use_cases/`):

Stations:
- `create_station.rs`, `update_station.rs`, `deactivate_station.rs`, `list_stations.rs`.

Tables:
- `create_table.rs`, `update_table.rs`, `set_table_status.rs`, `deactivate_table.rs`, `list_tables.rs`.

Modifiers:
- `create_modifier_group.rs`, `update_modifier_group.rs`, `add_modifier.rs`, `update_modifier.rs`, `list_groups_with_modifiers.rs`, `assign_groups_to_product.rs`, `get_modifiers_for_product.rs`.

KDS tickets:
- `create_kds_ticket.rs` — recibe `station_id`, `table_id` (Opt), `course`, `items: [{description, quantity, modifier_ids, special_instructions}]`. Resuelve `modifier_ids → modifiers_summary`. Auto-asigna `ticket_number` (next por store). Crea ticket en `Pending` + items en `Pending`. **Publica `TicketCreated` al broadcaster**.
- `send_ticket.rs` — `Pending → InProgress`. Setea `sent_at`. Publica `TicketStatusChanged`.
- `set_item_status.rs` — actualiza item, **auto-evalúa el ticket**: si todos los items quedan en mismo status no-terminal, el ticket transiciona acorde. Publica `ItemStatusChanged` y posiblemente `TicketStatusChanged`.
- `mark_ticket_ready.rs` — fuerza `InProgress → Ready` (mark all items Ready si no lo están ya).
- `serve_ticket.rs` — `Ready → Served`. Publica.
- `cancel_ticket.rs` — `{Pending, InProgress} → Canceled` con razón.
- `list_tickets_by_station.rs` — filtros: status, station, course.

**DTOs** (`dtos/{commands,responses}.rs`): commands para todo lo anterior; responses planos. `KdsTicketDetailResponse` incluye `items` para evitar N+1.

**Event subscriber** (`subscriber.rs`):

- `RestaurantOperationsEventSubscriber` — `interested_in: []` en v1.0 (passive). v1.1 escuchará `sale.item_added` para auto-crear tickets KDS.

**Eventos publicados** (vía outbox, in-tx en use cases): `restaurant_operations.kds_ticket.created`, `.canceled`, `.served`. v1.0 publica pero sin consumidor.

### Infrastructure (`modules/restaurant_operations/src/infrastructure/`)

- `persistence/` — 5 PgRepositories (group + modifier comparten una sola implementación combinada `PgMenuModifierRepository`).
- `broadcaster/tokio_broadcast_kds_broadcaster.rs` — `TokioBroadcastKdsBroadcaster { inner: Arc<RwLock<HashMap<KitchenStationId, broadcast::Sender<KdsEvent>>>> }`. `publish` busca/crea el sender de la estación y hace `send(event)` (silencioso si no hay receivers — comportamiento esperado de broadcast). Expone `subscribe(station_id) -> broadcast::Receiver<KdsEvent>` para que el handler SSE se suscriba.

### Migraciones

Continuando convención (service_orders terminó en `35`):

- `20260501000036_create_kitchen_stations_table.sql`
- `20260501000037_create_restaurant_tables_table.sql`
- `20260501000038_create_menu_modifier_groups_table.sql`
- `20260501000039_create_menu_modifiers_table.sql`
- `20260501000040_create_product_modifier_groups_table.sql` (M2M)
- `20260501000041_create_kds_tickets_table.sql`
- `20260501000042_create_kds_ticket_items_table.sql`
- `20260501000043_seed_restaurant_operations_permissions.sql`

Esquema clave (resumido):

```sql
CREATE TABLE kds_tickets (
    id UUID PRIMARY KEY,
    store_id UUID NOT NULL REFERENCES stores(id) ON DELETE CASCADE,
    station_id UUID NOT NULL REFERENCES kitchen_stations(id) ON DELETE RESTRICT,
    table_id UUID NULL REFERENCES restaurant_tables(id) ON DELETE SET NULL,
    sale_id UUID NULL,                                      -- v1.1 link
    ticket_number INTEGER NOT NULL,                         -- auto-increment per store
    status VARCHAR(16) NOT NULL DEFAULT 'pending',
    course VARCHAR(16) NOT NULL DEFAULT 'main',
    notes TEXT NULL,
    sent_at TIMESTAMPTZ NULL,
    ready_at TIMESTAMPTZ NULL,
    served_at TIMESTAMPTZ NULL,
    canceled_reason TEXT NULL,
    created_by UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (status IN ('pending','in_progress','ready','served','canceled')),
    CHECK (course IN ('appetizer','main','dessert','drink','other')),
    UNIQUE (store_id, ticket_number)
);
CREATE INDEX idx_kds_tickets_station_active ON kds_tickets (station_id, status)
    WHERE status IN ('pending','in_progress','ready');
CREATE INDEX idx_kds_tickets_table ON kds_tickets (table_id) WHERE table_id IS NOT NULL;
```

Permisos (formato `module:action`): `restaurant:read_station`, `restaurant:write_station`, `restaurant:read_table`, `restaurant:write_table`, `restaurant:read_modifier`, `restaurant:write_modifier`, `restaurant:read_ticket`, `restaurant:write_ticket`, `restaurant:transition_ticket`, `restaurant:cancel_ticket`. **Importante**: agregar también a `seed/src/data.rs` (PERMISSIONS + ROLE_PERMISSIONS de super_admin y store_admin).

### API Gateway

**Dependencias nuevas en `api-gateway/Cargo.toml`**: `restaurant_operations` (path), `tokio-stream` para `tokio_stream::wrappers::BroadcastStream` (ya disponible vía workspace si se agrega).

**Handlers** en `api-gateway/src/handlers/restaurant/`:
- `mod.rs`, `stations.rs`, `tables.rs`, `modifiers.rs`, `tickets.rs`, `kds_stream.rs`.

**`kds_stream_handler`** (el corazón del SSE):
```rust
async fn kds_stream_handler(
    State(state): State<AppState>,
    CurrentUser(ctx): CurrentUser,
    Path(station_id): Path<Uuid>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Response> {
    require_permission(&ctx, "restaurant:read_ticket")?;
    let receiver = state.kds_broadcaster().subscribe(KitchenStationId::from_uuid(station_id));
    let stream = BroadcastStream::new(receiver)
        .filter_map(|res| async move {
            match res {
                Ok(event) => Some(Ok(Event::default().json_data(event).unwrap())),
                Err(_lagged) => None, // skip lagged events; client reconnects to catch up
            }
        });
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
```

**Routers** en `api-gateway/src/routes/restaurant_routes.rs`:
- `restaurant_stations_router(state)` — auth.
- `restaurant_tables_router(state)` — auth.
- `restaurant_modifiers_router(state)` — auth.
- `kds_tickets_router(state)` — auth (incluye transitions e items).
- `kds_stream_router(state)` — auth (la conexión SSE inicial requiere JWT en el header `Authorization`).

**Endpoints**:

| Método | Path | Permiso | Descripción |
|---|---|---|---|
| GET | `/api/v1/restaurant/stations?store_id=` | `restaurant:read_station` | Listar estaciones |
| POST/PUT/DELETE | `/api/v1/restaurant/stations[/{id}]` | `restaurant:write_station` | CRUD |
| GET | `/api/v1/restaurant/tables?store_id=` | `restaurant:read_table` | Listar mesas |
| POST/PUT/DELETE | `/api/v1/restaurant/tables[/{id}]` | `restaurant:write_table` | CRUD |
| POST | `/api/v1/restaurant/tables/{id}/status` | `restaurant:write_table` | Cambiar status (free/seated/dirty/reserved) |
| GET | `/api/v1/restaurant/modifier-groups?store_id=` | `restaurant:read_modifier` | Listar grupos + sus modificadores |
| POST/PUT | `/api/v1/restaurant/modifier-groups[/{id}]` | `restaurant:write_modifier` | CRUD grupo |
| POST/PUT | `/api/v1/restaurant/modifier-groups/{id}/modifiers[/{mid}]` | `restaurant:write_modifier` | CRUD modificador |
| PUT | `/api/v1/restaurant/products/{product_id}/modifier-groups` | `restaurant:write_modifier` | M2M assignment |
| GET | `/api/v1/restaurant/products/{product_id}/modifier-groups` | `restaurant:read_modifier` | Modificadores de un producto |
| GET | `/api/v1/restaurant/kds/tickets?station_id=&status=` | `restaurant:read_ticket` | Listar tickets activos |
| POST | `/api/v1/restaurant/kds/tickets` | `restaurant:write_ticket` | Crear ticket (con items inline) |
| GET | `/api/v1/restaurant/kds/tickets/{id}` | `restaurant:read_ticket` | Detalle (con items) |
| POST | `/api/v1/restaurant/kds/tickets/{id}/send` | `restaurant:transition_ticket` | Pending → InProgress |
| POST | `/api/v1/restaurant/kds/tickets/{id}/ready` | `restaurant:transition_ticket` | InProgress → Ready (todos los items) |
| POST | `/api/v1/restaurant/kds/tickets/{id}/serve` | `restaurant:transition_ticket` | Ready → Served |
| POST | `/api/v1/restaurant/kds/tickets/{id}/cancel` | `restaurant:cancel_ticket` | → Canceled |
| POST | `/api/v1/restaurant/kds/tickets/{id}/items/{item_id}/status` | `restaurant:transition_ticket` | Cambiar status de un item |
| **GET** | `/api/v1/restaurant/kds/stations/{station_id}/stream` | `restaurant:read_ticket` | **SSE** stream de eventos |

**`AppState`** — agregar:
```rust
kitchen_station_repo, restaurant_table_repo, menu_modifier_repo,
kds_ticket_repo, kds_ticket_item_repo,
kds_broadcaster: Arc<dyn KdsBroadcaster>
```

### Mapeo de errores

`impl From<RestaurantOperationsError> for AppError`: 404 (`StationNotFound`, `TableNotFound`, `TicketNotFound`, `ItemNotFound`, `ModifierGroupNotFound`, `ModifierNotFound`, `ProductNotFound`); 409 (`InvalidStateTransition`, `CannotModifyTerminalTicket`); 400 (`InvalidStatus`/`InvalidCourse`/`InvalidTableStatus`/`Validation`); 500 (`Database`, `Serialization`).

### Roadmap interno del módulo

1. **v1.0 — núcleo F&B + SSE** (≈2.5 semanas)
   - Domain + value_objects + repos traits + state machines.
   - Use cases CRUD para stations/tables/modifiers.
   - Use cases del lifecycle del ticket KDS con auto-advance basado en items.
   - `KdsBroadcaster` trait + `TokioBroadcastKdsBroadcaster` impl.
   - Migraciones + Pg repos + handlers + routers + seed (con permisos).
   - SSE handler + smoke test que conecta a `/stream` y recibe un evento al crear un ticket.

2. **v1.1 — integraciones cross-módulo** (≈1.5 semanas)
   - `RestaurantOperationsEventSubscriber` activo: `sale.item_added` → auto-crea/extiende ticket KDS según mapping de routing.
   - Auto-decremento de stock para items que tengan `recipe_id` (vía `inventory::recipe`).
   - Reglas de routing configurables: `kds_routing_rules` (category_id → station_id).
   - `Tip` y `TipDistribution` (necesita pagos cerrados).
   - `SplitBill` (necesita `sales::Sale`).

3. **v1.2 — refinamientos** (≈1.5 semanas)
   - `FloorPlan` con coordenadas 2D (campo `layout_json` JSONB) y endpoint para drag & drop.
   - `TableReservation` reusando `booking::Appointment` con `resource_type = 'table'`.
   - Splitting de un sale en múltiples tickets por course con timing (`fire_at` por course).
   - Fan-out de SSE vía Postgres `LISTEN/NOTIFY` para soportar múltiples réplicas del gateway.

### Verificación end-to-end (v1.0)

- Seed: 2 stations (Hot Line + Bar), 4 tables, 2 modifier groups (Cocción, Extras) con 5 modificadores, 1 ticket KDS en `Pending` con 2 items en estación Hot Line.
- En una terminal: `curl -N "http://localhost:8000/api/v1/restaurant/kds/stations/{station_id}/stream"` (con header Authorization) — la conexión queda abierta esperando eventos.
- En otra terminal: `POST /kds/tickets/{id}/send` — verificar que el primer terminal recibe un event `ticket_status_changed { status: in_progress }`.
- `POST /kds/tickets/{id}/items/{item_id}/status {status: "ready"}` (uno) → `item_status_changed`. Después el segundo → `item_status_changed` + auto-`ticket_status_changed { status: ready }`.
- `POST /kds/tickets/{id}/serve` → `ticket_status_changed { status: served }` + el ticket sale de la lista activa.
- `POST /kds/tickets/{id}/cancel` (sobre otro nuevo) — cancel desde Pending OK; cancel desde Served → 409.

### Patrones existentes a copiar (no reinventar)

- Layout de crate: `modules/service_orders/`.
- VOs con `FromStr` + state machine: `modules/service_orders/src/domain/value_objects/`.
- Repo Pg con `QueryBuilder` para listas filtradas: `modules/booking/src/infrastructure/persistence/pg_appointment_repository.rs::list`.
- Handler con use case + permission check: `api-gateway/src/handlers/service_orders/transitions.rs`.
- Mapeo de errores: `api-gateway/src/error.rs::From<ServiceOrdersError>`.
- Seed permisos en data.rs: `seed/src/data.rs` PERMISSIONS + ROLE_PERMISSIONS para super_admin/store_admin.
- Auto-increment ticket_number con MAX+1 bajo lock pesimista: análogo a `fiscal_sequences::current_number` (ver migración fiscal).
- Generación UUIDs: `Uuid::new_v7(Timestamp::now(NoContext))`.

---

## Plan detallado — Módulo `tenancy`

Sigue el mismo layout que `modules/restaurant_operations` (último mergeado): `domain/{entities,repositories,value_objects} + application/{use_cases,dtos,subscriber} + infrastructure/persistence`. **Diferencia clave**: es el primer módulo que toca tablas raíz pre-existentes (`users`, `stores`) y por tanto requiere una estrategia explícita de **migración no destructiva**.

### Decisiones arquitectónicas v1.0

1. **`organization_id` NULLABLE en v1.0, NOT NULL en v1.2.** La columna se agrega a `users` y `stores` con FK opcional a `organizations(id)`. Una migración de datos (`20260501000051_create_default_organization.sql`) crea una organización default determinística (id `00000000-0000-0000-0000-000000000001`, slug `default`, plan `enterprise` con todas las features activas) y backfilliza todos los users/stores existentes. v1.2 hace `ALTER TABLE ... SET NOT NULL` después de validar que el backfill funcionó en producción.

2. **Solo `users` y `stores` reciben columna directa.** El resto (products, customers, sales, kds_tickets, appointments, service_orders, etc.) **hereda** la organización vía su FK a `store_id`. Razón: agregar la columna a 30+ tablas multiplica el riesgo de migración por 30; el modelo deja una sola fuente de verdad (`stores.organization_id`); cualquier query cross-store ya filtra por `store_id IN (SELECT id FROM stores WHERE organization_id = ?)`. Costo: las queries directamente cross-org (analytics consolidado, super-admin) requieren un JOIN extra. Beneficio: migración 30× más simple.

3. **Sin enforcement por middleware en v1.0.** Los endpoints CRUD del módulo existen, los datos se persisten, pero **ningún endpoint chequea que `JWT.organization_id == resource.organization_id`**. v1.0 solo introduce el modelo. v1.1 agrega un `OrganizationScope` extractor en `api-gateway/src/extractors/` que se inyecta en todos los handlers existentes y filtra automáticamente por `current_user.organization_id`. Esto permite shippear v1.0 sin riesgo y empezar a usar tenancy desde el dashboard de admin sin romper el comportamiento single-tenant actual.

4. **Feature flags solo se persisten en v1.0.** El JSONB en `organization_plans.feature_flags` (`{"booking": true, "restaurant": false, ...}`) se lee/escribe vía endpoints, pero no hay enforcement. v1.1 agrega un `RequireFeature` middleware que devuelve 403 si la ruta toca un módulo desactivado.

5. **Custom domains: solo el modelo + verification token.** v1.0 graba el dominio + emite un token de verificación, expone endpoints CRUD. **No hace lookup de DNS TXT** ni rutea por host header. v1.1 agrega un job nocturno que consulta DNS y marca `verified_at`, más middleware que resuelve `Host: tienda1.misitio.com` → `organization_id` para la storefront.

6. **JWT enrichment se difiere a v1.1.** v1.0 deja `auth_handlers::login` como está. Los endpoints del módulo no requieren `organization_id` en el JWT — usan el path param. v1.1 agrega `organization_id` al JWT (leyendo de `users.organization_id`, fallback al default), y el extractor lo consume.

### Workflow state machine

**Organization**:
```
PendingSetup ──activate──▶ Active ──suspend──▶ Suspended ──activate──▶ Active
```
v1.0: solo `Active` y `Suspended`. `PendingSetup` se reserva para v1.1 cuando entre el flujo de signup self-service.

**OrganizationDomain**: una org puede tener N dominios pero solo 1 `is_primary = TRUE` (forzado por unique partial index). `is_verified` empieza en `false` y v1.0 permite verificarlo manualmente vía endpoint (admin marca como verificado); v1.1 automatiza vía DNS TXT.

### Crate y dependencias

- Nuevo crate: `modules/tenancy/` (agregar a `Cargo.toml` workspace `members`).
- `Cargo.toml` espejo de los anteriores: `common`, `identity`, `events` + workspace deps. **No** importar otros módulos de negocio — tenancy es puramente "plataforma".

### Domain layer (`modules/tenancy/src/domain/`)

**Entidades** (`entities/`):

- `organization.rs` — `Organization { id, name, slug, contact_email, contact_phone, status, created_at, updated_at }`. Métodos: `register`, `reconstitute`, `update_contact`, `suspend`, `activate`. `slug` es URL-safe (`[a-z0-9-]+`, longitud 3-60), validado en el constructor.

- `organization_plan.rs` — `OrganizationPlan { id, organization_id, tier, feature_flags (JsonValue), seat_limit (Option<i32>), store_limit (Option<i32>), starts_at, expires_at (Option<DateTime>), created_at, updated_at }`. Una sola fila por org (UNIQUE `organization_id`). `feature_flags` es un objeto plano `{"booking": true, "restaurant": false, "service_orders": true, "loyalty": true}`. Default según tier (Free habilita solo retail, Pro suma 1-2 verticales, Enterprise suma todo).

- `organization_domain.rs` — `OrganizationDomain { id, organization_id, domain, is_verified, is_primary, verification_token (Option<String>), verified_at (Option<DateTime>), created_at }`. Métodos: `register` (genera token aleatorio v1.0), `mark_verified`, `set_primary`, `unset_primary`.

- `organization_branding.rs` — `OrganizationBranding { organization_id (PK + FK), logo_url, favicon_url, primary_color, secondary_color, accent_color, theme, custom_css, created_at, updated_at }`. Una sola fila por org (PK = FK a organizations). Métodos: `upsert_for_org` factory + setters.

**Value Objects** (`value_objects/`):

- `ids.rs` — `OrganizationId`, `OrganizationPlanId`, `OrganizationDomainId`. Branding usa `organization_id` como PK, no necesita id propio.
- `organization_status.rs` — enum `OrganizationStatus { Active, Suspended, PendingSetup }` con `FromStr` y `can_transition_to`.
- `plan_tier.rs` — enum `PlanTier { Free, Pro, Enterprise }` con `default_feature_flags() -> JsonValue` que devuelve el set por tier.
- `organization_theme.rs` — enum `OrganizationTheme { Light, Dark, System }`, default `System`.

**Repositorios** (`repositories/`):

- `organization_repository.rs` — `save`, `update`, `find_by_id`, `find_by_slug`, `list(only_active)`.
- `organization_plan_repository.rs` — `upsert` (1 row por org), `find_by_organization`.
- `organization_domain_repository.rs` — `save`, `update`, `delete`, `find_by_id`, `find_by_domain`, `list_by_organization`, `unset_primary_for_org` (transaccional helper para asegurar single-primary invariant).
- `organization_branding_repository.rs` — `upsert`, `find_by_organization`.

### Application layer

**Use cases** (`use_cases/`):

Organizations:
- `register_organization.rs` — valida slug único, crea Organization en `Active`, crea OrganizationPlan default según `tier` (default Free), retorna detail.
- `update_organization.rs` — name + contact (slug y status no se cambian aquí).
- `suspend_organization.rs` / `activate_organization.rs` — transiciones de status.
- `list_organizations.rs`, `get_organization.rs`.
- `get_organization_by_slug.rs` — usado por el endpoint público.

Plans:
- `set_plan.rs` — upsert tier + feature_flags + límites.
- `set_feature_flag.rs` — granular: `PUT /plan/features/{feature}` con `{enabled: true}`.
- `get_plan.rs`.

Domains:
- `register_domain.rs` — genera verification_token, persiste con `is_verified=false`.
- `verify_domain.rs` — marca verified (v1.0 manual; v1.1 DNS TXT).
- `set_primary_domain.rs` — transaccional: limpia `is_primary` en otros dominios de la org y setea el target.
- `delete_domain.rs`, `list_domains.rs`.
- `find_organization_by_domain.rs` — endpoint público para storefront.

Branding:
- `upsert_branding.rs`, `get_branding.rs`.

**DTOs** (`dtos/{commands,responses}.rs`): commands con validación serde; responses planos.

`OrganizationDetailResponse` incluye `plan + branding + domains` para evitar N+1 desde el dashboard.

`PublicOrganizationResponse` (para `by-slug` y `by-domain`): solo `{ id, name, slug, branding: { logo_url, theme, ... } }` — esconde plan, contacto, etc.

**Event subscriber** (`subscriber.rs`):

- `TenancyEventSubscriber` — `interested_in: []` en v1.0 (passive). v1.1 escuchará `users.created` para auto-asignar a la org del invitador, etc.

**Eventos publicados** (vía outbox, in-tx en use cases):
- `tenancy.organization.registered`, `.suspended`, `.activated`.
- `tenancy.plan.changed` (con tier antes/después y feature_flags antes/después — útil para auditoría).
- `tenancy.domain.verified`.

### Infrastructure (`modules/tenancy/src/infrastructure/persistence/`)

4 implementaciones SQLx siguiendo el patrón de `modules/restaurant_operations/`:
- `pg_organization_repository.rs`, `pg_organization_plan_repository.rs`, `pg_organization_domain_repository.rs`, `pg_organization_branding_repository.rs`.
- `set_primary` envuelve dos queries en una transacción: `UPDATE ... SET is_primary = FALSE WHERE organization_id = $1` + `UPDATE ... SET is_primary = TRUE WHERE id = $2`. El partial unique index garantiza que la transacción falla si quedan dos primaries.

### Migraciones

Continuando convención (restaurant_operations terminó en `43`):

- `20260501000044_create_organizations_table.sql`
- `20260501000045_create_organization_plans_table.sql`
- `20260501000046_create_organization_domains_table.sql`
- `20260501000047_create_organization_branding_table.sql`
- `20260501000048_add_organization_id_to_users_and_stores.sql` — **transversal** (ALTER TABLE).
- `20260501000049_seed_tenancy_permissions.sql`
- `20260501000050_create_default_organization.sql` — data migration (crea default org + plan + asigna users/stores existentes).

Esquema clave (resumido):

```sql
CREATE TABLE organizations (
    id UUID PRIMARY KEY,
    name VARCHAR(160) NOT NULL,
    slug VARCHAR(60) NOT NULL UNIQUE,
    contact_email VARCHAR(160) NOT NULL,
    contact_phone VARCHAR(40) NULL,
    status VARCHAR(16) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (status IN ('active','suspended','pending_setup')),
    CHECK (slug ~ '^[a-z0-9][a-z0-9-]{1,58}[a-z0-9]$')
);

CREATE TABLE organization_plans (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL UNIQUE REFERENCES organizations(id) ON DELETE CASCADE,
    tier VARCHAR(16) NOT NULL DEFAULT 'free',
    feature_flags JSONB NOT NULL DEFAULT '{}'::JSONB,
    seat_limit INTEGER NULL,
    store_limit INTEGER NULL,
    starts_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (tier IN ('free','pro','enterprise'))
);

CREATE TABLE organization_domains (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    domain VARCHAR(253) NOT NULL UNIQUE,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    verification_token VARCHAR(64) NULL,
    verified_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX idx_org_domains_one_primary
    ON organization_domains (organization_id) WHERE is_primary = TRUE;

CREATE TABLE organization_branding (
    organization_id UUID PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    logo_url VARCHAR(500) NULL,
    favicon_url VARCHAR(500) NULL,
    primary_color VARCHAR(7) NULL,
    secondary_color VARCHAR(7) NULL,
    accent_color VARCHAR(7) NULL,
    theme VARCHAR(16) NOT NULL DEFAULT 'system',
    custom_css TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (theme IN ('light','dark','system'))
);

-- Transversal:
ALTER TABLE users  ADD COLUMN organization_id UUID NULL REFERENCES organizations(id) ON DELETE SET NULL;
ALTER TABLE stores ADD COLUMN organization_id UUID NULL REFERENCES organizations(id) ON DELETE SET NULL;
CREATE INDEX idx_users_organization  ON users  (organization_id) WHERE organization_id IS NOT NULL;
CREATE INDEX idx_stores_organization ON stores (organization_id) WHERE organization_id IS NOT NULL;
```

Permisos: `tenancy:read_org`, `tenancy:write_org`, `tenancy:suspend_org`, `tenancy:read_plan`, `tenancy:write_plan`, `tenancy:read_domain`, `tenancy:write_domain`, `tenancy:verify_domain`, `tenancy:read_branding`, `tenancy:write_branding`. v1.0 los asigna solo a `super_admin`. v1.1 introducirá `org_admin` con un subset (read_org, write_branding, write_domain pero no write_plan).

### API Gateway

**Handlers** en `api-gateway/src/handlers/tenancy/`:
- `mod.rs`, `organizations.rs`, `plans.rs`, `domains.rs`, `branding.rs`, `public.rs`.

**Routers** en `api-gateway/src/routes/tenancy_routes.rs`:
- `organizations_router(state)` — auth, super_admin only en v1.0.
- `public_tenancy_router()` — sin auth (by-slug, by-domain).

**Endpoints**:

| Método | Path | Permiso | Descripción |
|---|---|---|---|
| GET | `/api/v1/organizations` | `tenancy:read_org` | Listar (super_admin) |
| POST | `/api/v1/organizations` | `tenancy:write_org` | Registrar |
| GET | `/api/v1/organizations/{id}` | `tenancy:read_org` | Detalle (con plan/domains/branding) |
| PUT | `/api/v1/organizations/{id}` | `tenancy:write_org` | Actualizar contacto |
| POST | `/api/v1/organizations/{id}/suspend` | `tenancy:suspend_org` | Suspender |
| POST | `/api/v1/organizations/{id}/activate` | `tenancy:suspend_org` | Re-activar |
| GET | `/api/v1/organizations/{id}/plan` | `tenancy:read_plan` | Plan actual |
| PUT | `/api/v1/organizations/{id}/plan` | `tenancy:write_plan` | Set tier + feature flags + limits |
| GET | `/api/v1/organizations/{id}/domains` | `tenancy:read_domain` | Listar dominios |
| POST | `/api/v1/organizations/{id}/domains` | `tenancy:write_domain` | Registrar (genera token) |
| POST | `/api/v1/organizations/{id}/domains/{did}/verify` | `tenancy:verify_domain` | Marcar verificado (manual v1.0) |
| POST | `/api/v1/organizations/{id}/domains/{did}/set-primary` | `tenancy:write_domain` | Marcar como primary |
| DELETE | `/api/v1/organizations/{id}/domains/{did}` | `tenancy:write_domain` | Eliminar |
| GET | `/api/v1/organizations/{id}/branding` | `tenancy:read_branding` | Branding actual |
| PUT | `/api/v1/organizations/{id}/branding` | `tenancy:write_branding` | Upsert |
| **GET** | `/api/v1/public/organizations/by-slug/{slug}` | — (público) | Lookup para storefront |
| **GET** | `/api/v1/public/organizations/by-domain/{domain}` | — (público) | Lookup por host |

### Mapeo de errores

`impl From<TenancyError> for AppError`: 404 (`OrganizationNotFound`, `PlanNotFound`, `DomainNotFound`, `BrandingNotFound`); 409 (`InvalidStatusTransition`, `SlugAlreadyTaken`, `DomainAlreadyTaken`); 400 (`InvalidStatus`/`InvalidTier`/`InvalidTheme`/`InvalidSlug`/`InvalidDomain`/`Validation`); 500 (`Database`, `Serialization`).

### Roadmap interno del módulo

1. **v1.0 — modelo + endpoints sin enforcement** (≈2 semanas)
   - Domain + value_objects + repos + use cases CRUD.
   - Migraciones (incluida la transversal NULLABLE + backfill).
   - Endpoints públicos (by-slug, by-domain) y autenticados.
   - Seed + permisos.
   - Smoke test: crear org, asignar plan, registrar domain, verificar manualmente, upsert branding, leer público.

2. **v1.1 — enforcement por middleware + JWT enrichment + role org_admin** (≈2 semanas) — **COMPLETADO**
   - **JWT enrichment**: `TokenClaims` gana campo `organization_id: Option<Uuid>` (con `#[serde(default)]` para backward-compat). `User` entity gana `organization_id`. `JwtTokenService::generate_access_token` lo lee y embebe.
   - **`UserContext` enrichment**: `auth_middleware` copia `claims.organization_id` al `UserContext`, expuesto vía `ctx.organization_id() -> Option<Uuid>`.
   - **`require_org_match(ctx, target_org_id)`**, **`verify_store_in_org(pool, ctx, store_id)`**, **`require_feature(pool, ctx, feature_name)`**: helpers en `api-gateway/src/middleware/org_scope.rs`. `super_admin` bypassa todo; cualquier otro role solo pasa si pertenece a la org. Devuelven 403 con códigos `CROSS_ORG_ACCESS_DENIED`, `CROSS_ORG_STORE_DENIED`, `FEATURE_DISABLED`.
   - **`org_admin` role nuevo**: subset de tenancy perms (read_org, read_plan, read_domain, write_domain, verify_domain, read_branding, write_branding). **NO** tiene write_org, write_plan, suspend_org. Asignado en seed a `demo_resto_admin@example.com` ligado a `demo-resto`.
   - **Aplicación del patrón a 2 verticales como prueba**:
     - **tenancy**: handlers protegidos con `require_org_match`; `list_organizations` filtra a own-org si no es super_admin.
     - **restaurant**: handlers wrapped con `require_feature("restaurant")` + `verify_store_in_org` donde aplica.
   - **Tier change snap a defaults**: `SetPlanUseCase` usa `tier.default_feature_flags()` cuando cambia el tier sin enviar flags explícitos.
   - Smoke test: 10 escenarios cross-org validados (super_admin ve todas; org_admin solo demo-resto; cross-org → 403; feature off → 403; store ajena → 403).

3. **v1.2 — rollout completo del enforcement + cache + global perms** (≈2 semanas) — **EN CURSO**
   - **TTL cache `(store_id → org_id)`** con 60s en `verify_store_in_org` para evitar la query por request. Implementación in-memory simple (`Arc<RwLock<HashMap<Uuid, (Uuid, Instant)>>>`) inyectada vía `AppState`. Invalidación por TTL únicamente en v1.2; invalidación explícita en mutaciones de stores se difiere.
   - **Global permissions en JWT**: `TokenClaims` gana `global_permissions: Vec<String>` (permisos no ligados a store, ej. todos los `tenancy:*`). `auth_middleware` los carga como fallback cuando no hay `X-Store-Id`. Resuelve el roce actual donde un org_admin necesita mandar `X-Store-Id` para usar endpoints de tenancy.
   - **Rollout enforcement a verticales con feature flag**:
     - `booking` (26 handlers, feature `booking`).
     - `service_orders` (19 handlers, feature `service_orders`).
     - `loyalty` (13 handlers, feature `loyalty`).
   - **Rollout `verify_store_in_org` a módulos core** (~195+ handlers, sin feature flag): sales, inventory, purchasing, fiscal, accounting, analytics, cash_management, catalog, demand_planning, shipping, payments. Patrón mecánico:
     ```rust
     require_permission(&ctx, "module:action")?;
     require_feature(state.pool(), &ctx, "feature_name").await?;  // solo verticales
     verify_store_in_org(state.pool(), &ctx, store_id).await?;     // donde el cliente envía store_id
     ```
   - Validación final: `cargo fmt && cargo clippy --workspace -- -D warnings && docker compose up`.

4. **v1.3 — signup self-service + DNS + quotas + endurecimiento DB** (≈2 semanas)
   - Self-service signup: `POST /api/v1/auth/register-organization` que crea org + admin user en una transacción.
   - Host-header → org middleware: lee `Host:`, busca en `organization_domains` con `is_verified = TRUE`, setea `organization_id` en el request scope.
   - Job nocturno `domain_verification.rs`: para cada dominio no verificado, lookup TXT y marca `verified_at` si el token coincide.
   - Quotas: `seat_limit` y `store_limit` enforceados en endpoints de creación de users/stores.
   - `ALTER TABLE users SET NOT NULL organization_id` y `ALTER TABLE stores SET NOT NULL organization_id` después de validar en producción.
   - Audit log dedicado para cambios de plan (qué, cuándo, quién).
   - Invalidación explícita del cache `(store_id → org_id)` en mutaciones de stores.

### Verificación end-to-end (v1.0)

- Seed: default org + 1 demo org adicional ("Restaurante Demo", slug `demo-resto`, plan `pro` con `restaurant=true,booking=false`, branding con primary_color, 1 dominio `demo-resto.example.com`).
- `GET /api/v1/organizations` → 200, lista 2 orgs.
- `GET /api/v1/public/organizations/by-slug/demo-resto` → 200 con branding (sin plan/contacto).
- `POST /api/v1/organizations` con slug nuevo → 201; mismo slug → 409.
- `PUT /api/v1/organizations/{id}/plan` cambiando feature_flags → 200; verificar que `GET .../plan` devuelve el JSONB actualizado.
- `POST .../domains` → 201 con `verification_token`. `POST .../verify` → 200 con `verified_at`. `POST .../set-primary` → 200; verificar partial unique index permite re-asignar primary y rechaza tener dos.
- `PUT .../branding` upsert con `primary_color: "#ef4444"`, `theme: "light"` → 200; `GET .../branding` retorna lo escrito.
- En la DB: `SELECT organization_id FROM users` y `FROM stores` debe estar 100% poblado con la default org tras el backfill.

### Patrones existentes a copiar (no reinventar)

- Layout completo: `modules/restaurant_operations/`.
- Macro de IDs y `FromStr` en VOs: `modules/restaurant_operations/src/domain/value_objects/`.
- Repo Pg con `set_primary` transaccional: análogo al `assign_groups_to_product` de `pg_menu_modifier_repository.rs`.
- Handler público sin `CurrentUser`: `api-gateway/src/handlers/booking/public.rs`.
- Router público sin auth layer: `api-gateway/src/routes/booking_routes.rs::public_booking_router`.
- Seed permisos en data.rs: `seed/src/data.rs` PERMISSIONS + ROLE_PERMISSIONS para super_admin (no agregar a store_admin en v1.0 — tenancy es super-admin only).
- Migración de datos no destructiva: ver migraciones de loyalty/booking/restaurant para el patrón de `INSERT ... ON CONFLICT DO NOTHING` + `UPDATE ... WHERE x IS NULL` para backfill.
- Generación UUIDs: `Uuid::new_v7(Timestamp::now(NoContext))`.

---

## Plan detallado — Módulo `subscriptions`

### Alcance v1.0 — SaaS billing del propio platform

v1.0 cobra recurrentemente a las `Organization`s por usar el sistema. La venta de membresías a customers (gimnasios, café del mes) se difiere a v1.1 reutilizando las mismas entidades.

### Decisiones de diseño

1. **`SubscriptionPlan` separado de `OrganizationPlan`**. `OrganizationPlan` (tenancy) lleva los **feature_flags** y límites del tier de un tenant; `SubscriptionPlan` (subscriptions) lleva el **precio** y la cadencia. Una org puede tener `OrganizationPlan` tier=`pro` + `Subscription` con `SubscriptionPlan` "Pro Mensual $49" o "Pro Anual $490". El `tier` en `OrganizationPlan` se actualiza cuando una `Subscription` activa cambia de plan (suscriptor del evento `subscription.activated`).

2. **Cadencia mensual únicamente en v1.0**. `BillingInterval { Monthly }`. `Quarterly`, `Annual` se difieren a v1.1.

3. **Trial opcional por plan**. `SubscriptionPlan.trial_days: Option<i32>`. Si > 0, la primera `BillingCycle` arranca con `status='trialing'` y no genera invoice; al expirar el trial, se transiciona a `active` y se factura el primer ciclo real.

4. **Sin payment method almacenado en v1.0**. El job de billing genera la `Invoice` (vía `fiscal::invoice`) y crea una `Transaction` `pending` (vía `payments`); el cobro real depende del gateway. Manual gateway = el org_admin paga via referencia bancaria; webhook del gateway confirma. v1.1 agrega `OrganizationPaymentMethod` con tokens de Stripe/Ficohsa.

5. **Dunning policy fija v1.0**: 3 reintentos a +1d, +3d, +7d. Tras el 3er fallo: `subscription.status = past_due`, evento `subscription.past_due` (notifica al org_admin). Si el org no paga en 14d adicionales: `canceled` (downgrade a tier `free`, suspende features pagos).

6. **Sin proration en v1.0**. Cambio de plan se aplica al fin del ciclo actual (`change_at_period_end = true` por default). Cancelación: por default `cancel_at_period_end`, opción `cancel_immediately` para super_admin.

### Entidades (`modules/subscriptions/src/domain/entities/`)

- `subscription_plan.rs` — `SubscriptionPlan { id, code (UNIQUE), name, description, tier (mirror del enum de tenancy), interval (Monthly), price_cents (i64), currency (ISO-4217), trial_days (Option<i32>), is_active, sort_order, created_at, updated_at }`. Borrado = soft (is_active=false).
- `subscription.rs` — `Subscription { id, organization_id, plan_id, status (Trialing|Active|PastDue|Canceled), current_period_start, current_period_end, trial_end (Option), cancel_at_period_end (bool), canceled_at (Option), created_at, updated_at, version }`. Una org tiene a lo sumo una Subscription no-Canceled (UNIQUE partial index). State machine: `start_trial`, `activate`, `mark_past_due`, `cancel(at_period_end)`, `resume`. Cada transición incrementa `version`.
- `billing_cycle.rs` — `BillingCycle { id, subscription_id, period_start, period_end, status (Pending|Trialing|Invoiced|Paid|Failed|Skipped), invoice_id (Option), transaction_id (Option), amount_cents, currency, attempted_at (Option), settled_at (Option), failure_reason (Option<String>), created_at }`. Generado por el job. Trialing → Skipped con amount=0.
- `dunning_attempt.rs` — `DunningAttempt { id, billing_cycle_id, attempt_number (1..=3), scheduled_at, executed_at (Option), outcome (Pending|Succeeded|Failed|Skipped), failure_reason (Option), transaction_id (Option), created_at }`.

### Value objects

`subscription_plan_id`, `subscription_id`, `billing_cycle_id`, `dunning_attempt_id` (Uuid newtypes via macro), `subscription_status`, `billing_cycle_status`, `billing_interval` (Monthly + `next_period(start)`), `dunning_outcome`, `plan_code` (newtype validador `[a-z0-9_]+`, max 32 chars).

### Repositorios (traits + Pg implementations)

- `SubscriptionPlanRepository` — `save`, `update`, `find_by_id`, `find_by_code`, `find_active`, `list_all (paginated)`.
- `SubscriptionRepository` — `save`, `update_with_version` (optimistic), `find_by_id`, `find_active_by_organization`, `list_due_for_billing(now)`, `list_past_due_pending_cancellation(now)`.
- `BillingCycleRepository` — `save`, `update`, `find_by_id`, `find_by_subscription`, `find_pending_due(now)`, `find_failed_pending_dunning`.
- `DunningAttemptRepository` — `save`, `update`, `find_by_billing_cycle`, `find_due(now)`.

### Errores (`SubscriptionError`)

`PlanNotFound`, `SubscriptionNotFound`, `BillingCycleNotFound`, `DunningAttemptNotFound`, `InvalidStatusTransition { from, to }`, `OrganizationAlreadySubscribed`, `PlanInactive`, `CodeAlreadyTaken`, `InvalidPlanCode`, `OptimisticLockFailed`, `Database(sqlx::Error)`, `FiscalIntegration(String)`, `PaymentIntegration(String)`.

### Eventos

- `subscription.created`, `subscription.activated` (Trial → Active; **suscriber dentro de tenancy** lo escucha y actualiza `OrganizationPlan.tier`/`feature_flags`), `subscription.canceled`, `subscription.past_due`.
- `billing_cycle.invoiced`, `billing_cycle.paid`, `billing_cycle.failed`.

`SubscriptionsEventSubscriber` — `interested_in: ["payment.confirmed", "payment.rejected"]`. Cuando llega `payment.confirmed` con `transaction_id` que corresponde a un `BillingCycle.transaction_id`, marca el cycle Paid y la subscription Active (si estaba PastDue).

### Migración

`20260601000001_create_subscriptions.sql` (timestamp arbitrario después de tenancy):

- `subscription_plans (id, code UNIQUE, name, description, tier, interval, price_cents, currency, trial_days, is_active, sort_order, created_at, updated_at)`.
- `subscriptions (id, organization_id FK, plan_id FK, status, current_period_start/end, trial_end, cancel_at_period_end, canceled_at, version, created_at, updated_at)` + UNIQUE partial index `WHERE status <> 'canceled'`.
- `billing_cycles (id, subscription_id FK, period_start/end, status, invoice_id, transaction_id, amount_cents, currency, attempted_at, settled_at, failure_reason, created_at)`.
- `dunning_attempts (id, billing_cycle_id FK, attempt_number, scheduled_at, executed_at, outcome, transaction_id, failure_reason, created_at)`.

Sin backfill v1.0 (módulo nuevo).

### Use cases

- `create_plan` / `update_plan` (no permite cambiar code/price/interval/tier — crear nuevo plan + migrar) / `deactivate_plan` / `list_plans` / `get_plan`.
- `subscribe_organization` — valida no haya activa; si plan tiene trial_days, status=Trialing; first BillingCycle Trialing-Skipped. Publica `subscription.created`.
- `cancel_subscription(at_period_end | immediately)` — `immediately` solo super_admin.
- `resume_subscription` — solo si `cancel_at_period_end=true` y aún no llegó period_end.
- `change_plan` — defer al fin del periodo (v1.0 cambia plan_id directo via job; sin proration).
- `process_billing_cycle` — invocado por el job. Genera invoice (fiscal), crea transaction (payments), marca cycle Invoiced.
- `record_payment_outcome` — invocado por subscriber de `payment.confirmed`/`payment.rejected`. Marca Paid/Failed, transiciona subscription, crea DunningAttempts si Failed.
- `process_dunning_attempt` — invocado por job. Reintenta cobro. Si las 3 attempts agotaron → cycle Failed definitivo + sub PastDue.
- `tick_past_due_subscriptions` — subs PastDue > 14d → Canceled + downgrade a free.

### Infraestructura

- `infrastructure/persistence/`: PgRepositorios (mismo patrón de tenancy).
- `infrastructure/fiscal_invoice_adapter.rs` — trait `BillingInvoiceGateway` que llama a `fiscal::CreateInvoiceUseCase`. Permite mockear en tests.
- `infrastructure/payments_charge_adapter.rs` — trait `BillingPaymentGateway` que invoca `payments::ProcessPaymentUseCase`.

### Job (`api-gateway/src/jobs/subscription_billing.rs`)

Corre cada hora. En cada tick:

1. Subs con `current_period_end <= now() AND status IN ('trialing','active')`: si trial expiró, activate (Trial → Active, publica `subscription.activated`). Crea siguiente BillingCycle Pending. Avanza period en la subscription.
2. BillingCycles Pending con `period_start <= now()` → ejecuta `process_billing_cycle`.
3. DunningAttempts Pending con `scheduled_at <= now()` → ejecuta `process_dunning_attempt`.
4. `tick_past_due_subscriptions` para cancelar las que agotaron grace period.

### API Gateway

**Permisos**: `subscriptions:read_plan`, `subscriptions:write_plan`, `subscriptions:read_subscription`, `subscriptions:write_subscription`, `subscriptions:cancel_subscription`. v1.0: super_admin tiene todos; org_admin tiene `read_plan`, `read_subscription` (su org), `cancel_subscription` (su sub al fin del periodo).

**Endpoints**:

| Método | Ruta | Permiso |
|---|---|---|
| GET | `/api/v1/subscription-plans` | (público) |
| GET | `/api/v1/subscription-plans/{id}` | (público) |
| POST | `/api/v1/subscription-plans` | `subscriptions:write_plan` |
| PUT | `/api/v1/subscription-plans/{id}` | `subscriptions:write_plan` |
| DELETE | `/api/v1/subscription-plans/{id}` | `subscriptions:write_plan` |
| POST | `/api/v1/organizations/{org_id}/subscription` | `subscriptions:write_subscription` |
| GET | `/api/v1/organizations/{org_id}/subscription` | `subscriptions:read_subscription` |
| GET | `/api/v1/organizations/{org_id}/subscription/cycles` | `subscriptions:read_subscription` |
| POST | `/api/v1/organizations/{org_id}/subscription/cancel` | `subscriptions:cancel_subscription` |
| POST | `/api/v1/organizations/{org_id}/subscription/resume` | `subscriptions:cancel_subscription` |
| POST | `/api/v1/organizations/{org_id}/subscription/change-plan` | `subscriptions:write_subscription` |
| GET | `/api/v1/admin/subscriptions` | `subscriptions:read_subscription` (super_admin) |

Cross-org check `require_org_match(&ctx, org_id)` en todos los `/organizations/{org_id}/subscription/*`. Globals JWT v1.2: las perms `subscriptions:*` se incluyen en `GLOBAL_PERMISSION_PREFIXES` para que el org_admin no necesite `X-Store-Id`.

### Seed

- Permisos en `seed/data.rs::PERMISSIONS` + asignados a super_admin (todos) y org_admin (read_plan/read_subscription/cancel_subscription).
- Plans iniciales: `free_monthly` ($0, free, no trial), `starter_monthly` ($19, starter, trial=14d), `pro_monthly` ($49, pro, trial=14d), `enterprise_monthly` ($199, enterprise, trial=30d).
- Demo subscription: `demo-resto` en `pro_monthly` activa, trial agotado, pagada.

### Verificación end-to-end

- Listar planes públicos → 4 plans.
- Subscribir org demo a `starter_monthly` → 201, status=Trialing, trial_end=+14d.
- Forzar trial_end < now → job activa, `OrganizationPlan.tier` cambia a `starter`.
- Forzar `current_period_end < now` → job genera BillingCycle Pending, luego Invoiced con invoice_id + transaction_id.
- Webhook `payment.confirmed` → cycle Paid, subscription Active.
- Webhook `payment.rejected` → cycle Failed, 3 DunningAttempts. Forzar 3 fallos → sub PastDue. Esperar 14d → Canceled, OrganizationPlan tier vuelve a `free`.

### Roadmap interno

1. **v1.0** (≈2 semanas) — todo lo de arriba. SaaS billing, monthly only, sin payment method almacenado.
2. **v1.1** (≈1 semana) — annual interval, proration en change_plan, `OrganizationPaymentMethod` con tokens.
3. **v1.2** (≈1.5 semanas) — Membresías que el comercio vende a sus customers.
4. **v1.3** (≈1 semana) — usage-based pricing.

### Patrones a copiar

- Layout: `modules/loyalty/`.
- Optimistic locking con `version`: `modules/inventory::stock`.
- State machine con `can_transition_to`: `modules/sales::sale`.
- Suscriptor a eventos: `modules/accounting::AccountingEventSubscriber`.
- Job pattern: `api-gateway/src/jobs/notification_dispatcher.rs`.
- Cross-org enforcement en handlers: `api-gateway/src/handlers/tenancy/`.
- Generación UUIDs: `Uuid::new_v7(Timestamp::now(NoContext))`.
