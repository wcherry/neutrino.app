# Neutrino

A Google Drive-competitive cloud storage and collaboration platform. Built with Rust on the backend and Next.js on the frontend, designed to be self-hosted or used via the managed service at [neutrino.app](https://neutrino.app).

## Features

**Phase 1 (current)**
- File upload with multipart support — files up to 10 GB
- File download with HTTP Range (resume support)
- Per-user storage quotas and daily upload caps
- Folder hierarchy with unlimited nesting
- Star/favorite, color-label folders, file shortcuts
- Bulk move, delete, and ZIP download
- Trash with 30-day retention and restore
- JWT authentication (access + refresh tokens)

**Planned**
- In-browser file previews (PDF, image, video, code)
- File versioning with history and restore
- Sharing with role-based permissions (Owner, Editor, Viewer)
- Native Docs, Sheets, Slides editors
- Real-time collaboration
- Desktop sync client and mobile apps
- AI writing and data assistance
- Enterprise admin, DLP, audit logs

## Tech Stack

| Layer | Technology |
|---|---|
| API | Rust + actix-web 4 |
| Database | SQLite (via Diesel 2, embedded) |
| Auth | argon2 (passwords), jsonwebtoken 9 (JWT) |
| File storage | Local filesystem (MinIO planned) |
| API docs | utoipa 4 + Swagger UI |
| Frontend | Next.js 15, React 18, TypeScript 5 |
| Styling | CSS custom properties (no Tailwind) |
| State | TanStack Query + Zustand |
| Forms | React Hook Form + Zod |
| Package manager | pnpm workspaces |

## Project Structure

```
neutrino/
├── apps/
│   └── web/                  # Next.js 15 web application
│       └── src/app/
│           ├── page.tsx       # Landing page
│           ├── sign-in/       # Auth pages
│           └── register/
├── packages/
│   └── ui/                   # Shared component library (32 components)
│       └── src/
│           ├── tokens/        # Design tokens (colors, spacing, typography)
│           ├── components/    # Button, Modal, Toast, etc.
│           └── motion/        # Framer Motion variants
├── backend/
│   ├── drive/                # Main API server
│   │   ├── src/
│   │   │   ├── features/
│   │   │   │   ├── auth/     # Register, login, token refresh
│   │   │   │   ├── storage/  # Upload, download, metadata, quotas
│   │   │   │   └── filesystem/ # Folders, trash, bulk ops, shortcuts
│   │   │   ├── config.rs
│   │   │   ├── schema.rs
│   │   │   └── main.rs
│   │   └── migrations/
│   ├── shared/               # Shared library crate
│   └── worker/               # Background worker (stub)
└── docs/
    └── ui/                   # Component and design system docs
```

## Getting Started

### Prerequisites

- Rust (stable) — [rustup.rs](https://rustup.rs)
- Node.js 20+ and pnpm — `npm install -g pnpm`
- SQLite (bundled — no install needed)

### Backend

```bash
cd backend/drive

# Copy and configure environment
cp .env.example .env
# Edit .env: set JWT_SECRET, DATABASE_URL, STORAGE_PATH, PORT

# Run migrations and start the server
cargo run
```

The API will be available at `http://localhost:8080`.
Swagger UI: `http://localhost:8080/swagger-ui/`

### Frontend

```bash
# From the repo root
pnpm install
pnpm dev
```

The web app will be available at `http://localhost:3000`.

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `neutrino.db` | SQLite database file path |
| `PORT` | `8080` | API server port |
| `JWT_SECRET` | *(required)* | Secret for signing JWTs |
| `JWT_ACCESS_EXPIRY_SECS` | `900` | Access token lifetime (15 min) |
| `JWT_REFRESH_EXPIRY_SECS` | `604800` | Refresh token lifetime (7 days) |
| `STORAGE_PATH` | `./storage` | Directory for uploaded files |
| `LOG_LEVEL` | `info` | Log verbosity |

### Docker (self-hosted)

```bash
docker run -d \
  -e JWT_SECRET=your-secret-here \
  -e STORAGE_PATH=/data/files \
  -v neutrino-data:/data \
  -p 8080:8080 \
  ghcr.io/your-org/neutrino:latest
```

## API Reference

Full interactive docs are available at `/swagger-ui/` when the server is running.

**Auth**
```
POST /api/v1/auth/register
POST /api/v1/auth/login
POST /api/v1/auth/refresh
POST /api/v1/auth/logout
```

**Storage**
```
POST   /api/v1/storage/files           Upload file (multipart)
GET    /api/v1/storage/files           List files (paginated)
GET    /api/v1/storage/files/{id}      Download file (supports Range)
GET    /api/v1/storage/files/{id}/metadata
GET    /api/v1/storage/quota
```

**Filesystem**
```
POST   /api/v1/fs/folders              Create folder
GET    /api/v1/fs/                     Root contents
GET    /api/v1/fs/folders/{id}         Folder contents
PATCH  /api/v1/fs/folders/{id}         Rename / color / star
DELETE /api/v1/fs/folders/{id}         Move to trash
PATCH  /api/v1/fs/files/{id}           Rename / move / star
DELETE /api/v1/fs/files/{id}           Move to trash
POST   /api/v1/fs/shortcuts            Create shortcut
DELETE /api/v1/fs/shortcuts/{id}       Delete shortcut
POST   /api/v1/fs/bulk/move            Bulk move
POST   /api/v1/fs/bulk/trash           Bulk trash
GET    /api/v1/fs/bulk/download        Bulk download as ZIP
GET    /api/v1/fs/trash                List trash
DELETE /api/v1/fs/trash                Empty trash
POST   /api/v1/fs/trash/files/{id}/restore
DELETE /api/v1/fs/trash/files/{id}     Permanently delete
POST   /api/v1/fs/trash/folders/{id}/restore
DELETE /api/v1/fs/trash/folders/{id}   Permanently delete
```

## Contributing

1. Fork and clone the repository
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Follow the [Backend Design Guide](agent_docs/Backend/Backend-Design-Guide.md) conventions
4. Open a pull request

## License

MIT
