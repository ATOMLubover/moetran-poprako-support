CREATE TABLE IF NOT EXISTS t_proj (
    f_proj_id VARCHAR(255) PRIMARY KEY NOT NULL,
    f_proj_name VARCHAR(100) NOT NULL,
  
    f_projset_id VARCHAR(255) NOT NULL,
    f_projset_serial INTEGER NOT NULL,
    f_projset_index INTEGER NOT NULL,

    f_created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (f_proj_name, f_projset_id),
    FOREIGN KEY (f_projset_id) REFERENCES t_projset(f_projset_id) ON DELETE CASCADE
);