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
    ("inventory:adjust", "Create and manage inventory adjustments"),
    ("inventory:approve_adjustments", "Approve inventory adjustments"),
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
    ("adjustments:apply", "Apply approved adjustments to inventory"),
    
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
    ("sales:read", "View sales transactions"),
    ("sales:void", "Void sales transactions"),
    ("sales:list", "List sales transactions"),
    ("sales:reports", "View sales reports"),
    
    ("customers:create", "Create customers"),
    ("customers:read", "View customer information"),
    ("customers:update", "Update customer information"),
    ("customers:delete", "Delete customers"),
    ("customers:list", "List all customers"),
    
    // Purchasing module permissions
    ("purchases:create", "Create purchase orders"),
    ("purchases:read", "View purchase orders"),
    ("purchases:update", "Update purchase orders"),
    ("purchases:approve", "Approve purchase orders"),
    ("purchases:receive", "Receive purchase orders"),
    ("purchases:list", "List purchase orders"),
    
    ("suppliers:create", "Create suppliers"),
    ("suppliers:read", "View supplier information"),
    ("suppliers:update", "Update supplier information"),
    ("suppliers:delete", "Delete suppliers"),
    ("suppliers:list", "List all suppliers"),
    
    // Reports permissions
    ("reports:sales", "Access sales reports"),
    ("reports:inventory", "Access inventory reports"),
    ("reports:inventory_history", "Access history stock reports"),
    ("reports:inventory_valuation", "Access inventory valuation reports"),
    ("reports:inventory_low_stock", "Access low stock reports"),
    ("reports:purchases", "Access purchasing reports"),
    ("reports:financial", "Access financial reports"),
    
    // System permissions
    ("system:admin", "Full system administration access (super_admin only)"),
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
    (
        "store_admin",
        "Full access to a specific store",
        true,
    ),
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
    (
        "inventory_clerk",
        "Manage inventory and stock levels",
        true,
    ),
    (
        "purchasing_agent",
        "Create and manage purchase orders",
        false,
    ),
    (
        "viewer",
        "Read-only access to store data",
        false,
    ),
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
            "users:create", "users:read", "users:update", "users:delete", "users:list",
            "roles:create", "roles:read", "roles:update", "roles:delete", "roles:list", "roles:assign_permissions",
            "permissions:read", "permissions:list",
            // Stores
            "stores:create", "stores:read", "stores:update", "stores:delete", "stores:list", "stores:assign_users",
            // Products
            "products:create", "products:read", "products:update", "products:delete", "products:list",
            "variants:create", "variants:read", "variants:update", "variants:delete",
            "recipes:create", "recipes:read", "recipes:update", "recipes:delete", "recipes:calculate_cost",
            // Inventory
            "inventory:read", "inventory:write", "inventory:update", "inventory:transfer", "inventory:adjust", "inventory:approve_adjustments", "inventory:reserve",
            "reservations:create", "reservations:read", "reservations:confirm", "reservations:cancel", "reservations:expire",
            "adjustments:create", "adjustments:read", "adjustments:submit", "adjustments:approve", "adjustments:reject", "adjustments:apply",
            "transfers:create", "transfers:read", "transfers:ship", "transfers:receive", "transfers:cancel",
            "categories:create", "categories:read", "categories:update", "categories:delete", "categories:list",
            // Sales
            "sales:create", "sales:read", "sales:void", "sales:list", "sales:reports",
            "customers:create", "customers:read", "customers:update", "customers:delete", "customers:list",
            // Purchasing
            "purchases:create", "purchases:read", "purchases:update", "purchases:approve", "purchases:receive", "purchases:list",
            "suppliers:create", "suppliers:read", "suppliers:update", "suppliers:delete", "suppliers:list",
            // Reports
            "reports:sales", "reports:inventory", "reports:inventory_history", "reports:inventory_valuation", "reports:inventory_low_stock",
            "reports:purchases", "reports:financial",
            // System
            "system:admin", "system:settings", "system:audit_log", "system:backup",
        ],
    ),
    
    // Store admin - full store access
    (
        "store_admin",
        &[
            // Identity
            "users:create", "users:read", "users:update", "users:list",
            "roles:read", "roles:list",
            "permissions:read", "permissions:list",
            // Stores
            "stores:read", "stores:update", "stores:assign_users",
            // Products
            "products:create", "products:read", "products:update", "products:delete", "products:list",
            "variants:create", "variants:read", "variants:update", "variants:delete",
            "recipes:create", "recipes:read", "recipes:update", "recipes:delete", "recipes:calculate_cost",
            // Inventory
            "inventory:read", "inventory:write", "inventory:update", "inventory:transfer", "inventory:adjust", "inventory:approve_adjustments", "inventory:reserve",
            "reservations:create", "reservations:read", "reservations:confirm", "reservations:cancel",
            "adjustments:create", "adjustments:read", "adjustments:submit", "adjustments:approve", "adjustments:reject", "adjustments:apply",
            "transfers:create", "transfers:read", "transfers:ship", "transfers:receive", "transfers:cancel",
            "categories:create", "categories:read", "categories:update", "categories:delete", "categories:list",
            // Sales
            "sales:create", "sales:read", "sales:void", "sales:list", "sales:reports",
            "customers:create", "customers:read", "customers:update", "customers:delete", "customers:list",
            // Purchasing
            "purchases:create", "purchases:read", "purchases:update", "purchases:approve", "purchases:receive", "purchases:list",
            "suppliers:create", "suppliers:read", "suppliers:update", "suppliers:delete", "suppliers:list",
            // Reports
            "reports:sales", "reports:inventory", "reports:inventory_history", "reports:inventory_valuation", "reports:inventory_low_stock",
            "reports:purchases",
        ],
    ),

    // Store manager
    (
        "store_manager",
        &[
            // Identity
            "users:read", "users:list",
            "roles:read", "roles:list",
            // Stores
            "stores:read",
            // Products
            "products:read", "products:update", "products:list",
            "variants:create", "variants:read", "variants:update",
            "recipes:create", "recipes:read", "recipes:update", "recipes:calculate_cost",
            // Inventory
            "inventory:read", "inventory:write", "inventory:update", "inventory:adjust", "inventory:approve_adjustments", "inventory:transfer", "inventory:reserve",
            "reservations:create", "reservations:read", "reservations:confirm", "reservations:cancel",
            "adjustments:create", "adjustments:read", "adjustments:submit", "adjustments:approve",
            "transfers:create", "transfers:read", "transfers:ship", "transfers:receive", "transfers:cancel",
            "categories:read", "categories:list",
            // Sales
            "sales:create", "sales:read", "sales:void", "sales:list", "sales:reports",
            "customers:create", "customers:read", "customers:update", "customers:list",
            // Purchasing
            "purchases:create", "purchases:read", "purchases:update", "purchases:list",
            "suppliers:read", "suppliers:list",
            // Reports
            "reports:sales", "reports:inventory", "reports:inventory_history", "reports:inventory_valuation", "reports:inventory_low_stock",
        ],
    ),

    // Cashier
    (
        "cashier",
        &[
            // Products
            "products:read", "products:list",
            "variants:read",
            "recipes:read",
            "categories:read", "categories:list",
            // Inventory
            "inventory:read", "inventory:reserve",
            "reservations:create", "reservations:confirm", "reservations:cancel",
            // Sales
            "sales:create", "sales:read", "sales:list",
            "customers:create", "customers:read", "customers:update", "customers:list",
        ],
    ),
    
    // Inventory clerk
    (
        "inventory_clerk",
        &[
            // Products
            "products:read", "products:update", "products:list",
            "variants:read", "variants:update",
            "recipes:read",
            "categories:read", "categories:list",
            // Inventory
            "inventory:read", "inventory:write", "inventory:update", "inventory:adjust",
            "adjustments:create", "adjustments:read", "adjustments:submit",
            "transfers:read",
            // Purchasing
            "purchases:read", "purchases:receive", "purchases:list",
            "suppliers:read", "suppliers:list",
            // Reports
            "reports:inventory", "reports:inventory_history", "reports:inventory_low_stock",
        ],
    ),

    // Purchasing agent
    (
        "purchasing_agent",
        &[
            // Products
            "products:read", "products:list",
            "variants:read",
            "categories:read", "categories:list",
            // Inventory
            "inventory:read",
            // Purchasing
            "purchases:create", "purchases:read", "purchases:update", "purchases:list",
            "suppliers:create", "suppliers:read", "suppliers:update", "suppliers:list",
            // Reports
            "reports:purchases",
        ],
    ),
    
    // Viewer - read only
    (
        "viewer",
        &[
            // Products
            "products:read", "products:list",
            "variants:read",
            "recipes:read",
            "categories:read", "categories:list",
            // Inventory
            "inventory:read",
            "adjustments:read",
            "transfers:read",
            // Sales
            "sales:read", "sales:list",
            "customers:read", "customers:list",
            // Purchasing
            "purchases:read", "purchases:list",
            "suppliers:read", "suppliers:list",
        ],
    ),
    
    // Customer - e-commerce user
    (
        "customer",
        &[
            // Products (read-only for browsing)
            "products:read", "products:list",
            "variants:read",
            "categories:read", "categories:list",
            // Inventory (only to check availability)
            "inventory:read",
            // Shopping
            "cart:view", "cart:add", "cart:update", "cart:remove", "cart:checkout",
            "reservations:create", "reservations:cancel", // Only own reservations
            "orders:create", "orders:view_own", "orders:cancel_own",
            // Profile
            "profile:view", "profile:update",
            "wishlist:manage",
            "reviews:create", "reviews:view",
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