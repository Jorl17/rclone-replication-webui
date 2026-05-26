---
layout: default
title: Installation
nav_order: 2
has_children: true
permalink: /installation/
---

# Installation

rclone-replication-ui peut être installé de plusieurs façons selon ton environnement :

| Méthode | Cas d'usage |
|---|---|
| [Docker (PostgreSQL inclus)](docker-postgres) | Démarrage rapide, prototypage, machine de dev |
| [Docker (PostgreSQL externe)](docker-external) | Production avec une base existante |
| [Sans Docker](manual) | Développement local sur le code source |

## Prérequis communs

- **Frontend nginx + backend Rust** : tournent dans 2 containers séparés (ports 80 et 3000)
- **PostgreSQL** : nécessaire dans tous les cas (interne ou externe)
- **Réseau** : le backend doit pouvoir joindre les stockages distants à répliquer

## Architecture

```
┌─────────────────────────────────────┐
│  Frontend (nginx)                   │
│  ├─ Static files (React build)      │
│  └─ /api/* → proxy vers backend     │
└─────────────────────────────────────┘
                 ↕
┌─────────────────────────────────────┐
│  Backend (Rust + Axum)              │
│  ├─ REST API + SSE                  │
│  ├─ Scheduler cron                  │
│  ├─ Spawn rclone + apprise          │
│  └─ Secrets via SecretStore         │
└─────────────────────────────────────┘
                 ↕ SQL
┌─────────────────────────────────────┐
│  PostgreSQL                         │
└─────────────────────────────────────┘
```

Continuez sur l'une des pages d'installation à gauche pour la suite.
