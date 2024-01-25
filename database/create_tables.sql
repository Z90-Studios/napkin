CREATE EXTENSION IF NOT EXISTS "vector";
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE OR REPLACE FUNCTION generate_ulid() RETURNS uuid
	AS $$
		SELECT (lpad(to_hex(floor(extract(epoch FROM clock_timestamp()) * 1000)::bigint), 12, '0') || encode(gen_random_bytes(10), 'hex'))::uuid;
	$$ LANGUAGE SQL;

CREATE TABLE IF NOT EXISTS projects (
	id uuid DEFAULT generate_ulid (),
	name TEXT NOT NULL,
	PRIMARY KEY (id)
);

ALTER TABLE projects OWNER TO postgres;

CREATE TABLE IF NOT EXISTS nodes (
	id uuid DEFAULT generate_ulid (),
	project UUID,
	title TEXT NOT NULL,
	data JSONB NOT NULL,
	PRIMARY KEY (id),
	CONSTRAINT n_project
		FOREIGN KEY(project)
			REFERENCES projects(id)
);

CREATE TABLE IF NOT EXISTS edges (
	id UUID DEFAULT generate_ulid (),
	project UUID,
	source UUID,
	target UUID,
	data JSONB NOT NULL,
	PRIMARY KEY (id),
	CONSTRAINT e_project
		FOREIGN KEY(project)
			REFERENCES projects(id),
	CONSTRAINT e_source
		FOREIGN KEY(source)
			REFERENCES nodes(id),
	CONSTRAINT e_target
		FOREIGN KEY(target)
			REFERENCES nodes(id)
);

CREATE TABLE IF NOT EXISTS node_metadata (
	owner_id UUID NOT NULL,
	name TEXT NOT NULL,
	value JSONB NOT NULL,
	CONSTRAINT node_tag_pkey PRIMARY KEY (owner_id, name),
	CONSTRAINT node_tag_owner_id_fkey FOREIGN KEY (owner_id)
		REFERENCES nodes (id) ON DELETE CASCADE

);

CREATE TABLE IF NOT EXISTS edge_metadata (
	owner_id UUID NOT NULL,
	name TEXT NOT NULL,
	value JSONB NOT NULL,
	CONSTRAINT edge_tag_pkey PRIMARY KEY (owner_id, name),
	CONSTRAINT edge_tag_owner_id_fkey FOREIGN KEY (owner_id)
		REFERENCES edges (id) ON DELETE CASCADE
);
