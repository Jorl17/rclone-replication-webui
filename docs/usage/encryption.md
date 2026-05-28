---
layout: default
title: Chiffrement
parent: Utilisation
nav_order: 3
---

# Chiffrement de la destination

Une tâche peut **chiffrer les données avant de les envoyer** vers la destination, avec un schéma
**asymétrique** (clé publique / clé privée) basé sur [age](https://age-encryption.org) (X25519).

## Modèle de sécurité

| | |
|---|---|
| **Clé publique** (`age1...`) | Stockée sur la tâche. Sert **uniquement à chiffrer**. |
| **Clé privée** (`AGE-SECRET-KEY-1...`) | **Jamais** envoyée ni stockée par l'application. Saisie à la restauration, gardée en mémoire le temps du run, puis effacée. |

Conséquence : **le serveur peut sauvegarder en automatique (cron) sans jamais pouvoir déchiffrer**.
Même s'il est entièrement compromis, un attaquant ne récupère que la clé publique — les données sur la
destination restent confidentielles.

> Le chiffrement est **hybride** : age scelle une clé de session symétrique éphémère
> (ChaCha20-Poly1305, authentifiée) pour le destinataire via X25519. Chaque fichier est chiffré
> indépendamment, **en flux** (pas de chargement complet en mémoire). Toute altération du fichier
> chiffré est détectée au déchiffrement.

## Activer le chiffrement sur une tâche

Dans le formulaire de tâche (création ou édition), section **Chiffrement de la destination** :

1. Cochez **« Chiffrer les données avant l'envoi »**.
2. Renseignez une **clé publique age** :
   - soit en cliquant sur **« Générer une paire »** (génération **côté navigateur**, la clé privée
     ne quitte jamais votre poste — voir ci-dessous) ;
   - soit en collant une clé publique age existante (`age1...`).

Les tâches chiffrées affichent un cadenas <kbd>🔒 Chiffré</kbd> dans la liste et sur leur page.

### Générer une paire de clés

Le bouton **« Générer une paire »** ouvre une fenêtre qui crée une nouvelle paire **dans le
navigateur** :

- La **clé publique** est pré-remplie dans le formulaire (et sera enregistrée).
- La **clé privée** est affichée **une seule fois**. Elle n'est **pas** transmise au serveur ni
  sauvegardée. **Copiez-la immédiatement** dans un gestionnaire de secrets : sans elle, vos
  sauvegardes sont **définitivement irrécupérables**.

Vous devez cocher « J'ai sauvegardé ma clé privée » pour valider.

## Comment fonctionne la sauvegarde chiffrée

Au lieu d'un `rclone sync`, la tâche exécute, pour chaque fichier nouveau ou modifié :

```text
rclone cat SRC:fichier  →  age (clé publique)  →  rclone rcat DST:fichier.age
```

- **Incrémental** : un manifeste (taille + date de modification par fichier) permet de ne
  re-chiffrer que ce qui a changé côté source. Les fichiers disparus de la source voient leur `.age`
  supprimé côté destination.
- **Noms de fichiers en clair** : l'arborescence est conservée, chaque fichier reçoit le suffixe
  `.age`. Le **contenu** est chiffré, pas les noms.
- Les retries et notifications fonctionnent comme pour une tâche normale ; les statistiques
  (fichiers chiffrés, inchangés, supprimés, erreurs) s'affichent dans l'historique.

## Restaurer une destination chiffrée

Depuis la liste ou la page de la tâche, cliquez sur **Restaurer**. La fenêtre demande :

| Champ | Description |
|---|---|
| **Clé privée** | Obligatoire si la destination est chiffrée. Collez la clé `AGE-SECRET-KEY-1...` ou importez un fichier de clé. **Jamais sauvegardée.** |
| **Stockage / chemin cible** | Où écrire les données déchiffrées. **Par défaut : la source d'origine** de la tâche, modifiable. |

La restauration exécute l'inverse :

```text
rclone cat DST:fichier.age  →  age -d (clé privée)  →  rclone rcat CIBLE:fichier
```

La clé privée est utilisée uniquement pendant l'exécution puis effacée de la mémoire.

## Bonnes pratiques & limites

- **Sauvegardez la clé privée hors-ligne.** C'est le seul moyen de déchiffrer. Aucune récupération
  n'est possible si elle est perdue.
- **Changer la clé publique** d'une tâche existante : les `.age` déjà écrits restent chiffrés avec
  l'**ancienne** clé (leur restauration exige l'ancienne clé privée) ; seuls les fichiers
  nouveaux/modifiés utilisent la nouvelle clé. Évitez de changer de clé en cours de route.
- **HTTPS** : exposez l'application derrière TLS — la clé privée transite (en mémoire) lors de la
  restauration.
- **Métadonnées** : tailles et noms de fichiers restent visibles côté destination (mode « noms en
  clair »). Seul le contenu est confidentiel.
