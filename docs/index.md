---
layout: default
title: Accueil
nav_order: 1
description: "Documentation officielle de rclone-replication-ui"
permalink: /
---

# rclone-replication-ui
{: .fs-9 }

Interface web de gestion de réplication de fichiers basée sur [rclone](https://rclone.org/).
{: .fs-6 .fw-300 }

[Installation](installation/){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 }
[Utilisation](usage/){: .btn .fs-5 .mb-4 .mb-md-0 .mr-2 }
[Code source](https://github.com/SIXMON/rclone-replication-webui){: .btn .fs-5 .mb-4 .mb-md-0 }

---

## Aperçu

rclone-replication-ui est une application self-hosted qui transforme la CLI [rclone](https://rclone.org/) en une interface web moderne, simple et puissante.

![Liste des tâches](https://raw.githubusercontent.com/SIXMON/rclone-replication-webui/main/docs/screenshots/tasks-list.png)

## Fonctionnalités

- **Stockages distants** — Formulaires guidés pour S3, SFTP, FTP, SMB, Azure Blob Storage, SharePoint, Google Drive, Dropbox, Backblaze B2 et stockage local. Éditeur clé/valeur pour les autres types rclone.
- **Tâches de réplication** — Planification cron (5 ou 6 champs, macros `@daily`, `@hourly`), aperçu en direct des prochaines exécutions, déclenchement manuel, mode restauration (sync inverse).
- **Retry automatique** — Relance configurable en cas d'échec avec backoff linéaire.
- **Suivi temps réel** — Progression SSE et logs en direct pendant l'exécution.
- **Historique** — 100 dernières exécutions par tâche avec stats rclone (transferts, volume, erreurs).
- **Notifications** — Alertes markdown via Apprise (Slack, Mattermost, email, webhooks).
- **Protection anti-chevauchement** — Les déclenchements concurrents sont ignorés.
- **OAuth intégré** — Connexion en un clic pour Dropbox et Google Drive.
- **Secret Manager** — Stockage des credentials dans Scaleway, AWS, Azure Key Vault, GCP, HashiCorp Vault, Infisical ou Doppler.

## Stack technique

| Composant | Technologie |
|---|---|
| Backend | Rust (Axum 0.8) |
| ORM | SeaORM 1.x |
| Frontend | React 19 + TypeScript + Vite 8 + Tailwind 4 |
| Base de données | PostgreSQL |
| Planification | tokio-cron-scheduler |
| Réplication | [rclone](https://rclone.org/) |
| Notifications | [apprise-go](https://github.com/unraid/apprise-go) |

## Premier pas

Le plus rapide pour essayer le projet :

```bash
git clone https://github.com/SIXMON/rclone-replication-webui.git
cd rclone-replication-webui
docker compose -f docker-compose.postgres.yml up --build
```

Ouvrez ensuite [http://localhost](http://localhost).

Pour les options avancées (PostgreSQL externe, Secret Manager, etc.), consultez la section [Installation](installation/).
