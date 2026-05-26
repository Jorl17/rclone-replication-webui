---
layout: default
title: Notifications
parent: Utilisation
nav_order: 3
---

# Canaux de notification

Les notifications sont envoyées via [apprise-go](https://github.com/unraid/apprise-go), un binaire Go qui supporte 80+ services (Slack, Mattermost, Discord, email, Telegram, ntfy, Gotify, etc.).

## Créer un canal

Depuis la page **Notifications**, cliquez sur **Ajouter**.

| Champ | Description |
|---|---|
| **Nom** | Identifiant lisible (ex. « Slack équipe infra ») |
| **URL Apprise** | URL au format Apprise (voir ci-dessous) |
| **Canal activé** | Permet de désactiver temporairement sans supprimer |

## Format des URL Apprise

Quelques exemples courants :

### Slack

```
slack://TokenA/TokenB/TokenC
```

Créez une [Incoming Webhook](https://api.slack.com/messaging/webhooks) Slack puis extrayez les 3 tokens depuis l'URL (`hooks.slack.com/services/A/B/C`).

### Mattermost

```
mmost://hostname/AuthToken
mmost://hostname/AuthToken?channel=mon-channel
```

`AuthToken` est l'ID du webhook Mattermost (`/hooks/<token>`).

### Email

```
mailtos://user:password@smtp.gmail.com:587?from=alerts@example.com&to=admin@example.com
mailto://user:password@smtp.example.com:25
```

### Discord

```
discord://webhook_id/webhook_token
```

### Telegram

```
tgram://bot_token/chat_id
```

### ntfy

```
ntfys://ntfy.sh/mon-topic
```

### Gotify

```
gotifys://gotify.example.com/AppToken
```

La liste complète : voir [la doc Apprise](https://github.com/caronc/apprise/wiki).

## Tester un canal

Sur la page Notifications, l'icône **envoyer** (📤) à droite de chaque canal envoie un message de test avec un titre `Test — rclone-ui` et un body markdown. Le résultat (✓ Envoyé / ✗ erreur) s'affiche à côté du bouton.

## Associer un canal à une tâche

Dans le formulaire de tâche (création ou édition) :

1. Sélectionnez le canal dans la liste déroulante « Notifications »
2. Cochez les cas qui doivent déclencher une notification :

| Case | Quand |
|---|---|
| **Erreur** | La tâche échoue (après tous les retries) |
| **Succès** | La tâche se termine avec succès |
| **Ignorée** | Le cron a tenté de lancer la tâche pendant qu'elle tournait déjà |

## Format des messages

Les notifications sont envoyées **en markdown** (apprise-go convertit automatiquement vers le format du service cible).

### Échec

```markdown
**Tâche** : `<task_id>`
**Run** : `<run_id>`
**Tentatives** : 4
**Code de sortie** : `1`

**Logs** :
```
Transferred: 0 B / 0 B
[ERROR] something went wrong
```
```

Tronqué à 4000 caractères. Les 50 premières lignes lisibles sont incluses.

### Succès

```markdown
**Tâche** : `<task_id>`
**Run** : `<run_id>`

La synchronisation s'est terminée avec succès.
```

### Ignorée

```markdown
**Tâche** : `<task_id>`
**Raison** : L'exécution précédente est encore en cours

La planification cron a tenté de lancer cette tâche, mais la synchronisation précédente n'est pas terminée. L'exécution a été _ignorée_.
```

## Suppression

Un canal **ne peut pas être supprimé** s'il est référencé par au moins une tâche. Le bouton de suppression est désactivé avec une infobulle expliquant pourquoi. Désassociez d'abord les tâches concernées.
