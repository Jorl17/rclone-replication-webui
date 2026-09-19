import { useEffect, useState } from 'react';
import { KeyRound, Copy, Check, ShieldAlert, RefreshCw, X } from 'lucide-react';
import { Trans, useTranslation } from 'react-i18next';
import { generateAgeKeypair, type AgeKeypair } from '../../lib/age';

interface Props {
  open: boolean;
  onClose: () => void;
  /** Appelé avec la clé publique lorsque l'utilisateur confirme avoir sauvegardé sa clé privée. */
  onUse: (publicKey: string) => void;
}

export function KeygenModal({ open, onClose, onUse }: Props) {
  const { t } = useTranslation();
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
      setError((e as Error).message || t('keygen.failed'));
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
    // generate is recreated each render; opening the modal is the only trigger we want.
    // eslint-disable-next-line react-hooks/exhaustive-deps
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
            <h2 className="text-base font-semibold text-surface-900">{t('keygen.title')}</h2>
          </div>
          <button onClick={onClose} className="text-surface-400 hover:text-surface-600 transition-colors">
            <X size={18} />
          </button>
        </div>

        {loading && <p className="text-sm text-surface-500 py-10 text-center animate-pulse">{t('keygen.generating')}</p>}
        {error && <p className="text-sm text-red-600 py-4">{error}</p>}

        {keys && (
          <div className="space-y-4">
            <div>
              <label className="block text-xs font-semibold text-surface-500 uppercase tracking-wider mb-1">
                {t('keygen.publicKey')}
              </label>
              <div className="flex gap-2">
                <code className="flex-1 text-xs bg-surface-50 border border-surface-200 rounded-lg px-3 py-2 break-all font-mono text-surface-700">
                  {keys.publicKey}
                </code>
                <button
                  type="button"
                  onClick={() => copy(keys.publicKey, 'pub')}
                  className="shrink-0 px-2 border border-surface-200 rounded-lg hover:bg-surface-50 transition-colors"
                  aria-label={t('keygen.copyPublic')}
                >
                  {copied === 'pub' ? <Check size={14} className="text-emerald-600" /> : <Copy size={14} className="text-surface-500" />}
                </button>
              </div>
            </div>

            <div>
              <label className="block text-xs font-semibold text-red-600 uppercase tracking-wider mb-1">
                {t('keygen.privateKey')}
              </label>
              <div className="flex gap-2">
                <code className="flex-1 text-xs bg-red-50 border border-red-200 rounded-lg px-3 py-2 break-all font-mono text-red-900">
                  {keys.privateKey}
                </code>
                <button
                  type="button"
                  onClick={() => copy(keys.privateKey, 'priv')}
                  className="shrink-0 px-2 border border-red-200 rounded-lg hover:bg-red-50 transition-colors"
                  aria-label={t('keygen.copyPrivate')}
                >
                  {copied === 'priv' ? <Check size={14} className="text-emerald-600" /> : <Copy size={14} className="text-red-500" />}
                </button>
              </div>
            </div>

            <div className="flex items-start gap-2 text-xs text-red-700 bg-red-50 border border-red-200 rounded-lg px-3 py-2.5">
              <ShieldAlert size={16} className="shrink-0 mt-0.5" />
              <span>
                <Trans i18nKey="keygen.warning" components={{ bold: <strong /> }} />
              </span>
            </div>

            <label className="flex items-center gap-2 text-sm text-surface-700 cursor-pointer">
              <input
                type="checkbox"
                checked={saved}
                onChange={(e) => setSaved(e.target.checked)}
                className="rounded border-surface-300 text-brand-600 focus:ring-brand-500"
              />
              {t('keygen.savedConfirm')}
            </label>

            <div className="flex justify-between gap-2 pt-2">
              <button
                type="button"
                onClick={() => void generate()}
                className="flex items-center gap-1.5 px-3 py-2 text-sm text-surface-600 border border-surface-200 rounded-lg hover:bg-surface-50 transition-colors"
              >
                <RefreshCw size={14} /> {t('keygen.regenerate')}
              </button>
              <button
                type="button"
                disabled={!saved}
                onClick={() => { onUse(keys.publicKey); onClose(); }}
                className="px-4 py-2 text-sm font-medium text-white bg-brand-600 rounded-lg hover:bg-brand-700 disabled:opacity-40 transition-colors"
              >
                {t('keygen.usePublic')}
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
