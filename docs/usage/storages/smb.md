---
layout: default
title: SMB (Windows Share)
parent: Stockages
grand_parent: Utilisation
nav_order: 5
---

# SMB / CIFS (Windows Share)

Protocole de partage de fichiers Windows. Aussi utilisé par Samba sur Linux, et par les NAS (Synology, QNAP, TrueNAS).

## Côté provider

### Windows Server

1. **Computer Management** → **System Tools** → **Local Users and Groups** → créer un utilisateur dédié
2. Sur le dossier à partager, **Properties** → **Sharing** → **Advanced Sharing** → cocher "Share this folder", **Permissions** : donner Full Control à l'utilisateur créé
3. Noter l'IP du serveur ou son hostname AD

### Samba (Linux)

`/etc/samba/smb.conf` :

```ini
[backups]
   path = /srv/backups
   read only = no
   valid users = rclone-user
   create mask = 0644
   directory mask = 0755
```

```bash
sudo smbpasswd -a rclone-user
sudo systemctl restart smbd
```

### NAS (Synology / QNAP)

Crée un utilisateur dédié + un share, et donne-lui les droits R/W sur ce share.

## Côté rclone-ui

| Champ | Description |
|---|---|
| **Hôte** | IP ou hostname du serveur SMB (ex. `192.168.1.100`) |
| **Port** | Défaut `445` |
| **Utilisateur** | Login SMB |
| **Mot de passe** | Password (champ sensible) |
| **Domaine** | Domaine Active Directory si applicable. Vide pour SMB autonome ou utiliser `WORKGROUP` |

## Chemin

Le chemin commence par le **nom du share**, suivi optionnellement d'un sous-dossier :

```
backups
backups/2026
documents/factures
```

## Astuces

- Si tu obtiens `NT_STATUS_LOGON_FAILURE` : vérifie le domaine et le format du username (parfois `DOMAIN\user` est requis dans le champ utilisateur sur les vieux serveurs)
- **--smb-idle-timeout 1m** pour économiser des connexions sur de gros sync

Voir la [documentation rclone SMB](https://rclone.org/smb/).
