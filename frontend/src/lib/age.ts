export interface AgeKeypair {
  /** Clé publique partageable (« recipient »), commence par `age1`. */
  publicKey: string;
  /** Clé privée secrète (« identity »), commence par `AGE-SECRET-KEY-1`. À ne jamais transmettre au serveur. */
  privateKey: string;
}

/**
 * Génère une paire de clés age (X25519) entièrement côté navigateur.
 * La clé privée ne quitte jamais le poste de l'utilisateur.
 *
 * La bibliothèque `age-encryption` (et ses dépendances cryptographiques) est chargée
 * dynamiquement, uniquement quand l'utilisateur génère une clé — ça la sort du bundle principal.
 */
export async function generateAgeKeypair(): Promise<AgeKeypair> {
  const age = await import('age-encryption');
  const privateKey = await age.generateIdentity();
  const publicKey = await age.identityToRecipient(privateKey);
  return { publicKey, privateKey };
}
