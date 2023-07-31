CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS projects (
	id uuid DEFAULT uuid_generate_v4 (),
	name TEXT NOT NULL,
	PRIMARY KEY (id)
);

ALTER TABLE projects OWNER TO postgres;

INSERT INTO projects (name) VALUES('Project: Napkin');
