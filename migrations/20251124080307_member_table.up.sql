CREATE TABLE IF NOT EXISTS t_member (
    f_member_id VARCHAR(255) PRIMARY KEY NOT NULL,
    f_user_id VARCHAR(255) NOT NULL,
    f_team_id VARCHAR(255) NOT NULL,
    
    f_is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    f_is_translator BOOLEAN NOT NULL DEFAULT FALSE,
    f_is_proofreader BOOLEAN NOT NULL DEFAULT FALSE,
    f_is_typesetter BOOLEAN NOT NULL DEFAULT FALSE,
    f_is_principal BOOLEAN NOT NULL DEFAULT FALSE,

    f_created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  
    UNIQUE (f_user_id, f_team_id),
  
    FOREIGN KEY (f_user_id) REFERENCES t_user(f_user_id) ON DELETE CASCADE,
    FOREIGN KEY (f_team_id) REFERENCES t_team(f_team_id) ON DELETE CASCADE
);