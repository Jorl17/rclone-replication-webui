import { useEffect, useState } from 'react';
import { KeyRound, Copy, Check, ShieldAlert, RefreshCw, X } from 'lucide-react';
import { generateAgeKeypair, type AgeKeypair } from '../../lib/age';

interface Props {
  open: boolean;
  onClose: () => void;
  /** Appelé avec la clé publique lorsque l'utilisateur confirme avoir sauvegardé sa clé privée. */
  onUse: (publicKey: string) => void;
}

export function KeygenModal({ open, onClose, onUse }: Props) {
  const [keys, setKeys] = useState<AgeKeypair | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const [copied, setCopied] = useState<'pub' | 'priv' | null>(null);

  const generate = async () => {
    setLoading(true);
    setError(null);
    setSaved(false);
    setKeys(null);
    try {
      setKeys(await generateAgeKeypair());
    } catch (e) {
      setError((e as Error).message || 'Échec de la génération de la clé');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (open) {
      void generate();
    } else {
      setKeys(null);
      setSaved(false);
      setError(null);
      setCopied(null);
    }
  }, [open]);

  if (!open) return null;

  const copy = async (text: string, which: 'pub' | 'priv') => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(which);
      setTimeout(() => setCopied(null), 1500);
    } catch {
      /* clipboard indisponible */
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-surface-950/50 backdrop-blur-sm animate-fade-in">
      <div className="bg-white rounded-xl shadow-2xl p-6 max-w-lg w-full mx-4 animate-slide-up">
        <div className="flex items-start justify-between mb-4">
          <div className="flex items-center gap-2">
            <div className="w-9 h-9 rounded-full bg-brand-100 flex items-center justify-center">
              <KeyRound size={18} className="text-brand-600" />
            </div>
            <h2 className="text-base font-semibold text-surface-900">Générer une paire de clés</h2>
          </div>
          <button onClick={onClose} className="text-surface-400 hover:text-surface-600 transition-colors">
            <X size={18} />
          </button>
        </div>

        {loading && <p className="text-sm text-surface-500 py-10 text-center animate-pulse">Génération en cours...</p>}
        {error && <p className="text-sm text-red-600 py-4">{error}</p>}

        {keys && (
          <div className="space-y-4">
            <div>
              <label className="block text-xs font-semibold text-surface-500 uppercase tracking-wider mb-1">
                Clé publique (sera enregistrée sur la tâche)
              </label>
              <div className="flex gap-2">
                <code className="flex-1 text-xs bg-surface-50 border border-surface-200 rounded-lg px-3 py-2 break-all font-mono text-surface-700">
                  {keys.publicKey}
                </code>
                <button
                  type="button"
                  onClick={() => copy(keys.publicKey, 'pub')}
                  className="shrink-0 px-2 border border-surface-200 rounded-lg hover:bg-surface-50 transition-colors"
                  aria-label="Copier la clé publique"
                >
                  {copied === 'pub' ? <Check size={14} className="text-emerald-600" /> : <Copy size={14} className="text-surface-500" />}
                </button>
              </div>
            </div>

            <div>
              <label className="block text-xs font-semibold text-red-600 uppercase tracking-wider mb-1">
                Clé privée (NON enregistrée — sauvegardez-la maintenant)
              </label>
              <div className="flex gap-2">
                <code className="flex-1 text-xs bg-red-50 border border-red-200 rounded-lg px-3 py-2 break-all font-mono text-red-900">
                  {keys.privateKey}
                </code>
                <button
                  type="button"
                  onClick={() => copy(keys.privateKey, 'priv')}
                  className="shrink-0 px-2 border border-red-200 rounded-lg hover:bg-red-50 transition-colors"
                  aria-label="Copier la clé privée"
                >
                  {copied === 'priv' ? <Check size={14} className="text-emerald-600" /> : <Copy size={14} className="text-red-500" />}
                </button>
              </div>
            </div>

            <div className="flex items-start gap-2 text-xs text-red-700 bg-red-50 border border-red-200 rounded-lg px-3 py-2.5">
              <ShieldAlert size={16} className="shrink-0 mt-0.5" />
              <span>
                Cette clé privée est la <strong>seule</strong> façon de déchiffrer vos sauvegardes. Elle n'est{' '}
                <strong>pas conservée</strong> par l'application et ne sera <strong>plus jamais affichée</strong>.
                Stockez-la dans un gestionnaire de secrets.
              </span>
            </div>

            <label className="flex items-center gap-2 text-sm text-surface-700 cursor-pointer">
              <input
                type="checkbox"
                checked={saved}
                onChange={(e) => setSaved(e.target.checked)}
                className="rounded border-surface-300 text-brand-600 focus:ring-brand-500"
              />
              J'ai sauvegardé ma clé privée en lieu sûr
            </label>

            <div className="flex justify-between gap-2 pt-2">
              <button
                type="button"
                onClick={() => void generate()}
                className="flex items-center gap-1.5 px-3 py-2 text-sm text-surface-600 border border-surface-200 rounded-lg hover:bg-surface-50 transition-colors"
              >
                <RefreshCw size={14} /> Régénérer
              </button>
              <button
                type="button"
                disabled={!saved}
                onClick={() => { onUse(keys.publicKey); onClose(); }}
                className="px-4 py-2 text-sm font-medium text-white bg-brand-600 rounded-lg hover:bg-brand-700 disabled:opacity-40 transition-colors"
              >
                Utiliser cette clé publique
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
