---
layout: default
title: Local
parent: Stockages
grand_parent: Utilisation
nav_order: 1
---

# Local (dossier local)

Le type le plus simple : un dossier sur le système de fichiers où tourne le container backend.

## Côté provider

Aucune configuration. Assure-toi simplement que le dossier existe et qu'il est **accessible depuis le container backend**.

Dans Docker, monte le chemin de l'hôte en volume :

```yaml
backend:
  volumes:
    - /chemin/sur/lhote:/mnt/data
```

Le backend tourne avec un utilisateur non-root (`appuser`). Vérifie que les permissions du dossier le permettent (`chmod 755` ou `chown -R 999:999`).

## Côté rclone-ui

| Champ | Exemple | Description |
|---|---|---|
| **Nom du stockage** | `local-backups` | Identifiant interne |
| **Type** | `Local (dossier local)` | — |
| **Chemin du dossier** | `/mnt/data/backups` | Racine du stockage |

Le chemin spécifié est concaténé au chemin de la tâche au moment de l'exécution. Exemple :

- Stockage Local avec `root = /mnt/data`
- Chemin source de la tâche : `important/`
- Commande rclone effective : `local:/mnt/data/important/`

## Cas d'usage

- Réplication d'un dossier serveur vers le cloud (Local → S3)
- Restauration depuis le cloud (S3 → Local)
- Tests rapides sans configurer de service distant
