import { useState } from 'react';
import { RotateCcw, ShieldAlert, Upload, X, Lock } from 'lucide-react';
import type { Task, RestorePayload } from '../../types/task';
import type { Remote } from '../../types/remote';

interface Props {
  open: boolean;
  task: Task;
  remotes: Remote[];
  onConfirm: (payload: RestorePayload) => void;
  onCancel: () => void;
}

/**
 * Modale de restauration : choix de la cible (par défaut la source d'origine) et, si la
 * destination est chiffrée, saisie de la clé privée (jamais sauvegardée).
 */
// Monté à l'ouverture uniquement (les parents rendent conditionnellement + `key`), donc l'état
// initialisé depuis la tâche est toujours frais et la clé privée est effacée à chaque fermeture.
export function RestoreModal({ open, task, remotes, onConfirm, onCancel }: Props) {
  const [privateKey, setPrivateKey] = useState('');
  const [targetRemoteId, setTargetRemoteId] = useState(task.source_remote_id);
  const [targetPath, setTargetPath] = useState(task.source_path);

  if (!open) return null;

  const encrypted = task.encryption_enabled;
  const keyMissing = encrypted && !privateKey.trim();

  const handleFile = async (file: File | undefined) => {
    if (!file) return;
    const text = await file.text();
    // Si c'est un fichier de clé age avec commentaires, on extrait la ligne AGE-SECRET-KEY.
    const line = text.split(/\r?\n/).find((l) => l.trim().startsWith('AGE-SECRET-KEY-'));
    setPrivateKey((line ?? text).trim());
  };

  const submit = () => {
    if (keyMissing) return;
    const payload: RestorePayload = {
      target_remote_id: targetRemoteId,
      target_path: targetPath,
    };
    if (encrypted) payload.private_key = privateKey.trim();
    onConfirm(payload);
  };

  const inputCls =
    'w-full border border-surface-300 rounded-lg px-3 py-2.5 text-sm focus:ring-2 focus:ring-brand-500 focus:border-brand-500';

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-surface-950/50 backdrop-blur-sm animate-fade-in">
      <div className="bg-white rounded-xl shadow-2xl p-6 max-w-md w-full mx-4 animate-slide-up">
        <div className="flex items-start justify-between mb-4">
          <div className="flex items-center gap-2">
            <div className="w-9 h-9 rounded-full bg-orange-100 flex items-center justify-center">
              <RotateCcw size={18} className="text-orange-600" />
            </div>
            <h2 className="text-base font-semibold text-surface-900">Restaurer</h2>
          </div>
          <button onClick={onCancel} className="text-surface-400 hover:text-surface-600 transition-colors">
            <X size={18} />
          </button>
        </div>

        <p className="text-sm text-surface-500 mb-4">
          Les données de{' '}
          <span className="font-mono text-surface-700">
            {task.dest_remote_name}:{task.dest_path}
          </span>{' '}
          seront {encrypted ? 'déchiffrées puis ' : ''}écrites vers la cible ci-dessous.
        </p>

        {encrypted && (
          <div className="mb-4 space-y-2">
            <label className="flex items-center gap-1.5 text-sm font-medium text-surface-700">
              <Lock size={13} className="text-brand-600" /> Clé privée <span className="text-red-400">*</span>
            </label>
            <textarea
              value={privateKey}
              onChange={(e) => setPrivateKey(e.target.value)}
              rows={2}
              className={`${inputCls} font-mono`}
              placeholder="AGE-SECRET-KEY-1..."
            />
            <div className="flex items-center gap-2">
              <label className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-surface-600 border border-surface-200 rounded-lg hover:bg-surface-50 cursor-pointer transition-colors">
                <Upload size={13} /> Importer un fichier
                <input type="file" className="hidden" onChange={(e) => void handleFile(e.target.files?.[0])} />
              </label>
              <span className="text-xs text-surface-400">jamais sauvegardée, utilisée le temps de la restauration</span>
            </div>
          </div>
        )}

        <div className="grid grid-cols-2 gap-3 mb-4">
          <div>
            <label className="block text-sm font-medium text-surface-700 mb-1">Stockage cible</label>
            <select value={targetRemoteId} onChange={(e) => setTargetRemoteId(e.target.value)} className={inputCls}>
              {remotes.map((r) => (
                <option key={r.id} value={r.id}>
                  {r.name} ({r.remote_type})
                </option>
              ))}
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-surface-700 mb-1">Chemin cible</label>
            <input
              value={targetPath}
              onChange={(e) => setTargetPath(e.target.value)}
              className={`${inputCls} font-mono`}
              placeholder="dossier"
            />
          </div>
        </div>

        <div className="flex items-start gap-2 text-xs text-orange-700 bg-orange-50 border border-orange-200 rounded-lg px-3 py-2.5 mb-4">
          <ShieldAlert size={16} className="shrink-0 mt-0.5" />
          <span>
            Par défaut, la cible est la source d'origine de la tâche. Cette opération peut écraser des fichiers
            existants.
          </span>
        </div>

        <div className="flex justify-end gap-2">
          <button
            onClick={onCancel}
            className="px-4 py-2 text-sm font-medium text-surface-600 border border-surface-200 rounded-lg hover:bg-surface-50 transition-colors"
          >
            Annuler
          </button>
          <button
            onClick={submit}
            disabled={keyMissing}
            className="px-4 py-2 text-sm font-medium text-white bg-orange-600 rounded-lg hover:bg-orange-700 disabled:opacity-40 transition-colors"
          >
            Restaurer
          </button>
        </div>
      </div>
    </div>
  );
}
