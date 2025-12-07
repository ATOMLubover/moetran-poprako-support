CREATE TABLE IF NOT EXISTS t_proj_assgin (
    f_proj_assgin_id VARCHAR(255) PRIMARY KEY NOT NULL,
    f_proj_id VARCHAR(255) NOT NULL,
    f_user_id VARCHAR(255) NOT NULL,
    
    f_is_translator BOOLEAN NOT NULL DEFAULT FALSE,
    f_is_proofreader BOOLEAN NOT NULL DEFAULT FALSE,
    f_is_typesetter BOOLEAN NOT NULL DEFAULT FALSE,
    f_is_redrawer BOOLEAN NOT NULL DEFAULT FALSE,
    f_is_principal BOOLEAN NOT NULL DEFAULT FALSE,
    
    f_created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (f_proj_id, f_user_id),
    FOREIGN KEY (f_proj_id) REFERENCES t_proj(f_proj_id) ON DELETE CASCADE
);