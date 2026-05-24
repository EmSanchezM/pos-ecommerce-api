CREATE TABLE backoffice_user_roles (
    backoffice_user_id UUID NOT NULL REFERENCES backoffice_users (id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES backoffice_roles (id) ON DELETE CASCADE,
    PRIMARY KEY (backoffice_user_id, role_id)
);

CREATE TABLE backoffice_role_permissions (
    role_id UUID NOT NULL REFERENCES backoffice_roles (id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES backoffice_permissions (id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);
