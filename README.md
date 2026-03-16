# Kadyn Pearce Portfolio

A high-performance portfolio website built with Rust, Actix-web, and Supabase (PostgreSQL).

## Features

- **Tech Stack**: Rust, Actix-web, Supabase (PostgreSQL), Tera templates
- **Admin Dashboard**: Manage projects, blog posts, skills, and experience via `/admin`
- **GitHub Authentication**: Login with GitHub to access admin features
- **Contact Form**: Email integration via Resend
- **Responsive Design**: Modern, mobile-friendly UI

## Quick Start

### Local Development

```bash
# Clone the repository
git clone https://github.com/kadynjaipearce/new_portfolio.git
cd new_portfolio

# Copy environment variables
cp .env.example .env

# Run the server
cargo run
```

Visit `http://localhost:8080`

### Docker Deployment

```bash
# Build and run with Docker Compose
docker-compose up -d
```

Visit `http://localhost:8080`

## Environment Variables

Use either full `DATABASE_URL` or `SUPABASE_DB_URL` + `SUPABASE_SERVICE_KEY` for the database.

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | Full Postgres connection string (URI) | Yes* |
| `SUPABASE_DB_URL` | Postgres URL without password (e.g. `postgres://postgres.[ref]@...pooler.supabase.com:6543/postgres`) | Yes* |
| `SUPABASE_SERVICE_KEY` | Database password / service key (use as secret; not in URL) | Yes* |
| `GITHUB_CLIENT_ID` | GitHub OAuth app client ID | Yes |
| `GITHUB_CLIENT_SECRET` | GitHub OAuth app client secret | Yes |
| `GITHUB_REDIRECT_URI` | OAuth callback URL | Yes |
| `ADMIN_GITHUB_USERNAME` | GitHub username for admin access | Yes |
| `SESSION_SECRET` | Secret for session encryption | Yes |
| `RESEND_API_KEY` | Resend API key for emails | No |
| `RESEND_FROM_EMAIL` | Email sender address | No |
| `CONTACT_EMAIL` | Email address for contact form | Yes |

## GitHub OAuth Setup

1. Go to GitHub Settings > Developer settings > OAuth Apps
2. Create a new OAuth App
3. Set Homepage URL to your domain
4. Set Authorization callback URL to `https://yourdomain.com/auth/github/callback`
5. Copy Client ID and Client Secret to your environment variables

## Dokploy Deployment

1. Create a new project in Dokploy and connect your Git repository (or use the Dockerfile in this repo).
2. Configure the service to build from the Dockerfile and expose port **8080**.
3. In the service **Environment** (or Secrets), set:
   - `DATABASE_URL` – Supabase Postgres connection string
   - `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`, `GITHUB_REDIRECT_URI`
   - `ADMIN_GITHUB_USERNAME`, `SESSION_SECRET`, `CONTACT_EMAIL`
   - Optionally: `RESEND_API_KEY`, `RESEND_FROM_EMAIL`
4. Set **GITHUB_REDIRECT_URI** to your Dokploy app URL, e.g. `https://yourdomain.com/auth/github/callback`.
5. Deploy; the app will run migrations on startup.

## Managing Content

1. Visit `/admin` and login with GitHub
2. Add your projects, experience, skills, and blog posts
3. Content is stored in the database and persists across deployments

## Database schema

The app expects a **projects** table with the schema in [sql/schema.sql](sql/schema.sql) (id, name, company, description, project_type, url, site_image_url, client_image_url, tags, stars, sort_order, is_featured, created_at). If you already have this table, you don’t need to run the projects section. The migrations in `migrations/` create only **posts**, **experiences**, **skills**, and **messages**. For a full reference of all table definitions, run or copy from `sql/schema.sql`.

## Project structure

```
portfolio/
├── sql/schema.sql       # Full SQL for all tables (reference)
├── migrations/          # Applied on startup (posts, experiences, skills, messages only)
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── db/
│   ├── models/
│   ├── routes/
│   ├── services/
│   └── middleware/
├── templates/
├── static/
├── Dockerfile
├── docker-compose.yml
└── .env.example
```

## License

MIT
