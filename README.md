# Project Napkin

![Project Status: Active](https://img.shields.io/badge/project_status-active-green?style=for-the-badge)
![Built with: Rust/Actix-web](https://img.shields.io/badge/built_with-rust/actix_web-blue?style=for-the-badge)
![Database: PostgreSQL with pgvector](https://img.shields.io/badge/database-postgres_with_pgvector-yellow?style=for-the-badge)

Project Napkin is a knowledge-focused database API with AI agents. It is built with Rust and uses Actix-web for the web framework and PostgreSQL with pgvector for the database. The purpose of Project Napkin is to generate a standardized method of mapping out data for the AI agent using network graphs and vector databases. The goal is to more accurately describe the requirements or other information for the agent, promoting more accurate and efficient interactions with data.

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes.

### Prerequisites

You will need to have the following installed on your machine:

- [Rust](https://www.rust-lang.org/tools/install)
- [PostgreSQL](https://www.postgresql.org/download/)
- [pgvector](https://github.com/ankane/pgvector)

### Clone the Repository

```bash
git clone https://github.com/Z90-Studios/napkin.git
cd napkin
```

### Database Setup

Create a PostgreSQL database and configure the connection string in your .env file.

```bash
DATABASE_URL=postgres://username:password@localhost/your_database
```

Then, set up the pgvector extension:

```sql
CREATE EXTENSION IF NOT EXISTS pgvector;
```

### Running the Application

You can run the application with the following command:

```bash
cargo run
```

If you want hot reloading during coding, you can use:

```bash
cargo watch -x run
```

## Contributing

We would love for you to contribute to `Project Napkin` and help make it even better than it is today! Check out our [Contributing Guide](CONTRIBUTING.md) to get started.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE.md) file for details.
