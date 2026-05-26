---
layout: default
title: Utilisation
nav_order: 3
has_children: true
permalink: /usage/
---

# Utilisation

Une fois l'application installée et accessible (par défaut sur [http://localhost](http://localhost)), tu peux gérer trois types d'objets :

| Objet | Description |
|---|---|
| [Stockages distants](storages/) | Sources et destinations des réplications (S3, SFTP, Dropbox…) |
| [Tâches](tasks) | Jobs de réplication source → destination, planifiables |
| [Notifications](notifications) | Canaux Apprise pour recevoir des alertes |

## Workflow général

1. **Configurer ses stockages** distants (au moins une source et une destination)
2. **Créer un canal de notification** (optionnel)
3. **Créer une tâche** qui lie source + destination + planning + notification
4. **Lancer la tâche** manuellement ou laisser le cron s'en occuper
5. **Consulter l'historique** des exécutions avec les stats et logs

## Concepts importants

### Stockages (remotes rclone)

Un stockage est une configuration rclone nommée. La couche rclone gère la communication réelle avec les différents services (cloud, serveurs distants, disque local…).

L'app distingue **types standard** (formulaire guidé avec champs nommés) et **types avancés** (éditeur libre clé/valeur). Voir [Types standard](storages/).

### Tâches

Une tâche = `source → destination` avec :
- une planification cron (optionnelle)
- des flags rclone supplémentaires
- une politique de retry (nombre de tentatives + délai)
- un canal de notification (avec conditions : erreur, succès, ignorée)

### Anti-chevauchement

Si une tâche est déjà en cours et que le cron veut la déclencher à nouveau, l'exécution est **ignorée** (`skipped`) et logguée. Pas de file d'attente.

### Mode restauration

Permet de lancer une tâche dans le sens inverse (destination → source). Utile pour restaurer des fichiers depuis une sauvegarde.
