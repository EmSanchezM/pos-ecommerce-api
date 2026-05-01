// Application state for sharing dependencies across handlers
//
// This module defines the AppState struct that holds all shared dependencies
// for the API Gateway, following hexagonal architecture principles.

use std::sync::Arc;

use catalog::{
    DefaultImageStorageRegistry, ImageStorageRegistry, PgImageStorageProviderRepository,
    PgProductImageRepository, PgProductListingRepository, PgProductReviewRepository,
    PgWishlistRepository,
};
use events::{OutboxRepository, PgOutboxRepository, SubscriberRegistry};
use fiscal::{PgFiscalSequenceRepository, PgInvoiceRepository, PgTaxRateRepository};
use identity::{JwtTokenService, PgAuditRepository, PgStoreRepository, PgUserRepository};
use inventory::{
    PgAdjustmentRepository, PgCategoryRepository, PgInventoryMovementRepository,
    PgInventoryStockRepository, PgProductRepository, PgRecipeRepository, PgReservationRepository,
    PgTransferRepository,
};
use notifications::{
    DefaultNotificationAdapterRegistry, NotificationAdapterRegistry, PgNotificationRepository,
};
use payments::{
    DefaultGatewayAdapterRegistry, GatewayAdapterRegistry, PgPaymentGatewayRepository,
    PgPayoutRepository, PgTransactionRepository,
};
use pos_core::PgTerminalRepository;
use purchasing::{PgGoodsReceiptRepository, PgPurchaseOrderRepository, PgVendorRepository};
use sales::{
    PgCartRepository, PgCreditNoteRepository, PgCustomerRepository, PgPromotionRepository,
    PgSaleRepository, PgShiftRepository,
};
use shipping::{
    DefaultDeliveryProviderRegistry, DeliveryProviderRegistry, PgDeliveryProviderRepository,
    PgDriverRepository, PgShipmentRepository, PgShipmentTrackingEventRepository,
    PgShippingMethodRepository, PgShippingRateRepository, PgShippingZoneRepository, ShipmentDeps,
};
use sqlx::PgPool;

/// Application state shared across all HTTP handlers.
///
/// This struct holds Arc-wrapped instances of repositories and services
/// that are needed by the authentication handlers. Using Arc allows
/// efficient sharing across async tasks without cloning the underlying data.
///
/// # Architecture
///
/// The AppState uses concrete PostgreSQL implementations for production.
/// For testing, handlers can be tested with mock implementations directly.
#[derive(Clone)]
pub struct AppState {
    /// Direct access to the PostgreSQL connection pool for transactional operations
    pool: PgPool,
    /// User repository for user persistence operations
    user_repo: Arc<PgUserRepository>,
    /// Store repository for store persistence operations
    store_repo: Arc<PgStoreRepository>,
    /// Terminal repository for terminal persistence operations
    terminal_repo: Arc<PgTerminalRepository>,
    /// Audit repository for audit logging
    audit_repo: Arc<PgAuditRepository>,
    /// Token service for JWT generation and validation
    token_service: Arc<JwtTokenService>,
    // -------------------------------------------------------------------------
    // Inventory repositories
    // -------------------------------------------------------------------------
    /// Product repository for product catalog operations
    product_repo: Arc<PgProductRepository>,
    /// Category repository for product category operations
    category_repo: Arc<PgCategoryRepository>,
    /// Inventory stock repository for stock management
    stock_repo: Arc<PgInventoryStockRepository>,
    /// Reservation repository for stock reservations
    reservation_repo: Arc<PgReservationRepository>,
    /// Movement repository for inventory movement history (stock ledger)
    movement_repo: Arc<PgInventoryMovementRepository>,
    /// Recipe repository for recipe/BOM management
    recipe_repo: Arc<PgRecipeRepository>,
    /// Adjustment repository for stock adjustments
    adjustment_repo: Arc<PgAdjustmentRepository>,
    /// Transfer repository for inter-store transfers
    transfer_repo: Arc<PgTransferRepository>,
    // -------------------------------------------------------------------------
    // Purchasing repositories
    // -------------------------------------------------------------------------
    /// Vendor repository for vendor/supplier management
    vendor_repo: Arc<PgVendorRepository>,
    /// Purchase order repository for purchase order management
    purchase_order_repo: Arc<PgPurchaseOrderRepository>,
    /// Goods receipt repository for goods receipt management
    goods_receipt_repo: Arc<PgGoodsReceiptRepository>,
    // -------------------------------------------------------------------------
    // Sales repositories
    // -------------------------------------------------------------------------
    /// Customer repository for customer management
    customer_repo: Arc<PgCustomerRepository>,
    /// Sale repository for sale transaction operations
    sale_repo: Arc<PgSaleRepository>,
    /// Shift repository for cashier shift management
    shift_repo: Arc<PgShiftRepository>,
    /// Cart repository for e-commerce cart management
    cart_repo: Arc<PgCartRepository>,
    /// Credit note repository for returns management
    credit_note_repo: Arc<PgCreditNoteRepository>,
    /// Promotion repository for discount/coupon management
    promotion_repo: Arc<PgPromotionRepository>,
    // -------------------------------------------------------------------------
    // Fiscal repositories
    // -------------------------------------------------------------------------
    /// Invoice repository for fiscal invoice operations
    invoice_repo: Arc<PgInvoiceRepository>,
    /// Tax rate repository for tax rate management
    tax_rate_repo: Arc<PgTaxRateRepository>,
    /// Fiscal sequence repository for invoice numbering
    fiscal_sequence_repo: Arc<PgFiscalSequenceRepository>,
    // -------------------------------------------------------------------------
    // Payments repositories + adapter
    // -------------------------------------------------------------------------
    /// Payment gateway repository (super-admin managed)
    payment_gateway_repo: Arc<PgPaymentGatewayRepository>,
    /// Payment transaction repository
    transaction_repo: Arc<PgTransactionRepository>,
    /// Gateway payout repository
    payout_repo: Arc<PgPayoutRepository>,
    /// Registry that resolves a `GatewayType` to its adapter (Manual today,
    /// stubs for Stripe/PayPal/BAC/Ficohsa).
    gateway_registry: Arc<dyn GatewayAdapterRegistry>,
    // -------------------------------------------------------------------------
    // Shipping repositories + adapters
    // -------------------------------------------------------------------------
    shipping_method_repo: Arc<PgShippingMethodRepository>,
    shipping_zone_repo: Arc<PgShippingZoneRepository>,
    shipping_rate_repo: Arc<PgShippingRateRepository>,
    driver_repo: Arc<PgDriverRepository>,
    delivery_provider_repo: Arc<PgDeliveryProviderRepository>,
    shipment_repo: Arc<PgShipmentRepository>,
    shipment_event_repo: Arc<PgShipmentTrackingEventRepository>,
    delivery_registry: Arc<dyn DeliveryProviderRegistry>,
    /// Pre-built ShipmentDeps to avoid rewiring on every handler invocation.
    shipment_deps: Arc<ShipmentDeps>,
    // -------------------------------------------------------------------------
    // Catalog repositories + image adapter registry
    // -------------------------------------------------------------------------
    listing_repo: Arc<PgProductListingRepository>,
    image_repo: Arc<PgProductImageRepository>,
    review_repo: Arc<PgProductReviewRepository>,
    wishlist_repo: Arc<PgWishlistRepository>,
    image_storage_provider_repo: Arc<PgImageStorageProviderRepository>,
    image_storage_registry: Arc<dyn ImageStorageRegistry>,
    // -------------------------------------------------------------------------
    // Events (transactional outbox + in-process dispatch)
    // -------------------------------------------------------------------------
    outbox_repo: Arc<dyn OutboxRepository>,
    subscriber_registry: SubscriberRegistry,
    // -------------------------------------------------------------------------
    // Notifications (multi-channel outbound messaging + adapter registry)
    // -------------------------------------------------------------------------
    notification_repo: Arc<PgNotificationRepository>,
    notification_registry: Arc<dyn NotificationAdapterRegistry>,
}

impl AppState {
    /// Creates a new AppState with the given dependencies.
    ///
    /// # Arguments
    ///
    /// * `user_repo` - User repository implementation
    /// * `store_repo` - Store repository implementation
    /// * `terminal_repo` - Terminal repository implementation
    /// * `audit_repo` - Audit repository implementation
    /// * `token_service` - Token service implementation
    /// * `product_repo` - Product repository implementation
    /// * `category_repo` - Category repository implementation
    /// * `stock_repo` - Inventory stock repository implementation
    /// * `reservation_repo` - Reservation repository implementation
    /// * `movement_repo` - Inventory movement repository implementation
    /// * `recipe_repo` - Recipe repository implementation
    /// * `adjustment_repo` - Adjustment repository implementation
    /// * `transfer_repo` - Transfer repository implementation
    /// * `vendor_repo` - Vendor repository implementation
    /// * `purchase_order_repo` - Purchase order repository implementation
    /// * `goods_receipt_repo` - Goods receipt repository implementation
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pool: PgPool,
        user_repo: Arc<PgUserRepository>,
        store_repo: Arc<PgStoreRepository>,
        terminal_repo: Arc<PgTerminalRepository>,
        audit_repo: Arc<PgAuditRepository>,
        token_service: Arc<JwtTokenService>,
        product_repo: Arc<PgProductRepository>,
        category_repo: Arc<PgCategoryRepository>,
        stock_repo: Arc<PgInventoryStockRepository>,
        reservation_repo: Arc<PgReservationRepository>,
        movement_repo: Arc<PgInventoryMovementRepository>,
        recipe_repo: Arc<PgRecipeRepository>,
        adjustment_repo: Arc<PgAdjustmentRepository>,
        transfer_repo: Arc<PgTransferRepository>,
        vendor_repo: Arc<PgVendorRepository>,
        purchase_order_repo: Arc<PgPurchaseOrderRepository>,
        goods_receipt_repo: Arc<PgGoodsReceiptRepository>,
        customer_repo: Arc<PgCustomerRepository>,
        sale_repo: Arc<PgSaleRepository>,
        shift_repo: Arc<PgShiftRepository>,
        cart_repo: Arc<PgCartRepository>,
        credit_note_repo: Arc<PgCreditNoteRepository>,
        promotion_repo: Arc<PgPromotionRepository>,
        invoice_repo: Arc<PgInvoiceRepository>,
        tax_rate_repo: Arc<PgTaxRateRepository>,
        fiscal_sequence_repo: Arc<PgFiscalSequenceRepository>,
        payment_gateway_repo: Arc<PgPaymentGatewayRepository>,
        transaction_repo: Arc<PgTransactionRepository>,
        payout_repo: Arc<PgPayoutRepository>,
        gateway_registry: Arc<dyn GatewayAdapterRegistry>,
        shipping_method_repo: Arc<PgShippingMethodRepository>,
        shipping_zone_repo: Arc<PgShippingZoneRepository>,
        shipping_rate_repo: Arc<PgShippingRateRepository>,
        driver_repo: Arc<PgDriverRepository>,
        delivery_provider_repo: Arc<PgDeliveryProviderRepository>,
        shipment_repo: Arc<PgShipmentRepository>,
        shipment_event_repo: Arc<PgShipmentTrackingEventRepository>,
        delivery_registry: Arc<dyn DeliveryProviderRegistry>,
        shipment_deps: Arc<ShipmentDeps>,
        listing_repo: Arc<PgProductListingRepository>,
        image_repo: Arc<PgProductImageRepository>,
        review_repo: Arc<PgProductReviewRepository>,
        wishlist_repo: Arc<PgWishlistRepository>,
        image_storage_provider_repo: Arc<PgImageStorageProviderRepository>,
        image_storage_registry: Arc<dyn ImageStorageRegistry>,
        outbox_repo: Arc<dyn OutboxRepository>,
        subscriber_registry: SubscriberRegistry,
        notification_repo: Arc<PgNotificationRepository>,
        notification_registry: Arc<dyn NotificationAdapterRegistry>,
    ) -> Self {
        Self {
            pool,
            user_repo,
            store_repo,
            terminal_repo,
            audit_repo,
            token_service,
            product_repo,
            category_repo,
            stock_repo,
            reservation_repo,
            movement_repo,
            recipe_repo,
            adjustment_repo,
            transfer_repo,
            vendor_repo,
            purchase_order_repo,
            goods_receipt_repo,
            customer_repo,
            sale_repo,
            shift_repo,
            cart_repo,
            credit_note_repo,
            promotion_repo,
            invoice_repo,
            tax_rate_repo,
            fiscal_sequence_repo,
            payment_gateway_repo,
            transaction_repo,
            payout_repo,
            gateway_registry,
            shipping_method_repo,
            shipping_zone_repo,
            shipping_rate_repo,
            driver_repo,
            delivery_provider_repo,
            shipment_repo,
            shipment_event_repo,
            delivery_registry,
            shipment_deps,
            listing_repo,
            image_repo,
            review_repo,
            wishlist_repo,
            image_storage_provider_repo,
            image_storage_registry,
            outbox_repo,
            subscriber_registry,
            notification_repo,
            notification_registry,
        }
    }

    /// Creates an AppState from a PostgreSQL connection pool and JWT secret.
    ///
    /// This is a convenience constructor for production use that creates
    /// the concrete PostgreSQL repository implementations.
    ///
    /// # Arguments
    ///
    /// * `pool` - PostgreSQL connection pool
    /// * `jwt_secret` - Secret key for JWT signing (should be at least 32 bytes)
    pub fn from_pool(pool: PgPool, jwt_secret: String) -> Self {
        let pool_arc = Arc::new(pool.clone());

        // Identity repositories
        let user_repo = Arc::new(PgUserRepository::new((*pool_arc).clone()));
        let store_repo = Arc::new(PgStoreRepository::new((*pool_arc).clone()));
        let audit_repo = Arc::new(PgAuditRepository::new((*pool_arc).clone()));

        // Core repositories
        let terminal_repo = Arc::new(PgTerminalRepository::new(pool_arc.clone()));

        // Inventory repositories
        let product_repo = Arc::new(PgProductRepository::new((*pool_arc).clone()));
        let category_repo = Arc::new(PgCategoryRepository::new((*pool_arc).clone()));
        let stock_repo = Arc::new(PgInventoryStockRepository::new((*pool_arc).clone()));
        let reservation_repo = Arc::new(PgReservationRepository::new((*pool_arc).clone()));
        let movement_repo = Arc::new(PgInventoryMovementRepository::new((*pool_arc).clone()));
        let recipe_repo = Arc::new(PgRecipeRepository::new((*pool_arc).clone()));
        let adjustment_repo = Arc::new(PgAdjustmentRepository::new((*pool_arc).clone()));
        let transfer_repo = Arc::new(PgTransferRepository::new((*pool_arc).clone()));

        // Purchasing repositories
        let vendor_repo = Arc::new(PgVendorRepository::new((*pool_arc).clone()));
        let purchase_order_repo = Arc::new(PgPurchaseOrderRepository::new((*pool_arc).clone()));
        let goods_receipt_repo = Arc::new(PgGoodsReceiptRepository::new((*pool_arc).clone()));

        // Sales repositories
        let customer_repo = Arc::new(PgCustomerRepository::new((*pool_arc).clone()));
        let sale_repo = Arc::new(PgSaleRepository::new((*pool_arc).clone()));
        let shift_repo = Arc::new(PgShiftRepository::new((*pool_arc).clone()));
        let cart_repo = Arc::new(PgCartRepository::new((*pool_arc).clone()));
        let credit_note_repo = Arc::new(PgCreditNoteRepository::new((*pool_arc).clone()));
        let promotion_repo = Arc::new(PgPromotionRepository::new((*pool_arc).clone()));

        // Fiscal repositories
        let invoice_repo = Arc::new(PgInvoiceRepository::new((*pool_arc).clone()));
        let tax_rate_repo = Arc::new(PgTaxRateRepository::new((*pool_arc).clone()));
        let fiscal_sequence_repo = Arc::new(PgFiscalSequenceRepository::new((*pool_arc).clone()));

        // Payments repositories
        let payment_gateway_repo = Arc::new(PgPaymentGatewayRepository::new((*pool_arc).clone()));
        let transaction_repo = Arc::new(PgTransactionRepository::new((*pool_arc).clone()));
        let payout_repo = Arc::new(PgPayoutRepository::new((*pool_arc).clone()));
        // Default registry: Manual is fully wired today; Stripe/PayPal/BAC/
        // Ficohsa are stubs that fail loudly until their adapters are filled in.
        let gateway_registry: Arc<dyn GatewayAdapterRegistry> =
            Arc::new(DefaultGatewayAdapterRegistry::new());

        // Shipping repositories + adapters
        let shipping_method_repo = Arc::new(PgShippingMethodRepository::new((*pool_arc).clone()));
        let shipping_zone_repo = Arc::new(PgShippingZoneRepository::new((*pool_arc).clone()));
        let shipping_rate_repo = Arc::new(PgShippingRateRepository::new((*pool_arc).clone()));
        let driver_repo = Arc::new(PgDriverRepository::new((*pool_arc).clone()));
        let delivery_provider_repo =
            Arc::new(PgDeliveryProviderRepository::new((*pool_arc).clone()));
        let shipment_repo = Arc::new(PgShipmentRepository::new((*pool_arc).clone()));
        let shipment_event_repo =
            Arc::new(PgShipmentTrackingEventRepository::new((*pool_arc).clone()));
        let delivery_registry: Arc<dyn DeliveryProviderRegistry> =
            Arc::new(DefaultDeliveryProviderRegistry::new());
        let shipment_deps = Arc::new(ShipmentDeps {
            method_repo: shipping_method_repo.clone(),
            driver_repo: driver_repo.clone(),
            provider_repo: delivery_provider_repo.clone(),
            shipment_repo: shipment_repo.clone(),
            event_repo: shipment_event_repo.clone(),
            provider_registry: delivery_registry.clone(),
            transaction_repo: transaction_repo.clone(),
        });

        // Catalog repositories + image storage registry
        let listing_repo = Arc::new(PgProductListingRepository::new((*pool_arc).clone()));
        let image_repo = Arc::new(PgProductImageRepository::new((*pool_arc).clone()));
        let review_repo = Arc::new(PgProductReviewRepository::new((*pool_arc).clone()));
        let wishlist_repo = Arc::new(PgWishlistRepository::new((*pool_arc).clone()));
        let image_storage_provider_repo =
            Arc::new(PgImageStorageProviderRepository::new((*pool_arc).clone()));
        let image_storage_registry: Arc<dyn ImageStorageRegistry> =
            Arc::new(DefaultImageStorageRegistry::new());

        // Events repositories + (empty) subscriber registry
        let outbox_repo: Arc<dyn OutboxRepository> =
            Arc::new(PgOutboxRepository::new((*pool_arc).clone()));
        // Subscribers register themselves at startup; downstream modules
        // (analytics, accounting, notifications-webhooks, ...) push their
        // EventSubscriber implementations into this registry. Empty by default.
        let subscriber_registry = SubscriberRegistry::new();

        // Notifications repository + adapter registry
        let notification_repo = Arc::new(PgNotificationRepository::new((*pool_arc).clone()));
        let notification_registry: Arc<dyn NotificationAdapterRegistry> =
            Arc::new(DefaultNotificationAdapterRegistry::new());

        // Services
        let token_service = Arc::new(JwtTokenService::new(jwt_secret));

        Self {
            pool,
            user_repo,
            store_repo,
            terminal_repo,
            audit_repo,
            token_service,
            product_repo,
            category_repo,
            stock_repo,
            reservation_repo,
            movement_repo,
            recipe_repo,
            adjustment_repo,
            transfer_repo,
            vendor_repo,
            purchase_order_repo,
            goods_receipt_repo,
            customer_repo,
            sale_repo,
            shift_repo,
            cart_repo,
            credit_note_repo,
            promotion_repo,
            invoice_repo,
            tax_rate_repo,
            fiscal_sequence_repo,
            payment_gateway_repo,
            transaction_repo,
            payout_repo,
            gateway_registry,
            shipping_method_repo,
            shipping_zone_repo,
            shipping_rate_repo,
            driver_repo,
            delivery_provider_repo,
            shipment_repo,
            shipment_event_repo,
            delivery_registry,
            shipment_deps,
            listing_repo,
            image_repo,
            review_repo,
            wishlist_repo,
            image_storage_provider_repo,
            image_storage_registry,
            outbox_repo,
            subscriber_registry,
            notification_repo,
            notification_registry,
        }
    }

    /// Returns a reference to the PostgreSQL connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Returns a reference to the user repository.
    pub fn user_repo(&self) -> Arc<PgUserRepository> {
        self.user_repo.clone()
    }

    /// Returns a reference to the store repository.
    pub fn store_repo(&self) -> Arc<PgStoreRepository> {
        self.store_repo.clone()
    }

    /// Returns a reference to the terminal repository.
    pub fn terminal_repo(&self) -> Arc<PgTerminalRepository> {
        self.terminal_repo.clone()
    }

    /// Returns a reference to the audit repository.
    pub fn audit_repo(&self) -> Arc<PgAuditRepository> {
        self.audit_repo.clone()
    }

    /// Returns a reference to the token service.
    pub fn token_service(&self) -> Arc<JwtTokenService> {
        self.token_service.clone()
    }

    // -------------------------------------------------------------------------
    // Inventory repository accessors
    // -------------------------------------------------------------------------

    /// Returns a reference to the product repository.
    pub fn product_repo(&self) -> Arc<PgProductRepository> {
        self.product_repo.clone()
    }

    /// Returns a reference to the category repository.
    pub fn category_repo(&self) -> Arc<PgCategoryRepository> {
        self.category_repo.clone()
    }

    /// Returns a reference to the inventory stock repository.
    pub fn stock_repo(&self) -> Arc<PgInventoryStockRepository> {
        self.stock_repo.clone()
    }

    /// Returns a reference to the reservation repository.
    pub fn reservation_repo(&self) -> Arc<PgReservationRepository> {
        self.reservation_repo.clone()
    }

    /// Returns a reference to the inventory movement repository.
    pub fn movement_repo(&self) -> Arc<PgInventoryMovementRepository> {
        self.movement_repo.clone()
    }

    /// Returns a reference to the recipe repository.
    pub fn recipe_repo(&self) -> Arc<PgRecipeRepository> {
        self.recipe_repo.clone()
    }

    /// Returns a reference to the adjustment repository.
    pub fn adjustment_repo(&self) -> Arc<PgAdjustmentRepository> {
        self.adjustment_repo.clone()
    }

    /// Returns a reference to the transfer repository.
    pub fn transfer_repo(&self) -> Arc<PgTransferRepository> {
        self.transfer_repo.clone()
    }

    // -------------------------------------------------------------------------
    // Purchasing repository accessors
    // -------------------------------------------------------------------------

    /// Returns a reference to the vendor repository.
    pub fn vendor_repo(&self) -> Arc<PgVendorRepository> {
        self.vendor_repo.clone()
    }

    /// Returns a reference to the purchase order repository.
    pub fn purchase_order_repo(&self) -> Arc<PgPurchaseOrderRepository> {
        self.purchase_order_repo.clone()
    }

    /// Returns a reference to the goods receipt repository.
    pub fn goods_receipt_repo(&self) -> Arc<PgGoodsReceiptRepository> {
        self.goods_receipt_repo.clone()
    }

    // -------------------------------------------------------------------------
    // Sales repository accessors
    // -------------------------------------------------------------------------

    /// Returns a reference to the customer repository.
    pub fn customer_repo(&self) -> Arc<PgCustomerRepository> {
        self.customer_repo.clone()
    }

    /// Returns a reference to the sale repository.
    pub fn sale_repo(&self) -> Arc<PgSaleRepository> {
        self.sale_repo.clone()
    }

    /// Returns a reference to the shift repository.
    pub fn shift_repo(&self) -> Arc<PgShiftRepository> {
        self.shift_repo.clone()
    }

    /// Returns a reference to the cart repository.
    pub fn cart_repo(&self) -> Arc<PgCartRepository> {
        self.cart_repo.clone()
    }

    /// Returns a reference to the credit note repository.
    pub fn credit_note_repo(&self) -> Arc<PgCreditNoteRepository> {
        self.credit_note_repo.clone()
    }

    /// Returns a reference to the promotion repository.
    pub fn promotion_repo(&self) -> Arc<PgPromotionRepository> {
        self.promotion_repo.clone()
    }

    // -------------------------------------------------------------------------
    // Fiscal repository accessors
    // -------------------------------------------------------------------------

    /// Returns a reference to the invoice repository.
    pub fn invoice_repo(&self) -> Arc<PgInvoiceRepository> {
        self.invoice_repo.clone()
    }

    /// Returns a reference to the tax rate repository.
    pub fn tax_rate_repo(&self) -> Arc<PgTaxRateRepository> {
        self.tax_rate_repo.clone()
    }

    /// Returns a reference to the fiscal sequence repository.
    pub fn fiscal_sequence_repo(&self) -> Arc<PgFiscalSequenceRepository> {
        self.fiscal_sequence_repo.clone()
    }

    // -------------------------------------------------------------------------
    // Payments accessors
    // -------------------------------------------------------------------------

    /// Returns a reference to the payment gateway repository.
    pub fn payment_gateway_repo(&self) -> Arc<PgPaymentGatewayRepository> {
        self.payment_gateway_repo.clone()
    }

    /// Returns a reference to the payment transaction repository.
    pub fn transaction_repo(&self) -> Arc<PgTransactionRepository> {
        self.transaction_repo.clone()
    }

    /// Returns a reference to the payout repository.
    pub fn payout_repo(&self) -> Arc<PgPayoutRepository> {
        self.payout_repo.clone()
    }

    /// Returns the configured default gateway adapter.
    pub fn gateway_registry(&self) -> Arc<dyn GatewayAdapterRegistry> {
        self.gateway_registry.clone()
    }

    // -------------------------------------------------------------------------
    // Shipping accessors
    // -------------------------------------------------------------------------

    pub fn shipping_method_repo(&self) -> Arc<PgShippingMethodRepository> {
        self.shipping_method_repo.clone()
    }
    pub fn shipping_zone_repo(&self) -> Arc<PgShippingZoneRepository> {
        self.shipping_zone_repo.clone()
    }
    pub fn shipping_rate_repo(&self) -> Arc<PgShippingRateRepository> {
        self.shipping_rate_repo.clone()
    }
    pub fn driver_repo(&self) -> Arc<PgDriverRepository> {
        self.driver_repo.clone()
    }
    pub fn delivery_provider_repo(&self) -> Arc<PgDeliveryProviderRepository> {
        self.delivery_provider_repo.clone()
    }
    pub fn shipment_repo(&self) -> Arc<PgShipmentRepository> {
        self.shipment_repo.clone()
    }
    pub fn shipment_event_repo(&self) -> Arc<PgShipmentTrackingEventRepository> {
        self.shipment_event_repo.clone()
    }
    pub fn delivery_registry(&self) -> Arc<dyn DeliveryProviderRegistry> {
        self.delivery_registry.clone()
    }
    pub fn shipment_deps(&self) -> Arc<ShipmentDeps> {
        self.shipment_deps.clone()
    }

    // -------------------------------------------------------------------------
    // Catalog accessors
    // -------------------------------------------------------------------------

    pub fn listing_repo(&self) -> Arc<PgProductListingRepository> {
        self.listing_repo.clone()
    }
    pub fn catalog_image_repo(&self) -> Arc<PgProductImageRepository> {
        self.image_repo.clone()
    }
    pub fn review_repo(&self) -> Arc<PgProductReviewRepository> {
        self.review_repo.clone()
    }
    pub fn wishlist_repo(&self) -> Arc<PgWishlistRepository> {
        self.wishlist_repo.clone()
    }
    pub fn image_storage_provider_repo(&self) -> Arc<PgImageStorageProviderRepository> {
        self.image_storage_provider_repo.clone()
    }
    pub fn image_storage_registry(&self) -> Arc<dyn ImageStorageRegistry> {
        self.image_storage_registry.clone()
    }

    // -------------------------------------------------------------------------
    // Events accessors
    // -------------------------------------------------------------------------

    pub fn outbox_repo(&self) -> Arc<dyn OutboxRepository> {
        self.outbox_repo.clone()
    }
    pub fn subscriber_registry(&self) -> SubscriberRegistry {
        self.subscriber_registry.clone()
    }

    // -------------------------------------------------------------------------
    // Notifications accessors
    // -------------------------------------------------------------------------

    pub fn notification_repo(&self) -> Arc<PgNotificationRepository> {
        self.notification_repo.clone()
    }
    pub fn notification_registry(&self) -> Arc<dyn NotificationAdapterRegistry> {
        self.notification_registry.clone()
    }
}
