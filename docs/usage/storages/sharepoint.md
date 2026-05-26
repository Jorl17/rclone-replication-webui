---
layout: default
title: SharePoint
parent: Stockages
grand_parent: Utilisation
nav_order: 7
---

# SharePoint (Microsoft 365)

Bibliothèques de documents SharePoint Online. Utilise le backend rclone `onedrive` avec `drive_type=documentLibrary` et le flow OAuth **client_credentials** (app-only, sans utilisateur interactif).

## Côté Azure / SharePoint

### 1. Créer une App Registration Azure AD

1. Portail Azure → **Microsoft Entra ID** → **App registrations** → **+ New registration**
2. Donne un nom (ex. `rclone-ui-sharepoint`)
3. Account types : **Accounts in this organizational directory only**
4. Pas de Redirect URI nécessaire
5. **Register**
6. Note **Application (client) ID** et **Directory (tenant) ID**

### 2. Générer un Client Secret

1. Onglet **Certificates & secrets** → **New client secret**
2. Durée : 24 mois max
3. Note la **Value** (visible une seule fois)

### 3. Donner les permissions API

1. Onglet **API permissions** → **+ Add a permission** → **Microsoft Graph**
2. **Application permissions** (pas Delegated)
3. Cherche et coche :
   - `Sites.ReadWrite.All` (ou `Sites.Selected` pour limiter à certains sites)
   - `Files.ReadWrite.All` (optionnel selon les besoins)
4. **Add permissions**
5. **Grant admin consent for <tenant>** ← important, à faire par un admin

### 4. Trouver le Drive ID

Le `drive_id` est l'identifiant de la bibliothèque de documents SharePoint.

**Via Graph Explorer** ([developer.microsoft.com/graph](https://developer.microsoft.com/en-us/graph/graph-explorer)) :

1. Connecte-toi avec un compte admin
2. Récupère le site ID :
   ```
   GET https://graph.microsoft.com/v1.0/sites/<tenant>.sharepoint.com:/sites/<site-name>
   ```
   Réponse → champ `id`, format `<hostname>,<site-guid>,<web-guid>`
3. Liste les drives du site :
   ```
   GET https://graph.microsoft.com/v1.0/sites/<site-id>/drives
   ```
   Réponse → tableau `value`, chaque entrée a un `id` (le drive_id). Le nom est dans `name` (souvent `Documents`).

## Côté rclone-ui

| Champ | Description |
|---|---|
| **Tenant ID** | GUID du tenant Azure AD ou domaine (`contoso.onmicrosoft.com`) |
| **Client ID** | Application ID de l'app Azure |
| **Client Secret** | Secret généré (sensible) |
| **Drive ID du site SharePoint** | ID du drive obtenu via Graph |
| **Région** | Global, Allemagne, US Gov, ou Chine |

Le backend ajoute automatiquement `type=onedrive`, `drive_type=documentLibrary` et `client_credentials=true` lors de la génération de la config rclone.

## Chemin

Le chemin est relatif à la racine de la bibliothèque de documents :

```
/                     ← racine
/Partage/Backups
/Documents partagés/2026
```

## Limites SharePoint

- **Fichiers modifiés silencieusement** : SharePoint peut altérer les fichiers (changement d'encoding, ajout de metadata). Sur les sync sensibles, ajoute `--ignore-checksum --ignore-size`
- **Erreurs "item not found"** lors d'overwrites : utilise `--backup-dir` pour conserver l'ancien fichier dans un dossier dédié
- **Throttling** Microsoft : si tu hites des `429 Too Many Requests`, baisse `--transfers` (défaut 4) à 2

Voir la [documentation rclone OneDrive/SharePoint](https://rclone.org/onedrive/#sharepoint).
