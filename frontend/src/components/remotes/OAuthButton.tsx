import { useState } from 'react';
import { KeyRound, Loader2 } from 'lucide-react';

interface Props {
  /** Type rclone : 'dropbox', 'drive', 'onedrive' */
  provider: string;
  /** Label du provider affiché à l'utilisateur */
  label: string;
  clientId: string;
  clientSecret: string;
  onSuccess: (tokenJson: string) => void;
}

/**
 * Bouton qui ouvre une popup pour le flow OAuth 2.0 et récupère le token
 * via window.postMessage envoyé par la page de callback du backend.
 */
export function OAuthButton({ provider, label, clientId, clientSecret, onSuccess }: Props) {
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleClick = () => {
    setError(null);
    if (!clientId || !clientSecret) {
      setError('Renseignez d\'abord le Client ID et le Client Secret avant de connecter.');
      return;
    }

    const params = new URLSearchParams({
      provider,
      client_id: clientId,
      client_secret: clientSecret,
    });
    const url = `/api/oauth/start?${params.toString()}`;

    const popup = window.open(url, 'rclone-ui-oauth', 'width=600,height=720,scrollbars=yes');
    if (!popup) {
      setError('Impossible d\'ouvrir la fenêtre OAuth. Autorisez les popups pour ce site.');
      return;
    }

    setPending(true);

    const onMessage = (e: MessageEvent) => {
      if (e.data?.type !== 'rclone-ui-oauth') return;
      window.removeEventListener('message', onMessage);
      window.clearInterval(closedCheck);
      setPending(false);
      if (e.data.success) {
        onSuccess(JSON.stringify(e.data.token));
      } else {
        setError(e.data.error || 'Échec du flow OAuth.');
      }
    };
    window.addEventListener('message', onMessage);

    // Détecte la fermeture manuelle de la popup
    const closedCheck = window.setInterval(() => {
      if (popup.closed) {
        window.removeEventListener('message', onMessage);
        window.clearInterval(closedCheck);
        setPending(false);
      }
    }, 500);
  };

  return (
    <div className="bg-brand-50 border border-brand-200 rounded-lg p-3 mb-3">
      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={handleClick}
          disabled={pending}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-brand-600 text-white rounded-lg text-sm font-medium hover:bg-brand-700 disabled:opacity-50 transition-colors"
        >
          {pending ? <Loader2 size={14} className="animate-spin" /> : <KeyRound size={14} />}
          Connecter via OAuth ({label})
        </button>
        <p className="text-xs text-brand-700">
          Auth interactive — remplit le champ « Token » automatiquement.
        </p>
      </div>
      {error && <p className="text-xs text-red-600 mt-2">{error}</p>}
    </div>
  );
}
