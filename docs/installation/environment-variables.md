---
layout: default
title: Variables d'environnement
parent: Installation
nav_order: 4
---

# Variables d'environnement

Toutes les variables sont lues au démarrage par le backend Rust via [dotenvy](https://crates.io/crates/dotenvy). Elles peuvent être définies dans un fichier `.env`, dans `docker-compose.yml`, ou dans l'environnement système.

## Cœur de l'application

| Variable | Description | Défaut |
|---|---|---|
| `DATABASE_URL` | URL PostgreSQL au format `postgresql://user:pass@host:port/db` | **Requis** |
| `BIND_ADDR` | Adresse d'écoute du backend | `0.0.0.0:3000` |
| `RCLONE_BIN` | Chemin (ou nom dans `$PATH`) du binaire rclone | `rclone` |
| `APPRISE_BIN` | Chemin du binaire apprise-go | `apprise` |
| `RUST_LOG` | Niveau de logs ([env_logger](https://docs.rs/env_logger/)) | `info,rclone_replication_ui=debug,sea_orm=warn,sqlx=warn` |

## Secret Manager

Le provider est sélectionné via `SECRET_MANAGER_PROVIDER`. Voir [Secret Manager](../usage/secret-manager) pour la description et le setup côté provider.

| Variable | Valeurs | Défaut |
|---|---|---|
| `SECRET_MANAGER_PROVIDER` | `none`, `scaleway`, `aws`, `azure`, `gcp`, `vault`, `infisical`, `doppler` | `none` |

### Scaleway

| Variable | Défaut |
|---|---|
| `SCW_SECRET_KEY` | — |
| `SCW_PROJECT_ID` | — |
| `SCW_DEFAULT_REGION` | `fr-par` |
| `SCW_SECRET_PATH` | `/rclone-ui` |

### AWS

Utilise la [chaîne d'auth AWS standard](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/credentials.html) : `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, ou IAM Role/IRSA.

| Variable | Défaut |
|---|---|
| `AWS_REGION` | `eu-west-1` |
| `AWS_SECRET_PREFIX` | `rclone-ui/` |

### Azure Key Vault

| Variable | Défaut |
|---|---|
| `AZURE_TENANT_ID` | — |
| `AZURE_CLIENT_ID` | — |
| `AZURE_CLIENT_SECRET` | — |
| `AZURE_VAULT_URL` (ex `https://myvault.vault.azure.net`) | — |

### Google Cloud Secret Manager

Utilise les [Application Default Credentials](https://cloud.google.com/docs/authentication/application-default-credentials) (`GOOGLE_APPLICATION_CREDENTIALS`, metadata server GCE/GKE, etc.).

| Variable | Défaut |
|---|---|
| `GCP_PROJECT_ID` | — |

### HashiCorp Vault

| Variable | Défaut |
|---|---|
| `VAULT_ADDR` (ex `http://vault.local:8200`) | — |
| `VAULT_TOKEN` | — |
| `VAULT_MOUNT_PATH` | `secret` |
| `VAULT_PATH_PREFIX` | `rclone-ui` |

### Infisical

| Variable | Défaut |
|---|---|
| `INFISICAL_HOST` | `https://app.infisical.com` |
| `INFISICAL_CLIENT_ID` | — |
| `INFISICAL_CLIENT_SECRET` | — |
| `INFISICAL_PROJECT_ID` | — |
| `INFISICAL_ENVIRONMENT` | `prod` |
| `INFISICAL_SECRET_PATH` | `/rclone-ui` |

### Doppler

| Variable | Défaut |
|---|---|
| `DOPPLER_TOKEN` (Service Token) | — |
| `DOPPLER_PROJECT` | — |
| `DOPPLER_CONFIG` | `prd` |

## Exemple complet `.env`

```env
# Cœur
DATABASE_URL=postgresql://rclone:rclone@postgres:5432/rclone_ui
BIND_ADDR=0.0.0.0:3000
RUST_LOG=info,rclone_replication_ui=debug,sea_orm=warn,sqlx=warn

# Secret Manager (exemple Scaleway)
SECRET_MANAGER_PROVIDER=scaleway
SCW_SECRET_KEY=00000000-0000-0000-0000-000000000000
SCW_PROJECT_ID=00000000-0000-0000-0000-000000000000
SCW_DEFAULT_REGION=fr-par
```

## Logs : ajuster la verbosité

Pour voir les requêtes SQL :

```env
RUST_LOG=info,rclone_replication_ui=debug,sea_orm=debug,sqlx=debug
```

Pour ne logger que les erreurs :

```env
RUST_LOG=warn
```
