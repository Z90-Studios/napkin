CREATE EXTENSION IF NOT EXISTS "vector";
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS projects (
	id uuid DEFAULT uuid_generate_v4 (),
	name TEXT NOT NULL,
	PRIMARY KEY (id)
);

ALTER TABLE projects OWNER TO postgres;

INSERT INTO projects (name) VALUES('Project: Napkin');

CREATE TABLE IF NOT EXISTS node_tag (
	id uuid DEFAULT uuid_generate_v4 (),
	name TEXT NOT NULL,
	description TEXT NOT NULL,
	PRIMARY KEY (id)
)

CREATE TABLE IF NOT EXISTS nodes (
	id uuid DEFAULT uuid_generate_v4 (),
	project uuid,
	title TEXT NOT NULL,
	data json NOT NULL,
	embedding vector(3),
	tags uuid[],
	PRIMARY KEY (id),
	CONSTRAINT n_project
		FOREIGN KEY(project)
			REFERENCES projects(id),
	CONSTRAINT n_tag
		FOREIGN KEY(EACH ELEMENT OF tags)
			REFERENCES node_tag(id)
);