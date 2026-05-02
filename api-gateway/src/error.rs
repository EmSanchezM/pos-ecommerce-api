// API Gateway Error Handling
//
// This module provides a unified error type for the API Gateway that maps
// domain errors to appropriate HTTP responses.

use accounting::AccountingError;
use analytics::AnalyticsError;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use cash_management::CashManagementError;
use catalog::CatalogError;
use demand_planning::DemandPlanningError;
use fiscal::FiscalError;
use identity::{AuthError, ErrorResponse, IdentityError};
use inventory::InventoryError;
use payments::PaymentsError;
use pos_core::CoreError;
use purchasing::PurchasingError;
use sales::SalesError;
use shipping::ShippingError;

// =============================================================================
// AppError - Unified API Gateway Error Type
// =============================================================================

/// Unified error type for the API Gateway.
///
/// This struct wraps various domain errors and implements `IntoResponse`
/// to convert them into appropriate HTTP responses with JSON bodies.
///
/// # Error Mapping
///
/// | Domain Error | HTTP Status | Error Code |
/// |-------------|-------------|------------|
/// | InvalidCredentials | 401 | INVALID_CREDENTIALS |
/// | AccountDisabled | 401 | ACCOUNT_DISABLED |
/// | TokenExpired | 401 | TOKEN_EXPIRED |
/// | InvalidToken | 401 | INVALID_TOKEN |
/// | PasswordTooShort | 400 | VALIDATION_ERROR |
/// | InvalidEmailFormat | 400 | VALIDATION_ERROR |
/// | InvalidUsernameFormat | 400 | VALIDATION_ERROR |
/// | InvalidName | 400 | VALIDATION_ERROR |
/// | DuplicateEmail | 409 | DUPLICATE_EMAIL |
/// | DuplicateUsername | 409 | DUPLICATE_USERNAME |
/// | StoreNotFound | 404 | STORE_NOT_FOUND |
/// | Internal | 500 | INTERNAL_ERROR |
#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    response: ErrorResponse,
}

impl AppError {
    /// Creates a new AppError with the given status code and error response.
    pub fn new(status: StatusCode, response: ErrorResponse) -> Self {
        Self { status, response }
    }

    /// Returns the HTTP status code for this error.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Returns a reference to the error response.
    pub fn response(&self) -> &ErrorResponse {
        &self.response
    }
}

// =============================================================================
// IntoResponse Implementation
// =============================================================================

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.status, Json(self.response)).into_response()
    }
}

// =============================================================================
// From<AuthError> Implementation
// =============================================================================

impl From<AuthError> for AppError {
    fn from(err: AuthError) -> Self {
        let (status, response) = match &err {
            // 401 Unauthorized - Authentication failures
            AuthError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::invalid_credentials(),
            ),
            AuthError::AccountDisabled => {
                (StatusCode::UNAUTHORIZED, ErrorResponse::account_disabled())
            }
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, ErrorResponse::token_expired()),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, ErrorResponse::invalid_token()),

            // 400 Bad Request - Validation errors
            AuthError::PasswordTooShort => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error(
                    "Password too short: minimum 8 characters required",
                ),
            ),
            AuthError::InvalidEmailFormat => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid email format"),
            ),
            AuthError::InvalidUsernameFormat => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid username format"),
            ),
            AuthError::InvalidName(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error(msg),
            ),

            // 409 Conflict - Duplicate resources
            AuthError::DuplicateEmail(_) => {
                (StatusCode::CONFLICT, ErrorResponse::duplicate_email())
            }
            AuthError::DuplicateUsername(_) => {
                (StatusCode::CONFLICT, ErrorResponse::duplicate_username())
            }

            // 404 Not Found
            AuthError::StoreNotFound => (StatusCode::NOT_FOUND, ErrorResponse::store_not_found()),

            // 500 Internal Server Error - Don't expose internal details
            AuthError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
        };

        AppError::new(status, response)
    }
}

// =============================================================================
// From<IdentityError> Implementation
// =============================================================================

impl From<IdentityError> for AppError {
    fn from(err: IdentityError) -> Self {
        let (status, response) = match &err {
            // 400 Bad Request - Validation errors
            IdentityError::InvalidPermissionFormat => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid permission format"),
            ),
            IdentityError::InvalidEmailFormat => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid email format"),
            ),
            IdentityError::InvalidUsernameFormat => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid username format"),
            ),

            // 409 Conflict - Duplicate resources
            IdentityError::DuplicatePermission(name) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_PERMISSION",
                    format!("Permission '{}' already exists", name),
                ),
            ),
            IdentityError::DuplicateRole(name) => (
                StatusCode::CONFLICT,
                ErrorResponse::new("DUPLICATE_ROLE", format!("Role '{}' already exists", name)),
            ),
            IdentityError::DuplicateEmail(_) => {
                (StatusCode::CONFLICT, ErrorResponse::duplicate_email())
            }
            IdentityError::DuplicateUsername(_) => {
                (StatusCode::CONFLICT, ErrorResponse::duplicate_username())
            }

            // 404 Not Found
            IdentityError::PermissionNotFound(_) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("PERMISSION_NOT_FOUND", "Permission not found"),
            ),
            IdentityError::RoleNotFound(_) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("ROLE_NOT_FOUND", "Role not found"),
            ),
            IdentityError::UserNotFound(_) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("USER_NOT_FOUND", "User not found"),
            ),
            IdentityError::StoreNotFound(_) => {
                (StatusCode::NOT_FOUND, ErrorResponse::store_not_found())
            }
            IdentityError::StoreInactive(_) => {
                (StatusCode::NOT_FOUND, ErrorResponse::store_not_found())
            }
            IdentityError::UserNotInStore(_) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("USER_NOT_IN_STORE", "User is not a member of this store"),
            ),

            // 403 Forbidden - Protected resources
            IdentityError::ProtectedRoleCannotBeDeleted => (
                StatusCode::FORBIDDEN,
                ErrorResponse::new("PROTECTED_ROLE", "Cannot delete system-protected role"),
            ),

            // 401 Unauthorized - Account status
            IdentityError::UserInactive => {
                (StatusCode::UNAUTHORIZED, ErrorResponse::account_disabled())
            }

            // 500 Internal Server Error - Database and other errors
            IdentityError::AuditError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("AUDIT_ERROR", "Failed to record audit entry"),
            ),
            IdentityError::PasswordHashError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
            IdentityError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
            IdentityError::NotImplemented => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
        };

        AppError::new(status, response)
    }
}

// =============================================================================
// From<CoreError> Implementation
// =============================================================================

impl From<CoreError> for AppError {
    fn from(err: CoreError) -> Self {
        let (status, response) = match &err {
            // 404 Not Found
            CoreError::StoreNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("STORE_NOT_FOUND", format!("Store not found: {}", id)),
            ),
            CoreError::TerminalNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("TERMINAL_NOT_FOUND", format!("Terminal not found: {}", id)),
            ),

            // 400 Bad Request - Business rule violations
            CoreError::StoreInactive(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("STORE_INACTIVE", format!("Store is inactive: {}", id)),
            ),
            CoreError::TerminalInactive(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("TERMINAL_INACTIVE", format!("Terminal is inactive: {}", id)),
            ),
            CoreError::InvalidTerminalCode => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error(
                    "Invalid terminal code format: must be alphanumeric with hyphens, 3-20 characters",
                ),
            ),
            CoreError::InvalidCaiNumber => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid CAI number format"),
            ),
            CoreError::InvalidCaiRange => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid CAI range: start must be <= end"),
            ),
            CoreError::NoCaiAssigned(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "NO_CAI_ASSIGNED",
                    format!("No CAI assigned to terminal: {}", id),
                ),
            ),
            CoreError::CaiExpired(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "CAI_EXPIRED",
                    format!("CAI has expired for terminal: {}", id),
                ),
            ),
            CoreError::CaiRangeExhausted(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "CAI_RANGE_EXHAUSTED",
                    format!("CAI range exhausted for terminal: {}", id),
                ),
            ),

            // 409 Conflict - Duplicate resources
            CoreError::TerminalCodeExists(code) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "TERMINAL_CODE_EXISTS",
                    format!("Terminal code already exists: {}", code),
                ),
            ),
            CoreError::CaiRangeOverlap => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "CAI_RANGE_OVERLAP",
                    "CAI range overlaps with existing active range",
                ),
            ),

            // 403 Forbidden
            CoreError::Unauthorized => (
                StatusCode::FORBIDDEN,
                ErrorResponse::new("FORBIDDEN", "Unauthorized: requires super_admin role"),
            ),

            // 500 Internal Server Error
            CoreError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
        };

        AppError::new(status, response)
    }
}

// =============================================================================
// From<InventoryError> Implementation
// =============================================================================

impl From<InventoryError> for AppError {
    fn from(err: InventoryError) -> Self {
        let (status, response) = match &err {
            // -----------------------------------------------------------------
            // 404 Not Found - Resource not found errors
            // -----------------------------------------------------------------
            InventoryError::CategoryNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("CATEGORY_NOT_FOUND", format!("Category not found: {}", id)),
            ),
            InventoryError::ParentCategoryNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "PARENT_CATEGORY_NOT_FOUND",
                    format!("Parent category not found: {}", id),
                ),
            ),
            InventoryError::ProductNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("PRODUCT_NOT_FOUND", format!("Product not found: {}", id)),
            ),
            InventoryError::VariantNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("VARIANT_NOT_FOUND", format!("Variant not found: {}", id)),
            ),
            InventoryError::StockNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("STOCK_NOT_FOUND", format!("Stock record not found: {}", id)),
            ),
            InventoryError::ReservationNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "RESERVATION_NOT_FOUND",
                    format!("Reservation not found: {}", id),
                ),
            ),
            InventoryError::RecipeNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("RECIPE_NOT_FOUND", format!("Recipe not found: {}", id)),
            ),
            InventoryError::IngredientNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "INGREDIENT_NOT_FOUND",
                    format!("Ingredient not found: {}", id),
                ),
            ),
            InventoryError::SubstituteNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "SUBSTITUTE_NOT_FOUND",
                    format!("Substitute not found: {}", id),
                ),
            ),
            InventoryError::AdjustmentNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "ADJUSTMENT_NOT_FOUND",
                    format!("Adjustment not found: {}", id),
                ),
            ),
            InventoryError::TransferNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("TRANSFER_NOT_FOUND", format!("Transfer not found: {}", id)),
            ),

            // -----------------------------------------------------------------
            // 409 Conflict - Duplicate resources and version conflicts
            // -----------------------------------------------------------------
            InventoryError::DuplicateCategorySlug(slug) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_CATEGORY_SLUG",
                    format!("Category slug '{}' already exists", slug),
                ),
            ),
            InventoryError::DuplicateSku(sku) => (
                StatusCode::CONFLICT,
                ErrorResponse::new("DUPLICATE_SKU", format!("SKU '{}' already exists", sku)),
            ),
            InventoryError::DuplicateBarcode(barcode) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_BARCODE",
                    format!("Barcode '{}' already exists", barcode),
                ),
            ),
            InventoryError::ActiveRecipeExists => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "ACTIVE_RECIPE_EXISTS",
                    "Active recipe already exists for this product/variant",
                ),
            ),
            InventoryError::OptimisticLockError => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "VERSION_CONFLICT",
                    "Record was modified by another process, please retry",
                ),
            ),
            InventoryError::StockAlreadyExists {
                store_id,
                product_id,
                variant_id,
            } => {
                let item_type = if product_id.is_some() {
                    "product"
                } else {
                    "variant"
                };
                let item_id = product_id.or(*variant_id).unwrap_or(*store_id);
                (
                    StatusCode::CONFLICT,
                    ErrorResponse::new(
                        "STOCK_ALREADY_EXISTS",
                        format!(
                            "Stock already exists for {} {} in store {}",
                            item_type, item_id, store_id
                        ),
                    ),
                )
            }

            // -----------------------------------------------------------------
            // 400 Bad Request - Validation and business rule violations
            // -----------------------------------------------------------------
            InventoryError::VariantsNotEnabled => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Product does not have variants enabled"),
            ),
            InventoryError::InsufficientStock => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INSUFFICIENT_STOCK", "Insufficient stock available"),
            ),
            InventoryError::NegativeStock => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Cannot have negative stock"),
            ),
            InventoryError::ReservedExceedsQuantity => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Reserved quantity cannot exceed total quantity"),
            ),
            InventoryError::InvalidReleaseQuantity => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid release quantity"),
            ),
            InventoryError::ReservationExpired => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("RESERVATION_EXPIRED", "Reservation has expired"),
            ),
            InventoryError::InvalidReservationStatus => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "INVALID_RESERVATION_STATUS",
                    "Invalid reservation status transition",
                ),
            ),
            InventoryError::IngredientInUse => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "INGREDIENT_IN_USE",
                    "Cannot delete ingredient used in active recipes",
                ),
            ),
            InventoryError::InvalidYieldQuantity => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Yield quantity must be positive"),
            ),
            InventoryError::InvalidIngredientQuantity => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Ingredient quantity must be positive"),
            ),
            InventoryError::InvalidWastePercentage => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Waste percentage must be between 0 and 1"),
            ),
            InventoryError::InvalidConversionRatio => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Conversion ratio must be positive"),
            ),
            InventoryError::InvalidSubstitutePriority => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Substitute priority must be non-negative"),
            ),
            InventoryError::SubstitutesNotAllowed => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "SUBSTITUTES_NOT_ALLOWED",
                    "Ingredient does not allow substitutes",
                ),
            ),
            InventoryError::EmptyAdjustment => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Adjustment has no items"),
            ),
            InventoryError::AdjustmentAlreadyApplied => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "ADJUSTMENT_ALREADY_APPLIED",
                    "Cannot modify applied adjustment",
                ),
            ),
            InventoryError::SameStoreTransfer => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("SAME_STORE_TRANSFER", "Cannot transfer to the same store"),
            ),
            InventoryError::EmptyTransfer => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Transfer has no items"),
            ),
            InventoryError::InvalidStatusTransition => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_STATUS_TRANSITION", "Invalid status transition"),
            ),
            InventoryError::InvalidCurrency => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error(
                    "Invalid currency code: must be 3 uppercase letters (ISO 4217)",
                ),
            ),
            InventoryError::InvalidBarcode => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid barcode: maximum 100 characters"),
            ),
            InventoryError::InvalidUnitOfMeasure => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid unit of measure"),
            ),
            InventoryError::InvalidMovementType => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid movement type"),
            ),
            InventoryError::InvalidAdjustmentType => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid adjustment type"),
            ),
            InventoryError::InvalidAdjustmentReason => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid adjustment reason"),
            ),
            InventoryError::InvalidTransferStatus => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid transfer status"),
            ),
            InventoryError::InvalidReservationStatusValue => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid reservation status"),
            ),
            InventoryError::InvalidAdjustmentStatus => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid adjustment status"),
            ),
            InventoryError::InvalidProductVariantConstraint => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error(
                    "Must specify either product_id or variant_id, but not both",
                ),
            ),

            // -----------------------------------------------------------------
            // 500 Internal Server Error - Database and system errors
            // -----------------------------------------------------------------
            InventoryError::AuditError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("AUDIT_ERROR", "Failed to record audit entry"),
            ),
            InventoryError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
            InventoryError::InvalidOperation(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_OPERATION", msg),
            ),
            InventoryError::NotImplemented => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("NOT_IMPLEMENTED", "Feature not yet implemented"),
            ),
        };

        AppError::new(status, response)
    }
}

// =============================================================================
// From<PurchasingError> Implementation
// =============================================================================

impl From<PurchasingError> for AppError {
    fn from(err: PurchasingError) -> Self {
        let (status, response) = match &err {
            // -----------------------------------------------------------------
            // 404 Not Found - Resource not found errors
            // -----------------------------------------------------------------
            PurchasingError::VendorNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("VENDOR_NOT_FOUND", format!("Vendor not found: {}", id)),
            ),
            PurchasingError::PurchaseOrderNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "PURCHASE_ORDER_NOT_FOUND",
                    format!("Purchase order not found: {}", id),
                ),
            ),
            PurchasingError::PurchaseOrderItemNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "PURCHASE_ORDER_ITEM_NOT_FOUND",
                    format!("Purchase order item not found: {}", id),
                ),
            ),
            PurchasingError::GoodsReceiptNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "GOODS_RECEIPT_NOT_FOUND",
                    format!("Goods receipt not found: {}", id),
                ),
            ),
            PurchasingError::GoodsReceiptItemNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "GOODS_RECEIPT_ITEM_NOT_FOUND",
                    format!("Goods receipt item not found: {}", id),
                ),
            ),
            PurchasingError::ProductNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("PRODUCT_NOT_FOUND", format!("Product not found: {}", id)),
            ),
            PurchasingError::StoreNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("STORE_NOT_FOUND", format!("Store not found: {}", id)),
            ),

            // -----------------------------------------------------------------
            // 409 Conflict - Duplicate resources
            // -----------------------------------------------------------------
            PurchasingError::DuplicateVendorCode(code) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_VENDOR_CODE",
                    format!("Vendor code '{}' already exists", code),
                ),
            ),
            PurchasingError::DuplicateVendorTaxId(tax_id) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_VENDOR_TAX_ID",
                    format!("Vendor tax ID '{}' already exists", tax_id),
                ),
            ),
            PurchasingError::DuplicateOrderNumber(number) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_ORDER_NUMBER",
                    format!("Order number '{}' already exists", number),
                ),
            ),
            PurchasingError::DuplicateReceiptNumber(number) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_RECEIPT_NUMBER",
                    format!("Receipt number '{}' already exists", number),
                ),
            ),

            // -----------------------------------------------------------------
            // 400 Bad Request - Business rule violations
            // -----------------------------------------------------------------
            PurchasingError::VendorNotActive(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("VENDOR_NOT_ACTIVE", format!("Vendor is not active: {}", id)),
            ),
            PurchasingError::OrderNotEditable => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "ORDER_NOT_EDITABLE",
                    "Cannot modify purchase order: not in draft status",
                ),
            ),
            PurchasingError::EmptyPurchaseOrder => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Purchase order has no items"),
            ),
            PurchasingError::CannotApproveSelfCreatedOrder => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "CANNOT_APPROVE_OWN_ORDER",
                    "User cannot approve their own purchase order",
                ),
            ),
            PurchasingError::OrderAlreadyCancelled => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "ORDER_ALREADY_CANCELLED",
                    "Purchase order has already been cancelled",
                ),
            ),
            PurchasingError::OrderAlreadyClosed => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "ORDER_ALREADY_CLOSED",
                    "Purchase order has already been closed",
                ),
            ),
            PurchasingError::OrderNotApproved => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "ORDER_NOT_APPROVED",
                    "Cannot receive goods: purchase order not approved",
                ),
            ),
            PurchasingError::OrderHasReceivedGoods => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "ORDER_HAS_RECEIVED_GOODS",
                    "Cannot cancel: purchase order has received goods",
                ),
            ),
            PurchasingError::InvalidQuantityOrdered => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Quantity ordered must be positive"),
            ),
            PurchasingError::InvalidUnitCost => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Unit cost must be non-negative"),
            ),
            PurchasingError::ExceedsOrderedQuantity => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "EXCEEDS_ORDERED_QUANTITY",
                    "Cannot receive more than ordered quantity",
                ),
            ),
            PurchasingError::ReceiptNotEditable => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "RECEIPT_NOT_EDITABLE",
                    "Cannot modify goods receipt: not in draft status",
                ),
            ),
            PurchasingError::EmptyGoodsReceipt => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Goods receipt has no items"),
            ),
            PurchasingError::ReceiptAlreadyConfirmed => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "RECEIPT_ALREADY_CONFIRMED",
                    "Goods receipt has already been confirmed",
                ),
            ),
            PurchasingError::ReceiptAlreadyCancelled => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "RECEIPT_ALREADY_CANCELLED",
                    "Goods receipt has already been cancelled",
                ),
            ),
            PurchasingError::InvalidQuantityReceived => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Quantity received must be positive"),
            ),
            PurchasingError::InvalidStatusTransition => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_STATUS_TRANSITION", "Invalid status transition"),
            ),
            PurchasingError::InvalidCurrency => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error(
                    "Invalid currency code: must be 3 uppercase letters (ISO 4217)",
                ),
            ),
            PurchasingError::InvalidUnitOfMeasure => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid unit of measure"),
            ),
            PurchasingError::InvalidPurchaseOrderStatus => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid purchase order status"),
            ),
            PurchasingError::InvalidGoodsReceiptStatus => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid goods receipt status"),
            ),

            // -----------------------------------------------------------------
            // 500 Internal Server Error - Database and system errors
            // -----------------------------------------------------------------
            PurchasingError::AuditError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("AUDIT_ERROR", "Failed to record audit entry"),
            ),
            PurchasingError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
            PurchasingError::NotImplemented => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("NOT_IMPLEMENTED", "Feature not yet implemented"),
            ),
        };

        AppError::new(status, response)
    }
}

// =============================================================================
// From<SalesError> Implementation
// =============================================================================

impl From<SalesError> for AppError {
    fn from(err: SalesError) -> Self {
        let (status, response) = match &err {
            // -----------------------------------------------------------------
            // 404 Not Found
            // -----------------------------------------------------------------
            SalesError::CustomerNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("CUSTOMER_NOT_FOUND", format!("Customer not found: {}", id)),
            ),
            SalesError::ShiftNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("SHIFT_NOT_FOUND", format!("Shift not found: {}", id)),
            ),
            SalesError::SaleNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("SALE_NOT_FOUND", format!("Sale not found: {}", id)),
            ),
            SalesError::SaleItemNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "SALE_ITEM_NOT_FOUND",
                    format!("Sale item not found: {}", id),
                ),
            ),
            SalesError::PaymentNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("PAYMENT_NOT_FOUND", format!("Payment not found: {}", id)),
            ),
            SalesError::CartNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("CART_NOT_FOUND", format!("Cart not found: {}", id)),
            ),
            SalesError::CartItemNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "CART_ITEM_NOT_FOUND",
                    format!("Cart item not found: {}", id),
                ),
            ),
            SalesError::CreditNoteNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "CREDIT_NOTE_NOT_FOUND",
                    format!("Credit note not found: {}", id),
                ),
            ),
            SalesError::ProductNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("PRODUCT_NOT_FOUND", format!("Product not found: {}", id)),
            ),
            SalesError::StoreNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("STORE_NOT_FOUND", format!("Store not found: {}", id)),
            ),
            SalesError::TerminalNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("TERMINAL_NOT_FOUND", format!("Terminal not found: {}", id)),
            ),
            // -----------------------------------------------------------------
            // 409 Conflict - Duplicate or state conflicts
            // -----------------------------------------------------------------
            SalesError::DuplicateCustomerCode(code) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_CUSTOMER_CODE",
                    format!("Customer code '{}' already exists", code),
                ),
            ),
            SalesError::DuplicateCustomerEmail(email) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_CUSTOMER_EMAIL",
                    format!("Customer email '{}' already exists", email),
                ),
            ),
            SalesError::DuplicateSaleNumber(number) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_SALE_NUMBER",
                    format!("Sale number '{}' already exists", number),
                ),
            ),
            SalesError::DuplicateCreditNoteNumber(number) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_CREDIT_NOTE_NUMBER",
                    format!("Credit note number '{}' already exists", number),
                ),
            ),
            SalesError::TerminalHasOpenShift => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "TERMINAL_HAS_OPEN_SHIFT",
                    "Terminal already has an open shift",
                ),
            ),
            SalesError::CashierHasOpenShift => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "CASHIER_HAS_OPEN_SHIFT",
                    "Cashier already has an open shift",
                ),
            ),
            SalesError::SaleAlreadyCompleted => (
                StatusCode::CONFLICT,
                ErrorResponse::new("SALE_ALREADY_COMPLETED", "Sale has already been completed"),
            ),
            SalesError::SaleAlreadyVoided => (
                StatusCode::CONFLICT,
                ErrorResponse::new("SALE_ALREADY_VOIDED", "Sale has already been voided"),
            ),
            SalesError::PaymentAlreadyRefunded => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "PAYMENT_ALREADY_REFUNDED",
                    "Payment has already been refunded",
                ),
            ),
            SalesError::CreditNoteAlreadyApproved => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "CREDIT_NOTE_ALREADY_APPROVED",
                    "Credit note has already been approved",
                ),
            ),
            SalesError::CreditNoteAlreadyCancelled => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "CREDIT_NOTE_ALREADY_CANCELLED",
                    "Credit note has already been cancelled",
                ),
            ),
            SalesError::CreditNoteAlreadyApplied => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "CREDIT_NOTE_ALREADY_APPLIED",
                    "Credit note has already been applied",
                ),
            ),
            SalesError::ShiftAlreadyClosed => (
                StatusCode::CONFLICT,
                ErrorResponse::new("SHIFT_ALREADY_CLOSED", "Shift is already closed"),
            ),
            // -----------------------------------------------------------------
            // 400 Bad Request - Validation and business rule violations
            // -----------------------------------------------------------------
            SalesError::CustomerNotActive(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "CUSTOMER_NOT_ACTIVE",
                    format!("Customer is not active: {}", id),
                ),
            ),
            SalesError::TerminalNotActive(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "TERMINAL_NOT_ACTIVE",
                    format!("Terminal is not active: {}", id),
                ),
            ),
            SalesError::NoOpenShift => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("NO_OPEN_SHIFT", "No open shift found for terminal"),
            ),
            SalesError::InvalidOpeningBalance => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "INVALID_OPENING_BALANCE",
                    "Opening balance must be non-negative",
                ),
            ),
            SalesError::SaleNotEditable => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "SALE_NOT_EDITABLE",
                    "Cannot modify sale: not in draft status",
                ),
            ),
            SalesError::EmptySale => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("EMPTY_SALE", "Sale has no items"),
            ),
            SalesError::SaleNotFullyPaid => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("SALE_NOT_FULLY_PAID", "Sale is not fully paid"),
            ),
            SalesError::PosRequiresOpenShift => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("POS_REQUIRES_OPEN_SHIFT", "POS sale requires an open shift"),
            ),
            SalesError::PosRequiresTerminal => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("POS_REQUIRES_TERMINAL", "POS sale requires a terminal"),
            ),
            SalesError::PosRequiresCashier => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("POS_REQUIRES_CASHIER", "POS sale requires a cashier"),
            ),
            SalesError::InvalidQuantity => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_QUANTITY", "Quantity must be positive"),
            ),
            SalesError::InvalidUnitPrice => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_UNIT_PRICE", "Unit price must be non-negative"),
            ),
            SalesError::InsufficientStock(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "INSUFFICIENT_STOCK",
                    format!("Insufficient stock for product: {}", id),
                ),
            ),
            SalesError::InvalidPaymentAmount => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_PAYMENT_AMOUNT", "Payment amount must be positive"),
            ),
            SalesError::PaymentExceedsBalance => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "PAYMENT_EXCEEDS_BALANCE",
                    "Payment exceeds remaining balance",
                ),
            ),
            SalesError::CashRequiresAmountTendered => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "CASH_REQUIRES_AMOUNT_TENDERED",
                    "Cash payment requires amount tendered",
                ),
            ),
            SalesError::InsufficientAmountTendered => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "INSUFFICIENT_AMOUNT_TENDERED",
                    "Amount tendered is less than payment amount",
                ),
            ),
            SalesError::CartExpired => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("CART_EXPIRED", "Cart has expired"),
            ),
            SalesError::EmptyCart => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("EMPTY_CART", "Cart is empty"),
            ),
            SalesError::CreditNoteNotEditable => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "CREDIT_NOTE_NOT_EDITABLE",
                    "Cannot modify credit note: not in draft status",
                ),
            ),
            SalesError::EmptyCreditNote => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("EMPTY_CREDIT_NOTE", "Credit note has no items"),
            ),
            SalesError::CannotApproveSelfCreatedCreditNote => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "CANNOT_APPROVE_SELF_CREATED",
                    "User cannot approve their own credit note",
                ),
            ),
            SalesError::ReturnQuantityExceedsSaleQuantity => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "RETURN_QUANTITY_EXCEEDS",
                    "Return quantity exceeds original sale quantity",
                ),
            ),
            SalesError::SaleNotCompleted => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "SALE_NOT_COMPLETED",
                    "Cannot create return for incomplete sale",
                ),
            ),
            SalesError::CannotCancelShippedOrder => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "CANNOT_CANCEL_SHIPPED_ORDER",
                    "Cannot cancel order that has been shipped",
                ),
            ),
            SalesError::OrderNotPaid => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("ORDER_NOT_PAID", "Order has not been paid"),
            ),
            SalesError::OrderNotProcessing => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("ORDER_NOT_PROCESSING", "Order is not in processing status"),
            ),
            SalesError::OrderNotShipped => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("ORDER_NOT_SHIPPED", "Order has not been shipped"),
            ),
            SalesError::NoValidCai(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "NO_VALID_CAI",
                    format!("No valid CAI available for terminal: {}", id),
                ),
            ),
            SalesError::ReservationFailed => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "RESERVATION_FAILED",
                    "Failed to create inventory reservation",
                ),
            ),
            SalesError::ReservationConfirmFailed => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "RESERVATION_CONFIRM_FAILED",
                    "Failed to confirm inventory reservation",
                ),
            ),
            SalesError::ReservationCancelFailed => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "RESERVATION_CANCEL_FAILED",
                    "Failed to cancel inventory reservation",
                ),
            ),
            // -----------------------------------------------------------------
            // 400 Bad Request - Validation (enum parsing)
            // -----------------------------------------------------------------
            SalesError::InvalidStatusTransition => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_STATUS_TRANSITION", "Invalid status transition"),
            ),
            SalesError::InvalidCurrency => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_CURRENCY", "Invalid currency code"),
            ),
            SalesError::InvalidUnitOfMeasure => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_UNIT_OF_MEASURE", "Invalid unit of measure"),
            ),
            SalesError::InvalidSaleStatus => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_SALE_STATUS", "Invalid sale status"),
            ),
            SalesError::InvalidOrderStatus => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_ORDER_STATUS", "Invalid order status"),
            ),
            SalesError::InvalidPaymentMethod => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_PAYMENT_METHOD", "Invalid payment method"),
            ),
            SalesError::InvalidPaymentStatus => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_PAYMENT_STATUS", "Invalid payment status"),
            ),
            SalesError::InvalidDiscountType => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_DISCOUNT_TYPE", "Invalid discount type"),
            ),
            SalesError::InvalidCustomerType => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_CUSTOMER_TYPE", "Invalid customer type"),
            ),
            SalesError::InvalidReturnReason => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_RETURN_REASON", "Invalid return reason"),
            ),
            SalesError::InvalidCreditNoteStatus => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_CREDIT_NOTE_STATUS", "Invalid credit note status"),
            ),
            SalesError::InvalidDiscountPercentage => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "INVALID_DISCOUNT_PERCENTAGE",
                    "Discount percentage must be between 0 and 100",
                ),
            ),
            SalesError::InvalidTaxRate => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_TAX_RATE", "Tax rate must be non-negative"),
            ),
            // -----------------------------------------------------------------
            // 500 Internal Server Error
            // -----------------------------------------------------------------
            SalesError::AuditError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("AUDIT_ERROR", "Failed to record audit entry"),
            ),
            SalesError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
            SalesError::NotImplemented => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("NOT_IMPLEMENTED", "Feature not yet implemented"),
            ),

            // Promotion errors
            SalesError::PromotionNotFound(_) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("PROMOTION_NOT_FOUND", "Promotion not found"),
            ),
            SalesError::DuplicatePromotionCode(_) => (
                StatusCode::CONFLICT,
                ErrorResponse::new("DUPLICATE_PROMOTION_CODE", "Promotion code already exists"),
            ),
            SalesError::PromotionNotActive => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "PROMOTION_NOT_ACTIVE",
                    "Promotion is not active or has expired",
                ),
            ),
            SalesError::PromotionUsageLimitExceeded => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "PROMOTION_USAGE_LIMIT_EXCEEDED",
                    "Promotion usage limit exceeded",
                ),
            ),
            SalesError::MinimumPurchaseNotMet(_) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "MINIMUM_PURCHASE_NOT_MET",
                    "Minimum purchase requirement not met",
                ),
            ),
            SalesError::InvalidPromotionType => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_PROMOTION_TYPE", "Invalid promotion type"),
            ),
            SalesError::InvalidPromotionStatus => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_PROMOTION_STATUS", "Invalid promotion status"),
            ),
        };

        AppError::new(status, response)
    }
}

// =============================================================================
// From<FiscalError> Implementation
// =============================================================================

impl From<FiscalError> for AppError {
    fn from(err: FiscalError) -> Self {
        let (status, response) = match &err {
            // -----------------------------------------------------------------
            // 404 Not Found
            // -----------------------------------------------------------------
            FiscalError::InvoiceNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("INVOICE_NOT_FOUND", format!("Invoice not found: {}", id)),
            ),
            FiscalError::TaxRateNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("TAX_RATE_NOT_FOUND", format!("Tax rate not found: {}", id)),
            ),
            FiscalError::SaleNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("SALE_NOT_FOUND", format!("Sale not found: {}", id)),
            ),

            // -----------------------------------------------------------------
            // 409 Conflict
            // -----------------------------------------------------------------
            FiscalError::DuplicateInvoiceNumber(number) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_INVOICE_NUMBER",
                    format!("Invoice number '{}' already exists", number),
                ),
            ),
            FiscalError::DuplicateTaxRateName(name) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_TAX_RATE_NAME",
                    format!("Tax rate name '{}' already exists", name),
                ),
            ),
            FiscalError::InvoiceAlreadyExistsForSale(id) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "INVOICE_ALREADY_EXISTS_FOR_SALE",
                    format!("Invoice already exists for sale: {}", id),
                ),
            ),
            FiscalError::InvoiceAlreadyVoided => (
                StatusCode::CONFLICT,
                ErrorResponse::new("INVOICE_ALREADY_VOIDED", "Invoice has already been voided"),
            ),
            FiscalError::InvoiceCannotBeVoided => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "INVOICE_CANNOT_BE_VOIDED",
                    "Invoice cannot be voided in its current status",
                ),
            ),

            // -----------------------------------------------------------------
            // 400 Bad Request
            // -----------------------------------------------------------------
            FiscalError::VoidWindowExpired => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "VOID_WINDOW_EXPIRED",
                    "Cannot void emitted invoice older than 72 hours",
                ),
            ),
            FiscalError::CannotDeleteDefaultTaxRate => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "CANNOT_DELETE_DEFAULT_TAX_RATE",
                    "Cannot delete default tax rate",
                ),
            ),
            FiscalError::FiscalSequenceNotFound => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "FISCAL_SEQUENCE_NOT_FOUND",
                    "No fiscal sequence found for terminal",
                ),
            ),
            FiscalError::FiscalSequenceExhausted(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "FISCAL_SEQUENCE_EXHAUSTED",
                    format!("Fiscal sequence exhausted for terminal: {}", id),
                ),
            ),
            FiscalError::SequenceExhausted => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "SEQUENCE_EXHAUSTED",
                    "Fiscal sequence range has been fully exhausted",
                ),
            ),
            FiscalError::NoActiveCai(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "NO_ACTIVE_CAI",
                    format!("No active CAI for terminal: {}", id),
                ),
            ),
            FiscalError::CaiExpired(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("CAI_EXPIRED", format!("CAI expired for terminal: {}", id)),
            ),
            FiscalError::SaleNotCompleted(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("SALE_NOT_COMPLETED", format!("Sale not completed: {}", id)),
            ),
            FiscalError::InvalidInvoiceType => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid invoice type"),
            ),
            FiscalError::InvalidInvoiceStatus => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid invoice status"),
            ),
            FiscalError::InvalidTaxType => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid tax type"),
            ),
            FiscalError::InvalidTaxRate => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid tax rate: must be between 0 and 1"),
            ),
            FiscalError::InvalidTaxAppliesTo => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid tax applies to value"),
            ),
            FiscalError::OriginalInvoiceRequired => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "ORIGINAL_INVOICE_REQUIRED",
                    "Original invoice required for credit/debit note",
                ),
            ),
            FiscalError::CreditNoteExceedsOriginal => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "CREDIT_NOTE_EXCEEDS_ORIGINAL",
                    "Credit note exceeds original invoice total",
                ),
            ),

            // -----------------------------------------------------------------
            // 500 Internal Server Error
            // -----------------------------------------------------------------
            FiscalError::AuditError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("AUDIT_ERROR", "Failed to record audit entry"),
            ),
            FiscalError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
            FiscalError::NotImplemented => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("NOT_IMPLEMENTED", "Feature not yet implemented"),
            ),
        };

        AppError::new(status, response)
    }
}

// =============================================================================
// From<PaymentsError> Implementation
// =============================================================================

impl From<PaymentsError> for AppError {
    fn from(err: PaymentsError) -> Self {
        let (status, response) = match &err {
            // 404 Not Found
            PaymentsError::GatewayNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "PAYMENT_GATEWAY_NOT_FOUND",
                    format!("Payment gateway not found: {}", id),
                ),
            ),
            PaymentsError::TransactionNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "PAYMENT_TRANSACTION_NOT_FOUND",
                    format!("Transaction not found: {}", id),
                ),
            ),
            PaymentsError::PayoutNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("PAYOUT_NOT_FOUND", format!("Payout not found: {}", id)),
            ),
            PaymentsError::SaleNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("SALE_NOT_FOUND", format!("Sale not found: {}", id)),
            ),

            // 409 Conflict
            PaymentsError::DuplicateGatewayName(name) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_GATEWAY_NAME",
                    format!("Gateway name '{}' already exists for this store", name),
                ),
            ),
            PaymentsError::DuplicateIdempotencyKey(key) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_IDEMPOTENCY_KEY",
                    format!("Idempotency key already used: {}", key),
                ),
            ),
            PaymentsError::TransactionAlreadyProcessed(id) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "TRANSACTION_ALREADY_PROCESSED",
                    format!("Transaction already processed: {}", id),
                ),
            ),

            // 400 Bad Request
            PaymentsError::NoDefaultGateway(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "NO_DEFAULT_GATEWAY",
                    format!("No default payment gateway configured for store: {}", id),
                ),
            ),
            PaymentsError::GatewayNotActive(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "GATEWAY_NOT_ACTIVE",
                    format!("Gateway is not active: {}", id),
                ),
            ),
            PaymentsError::RefundExceedsOriginal => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "REFUND_EXCEEDS_ORIGINAL",
                    "Refund amount exceeds original transaction amount",
                ),
            ),
            PaymentsError::CannotRefundTransaction => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "CANNOT_REFUND_TRANSACTION",
                    "Transaction cannot be refunded in its current status",
                ),
            ),
            PaymentsError::ProcessingFailed(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("PAYMENT_PROCESSING_FAILED", msg.clone()),
            ),
            PaymentsError::InvalidAmount => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Amount must be positive"),
            ),
            PaymentsError::InvalidGatewayType => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid gateway type"),
            ),
            PaymentsError::InvalidTransactionType => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid transaction type"),
            ),
            PaymentsError::InvalidTransactionStatus => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid transaction status"),
            ),
            PaymentsError::InvalidManualPaymentKind => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid manual payment kind"),
            ),
            PaymentsError::TransactionNotPending(id) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "TRANSACTION_NOT_PENDING",
                    format!(
                        "Transaction is not pending and cannot be confirmed/rejected: {}",
                        id
                    ),
                ),
            ),
            PaymentsError::TransactionAlreadyConfirmed(id) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "TRANSACTION_ALREADY_CONFIRMED",
                    format!("Transaction has already been confirmed: {}", id),
                ),
            ),
            PaymentsError::TransactionAlreadyRejected(id) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "TRANSACTION_ALREADY_REJECTED",
                    format!("Transaction has already been rejected: {}", id),
                ),
            ),
            PaymentsError::InvalidPayoutStatus => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid payout status"),
            ),
            PaymentsError::UnsupportedPaymentMethod => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "UNSUPPORTED_PAYMENT_METHOD",
                    "Payment method not supported by the selected gateway",
                ),
            ),
            PaymentsError::UnsupportedCurrency => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "UNSUPPORTED_CURRENCY",
                    "Currency not supported by the selected gateway",
                ),
            ),

            // 401 Unauthorized
            PaymentsError::InvalidWebhookSignature => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::new("INVALID_WEBHOOK_SIGNATURE", "Invalid webhook signature"),
            ),

            // 502 / 500
            PaymentsError::GatewayError(msg) => (
                StatusCode::BAD_GATEWAY,
                ErrorResponse::new("GATEWAY_ERROR", msg.clone()),
            ),
            PaymentsError::AuditError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("AUDIT_ERROR", "Failed to record audit entry"),
            ),
            PaymentsError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
            PaymentsError::NotImplemented => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("NOT_IMPLEMENTED", "Feature not yet implemented"),
            ),
        };

        AppError::new(status, response)
    }
}

// =============================================================================
// From<ShippingError> Implementation
// =============================================================================

impl From<ShippingError> for AppError {
    fn from(err: ShippingError) -> Self {
        let (status, response) = match &err {
            ShippingError::ShippingMethodNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "SHIPPING_METHOD_NOT_FOUND",
                    format!("Shipping method not found: {}", id),
                ),
            ),
            ShippingError::ShippingZoneNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "SHIPPING_ZONE_NOT_FOUND",
                    format!("Shipping zone not found: {}", id),
                ),
            ),
            ShippingError::ShippingRateNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "SHIPPING_RATE_NOT_FOUND",
                    format!("Shipping rate not found: {}", id),
                ),
            ),
            ShippingError::DriverNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("DRIVER_NOT_FOUND", format!("Driver not found: {}", id)),
            ),
            ShippingError::DeliveryProviderNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "DELIVERY_PROVIDER_NOT_FOUND",
                    format!("Delivery provider not found: {}", id),
                ),
            ),
            ShippingError::ShipmentNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("SHIPMENT_NOT_FOUND", format!("Shipment not found: {}", id)),
            ),
            ShippingError::SaleNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("SALE_NOT_FOUND", format!("Sale not found: {}", id)),
            ),
            ShippingError::DuplicateMethodCode(code) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_METHOD_CODE",
                    format!(
                        "Shipping method code '{}' already exists for this store",
                        code
                    ),
                ),
            ),
            ShippingError::DuplicateDriverPhone(phone) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_DRIVER_PHONE",
                    format!("Driver phone '{}' already exists for this store", phone),
                ),
            ),
            ShippingError::DuplicateProviderName(name) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_PROVIDER_NAME",
                    format!("Delivery provider name '{}' already exists", name),
                ),
            ),
            ShippingError::ShipmentAlreadyExistsForSale(id) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "SHIPMENT_ALREADY_EXISTS_FOR_SALE",
                    format!("Shipment already exists for sale: {}", id),
                ),
            ),
            ShippingError::ShipmentAlreadyDelivered => (
                StatusCode::CONFLICT,
                ErrorResponse::new("SHIPMENT_ALREADY_DELIVERED", "Shipment already delivered"),
            ),
            ShippingError::ShipmentAlreadyCancelled => (
                StatusCode::CONFLICT,
                ErrorResponse::new("SHIPMENT_ALREADY_CANCELLED", "Shipment already cancelled"),
            ),
            ShippingError::DriverBusy => (
                StatusCode::CONFLICT,
                ErrorResponse::new("DRIVER_BUSY", "Driver is busy with another shipment"),
            ),
            ShippingError::NoMatchingZone => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "NO_MATCHING_ZONE",
                    "No shipping zone matches the destination",
                ),
            ),
            ShippingError::NoRatesAvailable => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "NO_RATES_AVAILABLE",
                    "No shipping rates available for this zone/method",
                ),
            ),
            ShippingError::ExceedsMaxWeight => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("EXCEEDS_MAX_WEIGHT", "Order exceeds maximum weight"),
            ),
            ShippingError::BelowMinimumAmount => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "BELOW_MINIMUM_AMOUNT",
                    "Order below minimum amount for this shipping method",
                ),
            ),
            ShippingError::MethodOutsideAvailability => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "METHOD_OUTSIDE_AVAILABILITY",
                    "Method not available right now (off-hours / closed day)",
                ),
            ),
            ShippingError::InvalidStatusTransition { from, to } => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "INVALID_STATUS_TRANSITION",
                    format!("Invalid status transition: {} -> {}", from, to),
                ),
            ),
            ShippingError::DriverNotActive => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("DRIVER_NOT_ACTIVE", "Driver is not active"),
            ),
            ShippingError::DriverAssignmentNotAllowed => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "DRIVER_ASSIGNMENT_NOT_ALLOWED",
                    "This shipment method does not allow driver assignment",
                ),
            ),
            ShippingError::ProviderAssignmentNotAllowed => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "PROVIDER_ASSIGNMENT_NOT_ALLOWED",
                    "This shipment method does not allow external provider",
                ),
            ),
            ShippingError::InvalidPickupCode => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("INVALID_PICKUP_CODE", "Pickup code is invalid"),
            ),
            ShippingError::PickupExpired => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("PICKUP_EXPIRED", "Pickup window has expired"),
            ),
            ShippingError::ProviderNotActive(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "PROVIDER_NOT_ACTIVE",
                    format!("Delivery provider not active: {}", id),
                ),
            ),
            ShippingError::ProviderZoneNotCovered => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "PROVIDER_ZONE_NOT_COVERED",
                    "Provider does not cover this zone",
                ),
            ),
            ShippingError::ProviderError(msg) => (
                StatusCode::BAD_GATEWAY,
                ErrorResponse::new("DELIVERY_PROVIDER_ERROR", msg.clone()),
            ),
            ShippingError::InvalidWebhookSignature => (
                StatusCode::UNAUTHORIZED,
                ErrorResponse::new(
                    "INVALID_WEBHOOK_SIGNATURE",
                    "Invalid delivery webhook signature",
                ),
            ),
            ShippingError::InvalidMethodType => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid shipping method type"),
            ),
            ShippingError::InvalidRateType => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid shipping rate type"),
            ),
            ShippingError::InvalidShipmentStatus => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid shipment status"),
            ),
            ShippingError::InvalidVehicleType => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid driver vehicle type"),
            ),
            ShippingError::InvalidDriverStatus => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid driver status"),
            ),
            ShippingError::InvalidProviderType => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid delivery provider type"),
            ),
            ShippingError::InvalidTrackingSource => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid tracking event source"),
            ),
            ShippingError::InvalidAmount => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid amount: must be non-negative"),
            ),
            ShippingError::PaymentConfirmationFailed(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("PAYMENT_CONFIRMATION_FAILED", msg.clone()),
            ),
            ShippingError::AuditError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("AUDIT_ERROR", "Failed to record audit entry"),
            ),
            ShippingError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
            ShippingError::NotImplemented => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("NOT_IMPLEMENTED", "Feature not yet implemented"),
            ),
        };

        AppError::new(status, response)
    }
}

// =============================================================================
// From<CatalogError> Implementation
// =============================================================================

impl From<CatalogError> for AppError {
    fn from(err: CatalogError) -> Self {
        let (status, response) = match &err {
            // 404
            CatalogError::ListingNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("LISTING_NOT_FOUND", format!("Listing not found: {}", id)),
            ),
            CatalogError::ImageNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("IMAGE_NOT_FOUND", format!("Image not found: {}", id)),
            ),
            CatalogError::ReviewNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("REVIEW_NOT_FOUND", format!("Review not found: {}", id)),
            ),
            CatalogError::WishlistNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("WISHLIST_NOT_FOUND", format!("Wishlist not found: {}", id)),
            ),
            CatalogError::WishlistItemNotFound => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("WISHLIST_ITEM_NOT_FOUND", "Wishlist item not found"),
            ),
            CatalogError::CustomerNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("CUSTOMER_NOT_FOUND", format!("Customer not found: {}", id)),
            ),
            CatalogError::ProductNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("PRODUCT_NOT_FOUND", format!("Product not found: {}", id)),
            ),
            CatalogError::StorageProviderNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "STORAGE_PROVIDER_NOT_FOUND",
                    format!("Image storage provider not found: {}", id),
                ),
            ),

            // 409 Conflict
            CatalogError::DuplicateSlug(s) => (
                StatusCode::CONFLICT,
                ErrorResponse::new("DUPLICATE_SLUG", format!("Slug '{}' already in use", s)),
            ),
            CatalogError::DuplicateProductListing(id) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_PRODUCT_LISTING",
                    format!("Listing already exists for product: {}", id),
                ),
            ),
            CatalogError::DuplicateReview => (
                StatusCode::CONFLICT,
                ErrorResponse::new("DUPLICATE_REVIEW", "Customer already reviewed this listing"),
            ),
            CatalogError::DuplicateProviderName(n) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_STORAGE_PROVIDER_NAME",
                    format!("Storage provider name '{}' already exists", n),
                ),
            ),
            CatalogError::ReviewAlreadyApproved => (
                StatusCode::CONFLICT,
                ErrorResponse::new("REVIEW_ALREADY_APPROVED", "Review already approved"),
            ),

            // 400 Bad Request
            CatalogError::InvalidSlug => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error(
                    "Invalid slug: must be lowercase letters/digits/hyphens only",
                ),
            ),
            CatalogError::InvalidRating => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Rating must be between 1 and 5"),
            ),
            CatalogError::ListingUnpublished => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("LISTING_UNPUBLISHED", "Listing is unpublished"),
            ),
            CatalogError::ImageUploadFailed(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("IMAGE_UPLOAD_FAILED", msg.clone()),
            ),
            CatalogError::MaxImagesExceeded(max) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "MAX_IMAGES_EXCEEDED",
                    format!("Maximum images exceeded ({})", max),
                ),
            ),
            CatalogError::UnsupportedContentType(ct) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "UNSUPPORTED_CONTENT_TYPE",
                    format!("Unsupported image content type: {}", ct),
                ),
            ),
            CatalogError::ImageTooLarge(bytes) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "IMAGE_TOO_LARGE",
                    format!("Image too large: {} bytes", bytes),
                ),
            ),
            CatalogError::NoDefaultStorageProvider(id) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new(
                    "NO_DEFAULT_STORAGE_PROVIDER",
                    format!("No default image storage provider for store: {}", id),
                ),
            ),
            CatalogError::InvalidStorageProviderType => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error("Invalid storage provider type"),
            ),
            CatalogError::StorageProviderError(msg) => (
                StatusCode::BAD_GATEWAY,
                ErrorResponse::new("STORAGE_PROVIDER_ERROR", msg.clone()),
            ),

            // 500 Internal
            CatalogError::Io(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("IO_ERROR", msg.clone()),
            ),
            CatalogError::AuditError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("AUDIT_ERROR", "Failed to record audit entry"),
            ),
            CatalogError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
            CatalogError::NotImplemented => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::new("NOT_IMPLEMENTED", "Feature not yet implemented"),
            ),
        };

        AppError::new(status, response)
    }
}

// =============================================================================
// From<AnalyticsError> Implementation
// =============================================================================

impl From<AnalyticsError> for AppError {
    fn from(err: AnalyticsError) -> Self {
        let (status, response) = match &err {
            // 404
            AnalyticsError::DashboardNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "DASHBOARD_NOT_FOUND",
                    format!("Dashboard not found: {}", id),
                ),
            ),
            AnalyticsError::WidgetNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("WIDGET_NOT_FOUND", format!("Widget not found: {}", id)),
            ),
            AnalyticsError::SnapshotNotFound(key) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "KPI_SNAPSHOT_NOT_FOUND",
                    format!("No snapshot for KPI '{}'", key),
                ),
            ),

            // 400
            AnalyticsError::UnknownKpiKey(_)
            | AnalyticsError::UnknownReportKind(_)
            | AnalyticsError::InvalidTimeWindow(_)
            | AnalyticsError::InvalidWidgetKind(_)
            | AnalyticsError::InvalidWidgetConfig(_) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error(err.to_string()),
            ),

            // 500
            AnalyticsError::Database(_)
            | AnalyticsError::Serialization(_)
            | AnalyticsError::Subscriber(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
        };

        AppError::new(status, response)
    }
}

// =============================================================================
// From<AccountingError> Implementation
// =============================================================================

impl From<AccountingError> for AppError {
    fn from(err: AccountingError) -> Self {
        let (status, response) = match &err {
            // 404
            AccountingError::AccountNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("ACCOUNT_NOT_FOUND", format!("Account not found: {}", id)),
            ),
            AccountingError::AccountCodeNotFound(code) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "ACCOUNT_NOT_FOUND",
                    format!("Account code not found: {}", code),
                ),
            ),
            AccountingError::PeriodNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("PERIOD_NOT_FOUND", format!("Period not found: {}", id)),
            ),
            AccountingError::JournalEntryNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "JOURNAL_ENTRY_NOT_FOUND",
                    format!("Journal entry not found: {}", id),
                ),
            ),

            // 409
            AccountingError::DuplicateAccountCode(code) => (
                StatusCode::CONFLICT,
                ErrorResponse::new(
                    "DUPLICATE_ACCOUNT_CODE",
                    format!("Account code already in use: {}", code),
                ),
            ),
            AccountingError::PeriodClosed(_) | AccountingError::PeriodNotOpen => (
                StatusCode::CONFLICT,
                ErrorResponse::new("PERIOD_NOT_OPEN", err.to_string()),
            ),
            AccountingError::InvalidStatusTransition { .. } => (
                StatusCode::CONFLICT,
                ErrorResponse::new("INVALID_STATUS_TRANSITION", err.to_string()),
            ),

            // 400 — validation
            AccountingError::InvalidPeriodRange
            | AccountingError::NotEnoughLines
            | AccountingError::Unbalanced { .. }
            | AccountingError::InvalidLineAmounts
            | AccountingError::NegativeAmount
            | AccountingError::InvalidAccountType(_)
            | AccountingError::InvalidPeriodStatus(_)
            | AccountingError::InvalidJournalEntryStatus(_) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error(err.to_string()),
            ),

            // 500
            AccountingError::Database(_)
            | AccountingError::Serialization(_)
            | AccountingError::Subscriber(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
        };

        AppError::new(status, response)
    }
}

// =============================================================================
// From<DemandPlanningError> Implementation
// =============================================================================

impl From<DemandPlanningError> for AppError {
    fn from(err: DemandPlanningError) -> Self {
        let (status, response) = match &err {
            // 404
            DemandPlanningError::ForecastNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new("FORECAST_NOT_FOUND", format!("Forecast not found: {}", id)),
            ),
            DemandPlanningError::ReorderPolicyNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "REORDER_POLICY_NOT_FOUND",
                    format!("Reorder policy not found: {}", id),
                ),
            ),
            DemandPlanningError::SuggestionNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "SUGGESTION_NOT_FOUND",
                    format!("Replenishment suggestion not found: {}", id),
                ),
            ),

            // 409
            DemandPlanningError::DuplicateReorderPolicy { .. } => (
                StatusCode::CONFLICT,
                ErrorResponse::new("DUPLICATE_REORDER_POLICY", err.to_string()),
            ),
            DemandPlanningError::PolicyVersionConflict(_) => (
                StatusCode::CONFLICT,
                ErrorResponse::new("POLICY_VERSION_CONFLICT", err.to_string()),
            ),
            DemandPlanningError::InvalidSuggestionTransition { .. } => (
                StatusCode::CONFLICT,
                ErrorResponse::new("INVALID_STATUS_TRANSITION", err.to_string()),
            ),

            // 400 — validation
            DemandPlanningError::InvalidPolicyRange
            | DemandPlanningError::InvalidPolicyDays
            | DemandPlanningError::NegativeQuantity
            | DemandPlanningError::InsufficientHistory { .. }
            | DemandPlanningError::InvalidForecastMethod(_)
            | DemandPlanningError::InvalidForecastPeriod(_)
            | DemandPlanningError::InvalidSuggestionStatus(_)
            | DemandPlanningError::InvalidAbcClass(_)
            | DemandPlanningError::DismissReasonRequired
            | DemandPlanningError::ForecastingFailed(_) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error(err.to_string()),
            ),

            // 502 — downstream module (purchasing) refused the request. Surface
            // its message so operators don't have to grep server logs to know
            // why the auto-PO didn't get created.
            DemandPlanningError::Subscriber(_) => (
                StatusCode::BAD_GATEWAY,
                ErrorResponse::new("DOWNSTREAM_ERROR", err.to_string()),
            ),

            // 500
            DemandPlanningError::Database(_) | DemandPlanningError::Serialization(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
        };
        AppError::new(status, response)
    }
}

// =============================================================================
// From<CashManagementError> Implementation
// =============================================================================

impl From<CashManagementError> for AppError {
    fn from(err: CashManagementError) -> Self {
        let (status, response) = match &err {
            // 404
            CashManagementError::BankAccountNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "BANK_ACCOUNT_NOT_FOUND",
                    format!("Bank account not found: {}", id),
                ),
            ),
            CashManagementError::BankTransactionNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "BANK_TRANSACTION_NOT_FOUND",
                    format!("Bank transaction not found: {}", id),
                ),
            ),
            CashManagementError::CashDepositNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "CASH_DEPOSIT_NOT_FOUND",
                    format!("Cash deposit not found: {}", id),
                ),
            ),
            CashManagementError::ReconciliationNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "RECONCILIATION_NOT_FOUND",
                    format!("Reconciliation not found: {}", id),
                ),
            ),
            CashManagementError::ShiftNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse::new(
                    "SHIFT_NOT_FOUND",
                    format!("Cashier shift not found: {}", id),
                ),
            ),

            // 409
            CashManagementError::DuplicateAccountNumber(_) => (
                StatusCode::CONFLICT,
                ErrorResponse::new("DUPLICATE_ACCOUNT_NUMBER", err.to_string()),
            ),
            CashManagementError::AccountVersionConflict(_) => (
                StatusCode::CONFLICT,
                ErrorResponse::new("ACCOUNT_VERSION_CONFLICT", err.to_string()),
            ),
            CashManagementError::ShiftNotClosed(_) => (
                StatusCode::CONFLICT,
                ErrorResponse::new("SHIFT_NOT_CLOSED", err.to_string()),
            ),
            CashManagementError::InvalidDepositTransition { .. }
            | CashManagementError::InvalidReconciliationTransition { .. } => (
                StatusCode::CONFLICT,
                ErrorResponse::new("INVALID_STATUS_TRANSITION", err.to_string()),
            ),
            CashManagementError::TransactionAlreadyLinked => (
                StatusCode::CONFLICT,
                ErrorResponse::new("TRANSACTION_ALREADY_LINKED", err.to_string()),
            ),

            // 400 — validation
            CashManagementError::InvalidReconciliationRange
            | CashManagementError::ReconciliationUnbalanced { .. }
            | CashManagementError::TransactionAccountMismatch
            | CashManagementError::TransactionAmountMismatch { .. }
            | CashManagementError::NegativeAmount
            | CashManagementError::InvalidTransactionType(_)
            | CashManagementError::InvalidAccountType(_)
            | CashManagementError::InvalidDepositStatus(_)
            | CashManagementError::InvalidReconciliationStatus(_) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::validation_error(err.to_string()),
            ),

            // 502 — downstream module error (e.g. sales/cashier_shifts SQL).
            CashManagementError::Subscriber(_) => (
                StatusCode::BAD_GATEWAY,
                ErrorResponse::new("DOWNSTREAM_ERROR", err.to_string()),
            ),

            // 500
            CashManagementError::Database(_) | CashManagementError::Serialization(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorResponse::internal_error(),
            ),
        };
        AppError::new(status, response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // =========================================================================
    // AuthError Mapping Tests
    // =========================================================================

    #[test]
    fn test_auth_error_invalid_credentials_maps_to_401() {
        let app_error: AppError = AuthError::InvalidCredentials.into();
        assert_eq!(app_error.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(app_error.response().error_code, "INVALID_CREDENTIALS");
    }

    #[test]
    fn test_auth_error_account_disabled_maps_to_401() {
        let app_error: AppError = AuthError::AccountDisabled.into();
        assert_eq!(app_error.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(app_error.response().error_code, "ACCOUNT_DISABLED");
    }

    #[test]
    fn test_auth_error_token_expired_maps_to_401() {
        let app_error: AppError = AuthError::TokenExpired.into();
        assert_eq!(app_error.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(app_error.response().error_code, "TOKEN_EXPIRED");
    }

    #[test]
    fn test_auth_error_invalid_token_maps_to_401() {
        let app_error: AppError = AuthError::InvalidToken.into();
        assert_eq!(app_error.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(app_error.response().error_code, "INVALID_TOKEN");
    }

    #[test]
    fn test_auth_error_password_too_short_maps_to_400() {
        let app_error: AppError = AuthError::PasswordTooShort.into();
        assert_eq!(app_error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(app_error.response().error_code, "VALIDATION_ERROR");
    }

    #[test]
    fn test_auth_error_invalid_email_format_maps_to_400() {
        let app_error: AppError = AuthError::InvalidEmailFormat.into();
        assert_eq!(app_error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(app_error.response().error_code, "VALIDATION_ERROR");
    }

    #[test]
    fn test_auth_error_invalid_username_format_maps_to_400() {
        let app_error: AppError = AuthError::InvalidUsernameFormat.into();
        assert_eq!(app_error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(app_error.response().error_code, "VALIDATION_ERROR");
    }

    #[test]
    fn test_auth_error_invalid_name_maps_to_400() {
        let app_error: AppError = AuthError::InvalidName("Name too long".to_string()).into();
        assert_eq!(app_error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(app_error.response().error_code, "VALIDATION_ERROR");
        assert_eq!(app_error.response().message, "Name too long");
    }

    #[test]
    fn test_auth_error_duplicate_email_maps_to_409() {
        let app_error: AppError = AuthError::DuplicateEmail("test@example.com".to_string()).into();
        assert_eq!(app_error.status(), StatusCode::CONFLICT);
        assert_eq!(app_error.response().error_code, "DUPLICATE_EMAIL");
    }

    #[test]
    fn test_auth_error_duplicate_username_maps_to_409() {
        let app_error: AppError = AuthError::DuplicateUsername("john_doe".to_string()).into();
        assert_eq!(app_error.status(), StatusCode::CONFLICT);
        assert_eq!(app_error.response().error_code, "DUPLICATE_USERNAME");
    }

    #[test]
    fn test_auth_error_store_not_found_maps_to_404() {
        let app_error: AppError = AuthError::StoreNotFound.into();
        assert_eq!(app_error.status(), StatusCode::NOT_FOUND);
        assert_eq!(app_error.response().error_code, "STORE_NOT_FOUND");
    }

    #[test]
    fn test_auth_error_internal_maps_to_500() {
        let app_error: AppError = AuthError::Internal("Database error".to_string()).into();
        assert_eq!(app_error.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(app_error.response().error_code, "INTERNAL_ERROR");
        // Internal details should not be exposed
        assert_eq!(app_error.response().message, "Internal error");
    }

    // =========================================================================
    // IdentityError Mapping Tests
    // =========================================================================

    #[test]
    fn test_identity_error_user_not_found_maps_to_404() {
        let app_error: AppError = IdentityError::UserNotFound(Uuid::nil()).into();
        assert_eq!(app_error.status(), StatusCode::NOT_FOUND);
        assert_eq!(app_error.response().error_code, "USER_NOT_FOUND");
    }

    #[test]
    fn test_identity_error_protected_role_maps_to_403() {
        let app_error: AppError = IdentityError::ProtectedRoleCannotBeDeleted.into();
        assert_eq!(app_error.status(), StatusCode::FORBIDDEN);
        assert_eq!(app_error.response().error_code, "PROTECTED_ROLE");
    }

    #[test]
    fn test_identity_error_user_inactive_maps_to_401() {
        let app_error: AppError = IdentityError::UserInactive.into();
        assert_eq!(app_error.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(app_error.response().error_code, "ACCOUNT_DISABLED");
    }

    // =========================================================================
    // InventoryError Mapping Tests
    // =========================================================================

    #[test]
    fn test_inventory_error_product_not_found_maps_to_404() {
        let app_error: AppError = InventoryError::ProductNotFound(Uuid::nil()).into();
        assert_eq!(app_error.status(), StatusCode::NOT_FOUND);
        assert_eq!(app_error.response().error_code, "PRODUCT_NOT_FOUND");
    }

    #[test]
    fn test_inventory_error_variant_not_found_maps_to_404() {
        let app_error: AppError = InventoryError::VariantNotFound(Uuid::nil()).into();
        assert_eq!(app_error.status(), StatusCode::NOT_FOUND);
        assert_eq!(app_error.response().error_code, "VARIANT_NOT_FOUND");
    }

    #[test]
    fn test_inventory_error_stock_not_found_maps_to_404() {
        let app_error: AppError = InventoryError::StockNotFound(Uuid::nil()).into();
        assert_eq!(app_error.status(), StatusCode::NOT_FOUND);
        assert_eq!(app_error.response().error_code, "STOCK_NOT_FOUND");
    }

    #[test]
    fn test_inventory_error_reservation_not_found_maps_to_404() {
        let app_error: AppError = InventoryError::ReservationNotFound(Uuid::nil()).into();
        assert_eq!(app_error.status(), StatusCode::NOT_FOUND);
        assert_eq!(app_error.response().error_code, "RESERVATION_NOT_FOUND");
    }

    #[test]
    fn test_inventory_error_recipe_not_found_maps_to_404() {
        let app_error: AppError = InventoryError::RecipeNotFound(Uuid::nil()).into();
        assert_eq!(app_error.status(), StatusCode::NOT_FOUND);
        assert_eq!(app_error.response().error_code, "RECIPE_NOT_FOUND");
    }

    #[test]
    fn test_inventory_error_adjustment_not_found_maps_to_404() {
        let app_error: AppError = InventoryError::AdjustmentNotFound(Uuid::nil()).into();
        assert_eq!(app_error.status(), StatusCode::NOT_FOUND);
        assert_eq!(app_error.response().error_code, "ADJUSTMENT_NOT_FOUND");
    }

    #[test]
    fn test_inventory_error_transfer_not_found_maps_to_404() {
        let app_error: AppError = InventoryError::TransferNotFound(Uuid::nil()).into();
        assert_eq!(app_error.status(), StatusCode::NOT_FOUND);
        assert_eq!(app_error.response().error_code, "TRANSFER_NOT_FOUND");
    }

    #[test]
    fn test_inventory_error_duplicate_sku_maps_to_409() {
        let app_error: AppError = InventoryError::DuplicateSku("SKU-001".to_string()).into();
        assert_eq!(app_error.status(), StatusCode::CONFLICT);
        assert_eq!(app_error.response().error_code, "DUPLICATE_SKU");
    }

    #[test]
    fn test_inventory_error_duplicate_barcode_maps_to_409() {
        let app_error: AppError = InventoryError::DuplicateBarcode("1234567890".to_string()).into();
        assert_eq!(app_error.status(), StatusCode::CONFLICT);
        assert_eq!(app_error.response().error_code, "DUPLICATE_BARCODE");
    }

    #[test]
    fn test_inventory_error_optimistic_lock_maps_to_409() {
        let app_error: AppError = InventoryError::OptimisticLockError.into();
        assert_eq!(app_error.status(), StatusCode::CONFLICT);
        assert_eq!(app_error.response().error_code, "VERSION_CONFLICT");
    }

    #[test]
    fn test_inventory_error_insufficient_stock_maps_to_400() {
        let app_error: AppError = InventoryError::InsufficientStock.into();
        assert_eq!(app_error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(app_error.response().error_code, "INSUFFICIENT_STOCK");
    }

    #[test]
    fn test_inventory_error_same_store_transfer_maps_to_400() {
        let app_error: AppError = InventoryError::SameStoreTransfer.into();
        assert_eq!(app_error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(app_error.response().error_code, "SAME_STORE_TRANSFER");
    }

    #[test]
    fn test_inventory_error_invalid_status_transition_maps_to_400() {
        let app_error: AppError = InventoryError::InvalidStatusTransition.into();
        assert_eq!(app_error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(app_error.response().error_code, "INVALID_STATUS_TRANSITION");
    }

    #[test]
    fn test_inventory_error_database_maps_to_500() {
        let db_error = sqlx::Error::RowNotFound;
        let app_error: AppError = InventoryError::Database(db_error).into();
        assert_eq!(app_error.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(app_error.response().error_code, "INTERNAL_ERROR");
        // Internal details should not be exposed
        assert_eq!(app_error.response().message, "Internal error");
    }
}
