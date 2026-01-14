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
    
    // Inventory module permissions
    ("products:create", "Create new products"),
    ("products:read", "View product information"),
    ("products:update", "Update product information"),
    ("products:delete", "Delete products"),
    ("products:list", "List all products"),
    
    ("inventory:read", "View inventory levels"),
    ("inventory:update", "Update inventory levels"),
    ("inventory:transfer", "Transfer inventory between stores"),
    ("inventory:adjust", "Adjust inventory quantities"),
    
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
            "users:create", "users:read", "users:update", "users:delete", "users:list",
            "roles:create", "roles:read", "roles:update", "roles:delete", "roles:list", "roles:assign_permissions",
            "permissions:read", "permissions:list",
            "stores:create", "stores:read", "stores:update", "stores:delete", "stores:list", "stores:assign_users",
            "products:create", "products:read", "products:update", "products:delete", "products:list",
            "inventory:read", "inventory:update", "inventory:transfer", "inventory:adjust",
            "categories:create", "categories:read", "categories:update", "categories:delete", "categories:list",
            "sales:create", "sales:read", "sales:void", "sales:list", "sales:reports",
            "customers:create", "customers:read", "customers:update", "customers:delete", "customers:list",
            "purchases:create", "purchases:read", "purchases:update", "purchases:approve", "purchases:receive", "purchases:list",
            "suppliers:create", "suppliers:read", "suppliers:update", "suppliers:delete", "suppliers:list",
            "reports:sales", "reports:inventory", "reports:purchases", "reports:financial",
            "system:admin", "system:settings", "system:audit_log", "system:backup",
        ],
    ),
    // Store admin - full store access
    (
        "store_admin",
        &[
            "users:create", "users:read", "users:update", "users:list",
            "roles:read", "roles:list",
            "permissions:read", "permissions:list",
            "stores:read", "stores:update", "stores:assign_users",
            "products:create", "products:read", "products:update", "products:delete", "products:list",
            "inventory:read", "inventory:update", "inventory:transfer", "inventory:adjust",
            "categories:create", "categories:read", "categories:update", "categories:delete", "categories:list",
            "sales:create", "sales:read", "sales:void", "sales:list", "sales:reports",
            "customers:create", "customers:read", "customers:update", "customers:delete", "customers:list",
            "purchases:create", "purchases:read", "purchases:update", "purchases:approve", "purchases:receive", "purchases:list",
            "suppliers:create", "suppliers:read", "suppliers:update", "suppliers:delete", "suppliers:list",
            "reports:sales", "reports:inventory", "reports:purchases",
        ],
    ),
    // Store manager
    (
        "store_manager",
        &[
            "users:read", "users:list",
            "roles:read", "roles:list",
            "stores:read",
            "products:read", "products:update", "products:list",
            "inventory:read", "inventory:update", "inventory:adjust",
            "categories:read", "categories:list",
            "sales:create", "sales:read", "sales:void", "sales:list", "sales:reports",
            "customers:create", "customers:read", "customers:update", "customers:list",
            "purchases:create", "purchases:read", "purchases:update", "purchases:list",
            "suppliers:read", "suppliers:list",
            "reports:sales", "reports:inventory",
        ],
    ),
    // Cashier
    (
        "cashier",
        &[
            "products:read", "products:list",
            "inventory:read",
            "categories:read", "categories:list",
            "sales:create", "sales:read", "sales:list",
            "customers:create", "customers:read", "customers:update", "customers:list",
        ],
    ),
    // Inventory clerk
    (
        "inventory_clerk",
        &[
            "products:read", "products:update", "products:list",
            "inventory:read", "inventory:update", "inventory:adjust",
            "categories:read", "categories:list",
            "purchases:read", "purchases:receive", "purchases:list",
            "suppliers:read", "suppliers:list",
            "reports:inventory",
        ],
    ),
    // Purchasing agent
    (
        "purchasing_agent",
        &[
            "products:read", "products:list",
            "inventory:read",
            "purchases:create", "purchases:read", "purchases:update", "purchases:list",
            "suppliers:create", "suppliers:read", "suppliers:update", "suppliers:list",
            "reports:purchases",
        ],
    ),
    // Viewer - read only
    (
        "viewer",
        &[
            "products:read", "products:list",
            "inventory:read",
            "categories:read", "categories:list",
            "sales:read", "sales:list",
            "customers:read", "customers:list",
            "purchases:read", "purchases:list",
            "suppliers:read", "suppliers:list",
        ],
    ),
    // Customer - e-commerce user
    (
        "customer",
        &[
            "products:read", "products:list",
            "categories:read", "categories:list",
            "cart:view", "cart:add", "cart:update", "cart:remove", "cart:checkout",
            "orders:create", "orders:view_own", "orders:cancel_own",
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
