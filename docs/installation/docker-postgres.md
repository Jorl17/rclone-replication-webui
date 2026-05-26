---
layout: default
title: Docker (PostgreSQL inclus)
parent: Installation
nav_order: 1
---

# Docker avec PostgreSQL intégré

Méthode recommandée pour tester le projet ou pour les petits déploiements.

## Prérequis

- Docker 20+ et Docker Compose v2
- 1 Go de RAM minimum
- Un port libre (80 pour le frontend, 3000 pour le backend, 5432 pour Postgres)

## Lancement

```bash
git clone https://github.com/SIXMON/rclone-replication-webui.git
cd rclone-replication-webui
docker compose -f docker-compose.postgres.yml up --build -d
```

Trois containers démarrent :

| Service | Port | Image |
|---|---|---|
| `postgres` | 5432 | `postgres:18-alpine` |
| `backend` | 3000 | construit depuis `./backend/Dockerfile` (image finale `FROM scratch`) |
| `frontend` | 80 | construit depuis `./frontend/Dockerfile` (`nginx:alpine`) |

Ouvrez ensuite [http://localhost](http://localhost).

## Configuration par défaut

Le fichier `docker-compose.postgres.yml` est livré avec :

```yaml
postgres:
  image: postgres:18-alpine
  environment:
    POSTGRES_USER: rclone
    POSTGRES_PASSWORD: rclone
    POSTGRES_DB: rclone_ui
```

⚠️ **Pour un déploiement public**, change au moins le mot de passe et désactive l'exposition du port 5432.

## Persistance des données

Les données PostgreSQL sont stockées dans un volume Docker nommé `pgdata`. Il survit aux redémarrages mais pas à un `docker compose down -v`.

```bash
# Voir le volume
docker volume inspect rclone-replication-webui_pgdata

# Sauvegarder
docker compose -f docker-compose.postgres.yml exec postgres \
  pg_dump -U rclone rclone_ui > backup.sql

# Restaurer
cat backup.sql | docker compose -f docker-compose.postgres.yml exec -T postgres \
  psql -U rclone rclone_ui
```

## Configurer le Secret Manager

Voir la page [Variables d'environnement](environment-variables) pour la liste complète, et la page [Secret Manager](../usage/secret-manager) pour le détail de chaque provider.

Exemple pour activer Scaleway Secret Manager :

```yaml
backend:
  environment:
    DATABASE_URL: "postgresql://rclone:rclone@postgres:5432/rclone_ui"
    SECRET_MANAGER_PROVIDER: scaleway
    SCW_SECRET_KEY: "${SCW_SECRET_KEY}"
    SCW_PROJECT_ID: "${SCW_PROJECT_ID}"
```

Pour les secrets, utilisez plutôt un fichier `.env` à la racine que vous **n'ajouterez pas** au repo Git :

```env
SCW_SECRET_KEY=00000000-0000-0000-0000-000000000000
SCW_PROJECT_ID=00000000-0000-0000-0000-000000000000
```

## Arrêt et nettoyage

```bash
# Arrêt simple
docker compose -f docker-compose.postgres.yml down

# Arrêt + suppression des volumes (perte de données !)
docker compose -f docker-compose.postgres.yml down -v
```
