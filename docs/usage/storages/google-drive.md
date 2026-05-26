---
layout: default
title: Google Drive
parent: Stockages
grand_parent: Utilisation
nav_order: 8
---

# Google Drive

Deux méthodes d'authentification disponibles :

| Méthode | Cas d'usage |
|---|---|
| **OAuth 2.0** (bouton intégré) | Drive personnel ou Workspace, utilisateur interactif |
| **Service Account** | Backup automatisé Workspace sans utilisateur, Shared Drives |

## Méthode 1 : OAuth 2.0 (recommandé pour Drive personnel)

### Côté Google Cloud Console

1. Va sur [console.cloud.google.com](https://console.cloud.google.com) → choisis ou crée un projet
2. **API & Services** → **Library** → active **Google Drive API**
3. **API & Services** → **OAuth consent screen** → configure :
   - User Type : **External** (sauf si tu es sur Workspace, alors **Internal**)
   - Scopes : ajoute `.../auth/drive` (drive complet) ou `.../auth/drive.file` (fichiers créés par rclone seulement)
   - **Publish app** (sinon limité à 100 utilisateurs en mode Testing)
4. **API & Services** → **Credentials** → **+ Create Credentials** → **OAuth client ID**
   - Application type : **Web application**
   - Authorized redirect URIs : `https://<ton-app>/api/oauth/callback`
5. Note **Client ID** et **Client Secret**

### Côté rclone-ui

| Champ | Action |
|---|---|
| **Client ID OAuth** | Colle le Client ID |
| **Client Secret OAuth** | Colle le Client Secret |
| Bouton **Connecter via OAuth (Google Drive)** | Clique-le. Une popup s'ouvre → connecte ton compte Google → accepte les permissions |

Le champ **Token OAuth** se remplit automatiquement après la popup. C'est terminé.

| Champ optionnel | Description |
|---|---|
| **Shared Drive ID** | Pour cibler un Shared Drive plutôt que My Drive |
| **Dossier racine** | ID Google d'un sous-dossier (depuis l'URL après `/folders/`) |
| **Étendue d'accès (scope)** | Drive (défaut), drive.readonly, drive.file, etc. |

## Méthode 2 : Service Account (recommandé pour Workspace automatisé)

### Côté Google Cloud Console

1. Console → projet → **IAM & Admin** → **Service Accounts** → **Create service account**
2. Nom : `rclone-ui-bot`. Pas de rôle global nécessaire
3. Onglet **Keys** → **Add Key** → **Create new key** → **JSON** → télécharger
4. Si tu cibles un **Shared Drive** :
   - Ouvre le Shared Drive dans Drive web
   - **Partager** → ajoute l'email du service account (format `xxx@xxx.iam.gserviceaccount.com`) avec rôle Manager
5. Si tu cibles **My Drive** d'un utilisateur Workspace :
   - Active **Domain-wide delegation** sur le service account
   - Dans Google Workspace Admin → Sécurité → API controls → ajoute le scope `https://www.googleapis.com/auth/drive`

### Côté rclone-ui

| Champ | Action |
|---|---|
| **Service Account (JSON)** | Colle le contenu **complet** du fichier JSON téléchargé (champ sensible) |
| **Shared Drive ID** | ID du Shared Drive (si applicable) |
| **Dossier racine** | ID d'un sous-dossier (optionnel) |

Laisse les champs **Token OAuth**, **Client ID** et **Client Secret** vides.

## Trouver les IDs

- **Shared Drive ID** : ouvre le Shared Drive, copie depuis l'URL `https://drive.google.com/drive/folders/<ID>`
- **Dossier racine** : ouvre le dossier, copie depuis l'URL `https://drive.google.com/drive/folders/<ID>`

## Chemin

Le chemin est relatif à la racine choisie (My Drive ou root_folder_id) :

```
backups
backups/2026/01
photos
```

## Astuces

- **--drive-server-side-across-configs** permet les copies serveur-à-serveur entre deux remotes Drive (rapide, pas de download)
- **--drive-chunk-size 64M** accélère l'upload des gros fichiers
- **Quota** : 750 Go/jour pour les comptes Workspace. Si tu hites ça, splittes en plusieurs tâches sur plusieurs jours.

Voir la [documentation rclone Google Drive](https://rclone.org/drive/).
