CREATE TABLE IF NOT EXISTS t_projset (
    f_projset_id VARCHAR(255) PRIMARY KEY NOT NULL,
    f_projset_name VARCHAR(100) NOT NULL,
    f_projset_description TEXT,

    f_projset_serial INTEGER NOT NULL,

    f_team_id VARCHAR(255) NOT NULL,

    f_created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (f_projset_name, f_team_id),
    FOREIGN KEY (f_team_id) REFERENCES t_team(f_team_id) ON DELETE CASCADE
);