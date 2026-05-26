---
layout: default
title: FTP
parent: Stockages
grand_parent: Utilisation
nav_order: 4
---

# FTP

Protocole historique. Préférer SFTP quand c'est possible.

⚠️ **FTP transmet les credentials en clair**. À éviter en dehors d'un réseau privé. Pour du transit chiffré, utilise FTP-S (FTP over TLS) — supporté via les options rclone avancées.

## Côté provider

Configure un serveur FTP comme `vsftpd`, `proftpd` ou utilise un service hébergé (OVH, IONOS, etc.). Crée un utilisateur avec accès au répertoire cible.

```bash
sudo adduser rclone-ftp
sudo passwd rclone-ftp
sudo mkdir -p /srv/ftp/rclone
sudo chown rclone-ftp:rclone-ftp /srv/ftp/rclone
```

## Côté rclone-ui

| Champ | Description |
|---|---|
| **Hôte** | `ftp.example.com` |
| **Port** | Défaut `21` |
| **Utilisateur** | Login |
| **Mot de passe** | Password (champ sensible) |

## FTP-S (TLS)

Pour activer FTP over TLS, ajoute dans **Options rclone supplémentaires** de la tâche :

```
--ftp-tls --ftp-explicit-tls
```

Voir la [documentation rclone FTP](https://rclone.org/ftp/).
