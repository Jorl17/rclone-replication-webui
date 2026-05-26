---
layout: default
title: Azure Blob Storage
parent: Stockages
grand_parent: Utilisation
nav_order: 6
---

# Azure Blob Storage

Object storage de Microsoft Azure. Pour les fichiers de SharePoint/OneDrive utilise plutôt les types dédiés ([SharePoint](sharepoint)).

## Côté Azure

### Méthode 1 : Account Key

1. Portail Azure → **Storage accounts** → choisis ou crée un compte
2. **Security + networking** → **Access keys** → révèle `key1` ou `key2`
3. Note **Storage account name** + **Key**

### Méthode 2 : SAS URL (recommandé pour la sécurité)

1. Portail Azure → Storage account → **Shared access signature**
2. Coche les permissions : `Read`, `Write`, `Delete`, `List`, `Add`, `Create`
3. Définis une **Start time** et **Expiry time**
4. Clique **Generate SAS and connection string**
5. Copie le `Blob service SAS URL` (commence par `https://...?sv=...`)

Le SAS expire à la date choisie — pense à le renouveler avant.

## Côté rclone-ui

| Champ | Description |
|---|---|
| **Nom du compte de stockage** | Le Storage Account Name. Vide si tu utilises SAS URL. |
| **Clé du compte** | Account key (champ sensible). Vide si SAS URL. |
| **URL SAS** | URL SAS complète, alternative à la clé. |
| **Endpoint** | Vide pour les endpoints Azure standard. Renseigne pour Azure Germany, US Gov, China ou Azurite. |

### Exemple Account Key

| Champ | Valeur |
|---|---|
| Nom du compte | `monstorage` |
| Clé du compte | `abc123...==` |
| URL SAS | (vide) |
| Endpoint | (vide) |

### Exemple SAS URL

| Champ | Valeur |
|---|---|
| Nom du compte | (vide) |
| Clé du compte | (vide) |
| URL SAS | `https://monstorage.blob.core.windows.net/mycontainer?sv=2022-11-02&ss=b&srt=co&sp=rwdlacx&se=2027-01-01T00:00:00Z&sig=...` |
| Endpoint | (vide) |

## Chemin

Le chemin commence par le **nom du container**, suivi optionnellement d'un préfixe :

```
mon-container
mon-container/backup/2026
```

## Astuces

- **--azureblob-archive-tier-delete** : permet de supprimer les blobs en `Archive` tier (sinon erreur)
- **--azureblob-access-tier=Cool** pour économiser sur le stockage rarement accédé

Voir la [documentation rclone Azure Blob](https://rclone.org/azureblob/).
