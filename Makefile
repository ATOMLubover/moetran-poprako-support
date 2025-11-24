migadd:
	sqlx migrate add -r $(name)

migrev:
	sqlx migrate revert

migrun:
	sqlx migrate run 

dbrst:
	sqlx database reset

prepare:
	cargo sqlx prepare