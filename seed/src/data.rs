/// Permissions: (code, description)
/// Format: module:action
pub const PERMISSIONS: &[(&str, &str)] = &[
    // Identity module permissions
    ("users:create", "Create new users"),
    ("users:read", "View user information"),
    ("users:update", "Update user information"),
    ("users:delete", "Delete users"),
    ("users:list", "List all users"),
    ("roles:create", "Create new roles"),
    ("roles:read", "View role information"),
    ("roles:update", "Update role information"),
    ("roles:delete", "Delete roles"),
    ("roles:list", "List all roles"),
    ("roles:assign_permissions", "Assign permissions to roles"),
    ("permissions:read", "View permission information"),
    ("permissions:list", "List all permissions"),
    // Store management permissions
    ("stores:create", "Create new stores"),
    ("stores:read", "View store information"),
    ("stores:update", "Update store information"),
    ("stores:delete", "Delete stores"),
    ("stores:list", "List all stores"),
    ("stores:assign_users", "Assign users to stores"),
    // Product management permissions
    ("products:create", "Create new products"),
    ("products:read", "View product information"),
    ("products:update", "Update product information"),
    ("products:delete", "Delete products"),
    ("products:list", "List all products"),
    ("variants:create", "Create product variants"),
    ("variants:read", "View product variants"),
    ("variants:update", "Update product variants"),
    ("variants:delete", "Delete product variants"),
    ("recipes:create", "Create product recipes"),
    ("recipes:read", "View product recipes"),
    ("recipes:update", "Update product recipes"),
    ("recipes:delete", "Delete product recipes"),
    ("recipes:calculate_cost", "Calculate recipe costs"),
    // Inventory module permissions
    ("inventory:read", "View inventory levels"),
    ("inventory:write", "Create inventory stocks"),
    ("inventory:update", "Update inventory levels"),
    ("inventory:transfer", "Transfer inventory between stores"),
    (
        "inventory:adjust",
        "Create and manage inventory adjustments",
    ),
    (
        "inventory:approve_adjustments",
        "Approve inventory adjustments",
    ),
    ("inventory:reserve", "Create inventory reservations"),
    ("reservations:create", "Create inventory reservations"),
    ("reservations:read", "View reservations"),
    ("reservations:confirm", "Confirm reservations"),
    ("reservations:cancel", "Cancel reservations"),
    ("reservations:expire", "Expire pending reservations"),
    ("adjustments:create", "Create inventory adjustments"),
    ("adjustments:read", "View inventory adjustments"),
    ("adjustments:submit", "Submit adjustments for approval"),
    ("adjustments:approve", "Approve inventory adjustments"),
    ("adjustments:reject", "Reject inventory adjustments"),
    (
        "adjustments:apply",
        "Apply approved adjustments to inventory",
    ),
    ("transfers:create", "Create stock transfers"),
    ("transfers:read", "View stock transfers"),
    ("transfers:ship", "Ship stock transfers"),
    ("transfers:receive", "Receive stock transfers"),
    ("transfers:cancel", "Cancel stock transfers"),
    ("categories:create", "Create product categories"),
    ("categories:read", "View product categories"),
    ("categories:update", "Update product categories"),
    ("categories:delete", "Delete product categories"),
    ("categories:list", "List all categories"),
    // Sales module permissions
    ("sales:create", "Create sales transactions"),
    ("sales:update", "Update sales transactions"),
    ("sales:process_payment", "Process payments for sales"),
    ("sales:apply_discount", "Apply discounts to sales"),
    ("sales:read", "View sales transactions"),
    ("sales:void", "Void sales transactions"),
    ("sales:list", "List sales transactions"),
    ("sales:complete", "Complete sales transactions"),
    ("sales:reports", "View sales reports"),
    ("sales:manage_cart", "Manage shopping carts"),
    ("sales:manage_credit_note", "Manage credit notes"),
    ("sales:manage_shift", "Manage cashier shifts"),
    ("sales:read_shift", "View cashier shift details"),
    ("sales:approve_credit_note", "Approve credit notes"),
    ("sales:read_credit_note", "View credit note details"),
    ("sales:create_customer", "Create customers"),
    ("sales:read_customer", "View customer information"),
    ("sales:update_customer", "Update customer information"),
    // Promotions module permissions
    ("promotions:create", "Create promotions"),
    ("promotions:read", "View promotions"),
    ("promotions:update", "Update promotions"),
    ("promotions:apply", "Apply promotions to sales"),
    // E-commerce order workflow permissions
    ("orders:mark_paid", "Mark e-commerce order as paid"),
    ("orders:process", "Start processing an e-commerce order"),
    ("orders:ship", "Ship an e-commerce order"),
    ("orders:deliver", "Mark e-commerce order as delivered"),
    ("orders:cancel", "Cancel an e-commerce order"),
    // Purchasing module permissions
    ("vendors:create", "Create new vendors/suppliers"),
    ("vendors:read", "View vendor information"),
    ("vendors:update", "Update vendor information"),
    ("purchase_orders:create", "Create purchase orders"),
    ("purchase_orders:read", "View purchase order details"),
    ("purchase_orders:update", "Update draft purchase orders"),
    (
        "purchase_orders:submit",
        "Submit purchase orders for approval",
    ),
    (
        "purchase_orders:approve",
        "Approve or reject purchase orders",
    ),
    ("purchase_orders:cancel", "Cancel purchase orders"),
    ("purchase_orders:close", "Close completed purchase orders"),
    ("goods_receipts:create", "Create goods receipts"),
    ("goods_receipts:read", "View goods receipt details"),
    ("goods_receipts:confirm", "Confirm goods receipts"),
    ("goods_receipts:cancel", "Cancel goods receipts"),
    // Fiscal module permissions
    ("invoices:create", "Generate fiscal invoices"),
    ("invoices:read", "View fiscal invoices"),
    ("invoices:void", "Void fiscal invoices"),
    ("invoices:report", "Generate fiscal reports"),
    ("tax_rates:create", "Create tax rates"),
    ("tax_rates:read", "View tax rates"),
    ("tax_rates:update", "Update tax rates"),
    ("tax_rates:delete", "Delete tax rates"),
    // Payments module permissions
    (
        "payment_gateways:read",
        "View payment gateway configuration",
    ),
    (
        "payment_gateways:create",
        "Create payment gateway (super_admin only)",
    ),
    (
        "payment_gateways:update",
        "Update payment gateway (super_admin only)",
    ),
    (
        "payment_gateways:delete",
        "Delete payment gateway (super_admin only)",
    ),
    ("transactions:create", "Process online payment transactions"),
    ("transactions:read", "View payment transactions"),
    ("transactions:refund", "Refund payment transactions"),
    (
        "transactions:confirm",
        "Manually confirm/reject pending payment transactions",
    ),
    (
        "transactions:reconcile",
        "Reconcile payment transactions against gateway",
    ),
    ("payouts:read", "View gateway payouts/settlements"),
    // Shipping module permissions
    (
        "shipping:read",
        "Read shipping configuration (methods/zones/rates)",
    ),
    ("shipping:create", "Create shipping methods/zones/rates"),
    ("shipping:update", "Update shipping methods/zones/rates"),
    ("shipping:delete", "Delete shipping methods/zones/rates"),
    ("shipments:read", "Read shipments"),
    ("shipments:create", "Create shipments"),
    ("shipments:update", "Update shipment status / tracking"),
    (
        "shipments:assign",
        "Assign / reassign drivers / dispatch providers",
    ),
    ("shipments:cancel", "Cancel shipments"),
    ("drivers:read", "Read drivers"),
    ("drivers:create", "Create drivers"),
    ("drivers:update", "Update drivers"),
    ("drivers:delete", "Delete drivers"),
    (
        "delivery_providers:read",
        "Read delivery provider configuration",
    ),
    (
        "delivery_providers:create",
        "Create delivery provider (super_admin only)",
    ),
    (
        "delivery_providers:update",
        "Update delivery provider (super_admin only)",
    ),
    (
        "delivery_providers:delete",
        "Delete delivery provider (super_admin only)",
    ),
    // Catalog module permissions
    (
        "catalog:read",
        "Read product listings, images, approved reviews, wishlist",
    ),
    ("catalog:create", "Create product listings"),
    (
        "catalog:update",
        "Update listings, manage images, publish/unpublish",
    ),
    ("catalog:delete", "Delete listings"),
    ("catalog:review", "Submit product reviews (customer)"),
    ("catalog:moderate", "Approve / delete reviews (manager+)"),
    (
        "image_storage_providers:read",
        "Read image storage configuration",
    ),
    (
        "image_storage_providers:create",
        "Create image storage provider (super_admin only)",
    ),
    (
        "image_storage_providers:update",
        "Update image storage provider (super_admin only)",
    ),
    (
        "image_storage_providers:delete",
        "Delete image storage provider (super_admin only)",
    ),
    // Reports permissions
    ("reports:sales", "Access sales reports"),
    ("reports:inventory", "Access inventory reports"),
    ("reports:inventory_history", "Access history stock reports"),
    (
        "reports:inventory_valuation",
        "Access inventory valuation reports",
    ),
    ("reports:inventory_low_stock", "Access low stock reports"),
    ("reports:purchases", "Access purchasing reports"),
    ("reports:financial", "Access financial reports"),
    (
        "reports:analytics",
        "Read analytics dashboards, KPIs and reports",
    ),
    // Analytics module
    (
        "analytics:write",
        "Create dashboards and add/remove widgets",
    ),
    // Accounting module
    (
        "accounting:read",
        "Read chart of accounts, periods, journal entries, and reports",
    ),
    (
        "accounting:write",
        "Create accounts, open/close periods, post journal entries",
    ),
    // Demand planning module
    ("demand_planning:read_forecast", "Read demand forecasts"),
    ("demand_planning:read_policy", "Read reorder policies"),
    (
        "demand_planning:write_policy",
        "Create or update reorder policies",
    ),
    (
        "demand_planning:read_suggestion",
        "List replenishment suggestions",
    ),
    (
        "demand_planning:approve_suggestion",
        "Approve a replenishment suggestion (creates a Purchase Order)",
    ),
    (
        "demand_planning:dismiss_suggestion",
        "Dismiss a replenishment suggestion",
    ),
    (
        "demand_planning:read_abc",
        "Read ABC classification of products",
    ),
    // Cash management module
    ("cash_management:read_account", "Read bank accounts"),
    (
        "cash_management:write_account",
        "Create or update bank accounts",
    ),
    ("cash_management:read_transaction", "List bank transactions"),
    (
        "cash_management:write_transaction",
        "Record manual bank transactions",
    ),
    ("cash_management:read_deposit", "List cash deposits"),
    (
        "cash_management:write_deposit",
        "Create cash deposits and mark them sent to bank",
    ),
    (
        "cash_management:link_deposit",
        "Link a deposit to its matching bank transaction",
    ),
    (
        "cash_management:read_reconciliation",
        "List bank reconciliations",
    ),
    (
        "cash_management:write_reconciliation",
        "Start a bank reconciliation",
    ),
    (
        "cash_management:close_reconciliation",
        "Close a bank reconciliation (audit boundary)",
    ),
    // Loyalty module
    ("loyalty:read_program", "Read loyalty programs"),
    ("loyalty:write_program", "Create or update loyalty programs"),
    ("loyalty:read_tier", "List program tiers"),
    ("loyalty:write_tier", "Create/update program tiers"),
    ("loyalty:read_member", "Read loyalty members + ledgers"),
    ("loyalty:enroll_member", "Enroll a customer into a program"),
    (
        "loyalty:adjust_points",
        "Manually adjust a member's points (audit boundary)",
    ),
    ("loyalty:read_reward", "List rewards"),
    ("loyalty:write_reward", "Create/update rewards"),
    (
        "loyalty:redeem_reward",
        "Redeem a member's points for a reward",
    ),
    // Booking module
    (
        "booking:read_resource",
        "List/read booking resources (people, equipment, rooms)",
    ),
    (
        "booking:write_resource",
        "Create/update/deactivate booking resources and their calendars",
    ),
    ("booking:read_service", "List/read bookable services"),
    (
        "booking:write_service",
        "Create/update/deactivate bookable services and assign resources",
    ),
    ("booking:read_appointment", "List/read appointments"),
    (
        "booking:write_appointment",
        "Create appointments on behalf of a customer (walk-in, phone)",
    ),
    (
        "booking:transition_appointment",
        "Confirm/start/complete/no-show an appointment",
    ),
    ("booking:cancel_appointment", "Cancel an appointment"),
    ("booking:read_policy", "Read the per-store booking policy"),
    (
        "booking:write_policy",
        "Upsert the per-store booking policy",
    ),
    // Service orders module
    (
        "service_orders:read_asset",
        "List/read assets being serviced",
    ),
    (
        "service_orders:write_asset",
        "Register/update/deactivate assets",
    ),
    ("service_orders:read_order", "List/read service orders"),
    (
        "service_orders:write_order",
        "Create service orders (intake)",
    ),
    (
        "service_orders:transition_order",
        "Diagnose/start-repair/start-testing/mark-ready/deliver an order",
    ),
    ("service_orders:cancel_order", "Cancel a service order"),
    (
        "service_orders:write_item",
        "Add/update/remove labor or parts items",
    ),
    (
        "service_orders:write_diagnostic",
        "Record technician diagnostics",
    ),
    (
        "service_orders:write_quote",
        "Draft a quote from current items",
    ),
    (
        "service_orders:transition_quote",
        "Send/approve/reject a quote",
    ),
    // System permissions
    (
        "system:admin",
        "Full system administration access (super_admin only)",
    ),
    ("system:settings", "Manage system settings"),
    ("system:audit_log", "View audit logs"),
    ("system:backup", "Manage system backups"),
    // E-commerce customer permissions
    ("cart:view", "View shopping cart"),
    ("cart:add", "Add items to shopping cart"),
    ("cart:update", "Update cart item quantities"),
    ("cart:remove", "Remove items from shopping cart"),
    ("cart:checkout", "Proceed to checkout"),
    ("orders:create", "Place orders"),
    ("orders:view_own", "View own order history"),
    ("orders:cancel_own", "Cancel own pending orders"),
    ("profile:view", "View own profile"),
    ("profile:update", "Update own profile information"),
    ("wishlist:manage", "Manage product wishlist"),
    ("reviews:create", "Create product reviews"),
    ("reviews:view", "View product reviews"),
];

/// Roles: (name, description, is_system_protected)
pub const ROLES: &[(&str, &str, bool)] = &[
    (
        "super_admin",
        "Full system access with all permissions",
        true,
    ),
    ("store_admin", "Full access to a specific store", true),
    (
        "store_manager",
        "Manage store operations, inventory, and staff",
        true,
    ),
    (
        "cashier",
        "Process sales and handle customer transactions",
        true,
    ),
    ("inventory_clerk", "Manage inventory and stock levels", true),
    (
        "purchasing_agent",
        "Create and manage purchase orders",
        false,
    ),
    ("viewer", "Read-only access to store data", false),
    (
        "customer",
        "E-commerce customer with access to shopping cart, orders, profile, and product browsing",
        false,
    ),
];

/// Role permissions mapping: (role_name, &[permission_codes])
pub const ROLE_PERMISSIONS: &[(&str, &[&str])] = &[
    // Super admin gets all permissions
    (
        "super_admin",
        &[
            // Identity
            "users:create",
            "users:read",
            "users:update",
            "users:delete",
            "users:list",
            "roles:create",
            "roles:read",
            "roles:update",
            "roles:delete",
            "roles:list",
            "roles:assign_permissions",
            "permissions:read",
            "permissions:list",
            // Stores
            "stores:create",
            "stores:read",
            "stores:update",
            "stores:delete",
            "stores:list",
            "stores:assign_users",
            // Products
            "products:create",
            "products:read",
            "products:update",
            "products:delete",
            "products:list",
            "variants:create",
            "variants:read",
            "variants:update",
            "variants:delete",
            "recipes:create",
            "recipes:read",
            "recipes:update",
            "recipes:delete",
            "recipes:calculate_cost",
            // Inventory
            "inventory:read",
            "inventory:write",
            "inventory:update",
            "inventory:transfer",
            "inventory:adjust",
            "inventory:approve_adjustments",
            "inventory:reserve",
            "reservations:create",
            "reservations:read",
            "reservations:confirm",
            "reservations:cancel",
            "reservations:expire",
            "adjustments:create",
            "adjustments:read",
            "adjustments:submit",
            "adjustments:approve",
            "adjustments:reject",
            "adjustments:apply",
            "transfers:create",
            "transfers:read",
            "transfers:ship",
            "transfers:receive",
            "transfers:cancel",
            "categories:create",
            "categories:read",
            "categories:update",
            "categories:delete",
            "categories:list",
            // Sales
            "sales:create",
            "sales:read",
            "sales:update",
            "sales:void",
            "sales:list",
            "sales:complete",
            "sales:reports",
            "sales:process_payment",
            "sales:apply_discount",
            "sales:create_customer",
            "sales:read_customer",
            "sales:update_customer",
            "sales:manage_shift",
            "sales:read_shift",
            "sales:manage_cart",
            "sales:manage_credit_note",
            "sales:read_credit_note",
            "sales:approve_credit_note",
            // Promotions
            "promotions:create",
            "promotions:read",
            "promotions:update",
            "promotions:apply",
            // E-commerce orders
            "orders:mark_paid",
            "orders:process",
            "orders:ship",
            "orders:deliver",
            "orders:cancel",
            // Purchasing
            "vendors:create",
            "vendors:read",
            "vendors:update",
            "purchase_orders:create",
            "purchase_orders:read",
            "purchase_orders:update",
            "purchase_orders:submit",
            "purchase_orders:approve",
            "purchase_orders:cancel",
            "purchase_orders:close",
            "goods_receipts:create",
            "goods_receipts:read",
            "goods_receipts:confirm",
            "goods_receipts:cancel",
            // Fiscal
            "invoices:create",
            "invoices:read",
            "invoices:void",
            "invoices:report",
            "tax_rates:create",
            "tax_rates:read",
            "tax_rates:update",
            "tax_rates:delete",
            // Payments
            "payment_gateways:read",
            "payment_gateways:create",
            "payment_gateways:update",
            "payment_gateways:delete",
            "transactions:create",
            "transactions:read",
            "transactions:refund",
            "transactions:confirm",
            "transactions:reconcile",
            "payouts:read",
            // Shipping
            "shipping:read",
            "shipping:create",
            "shipping:update",
            "shipping:delete",
            "shipments:read",
            "shipments:create",
            "shipments:update",
            "shipments:assign",
            "shipments:cancel",
            "drivers:read",
            "drivers:create",
            "drivers:update",
            "drivers:delete",
            "delivery_providers:read",
            "delivery_providers:create",
            "delivery_providers:update",
            "delivery_providers:delete",
            // Catalog
            "catalog:read",
            "catalog:create",
            "catalog:update",
            "catalog:delete",
            "catalog:review",
            "catalog:moderate",
            "image_storage_providers:read",
            "image_storage_providers:create",
            "image_storage_providers:update",
            "image_storage_providers:delete",
            // Reports
            "reports:sales",
            "reports:inventory",
            "reports:inventory_history",
            "reports:inventory_valuation",
            "reports:inventory_low_stock",
            "reports:purchases",
            "reports:financial",
            "reports:analytics",
            // Analytics
            "analytics:write",
            // Accounting
            "accounting:read",
            "accounting:write",
            // Demand planning
            "demand_planning:read_forecast",
            "demand_planning:read_policy",
            "demand_planning:write_policy",
            "demand_planning:read_suggestion",
            "demand_planning:approve_suggestion",
            "demand_planning:dismiss_suggestion",
            "demand_planning:read_abc",
            // Cash management
            "cash_management:read_account",
            "cash_management:write_account",
            "cash_management:read_transaction",
            "cash_management:write_transaction",
            "cash_management:read_deposit",
            "cash_management:write_deposit",
            "cash_management:link_deposit",
            "cash_management:read_reconciliation",
            "cash_management:write_reconciliation",
            "cash_management:close_reconciliation",
            // Loyalty
            "loyalty:read_program",
            "loyalty:write_program",
            "loyalty:read_tier",
            "loyalty:write_tier",
            "loyalty:read_member",
            "loyalty:enroll_member",
            "loyalty:adjust_points",
            "loyalty:read_reward",
            "loyalty:write_reward",
            "loyalty:redeem_reward",
            // Booking
            "booking:read_resource",
            "booking:write_resource",
            "booking:read_service",
            "booking:write_service",
            "booking:read_appointment",
            "booking:write_appointment",
            "booking:transition_appointment",
            "booking:cancel_appointment",
            "booking:read_policy",
            "booking:write_policy",
            // Service orders
            "service_orders:read_asset",
            "service_orders:write_asset",
            "service_orders:read_order",
            "service_orders:write_order",
            "service_orders:transition_order",
            "service_orders:cancel_order",
            "service_orders:write_item",
            "service_orders:write_diagnostic",
            "service_orders:write_quote",
            "service_orders:transition_quote",
            // System
            "system:admin",
            "system:settings",
            "system:audit_log",
            "system:backup",
        ],
    ),
    // Store admin - full store access
    (
        "store_admin",
        &[
            // Identity
            "users:create",
            "users:read",
            "users:update",
            "users:list",
            "roles:read",
            "roles:list",
            "permissions:read",
            "permissions:list",
            // Stores
            "stores:read",
            "stores:update",
            "stores:assign_users",
            // Products
            "products:create",
            "products:read",
            "products:update",
            "products:delete",
            "products:list",
            "variants:create",
            "variants:read",
            "variants:update",
            "variants:delete",
            "recipes:create",
            "recipes:read",
            "recipes:update",
            "recipes:delete",
            "recipes:calculate_cost",
            // Inventory
            "inventory:read",
            "inventory:write",
            "inventory:update",
            "inventory:transfer",
            "inventory:adjust",
            "inventory:approve_adjustments",
            "inventory:reserve",
            "reservations:create",
            "reservations:read",
            "reservations:confirm",
            "reservations:cancel",
            "adjustments:create",
            "adjustments:read",
            "adjustments:submit",
            "adjustments:approve",
            "adjustments:reject",
            "adjustments:apply",
            "transfers:create",
            "transfers:read",
            "transfers:ship",
            "transfers:receive",
            "transfers:cancel",
            "categories:create",
            "categories:read",
            "categories:update",
            "categories:delete",
            "categories:list",
            // Sales
            "sales:create",
            "sales:read",
            "sales:update",
            "sales:void",
            "sales:list",
            "sales:complete",
            "sales:reports",
            "sales:process_payment",
            "sales:apply_discount",
            "sales:create_customer",
            "sales:read_customer",
            "sales:update_customer",
            "sales:manage_shift",
            "sales:read_shift",
            "sales:manage_cart",
            "sales:manage_credit_note",
            "sales:read_credit_note",
            "sales:approve_credit_note",
            // Promotions
            "promotions:create",
            "promotions:read",
            "promotions:update",
            "promotions:apply",
            // E-commerce orders
            "orders:mark_paid",
            "orders:process",
            "orders:ship",
            "orders:deliver",
            "orders:cancel",
            // Purchasing
            "vendors:create",
            "vendors:read",
            "vendors:update",
            "purchase_orders:create",
            "purchase_orders:read",
            "purchase_orders:update",
            "purchase_orders:submit",
            "purchase_orders:approve",
            "purchase_orders:cancel",
            "purchase_orders:close",
            "goods_receipts:create",
            "goods_receipts:read",
            "goods_receipts:confirm",
            "goods_receipts:cancel",
            // Fiscal
            "invoices:create",
            "invoices:read",
            "invoices:void",
            "invoices:report",
            "tax_rates:create",
            "tax_rates:read",
            "tax_rates:update",
            "tax_rates:delete",
            // Payments — store_admin can view gateways and operate transactions
            // (gateway create/update/delete is reserved to super_admin at the
            // handler layer)
            "payment_gateways:read",
            "transactions:create",
            "transactions:read",
            "transactions:refund",
            "transactions:confirm",
            "transactions:reconcile",
            "payouts:read",
            // Shipping (store_admin can configure methods/zones/rates and
            // operate shipments; delivery_providers CUD is super_admin only)
            "shipping:read",
            "shipping:create",
            "shipping:update",
            "shipping:delete",
            "shipments:read",
            "shipments:create",
            "shipments:update",
            "shipments:assign",
            "shipments:cancel",
            "drivers:read",
            "drivers:create",
            "drivers:update",
            "drivers:delete",
            "delivery_providers:read",
            // Catalog (store_admin can fully manage listings + moderate reviews;
            // image_storage_providers CUD stays super_admin-only)
            "catalog:read",
            "catalog:create",
            "catalog:update",
            "catalog:delete",
            "catalog:moderate",
            "image_storage_providers:read",
            // Reports
            "reports:sales",
            "reports:inventory",
            "reports:inventory_history",
            "reports:inventory_valuation",
            "reports:inventory_low_stock",
            "reports:purchases",
            "reports:analytics",
            // Analytics — dashboards owned by the store admin
            "analytics:write",
            // Accounting — chart of accounts, periods, JEs, P&L
            "accounting:read",
            "accounting:write",
            // Demand planning — forecasts, reorder policies, replenishment, ABC
            "demand_planning:read_forecast",
            "demand_planning:read_policy",
            "demand_planning:write_policy",
            "demand_planning:read_suggestion",
            "demand_planning:approve_suggestion",
            "demand_planning:dismiss_suggestion",
            "demand_planning:read_abc",
            // Cash management — bank accounts, manual transactions, deposits, reconciliations
            "cash_management:read_account",
            "cash_management:write_account",
            "cash_management:read_transaction",
            "cash_management:write_transaction",
            "cash_management:read_deposit",
            "cash_management:write_deposit",
            "cash_management:link_deposit",
            "cash_management:read_reconciliation",
            "cash_management:write_reconciliation",
            "cash_management:close_reconciliation",
            // Loyalty
            "loyalty:read_program",
            "loyalty:write_program",
            "loyalty:read_tier",
            "loyalty:write_tier",
            "loyalty:read_member",
            "loyalty:enroll_member",
            "loyalty:adjust_points",
            "loyalty:read_reward",
            "loyalty:write_reward",
            "loyalty:redeem_reward",
            // Booking
            "booking:read_resource",
            "booking:write_resource",
            "booking:read_service",
            "booking:write_service",
            "booking:read_appointment",
            "booking:write_appointment",
            "booking:transition_appointment",
            "booking:cancel_appointment",
            "booking:read_policy",
            "booking:write_policy",
            // Service orders
            "service_orders:read_asset",
            "service_orders:write_asset",
            "service_orders:read_order",
            "service_orders:write_order",
            "service_orders:transition_order",
            "service_orders:cancel_order",
            "service_orders:write_item",
            "service_orders:write_diagnostic",
            "service_orders:write_quote",
            "service_orders:transition_quote",
        ],
    ),
    // Store manager
    (
        "store_manager",
        &[
            // Identity
            "users:read",
            "users:list",
            "roles:read",
            "roles:list",
            // Stores
            "stores:read",
            // Products
            "products:read",
            "products:update",
            "products:list",
            "variants:create",
            "variants:read",
            "variants:update",
            "recipes:create",
            "recipes:read",
            "recipes:update",
            "recipes:calculate_cost",
            // Inventory
            "inventory:read",
            "inventory:write",
            "inventory:update",
            "inventory:adjust",
            "inventory:approve_adjustments",
            "inventory:transfer",
            "inventory:reserve",
            "reservations:create",
            "reservations:read",
            "reservations:confirm",
            "reservations:cancel",
            "adjustments:create",
            "adjustments:read",
            "adjustments:submit",
            "adjustments:approve",
            "transfers:create",
            "transfers:read",
            "transfers:ship",
            "transfers:receive",
            "transfers:cancel",
            "categories:read",
            "categories:list",
            // Sales
            "sales:create",
            "sales:read",
            "sales:update",
            "sales:void",
            "sales:list",
            "sales:complete",
            "sales:reports",
            "sales:process_payment",
            "sales:apply_discount",
            "sales:create_customer",
            "sales:read_customer",
            "sales:update_customer",
            "sales:manage_shift",
            "sales:read_shift",
            "sales:manage_cart",
            "sales:manage_credit_note",
            "sales:read_credit_note",
            "sales:approve_credit_note",
            // Promotions
            "promotions:create",
            "promotions:read",
            "promotions:update",
            "promotions:apply",
            // E-commerce orders
            "orders:process",
            "orders:ship",
            "orders:deliver",
            "orders:cancel",
            // Purchasing
            "vendors:read",
            "purchase_orders:create",
            "purchase_orders:read",
            "purchase_orders:update",
            "purchase_orders:submit",
            "goods_receipts:create",
            "goods_receipts:read",
            // Fiscal
            "invoices:create",
            "invoices:read",
            "invoices:void",
            "invoices:report",
            "tax_rates:read",
            // Payments — managers process and confirm transactions but don't
            // have reconcile-by-bank-statement (super_admin / store_admin)
            "payment_gateways:read",
            "transactions:create",
            "transactions:read",
            "transactions:refund",
            "transactions:confirm",
            "payouts:read",
            // Shipping (manager runs the day-to-day; cannot delete config)
            "shipping:read",
            "shipments:read",
            "shipments:create",
            "shipments:update",
            "shipments:assign",
            "shipments:cancel",
            "drivers:read",
            "drivers:create",
            "drivers:update",
            "delivery_providers:read",
            // Catalog (manager updates listings + moderates reviews; no delete)
            "catalog:read",
            "catalog:create",
            "catalog:update",
            "catalog:moderate",
            "image_storage_providers:read",
            // Reports
            "reports:sales",
            "reports:inventory",
            "reports:inventory_history",
            "reports:inventory_valuation",
            "reports:inventory_low_stock",
        ],
    ),
    // Cashier
    (
        "cashier",
        &[
            // Products
            "products:read",
            "products:list",
            "variants:read",
            "recipes:read",
            "categories:read",
            "categories:list",
            // Inventory
            "inventory:read",
            "inventory:reserve",
            "reservations:create",
            "reservations:confirm",
            "reservations:cancel",
            // Sales
            "sales:create",
            "sales:read",
            "sales:update",
            "sales:list",
            "sales:complete",
            "sales:process_payment",
            "sales:apply_discount",
            "sales:create_customer",
            "sales:read_customer",
            "sales:update_customer",
            "sales:manage_shift",
            "sales:read_shift",
            "sales:manage_cart",
            "sales:manage_credit_note",
            "sales:read_credit_note",
            // Promotions (read + apply at checkout)
            "promotions:read",
            "promotions:apply",
            // Fiscal (generate invoices at POS)
            "invoices:create",
            "invoices:read",
            "tax_rates:read",
            // Payments — cashier records charges (manual flow leaves them
            // pending for a manager to confirm)
            "payment_gateways:read",
            "transactions:create",
            "transactions:read",
            // Shipping (cashier may issue StorePickup/OwnDelivery shipments at POS)
            "shipping:read",
            "shipments:read",
            "shipments:create",
            "shipments:update",
            "drivers:read",
            // Catalog (read-only; cashier needs it to look up products by slug)
            "catalog:read",
        ],
    ),
    // Inventory clerk
    (
        "inventory_clerk",
        &[
            // Products
            "products:read",
            "products:update",
            "products:list",
            "variants:read",
            "variants:update",
            "recipes:read",
            "categories:read",
            "categories:list",
            // Inventory
            "inventory:read",
            "inventory:write",
            "inventory:update",
            "inventory:adjust",
            "adjustments:create",
            "adjustments:read",
            "adjustments:submit",
            "transfers:read",
            // Purchasing
            "vendors:read",
            "purchase_orders:read",
            "goods_receipts:create",
            "goods_receipts:read",
            "goods_receipts:confirm",
            // Reports
            "reports:inventory",
            "reports:inventory_history",
            "reports:inventory_low_stock",
        ],
    ),
    // Purchasing agent
    (
        "purchasing_agent",
        &[
            // Products
            "products:read",
            "products:list",
            "variants:read",
            "categories:read",
            "categories:list",
            // Inventory
            "inventory:read",
            // Purchasing
            "vendors:create",
            "vendors:read",
            "vendors:update",
            "purchase_orders:create",
            "purchase_orders:read",
            "purchase_orders:update",
            "purchase_orders:submit",
            "goods_receipts:create",
            "goods_receipts:read",
            // Reports
            "reports:purchases",
        ],
    ),
    // Viewer - read only
    (
        "viewer",
        &[
            // Products
            "products:read",
            "products:list",
            "variants:read",
            "recipes:read",
            "categories:read",
            "categories:list",
            // Inventory
            "inventory:read",
            "adjustments:read",
            "transfers:read",
            // Sales
            "sales:read",
            "sales:list",
            "sales:read_customer",
            "sales:read_shift",
            "sales:read_credit_note",
            // Purchasing
            "vendors:read",
            "purchase_orders:read",
            "goods_receipts:read",
            // Promotions
            "promotions:read",
            // Fiscal
            "invoices:read",
            "tax_rates:read",
            // Payments — read-only
            "payment_gateways:read",
            "transactions:read",
            "payouts:read",
            // Shipping read-only
            "shipping:read",
            "shipments:read",
            "drivers:read",
            "delivery_providers:read",
            // Catalog read-only
            "catalog:read",
        ],
    ),
    // Customer - e-commerce user
    (
        "customer",
        &[
            // Products (read-only for browsing)
            "products:read",
            "products:list",
            "variants:read",
            "categories:read",
            "categories:list",
            // Inventory (only to check availability)
            "inventory:read",
            // Shopping
            "cart:view",
            "cart:add",
            "cart:update",
            "cart:remove",
            "cart:checkout",
            "reservations:create",
            "reservations:cancel", // Only own reservations
            "orders:create",
            "orders:view_own",
            "orders:cancel_own",
            // Profile
            "profile:view",
            "profile:update",
            "wishlist:manage",
            "reviews:create",
            "reviews:view",
            // Catalog browsing + reviews + wishlist
            "catalog:read",
            "catalog:review",
        ],
    ),
];

/// Main store: (name, address, is_ecommerce, is_active)
pub const MAIN_STORE: (&str, &str, bool, bool) = (
    "Tienda Principal",
    "Dirección Principal, Ciudad",
    false,
    true,
);

/// Super admin user: (email, username, first_name, last_name, password)
/// Password: SuperAdmin123!
pub const SUPER_ADMIN_USER: (&str, &str, &str, &str, &str) = (
    "admin@pos-ecommerce.com",
    "super_admin",
    "Super",
    "Admin",
    "SuperAdmin123!",
);

/// Default chart of accounts aligned with a basic Honduran SME plan.
/// Format: (code, name, account_type)
///
/// account_type values match the `chart_of_accounts.account_type` CHECK
/// constraint: asset, liability, equity, revenue, expense.
pub const CHART_OF_ACCOUNTS: &[(&str, &str, &str)] = &[
    // Activos (1xxx)
    ("1010", "Caja", "asset"),
    ("1020", "Bancos", "asset"),
    ("1100", "Cuentas por Cobrar", "asset"),
    ("1200", "Inventario", "asset"),
    ("1500", "Activos Fijos", "asset"),
    // Pasivos (2xxx)
    ("2010", "Cuentas por Pagar", "liability"),
    ("2100", "ISV por Pagar", "liability"),
    ("2200", "Sueldos por Pagar", "liability"),
    // Patrimonio (3xxx)
    ("3010", "Capital Social", "equity"),
    ("3020", "Utilidades Retenidas", "equity"),
    // Ingresos (4xxx)
    ("4010", "Ingresos por Ventas", "revenue"),
    ("4020", "Ingresos por Servicios", "revenue"),
    ("4030", "Otros Ingresos", "revenue"),
    // Gastos (5xxx)
    ("5010", "Costo de Ventas", "expense"),
    ("5020", "Gastos de Operación", "expense"),
    ("5030", "Gastos de Personal", "expense"),
    ("5040", "Gastos Financieros", "expense"),
];

// ============================================================================
// Demand planning seed data
//
// Eight high-rotation grocery items typical of an HN bodega plus one slow
// mover. Each row carries enough fields to seed `products` + `inventory_stock`
// + `reorder_policies` together, and every product gets ~120 days of synthetic
// completed sales so the recompute job has signal on first run. The three
// "near-trigger" SKUs ship with a stock level just below `min + safety` so a
// `pending` suggestion appears immediately.
//
// Fields, in order:
//   sku, name, unit_of_measure, base_price, cost_price,
//   on_hand_qty, min_qty, max_qty, safety_stock_qty, lead_time_days,
//   review_cycle_days, daily_demand_mean, weekly_amplitude
// `daily_demand_mean` and `weekly_amplitude` drive the synthetic sales: each
// day samples `mean + amplitude * sin(2π * weekday / 7)` units, rounded down,
// floor 0.
// ============================================================================

pub struct DemandSeedItem {
    pub sku: &'static str,
    pub name: &'static str,
    pub uom: &'static str,
    pub base_price: f64,
    pub cost_price: f64,
    pub on_hand_qty: f64,
    pub min_qty: f64,
    pub max_qty: f64,
    pub safety_stock_qty: f64,
    pub lead_time_days: i32,
    pub review_cycle_days: i32,
    pub daily_demand_mean: f64,
    pub weekly_amplitude: f64,
}

pub const DEMAND_SEED_ITEMS: &[DemandSeedItem] = &[
    // High-rotation, currently below trigger → suggestion expected.
    DemandSeedItem {
        sku: "DP-ARROZ-1KG",
        name: "Arroz blanco 1 kg",
        uom: "kg",
        base_price: 32.00,
        cost_price: 22.00,
        on_hand_qty: 18.0,
        min_qty: 25.0,
        max_qty: 120.0,
        safety_stock_qty: 10.0,
        lead_time_days: 5,
        review_cycle_days: 7,
        daily_demand_mean: 12.0,
        weekly_amplitude: 4.0,
    },
    DemandSeedItem {
        sku: "DP-FRIJOL-1LB",
        name: "Frijoles rojos 1 lb",
        uom: "lb",
        base_price: 24.00,
        cost_price: 16.00,
        on_hand_qty: 22.0,
        min_qty: 30.0,
        max_qty: 150.0,
        safety_stock_qty: 12.0,
        lead_time_days: 5,
        review_cycle_days: 7,
        daily_demand_mean: 14.0,
        weekly_amplitude: 5.0,
    },
    DemandSeedItem {
        sku: "DP-ACEITE-1L",
        name: "Aceite vegetal 1 L",
        uom: "liter",
        base_price: 58.00,
        cost_price: 40.00,
        on_hand_qty: 9.0,
        min_qty: 15.0,
        max_qty: 80.0,
        safety_stock_qty: 6.0,
        lead_time_days: 7,
        review_cycle_days: 14,
        daily_demand_mean: 6.0,
        weekly_amplitude: 1.5,
    },
    // Stable rotation, well above trigger → no suggestion.
    DemandSeedItem {
        sku: "DP-LECHE-1L",
        name: "Leche entera UHT 1 L",
        uom: "liter",
        base_price: 36.00,
        cost_price: 26.00,
        on_hand_qty: 95.0,
        min_qty: 30.0,
        max_qty: 140.0,
        safety_stock_qty: 8.0,
        lead_time_days: 3,
        review_cycle_days: 7,
        daily_demand_mean: 18.0,
        weekly_amplitude: 6.0,
    },
    DemandSeedItem {
        sku: "DP-PAPEL-4PK",
        name: "Papel higiénico 4 rollos",
        uom: "unit",
        base_price: 78.00,
        cost_price: 55.00,
        on_hand_qty: 60.0,
        min_qty: 20.0,
        max_qty: 90.0,
        safety_stock_qty: 5.0,
        lead_time_days: 7,
        review_cycle_days: 14,
        daily_demand_mean: 5.0,
        weekly_amplitude: 1.0,
    },
    DemandSeedItem {
        sku: "DP-JABON-200G",
        name: "Jabón de baño 200 g",
        uom: "unit",
        base_price: 22.00,
        cost_price: 14.00,
        on_hand_qty: 50.0,
        min_qty: 25.0,
        max_qty: 110.0,
        safety_stock_qty: 8.0,
        lead_time_days: 7,
        review_cycle_days: 14,
        daily_demand_mean: 8.0,
        weekly_amplitude: 2.0,
    },
    // Strong weekend spike (high amplitude) → Holt-Winters value-add.
    DemandSeedItem {
        sku: "DP-COCA-2L",
        name: "Coca-Cola 2 L",
        uom: "liter",
        base_price: 45.00,
        cost_price: 32.00,
        on_hand_qty: 70.0,
        min_qty: 25.0,
        max_qty: 130.0,
        safety_stock_qty: 10.0,
        lead_time_days: 4,
        review_cycle_days: 7,
        daily_demand_mean: 14.0,
        weekly_amplitude: 9.0,
    },
    // Slow mover → likely class C.
    DemandSeedItem {
        sku: "DP-SAL-1KG",
        name: "Sal refinada 1 kg",
        uom: "kg",
        base_price: 12.00,
        cost_price: 7.00,
        on_hand_qty: 35.0,
        min_qty: 8.0,
        max_qty: 40.0,
        safety_stock_qty: 2.0,
        lead_time_days: 14,
        review_cycle_days: 30,
        daily_demand_mean: 1.2,
        weekly_amplitude: 0.4,
    },
    // Currently below trigger, slow mover → suggestion expected (small).
    DemandSeedItem {
        sku: "DP-CAFE-200G",
        name: "Café molido 200 g",
        uom: "unit",
        base_price: 88.00,
        cost_price: 62.00,
        on_hand_qty: 6.0,
        min_qty: 12.0,
        max_qty: 50.0,
        safety_stock_qty: 4.0,
        lead_time_days: 10,
        review_cycle_days: 14,
        daily_demand_mean: 2.5,
        weekly_amplitude: 0.5,
    },
];

/// Vendor used for every demo product so suggestion → PO has a target vendor.
/// (code, name, legal_name, tax_id, payment_terms_days)
pub const DEMAND_DEFAULT_VENDOR: (&str, &str, &str, &str, i32) = (
    "DP-VEN-001",
    "Distribuidora La Económica",
    "Distribuidora La Económica S. de R.L.",
    "08019999000123",
    15,
);

/// Category every demo product is filed under: (slug, name).
pub const DEMAND_DEFAULT_CATEGORY: (&str, &str) = ("abarrotes", "Abarrotes");

// ============================================================================
// Cash management seed data
//
// Default bank account so the API has somewhere to record deposits and
// transactions on first run. Numbers are placeholders — replace before going
// to production. (bank_name, account_number, account_type, currency,
// opening_balance)
// ============================================================================

pub const DEMO_BANK_ACCOUNT: (&str, &str, &str, &str, f64) = (
    "BAC Honduras",
    "10-200-300456",
    "checking",
    "HNL",
    25_000.00,
);

// ============================================================================
// Loyalty seed data
// ============================================================================

/// Default loyalty program for the main store: (name, description,
/// points_per_currency_unit, expiration_days). 1 pt per L 1 spent; points
/// expire after 365 days.
pub const DEMO_LOYALTY_PROGRAM: (&str, &str, f64, i32) = (
    "Cliente Frecuente",
    "Programa de fidelidad para clientes recurrentes.",
    1.0,
    365,
);

/// Tiers for the demo program: (name, threshold_points, benefits_json,
/// sort_order). Bronze→Silver→Gold ladder.
pub const DEMO_LOYALTY_TIERS: &[(&str, i64, &str, i32)] = &[
    ("Bronze", 0, r#"{"discount_percent":0}"#, 0),
    ("Silver", 500, r#"{"discount_percent":5}"#, 1),
    ("Gold", 2000, r#"{"discount_percent":10}"#, 2),
];

/// Rewards catalog seed entry. All Honduran-style amounts in HNL.
pub struct DemoLoyaltyReward {
    pub name: &'static str,
    pub description: &'static str,
    pub cost_points: i64,
    pub reward_type: &'static str,
    pub reward_value: f64,
    pub max_per_member: Option<i32>,
}

pub const DEMO_LOYALTY_REWARDS: &[DemoLoyaltyReward] = &[
    DemoLoyaltyReward {
        name: "Descuento L 50",
        description: "Cupón de L 50 de descuento en tu próxima compra.",
        cost_points: 500,
        reward_type: "discount_amount",
        reward_value: 50.0,
        max_per_member: None,
    },
    DemoLoyaltyReward {
        name: "10% en tu próxima visita",
        description: "Descuento del 10% sobre el subtotal de la próxima venta.",
        cost_points: 1_000,
        reward_type: "discount_percent",
        reward_value: 10.0,
        max_per_member: Some(2),
    },
];

// =============================================================================
// Booking demo data — gives booking endpoints a complete graph on first boot.
// =============================================================================

/// Resources scaffolded for the demo store: salon-style "stylist" people plus
/// one shared room. Color is a Tailwind-ish hex for UI calendars.
/// Format: (resource_type, name, color)
pub const DEMO_BOOKING_RESOURCES: &[(&str, &str, &str)] = &[
    ("person", "Ana — Estilista", "#f97316"),
    ("person", "Luis — Barbero", "#0ea5e9"),
    ("room", "Cabina 1", "#10b981"),
];

/// Weekly availability applied to every demo resource: Mon–Fri 09–17, Sat 09–13.
/// Format: (day_of_week, start_HH:MM, end_HH:MM). day_of_week: 0=Sun..6=Sat.
pub const DEMO_BOOKING_CALENDAR: &[(i16, &str, &str)] = &[
    (1, "09:00", "17:00"),
    (2, "09:00", "17:00"),
    (3, "09:00", "17:00"),
    (4, "09:00", "17:00"),
    (5, "09:00", "17:00"),
    (6, "09:00", "13:00"),
];

/// A demo bookable service. The `eligible_resource_names` field references
/// names from `DEMO_BOOKING_RESOURCES` so the M2M can be resolved by lookup.
pub struct DemoBookingService {
    pub name: &'static str,
    pub description: &'static str,
    pub duration_minutes: i32,
    pub price: f64,
    pub buffer_minutes_before: i32,
    pub buffer_minutes_after: i32,
    pub requires_deposit: bool,
    pub deposit_amount: Option<f64>,
    pub eligible_resource_names: &'static [&'static str],
}

pub const DEMO_BOOKING_SERVICES: &[DemoBookingService] = &[
    DemoBookingService {
        name: "Corte de cabello",
        description: "Corte clásico, lavado incluido.",
        duration_minutes: 30,
        price: 250.0,
        buffer_minutes_before: 0,
        buffer_minutes_after: 5,
        requires_deposit: false,
        deposit_amount: None,
        eligible_resource_names: &["Ana — Estilista", "Luis — Barbero"],
    },
    DemoBookingService {
        name: "Color y mechas",
        description: "Aplicación de color completo o mechas.",
        duration_minutes: 90,
        price: 1_200.0,
        buffer_minutes_before: 5,
        buffer_minutes_after: 10,
        requires_deposit: true,
        deposit_amount: Some(300.0),
        eligible_resource_names: &["Ana — Estilista"],
    },
    DemoBookingService {
        name: "Afeitado clásico",
        description: "Toalla caliente, navaja, masaje facial.",
        duration_minutes: 45,
        price: 350.0,
        buffer_minutes_before: 0,
        buffer_minutes_after: 5,
        requires_deposit: false,
        deposit_amount: None,
        eligible_resource_names: &["Luis — Barbero"],
    },
];

/// Per-store booking policy.
/// Format: (requires_deposit, deposit_percentage, cancellation_window_hours,
///          no_show_fee_amount, default_buffer_minutes, advance_booking_days_max).
pub const DEMO_BOOKING_POLICY: (bool, Option<f64>, i32, Option<f64>, i32, i32) =
    (false, None, 24, None, 5, 60);

// =============================================================================
// Service orders demo data — gives the workshop endpoints a complete graph on
// first boot. One customer, two assets (a car + a laptop) and one in-progress
// service order on the car (status = Diagnosis, with a diagnostic + 2 items
// already added so the staff can immediately draft a quote).
// =============================================================================

/// Demo customer. Format: (code, first_name, last_name, email, phone,
///                          customer_type, tax_id).
pub const DEMO_SERVICE_CUSTOMER: (&str, &str, &str, &str, &str, &str, &str) = (
    "DEMO-MARIO-001",
    "Mario",
    "Lopez",
    "mario.lopez@example.com",
    "+50432109876",
    "individual",
    "0801-1985-12345",
);

/// Demo asset registered for the customer.
/// Fields: (asset_type, brand, model, identifier, year, color, description).
pub type DemoServiceAsset = (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    Option<i32>,
    Option<&'static str>,
    Option<&'static str>,
);

pub const DEMO_SERVICE_ASSETS: &[DemoServiceAsset] = &[
    (
        "vehicle",
        "Toyota",
        "Corolla",
        "PEC-1234",
        Some(2018),
        Some("Gris"),
        Some("Sedan, transmision automatica"),
    ),
    (
        "electronic",
        "Apple",
        "MacBook Pro 14",
        "C02XK1XYZQ6L",
        Some(2022),
        Some("Space Gray"),
        Some("Pantalla con franja vertical intermitente"),
    ),
];

/// Demo service order intake. Mounted on the first asset (the car).
/// Format: (priority, intake_notes, customer_phone). status starts at 'intake'
/// and the seeder transitions it forward.
pub const DEMO_SERVICE_ORDER_INTAKE: (&str, &str) = (
    "high",
    "Cliente reporta ruido al frenar y vibracion en volante a alta velocidad.",
);

/// Demo diagnostic recorded against the order.
/// Format: (findings, recommended_actions, severity).
pub const DEMO_SERVICE_DIAGNOSTIC: (&str, &str, &str) = (
    "Pastillas de freno desgastadas (1mm). Disco delantero derecho con alabeo de 0.08mm.",
    "Cambiar pastillas delanteras y rectificar disco; balanceo de ruedas delanteras.",
    "high",
);

/// Demo line items (labor + parts) on the order.
/// Format: (item_type, description, quantity, unit_price, tax_rate).
pub const DEMO_SERVICE_ITEMS: &[(&str, &str, f64, f64, f64)] = &[
    (
        "labor",
        "Mano de obra: cambio de pastillas + rectificado",
        2.0,
        350.0,
        0.15,
    ),
    (
        "part",
        "Juego de pastillas delanteras OEM",
        1.0,
        1_200.0,
        0.15,
    ),
    ("labor", "Balanceo de ruedas delanteras", 1.0, 200.0, 0.15),
];
