---
layout: default
title: Sans Docker (dev)
parent: Installation
nav_order: 3
---

# Installation manuelle (développement)

Utile pour contribuer au code source ou pour débuguer.

## Prérequis

- **Rust** 1.75+ (`rustup install stable`)
- **Node.js** 24+ — la version est figée dans [`frontend/.node-version`](https://github.com/SIXMON/rclone-replication-webui/blob/main/frontend/.node-version)
- **PostgreSQL** 14+
- **rclone** installé localement (`brew install rclone` / `apt install rclone`)
- **apprise-go** (optionnel, pour les notifications)

## Backend

```bash
cd backend
export DATABASE_URL="postgresql://user:password@localhost:5432/rclone_ui"
cargo run
```

Le serveur démarre sur `http://localhost:3000`. Les migrations sont appliquées au démarrage.

Pour lancer en mode optimisé :

```bash
cargo build --release
./target/release/rclone-replication-ui
```

### Lint et formatage

```bash
cargo fmt           # formatage
cargo clippy        # lint
cargo build         # vérifie la compilation
```

### Healthcheck CLI

Le binaire accepte un flag `--health` qui se connecte à la BDD, exécute `SELECT 1` et quitte. Pratique pour les healthchecks Docker.

```bash
./target/release/rclone-replication-ui --health
# OK
```

## Frontend

Avec `nvm` :

```bash
cd frontend
nvm use         # lit .node-version → Node 24
npm install
npm run dev
```

Le serveur Vite démarre sur `http://localhost:5173` et proxifie automatiquement `/api/*` vers le backend (configuré dans `vite.config.ts`).

### Commandes utiles

```bash
npm run build       # build production dans dist/
npm run lint        # ESLint
npm run preview     # serveur statique pour tester le build
```

## Base de données locale

Option 1 : avec Docker

```bash
docker run -d --name postgres-rclone \
  -e POSTGRES_USER=rclone \
  -e POSTGRES_PASSWORD=rclone \
  -e POSTGRES_DB=rclone_ui \
  -p 5432:5432 \
  postgres:18-alpine
```

Option 2 : avec Postgres.app (macOS) ou un service système.

## Structure du repo

```
rclone-replication-ui/
├── backend/                    # Crate Rust
│   ├── src/
│   │   ├── entities/           # Entités SeaORM
│   │   ├── migration/          # Migrations BDD
│   │   ├── models/             # DTO requête/réponse
│   │   ├── routes/             # Handlers API
│   │   ├── services/           # Logique métier
│   │   │   ├── secrets/        # SecretStore (7 providers)
│   │   │   └── oauth/          # Flow OAuth (Dropbox, Drive, OneDrive)
│   │   └── sse/                # Server-Sent Events
│   └── Cargo.toml
├── frontend/                   # App React/TS
│   └── src/
│       ├── pages/
│       ├── components/
│       ├── hooks/
│       └── api/
├── docs/                       # Cette documentation (Jekyll)
├── docker-compose.yml          # PostgreSQL externe
└── docker-compose.postgres.yml # PostgreSQL inclus
```
