---
layout: default
title: Secret Manager
parent: Utilisation
nav_order: 5
---

# Secret Manager

Par défaut, les credentials sensibles (mots de passe, clés API, tokens OAuth) sont stockés **dans la base de données**, dans le champ `config` JSONB de la table `remotes`.

Pour les déploiements production, tu peux les **déporter vers un Secret Manager externe**. Le backend choisit alors automatiquement entre 7 providers selon la variable `SECRET_MANAGER_PROVIDER`.

## Pourquoi

- **Audit** : qui a accédé à quel secret, quand
- **Rotation** : changer un secret sans toucher à la BDD
- **Séparation** : la BDD ne contient plus que des données métier
- **IAM** : les permissions de lecture/écriture sont gérées par le provider

## Comment ça marche

Au démarrage du backend :

1. Si `SECRET_MANAGER_PROVIDER` est défini, le backend instancie le provider correspondant
2. Il scanne tous les remotes existants et **migre automatiquement** leurs champs sensibles vers le Secret Manager, puis les retire de la BDD
3. À chaque execution rclone, les secrets sont récupérés depuis le SM et fusionnés avec la config publique pour générer le fichier `rclone.conf` temporaire

Le mapping `remote_id → secret` est :

| Provider | Nom du secret |
|---|---|
| Scaleway | `<remote_id>` dans le path `/rclone-ui` |
| AWS | `rclone-ui/<remote_id>` |
| Azure Key Vault | `rclone-ui-<remote_id>` |
| GCP | `rclone-ui-<remote_id>` |
| Vault | `secret/data/rclone-ui/<remote_id>` |
| Infisical | `RCLONE_UI_<UUID_SNAKE_CASE>` |
| Doppler | `RCLONE_UI_<UUID_NO_DASH>` |

La valeur stockée est un JSON `{"<field_name>": "<value>", ...}` avec uniquement les champs marqués sensibles.

## Champs considérés sensibles

Par type de remote :

| Type | Champs |
|---|---|
| `s3` | `secret_access_key` |
| `sftp` / `ftp` / `smb` | `pass` |
| `azureblob` | `key`, `sas_url` |
| `sharepoint` | `client_secret` |
| `drive` | `service_account_credentials`, `client_secret`, `token` |
| `dropbox` | `token`, `client_secret` |
| `b2` | `key` |
| Types avancés | détection heuristique sur le nom (`pass`, `secret`, `token`, `key`, `auth`, `sas_url`, `credential`) |

## Édition d'un remote avec secrets stockés

Quand tu ouvres un remote existant en édition, les champs sensibles affichent un placeholder `•••••• (laisser vide pour conserver)` :

- **Laisse vide** : le secret stocké reste inchangé
- **Remplis-le** : le nouveau secret remplace l'ancien dans le SM

## Configuration par provider

### Scaleway Secret Manager

1. Console Scaleway → **Secret Manager** → activer dans le projet (région `fr-par` recommandée)
2. **IAM** → créer une API key avec le permission set `SecretManagerFullAccess`
3. Variables d'env :

```env
SECRET_MANAGER_PROVIDER=scaleway
SCW_SECRET_KEY=<la secret key, UUID>
SCW_PROJECT_ID=<UUID du projet>
SCW_DEFAULT_REGION=fr-par
SCW_SECRET_PATH=/rclone-ui
```

### AWS Secrets Manager

1. Console AWS → IAM → créer une policy avec :
   ```json
   {
     "Version": "2012-10-17",
     "Statement": [{
       "Effect": "Allow",
       "Action": [
         "secretsmanager:GetSecretValue",
         "secretsmanager:CreateSecret",
         "secretsmanager:PutSecretValue",
         "secretsmanager:DeleteSecret"
       ],
       "Resource": "arn:aws:secretsmanager:*:*:secret:rclone-ui/*"
     }]
   }
   ```
2. Attache cette policy à un utilisateur IAM ou un rôle (EC2/ECS/EKS/IRSA)
3. Variables d'env :

```env
SECRET_MANAGER_PROVIDER=aws
AWS_REGION=eu-west-1
AWS_ACCESS_KEY_ID=AKIA...
AWS_SECRET_ACCESS_KEY=...
AWS_SECRET_PREFIX=rclone-ui/
```

### Azure Key Vault

1. Azure Portal → **Key Vaults** → créer un vault
2. **App Registrations** → créer une app, noter le `tenant_id` et `client_id`
3. **Certificates & secrets** → créer un Client Secret
4. Sur le Key Vault, **Access policies** → autoriser l'app pour `Get / Set / Delete` sur les secrets
5. Variables d'env :

```env
SECRET_MANAGER_PROVIDER=azure
AZURE_TENANT_ID=...
AZURE_CLIENT_ID=...
AZURE_CLIENT_SECRET=...
AZURE_VAULT_URL=https://myvault.vault.azure.net
```

### Google Cloud Secret Manager

1. Console GCP → API Library → activer **Secret Manager API**
2. IAM → créer un Service Account avec le rôle `Secret Manager Admin`
3. Télécharger la clé JSON et l'exposer via `GOOGLE_APPLICATION_CREDENTIALS`
4. Variables d'env :

```env
SECRET_MANAGER_PROVIDER=gcp
GCP_PROJECT_ID=mon-projet-gcp
GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json
```

### HashiCorp Vault

Vault doit avoir le secrets engine **KV v2** activé (par défaut sur `secret/`).

1. Crée un token avec une policy autorisant `create/read/update/delete` sur `secret/data/rclone-ui/*`
2. Variables d'env :

```env
SECRET_MANAGER_PROVIDER=vault
VAULT_ADDR=http://vault.local:8200
VAULT_TOKEN=hvs.CAES...
VAULT_MOUNT_PATH=secret
VAULT_PATH_PREFIX=rclone-ui
```

### Infisical

1. Crée un projet sur [infisical.com](https://infisical.com) (ou self-host)
2. **Access Control** → **Identities** → créer une Universal Auth identity
3. Note `clientId` et `clientSecret`
4. Variables d'env :

```env
SECRET_MANAGER_PROVIDER=infisical
INFISICAL_HOST=https://app.infisical.com
INFISICAL_CLIENT_ID=...
INFISICAL_CLIENT_SECRET=...
INFISICAL_PROJECT_ID=...
INFISICAL_ENVIRONMENT=prod
INFISICAL_SECRET_PATH=/rclone-ui
```

### Doppler

1. Crée un projet sur [doppler.com](https://doppler.com)
2. **Access** → **Service Tokens** → créer un token sur le config voulu (`prd`, `dev`, etc.)
3. Variables d'env :

```env
SECRET_MANAGER_PROVIDER=doppler
DOPPLER_TOKEN=dp.st.prd...
DOPPLER_PROJECT=mon-projet
DOPPLER_CONFIG=prd
```

## Désactiver

Mets `SECRET_MANAGER_PROVIDER=none` (ou retire la variable). Les nouveaux remotes stockeront leurs credentials en BDD. **Attention** : les secrets déjà migrés restent dans le SM et le backend ne saura plus les retrouver tant que tu ne réactives pas le provider.
