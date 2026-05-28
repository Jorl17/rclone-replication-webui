---
layout: default
title: Tâches
parent: Utilisation
nav_order: 2
---

# Gérer les tâches de réplication

Une tâche définit ce qu'il faut copier, vers où, et quand.

## Créer une tâche

Depuis la page **Tâches**, cliquez sur **Nouvelle tâche**.

### Champs obligatoires

| Champ | Description |
|---|---|
| **Nom** | Identifiant lisible pour ta tâche |
| **Stockage source** | Sélectionne un [stockage](storages/) existant |
| **Chemin source** | Chemin dans le stockage source (ex. `backups/db/`). Laissé vide = racine. |
| **Stockage destination** | Sélectionne un autre stockage |
| **Chemin destination** | Chemin dans le stockage destination |

### Champs optionnels

| Champ | Description |
|---|---|
| **Planification automatique** | Expression cron (5 ou 6 champs) ou macro (`@daily`, `@hourly`…). Voir [Cron](#planification-cron) |
| **Options rclone supplémentaires** | Flags additionnels passés tels quels (`--bwlimit 10M --transfers 4`) |
| **Nombre de tentatives** | Retry en cas d'échec. Défaut : 3 |
| **Délai de base (secondes)** | Délai entre tentatives, multiplié par le numéro de tentative. Défaut : 15s |
| **Notifications** | Canal Apprise + cases « Erreur / Succès / Ignorée » |

## Planification cron

Le champ accepte trois formats :

| Format | Exemple | Description |
|---|---|---|
| **5 champs** (Unix standard) | `0 2 * * *` | minute, heure, jour, mois, jour-semaine |
| **6 champs** (avec secondes) | `*/30 * * * * *` | seconde, minute, heure, jour, mois, jour-semaine |
| **Macro** | `@hourly`, `@daily`, `@weekly`, `@monthly`, `@yearly` | Préréglages |

Un **aperçu en direct** s'affiche sous le champ et indique :
- Si l'expression est valide ou non
- Une traduction en français (« À 02:00 tous les jours »)
- Les 3 prochaines exécutions calculées

## Retry automatique

Lorsqu'une tâche échoue (exit code rclone ≠ 0), elle est relancée jusqu'à `max_retries` fois. Le délai entre tentatives est **linéaire** : `delay × n° de tentative`.

Exemple avec `max_retries = 3` et `retry_delay = 15s` :

| Tentative | Attente avant lancement |
|---|---|
| 1 (initiale) | 0 s |
| 2 | 15 s |
| 3 | 30 s |
| 4 | 45 s |

Les retries sont logués dans la sortie SSE en direct (`--- Tentative 2/4 dans 15s ---`) et inclus dans le `log_output` final. Une **seule** notification est envoyée à la fin (après le dernier retry).

## Déclencher une tâche manuellement

Depuis la page de la tâche, deux boutons :

- **Lancer** : exécution normale (source → destination)
- **Restaurer** : lit la destination et écrit vers une cible choisie (par défaut la source d'origine). Une fenêtre permet de choisir la cible et, si la destination est [chiffrée](encryption), de saisir la clé privée. Voir [Chiffrement](encryption).

Si la tâche est déjà en cours, les boutons sont désactivés. Si le cron tente de déclencher une tâche déjà en cours, l'exécution est enregistrée comme **ignorée** (`skipped`) et notifie si l'option « Notifier en cas d'exécution ignorée » est activée.

## Historique des exécutions

Pour chaque tâche, les **100 dernières exécutions** sont conservées (trigger SQL automatique). Le tableau affiche :

- Date de démarrage
- Déclencheur : `Manuel` / `Planifié` / `Restauration`
- Statut : `Succès` / `Échec` / `En cours` / `Ignorée`
- Durée
- **Stats rclone** : transferts (X/Y), volume transféré (formaté Ko/Mo/Go), vérifications, suppressions, erreurs
- Bouton **Logs** : déplie les logs bruts directement dans le tableau

## Suivi en temps réel

Pendant qu'une tâche tourne, la page de détail affiche un panneau « Progression en temps réel » alimenté en SSE. Les logs apparaissent ligne par ligne avec auto-scroll.

L'événement de fin (`done` SSE) ferme le panneau et déclenche le rafraîchissement automatique du tableau d'historique (via le SSE global `/api/events`).

## Stats parsées des logs rclone

L'app extrait la dernière ligne JSON contenant un objet `stats` produite par `rclone --use-json-log`. Sont récupérés :

- `transfers` / `totalTransfers` — fichiers transférés
- `bytes` — volume total (formaté en Ko/Mo/Go)
- `checks` — fichiers comparés sans transfert
- `deletes` — fichiers supprimés (affiché en orange si > 0)
- `errors` — erreurs (affiché en rouge si > 0)
