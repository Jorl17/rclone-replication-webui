---
layout: default
title: Backblaze B2
parent: Stockages
grand_parent: Utilisation
nav_order: 10
---

# Backblaze B2

Object storage économique de Backblaze. Compatible aussi avec l'API S3 (utilise alors le type [S3](s3) avec endpoint `https://s3.<region>.backblazeb2.com`).

## Côté Backblaze

### 1. Créer un bucket

1. Connecte-toi sur [secure.backblaze.com](https://secure.backblaze.com/user_signin.htm)
2. **B2 Cloud Storage** → **Buckets** → **Create a Bucket**
3. Donne un nom (globalement unique sur B2), choisis :
   - **Files in Bucket are : Private**
   - **Default Encryption : Enable** (recommandé)
   - **Object Lock : Disable** (sauf besoin spécifique)

### 2. Créer une Application Key

1. **Account** → **Application Keys** → **Add a New Application Key**
2. Configure :
   - **Name of Key** : `rclone-ui-<bucket-name>`
   - **Allow access to Bucket(s)** : sélectionne le bucket créé (limite l'accès)
   - **Type of Access** : `Read and Write`
   - **Allow List All Bucket Names** : décoche (sécurité)
   - **File name prefix** : laisse vide (ou restreint à un préfixe)
3. **Create New Key**
4. Note immédiatement le **keyID** et la **applicationKey** (la key ne sera plus affichée ensuite)

## Côté rclone-ui

| Champ | Description |
|---|---|
| **Application Key ID** | Le `keyID` (ex. `0012ab3c4d5e6f7g`) |
| **Application Key** | La clé secrète (champ sensible) |
| **Endpoint personnalisé** | Vide pour Backblaze cloud standard |
| **Suppression définitive** | Non = versioning (recommandé pour backups), Oui = suppression immédiate |

⚠️ Utilise toujours l'**Application Key ID** dans le champ « Application Key ID ». Si tu mets ton **Account ID master**, l'auth échouera.

## Chemin

Le chemin commence par le **nom du bucket**, suivi d'un préfixe optionnel :

```
mon-bucket
mon-bucket/dossier/sous-dossier
```

## Astuces

- **Versioning** : par défaut B2 conserve toutes les versions des fichiers. Avec `hard_delete=false`, supprimer un fichier le marque seulement « hidden ». Configure une **Lifecycle Rule** sur le bucket pour purger les anciennes versions après N jours.
- **--b2-versions** dans les options rclone pour lister toutes les versions
- **--b2-hard-delete** force la suppression immédiate (équivalent au toggle « Suppression définitive » du formulaire)
- **--b2-chunk-size 96M** = défaut, ne pas descendre en dessous de 5M
- **Class B/C transactions** : chaque listing/get coûte. Sur les sync incrémentales, `--fast-list` réduit le nombre d'appels.

Voir la [documentation rclone B2](https://rclone.org/b2/).
