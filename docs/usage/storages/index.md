---
layout: default
title: Stockages
parent: Utilisation
nav_order: 1
has_children: true
permalink: /usage/storages/
---

# Stockages distants

Un stockage représente un emplacement source ou destination dans rclone. L'app propose **9 types standard** avec formulaires guidés, et un éditeur **clé/valeur libre** pour les autres backends rclone.

## Types standard

Chacun a sa propre page avec les étapes de configuration côté provider **et** côté app :

| Type | Authentification | Page |
|---|---|---|
| **Local** | Aucune | [Local](local) |
| **Amazon S3** (et compatibles) | Access Key + Secret | [S3](s3) |
| **SFTP** | SSH (clé ou password) | [SFTP](sftp) |
| **FTP** | User / password | [FTP](ftp) |
| **SMB** (Windows Share) | User / password | [SMB](smb) |
| **Azure Blob Storage** | Account key ou SAS URL | [Azure Blob](azure-blob) |
| **SharePoint** | App Azure AD (client credentials) | [SharePoint](sharepoint) |
| **Google Drive** | OAuth 2.0 ou Service Account | [Google Drive](google-drive) |
| **Dropbox** | OAuth 2.0 | [Dropbox](dropbox) |
| **Backblaze B2** | Application Key | [Backblaze B2](backblaze-b2) |

## Types avancés

Pour tous les autres backends supportés par rclone (Azure Files, OneDrive, OpenStack Swift, WebDAV, HTTP, etc.), choisis le type correspondant dans la liste **Types avancés** du formulaire. Tu auras alors un éditeur clé/valeur libre où tu peux saisir n'importe quelle option de la documentation rclone.

Exemple pour OneDrive :

| Clé | Valeur |
|---|---|
| `token` | `{"access_token":"...", ...}` |
| `client_id` | `12345...` |
| `client_secret` | `...` |

## Astuces communes

### Chemins relatifs

Le **chemin** d'une tâche est concaténé à la racine du stockage. Par exemple :

- Stockage Local avec `root = /mnt/backups`
- Chemin de tâche : `important/`
- Résultat rclone : `local:/mnt/backups/important/`

### Compteur de références

La liste des stockages affiche une colonne **Utilisé par** indiquant le nombre de tâches référençant chaque stockage. Un stockage **ne peut pas être supprimé** s'il est utilisé : le bouton de suppression est désactivé avec une infobulle explicative.

### Tester un stockage

L'icône **Wi-Fi** (📶) à droite de chaque ligne exécute `rclone lsd <remote>:` et affiche le nombre de top-level entries trouvés. C'est un sanity check de connectivité.
