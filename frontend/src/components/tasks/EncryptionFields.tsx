import { useState } from 'react';
import { Lock, KeyRound, Info } from 'lucide-react';
import { KeygenModal } from './KeygenModal';

interface Props {
  enabled: boolean;
  publicKey: string;
  onEnabledChange: (v: boolean) => void;
  onPublicKeyChange: (v: string) => void;
  error?: string | null;
}

/**
 * Section de formulaire réutilisable pour activer le chiffrement asymétrique de la destination
 * et saisir / générer la clé publique age.
 */
export function EncryptionFields({ enabled, publicKey, onEnabledChange, onPublicKeyChange, error }: Props) {
  const [keygenOpen, setKeygenOpen] = useState(false);
  const inputCls =
    'w-full border border-surface-300 rounded-lg px-3 py-2.5 text-sm focus:ring-2 focus:ring-brand-500 focus:border-brand-500 transition-shadow';

  return (
    <fieldset className="border border-surface-200 rounded-lg p-4 space-y-3">
      <legend className="text-xs font-semibold text-surface-500 uppercase tracking-wider px-1 flex items-center gap-1.5">
        <Lock size={12} /> Chiffrement de la destination
      </legend>

      <label className="flex items-start gap-2.5 cursor-pointer">
        <input
          type="checkbox"
          checked={enabled}
          onChange={(e) => onEnabledChange(e.target.checked)}
          className="mt-0.5 rounded border-surface-300 text-brand-600 focus:ring-brand-500"
        />
        <span className="text-sm text-surface-700">
          Chiffrer les données avant l'envoi (age / X25519)
          <span className="block text-xs text-surface-400">
            Le serveur ne détient que la clé publique : il chiffre mais ne peut pas déchiffrer. La clé privée
            n'est demandée qu'au moment de la restauration.
          </span>
        </span>
      </label>

      {enabled && (
        <div className="pl-7 space-y-2">
          <label className="block text-sm font-medium text-surface-700">
            Clé publique <span className="text-red-400">*</span>
          </label>
          <textarea
            value={publicKey}
            onChange={(e) => onPublicKeyChange(e.target.value)}
            rows={2}
            className={`${inputCls} font-mono`}
            placeholder="age1..."
          />
          <div className="flex flex-wrap items-center gap-2">
            <button
              type="button"
              onClick={() => setKeygenOpen(true)}
              className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-brand-700 bg-brand-50 border border-brand-200 rounded-lg hover:bg-brand-100 transition-colors"
            >
              <KeyRound size={13} /> Générer une paire
            </button>
            <span className="text-xs text-surface-400">ou collez une clé publique age existante</span>
          </div>
          <p className="flex items-start gap-1 text-xs text-surface-400">
            <Info size={12} className="shrink-0 mt-0.5" />
            Conservez précieusement la clé privée correspondante : sans elle, les données chiffrées sont
            définitivement irrécupérables.
          </p>
          {error && <p className="text-red-500 text-xs">{error}</p>}
        </div>
      )}

      <KeygenModal open={keygenOpen} onClose={() => setKeygenOpen(false)} onUse={(pub) => onPublicKeyChange(pub)} />
    </fieldset>
  );
}
