---
layout: default
title: Dropbox
parent: Stockages
grand_parent: Utilisation
nav_order: 9
---

# Dropbox

Authentification OAuth 2.0 obligatoire. Le bouton intégré dans l'UI gère le flow complet.

## Côté Dropbox

### Créer une App Dropbox

1. Connecte-toi sur [dropbox.com/developers/apps](https://www.dropbox.com/developers/apps)
2. **Create app**
3. Choisis le **type d'API** :
   - **Scoped access** (recommandé)
4. Choisis le **type d'accès** :
   - **App folder** : isole les données dans un dossier `/Apps/<nom-app>`
   - **Full Dropbox** : accès à tout le compte
5. Donne un nom à l'app, valide

### Configurer les permissions

1. Dans l'app, onglet **Permissions** → coche au minimum :
   - `files.content.read`
   - `files.content.write`
   - `files.metadata.read`
   - `files.metadata.write`
2. Clique **Submit**

### Récupérer App key + App secret + Redirect URI

1. Onglet **Settings** de l'app
2. Note **App key** et **App secret** (cliquer "Show" pour le secret)
3. **OAuth 2 → Redirect URIs** → ajoute :
   ```
   https://<ton-app>/api/oauth/callback
   ```

## Côté rclone-ui

| Champ | Action |
|---|---|
| **App key (Client ID)** | Colle l'App key |
| **App secret (Client Secret)** | Colle l'App secret |
| Bouton **Connecter via OAuth (Dropbox)** | Clique. Popup → connexion Dropbox → autoriser |

Après la popup, le champ **Token OAuth (JSON)** est rempli automatiquement et le token est marqué comme champ sensible.

| Champ optionnel | Description |
|---|---|
| **Impersonate (Dropbox Business)** | Email d'un utilisateur à impersonner. Uniquement pour Dropbox Business avec un compte admin et l'option `team_member_file_access` |

## Chemin

Le chemin est relatif à la racine de l'app (cas **App folder**) ou de la racine Dropbox (cas **Full Dropbox**) :

```
/Documents
/Photos/2026
```

## Astuces

- **Refresh token** : le token JSON contient un `refresh_token` que rclone utilise pour rafraîchir automatiquement l'access token. Une fois autorisé, ça dure indéfiniment (sauf révocation manuelle).
- **--dropbox-chunk-size 128M** accélère l'upload des gros fichiers (limite Dropbox : 150 Mo par chunk)
- **--dropbox-shared-files** pour accéder aux fichiers partagés avec toi (read-only)

Voir la [documentation rclone Dropbox](https://rclone.org/dropbox/).
