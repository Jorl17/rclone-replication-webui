---
layout: default
title: SFTP
parent: Stockages
grand_parent: Utilisation
nav_order: 3
---

# SFTP (SSH File Transfer Protocol)

Transfert de fichiers sécurisé sur SSH. Standard pour les serveurs Linux.

## Côté provider

### Option A : Auth par mot de passe

Sur le serveur cible, assure-toi qu'un utilisateur existe et a un mot de passe :

```bash
sudo adduser rclone-user
sudo chown -R rclone-user:rclone-user /var/data
```

Le serveur SSH doit accepter `PasswordAuthentication yes` dans `/etc/ssh/sshd_config`.

### Option B : Auth par clé SSH (recommandé)

1. Génère une paire de clés **sans passphrase** :

```bash
ssh-keygen -t ed25519 -f rclone_ed25519 -N ""
```

2. Copie la clé publique sur le serveur :

```bash
ssh-copy-id -i rclone_ed25519.pub user@serveur.example.com
```

3. La clé privée (`rclone_ed25519`) sera utilisée par rclone-ui.

## Côté rclone-ui

| Champ | Description |
|---|---|
| **Hôte** | FQDN ou IP du serveur (ex. `backup.example.com`) |
| **Port** | Défaut `22` |
| **Utilisateur** | Login SSH |
| **Mot de passe** | À remplir si auth par password — laisse vide si tu utilises une clé SSH |
| **Clé SSH (chemin)** | Chemin **absolu** vers le fichier de clé privée dans le container backend |

### Auth par clé SSH dans Docker

La clé privée doit être accessible depuis le container backend. Deux approches :

**1. Volume monté**

```yaml
backend:
  volumes:
    - ./ssh-keys:/etc/rclone/keys:ro
  environment:
    # ...
```

Dans rclone-ui, mets `Clé SSH (chemin)` = `/etc/rclone/keys/id_ed25519`.

**2. Secret Docker / Kubernetes**

Si tu utilises Docker Swarm ou K8s, monte un secret au lieu d'un volume :

```yaml
backend:
  secrets:
    - source: rclone_ssh_key
      target: /etc/rclone/keys/id_ed25519
      mode: 0400
```

## Astuces

- **--sftp-disable-hashcheck** désactive les vérifications hash (gain de perf sur serveurs lents)
- **--sftp-set-modtime=false** si le serveur ne supporte pas `setstat`
- Pour les serveurs avec des hostkeys non standards, ajoute `--sftp-known-hosts-file /etc/ssh/known_hosts` dans les options rclone

Voir la [documentation rclone SFTP](https://rclone.org/sftp/) pour la liste complète.

## Sécurité

Le champ **Mot de passe** est stocké comme champ sensible (BDD ou Secret Manager). Le champ **Clé SSH (chemin)** ne contient que le **chemin** — la clé elle-même doit être sécurisée au niveau du filesystem du container.

Conseil : utilise des clés Ed25519 dédiées à rclone, avec un user dédié sur le serveur cible et des permissions minimales (`/var/data/...`).
