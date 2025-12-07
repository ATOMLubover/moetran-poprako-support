-- Mock data: projsets, projects, assignments, and status updates
-- Usage: psql -d yourdb -f docs/mock_data.sql

BEGIN;

CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- 1) Create some projsets for team `64aac7d91b30e3645d7f96c6`
INSERT INTO t_projset (f_projset_id, f_team_id, f_projset_name, f_projset_description, f_projset_serial)
VALUES
	('ps-001-64aac7d9','64aac7d91b30e3645d7f96c6','Main Translations','Primary translation queue',1),
	('ps-002-64aac7d9','64aac7d91b30e3645d7f96c6','Short Stories','One-shots and short pieces',2),
	('ps-003-64aac7d9','64aac7d91b30e3645d7f96c6','Side Projects','Experimental/side projects',3),
	('ps-004-64aac7d9','64aac7d91b30e3645d7f96c6','Legacy','Older/backlog projects',4),
	('ps-005-64aac7d9','64aac7d91b30e3645d7f96c6','Hotfixes','Urgent small tasks',5)
ON CONFLICT (f_projset_id) DO NOTHING;

-- 2) Create ~20 projects across the projsets
INSERT INTO t_proj (f_proj_id, f_proj_name, f_projset_id, f_projset_serial, f_projset_index, f_translating_status, f_proofreading_status, f_typesetting_status, f_reviewing_status, f_is_published)
VALUES
	('proj-001','Rising Dawn - Ch.1','ps-001-64aac7d9',1,1,1,0,0,0,false),
	('proj-002','Rising Dawn - Ch.2','ps-001-64aac7d9',1,2,2,0,0,0,false),
	('proj-003','Rising Dawn - Ch.3','ps-001-64aac7d9',1,3,3,1,0,0,false),
	('proj-004','Short: Evening Tea','ps-002-64aac7d9',2,1,0,0,0,0,false),
	('proj-005','Short: Autumn Walk','ps-002-64aac7d9',2,2,1,0,0,0,false),
	('proj-006','Side: Lab Notes','ps-003-64aac7d9',3,1,0,0,0,0,false),
	('proj-007','Side: Prototype','ps-003-64aac7d9',3,2,1,0,0,0,false),
	('proj-008','Legacy: Old Arc 1','ps-004-64aac7d9',4,1,0,0,0,0,false),
	('proj-009','Legacy: Old Arc 2','ps-004-64aac7d9',4,2,2,1,0,0,false),
	('proj-010','Hotfix: Translation typo #12','ps-005-64aac7d9',5,1,3,2,1,1,true),
	('proj-011','Rising Dawn - Ch.4','ps-001-64aac7d9',1,4,0,0,0,0,false),
	('proj-012','Rising Dawn - Extra','ps-001-64aac7d9',1,5,1,0,0,0,false),
	('proj-013','Short: Midnight Walk','ps-002-64aac7d9',2,3,0,0,0,0,false),
	('proj-014','Side: Design Doc','ps-003-64aac7d9',3,3,2,1,0,0,false),
	('proj-015','Legacy: Side Story','ps-004-64aac7d9',4,3,1,0,0,0,false),
	('proj-016','Hotfix: Image crop','ps-005-64aac7d9',5,2,0,0,0,0,false),
	('proj-017','Rising Dawn - Ch.5','ps-001-64aac7d9',1,6,0,0,0,0,false),
	('proj-018','One-shot: Snow','ps-002-64aac7d9',2,4,2,1,1,0,false),
	('proj-019','Side: Internal Test','ps-003-64aac7d9',3,4,0,0,0,0,false),
	('proj-020','Legacy: Archive Cleanup','ps-004-64aac7d9',4,4,0,0,0,0,false)
ON CONFLICT (f_proj_id) DO NOTHING;

-- 3) Assign some users to projects (translator / proofreader / typesetter)
-- Choose a handful of user ids that exist in the sample `insert_active_members.sql`
INSERT INTO t_proj_assgin (f_proj_assgin_id, f_proj_id, f_user_id, f_is_translator, f_is_proofreader, f_is_typesetter, f_is_redrawer, f_is_principal)
VALUES
	(gen_random_uuid()::text,'proj-001','63d9d0f78cdda1190a374cf8',TRUE,FALSE,FALSE,FALSE,TRUE),
	(gen_random_uuid()::text,'proj-001','63f219058cdda1190a378930',FALSE,TRUE,FALSE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-002','642ef01dde202dbcc5049016',TRUE,FALSE,FALSE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-002','64300235de202dbcc5049531',FALSE,TRUE,FALSE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-003','64ec5375f3e8846eb1b9a1e7',TRUE,TRUE,FALSE,FALSE,TRUE),
	(gen_random_uuid()::text,'proj-003','642ff17dde202dbcc504950f',FALSE,FALSE,TRUE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-004','643003f28cdda1190a3863e5',TRUE,FALSE,FALSE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-005','643001b18cdda1190a3863d7',FALSE,TRUE,FALSE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-006','642ffd98a73514f6e95317a2',TRUE,FALSE,FALSE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-007','643156ede97599997c289687',TRUE,FALSE,TRUE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-008','64315870a73514f6e9531ddb',FALSE,TRUE,FALSE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-009','64329163a73514f6e9532246',TRUE,TRUE,TRUE,FALSE,TRUE),
	(gen_random_uuid()::text,'proj-010','64f887797b9fe117f72c7158',FALSE,TRUE,TRUE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-011','64fe9209f72990522b18d680',TRUE,FALSE,FALSE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-012','65084d8cf72990522b193bdd',TRUE,FALSE,FALSE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-013','651a4f8687a85db2b6e7b720',FALSE,TRUE,FALSE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-014','652bc262939e86c710d1ea6c',TRUE,FALSE,TRUE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-015','6535bd1b87a85db2b6e8f349',FALSE,TRUE,FALSE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-016','654f4ab687a85db2b6ea1557',FALSE,FALSE,FALSE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-017','655620ff939e86c710d3c96d',TRUE,FALSE,FALSE,FALSE,TRUE),
	(gen_random_uuid()::text,'proj-018','6562cb67cf1b7cdbe0b6f1bf',TRUE,TRUE,TRUE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-019','656f1ec0fdbbd5756407fa3a',FALSE,FALSE,TRUE,FALSE,FALSE),
	(gen_random_uuid()::text,'proj-020','657d44c4ac6042435db02ef7',FALSE,TRUE,FALSE,FALSE,FALSE)
ON CONFLICT (f_proj_id, f_user_id) DO NOTHING;

-- 4) Update statuses for a few projects to demonstrate lifecycle
-- 0: not started, 1: in progress, 2: near complete, 3: done (example semantics)
UPDATE t_proj SET f_translating_status = 3, f_proofreading_status = 2, f_typesetting_status = 1, f_reviewing_status = 0, f_is_published = TRUE WHERE f_proj_id = 'proj-010';
UPDATE t_proj SET f_translating_status = 2, f_proofreading_status = 1, f_typesetting_status = 0, f_reviewing_status = 0, f_is_published = FALSE WHERE f_proj_id = 'proj-003';
UPDATE t_proj SET f_translating_status = 1, f_proofreading_status = 0, f_typesetting_status = 0, f_reviewing_status = 0 WHERE f_proj_id = 'proj-002';
UPDATE t_proj SET f_is_published = TRUE, f_translating_status = 3 WHERE f_proj_id = 'proj-001';

COMMIT;

-- End of mock data

