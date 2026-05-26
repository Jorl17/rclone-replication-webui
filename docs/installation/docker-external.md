---
layout: default
title: Docker (PostgreSQL externe)
parent: Installation
nav_order: 2
---

# Docker avec PostgreSQL externe

Méthode recommandée pour la production lorsque tu as déjà un serveur PostgreSQL géré (RDS, Scaleway, Supabase, etc.).

## Prérequis

- Docker 20+ et Docker Compose v2
- Une base PostgreSQL 14+ accessible depuis les containers
- Un utilisateur dédié avec droits `CREATE TABLE` sur la base cible

## Préparation de la base

Connectez-vous à votre serveur PostgreSQL et créez une base + un utilisateur dédiés :

```sql
CREATE USER rclone_ui WITH PASSWORD 'change-me';
CREATE DATABASE rclone_ui OWNER rclone_ui;
GRANT ALL PRIVILEGES ON DATABASE rclone_ui TO rclone_ui;
```

Les migrations SeaORM sont appliquées automatiquement au démarrage du backend.

## Lancement

Créez un fichier `.env` à la racine du projet :

```env
DATABASE_URL=postgresql://rclone_ui:change-me@host.docker.internal:5432/rclone_ui
```

> `host.docker.internal` pointe vers la machine hôte depuis un container.
> Sur Linux, ajoutez `extra_hosts: ["host.docker.internal:host-gateway"]` (déjà présent dans `docker-compose.yml`).

Puis lancez :

```bash
docker compose up --build -d
```

Deux containers démarrent :

| Service | Port | Image |
|---|---|---|
| `backend` | 3000 | `FROM scratch` |
| `frontend` | 80 | `nginx:alpine` |

## Utiliser l'image publiée

Pour ne pas builder localement, utilisez les images du registry GitHub :

```yaml
services:
  backend:
    image: ghcr.io/sixmon/rclone-replication-webui-backend:latest
    # ...
  frontend:
    image: ghcr.io/sixmon/rclone-replication-webui-frontend:latest
    # ...
```

Les images sont buildées automatiquement à chaque tag `v*.*.*` par le workflow GitHub Actions.

## Reverse proxy / HTTPS

Le frontend nginx écoute sur le port 80 en HTTP simple. Pour exposer en HTTPS, placez-le derrière un reverse proxy (Caddy, Traefik, nginx, Cloudflare) qui gère TLS.

Exemple Caddy :

```
mon-rclone-ui.example.com {
  reverse_proxy localhost:80
}
```

Pensez à ajouter `X-Forwarded-Proto: https` et `X-Forwarded-Host` dans les headers — le backend les lit pour construire les URLs de callback OAuth.

## Considérations production

- **Read-only** : les containers tournent en `read_only: true` avec `no-new-privileges`. Voir `docker-compose.yml`.
- **Healthcheck** : le backend expose `--health` (CLI) qui teste la connexion DB. Le compose redémarre automatiquement si la BDD tombe.
- **Sauvegardes** : vos backups PostgreSQL contiennent toute la config. Si vous utilisez un [Secret Manager externe](../usage/secret-manager), les credentials sensibles n'y sont pas (uniquement les références).
