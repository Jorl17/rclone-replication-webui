import { useEffect, useRef, useState } from 'react';

interface ProgressState {
  lines: string[];
  done: boolean;
  status: string | null;
}

/**
 * S'abonne au flux SSE de progression d'une tâche.
 *
 * Le backend renvoie d'abord toutes les lignes déjà accumulées (rattrapage des logs émis
 * avant la connexion), puis les nouvelles lignes en temps réel jusqu'à l'événement `done`.
 *
 * Cela permet de récupérer les logs live même après un rechargement de page, ou pour une
 * tâche déclenchée par cron sans interaction utilisateur préalable.
 *
 * @param taskId  - UUID de la tâche à suivre
 * @param running - true si la tâche est en cours (d'après le cache API)
 * @param forceConnect - true pour forcer la connexion sans attendre `running`
 */
export function useTaskProgress(taskId: string | null, running: boolean, forceConnect = false) {
  const [progress, setProgress] = useState<ProgressState>({ lines: [], done: false, status: null });
  const esRef = useRef<EventSource | null>(null);

  const shouldConnect = running || forceConnect;

  useEffect(() => {
    if (!taskId || !shouldConnect) return;

    // Réinitialisation volontaire de l'état à chaque (re)connexion / changement de tâche.
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setProgress({ lines: [], done: false, status: null });
    const es = new EventSource(`/api/tasks/${taskId}/progress`);
    esRef.current = es;

    es.addEventListener('log', (e) => {
      try {
        const data = JSON.parse(e.data);
        setProgress(prev => ({ ...prev, lines: [...prev.lines, data.line] }));
      } catch { /* ignore */ }
    });

    es.addEventListener('done', (e) => {
      try {
        const data = JSON.parse(e.data);
        setProgress(prev => ({ ...prev, done: true, status: data.status }));
      } catch { /* ignore */ }
      es.close();
    });

    es.addEventListener('idle', () => {
      // La tâche n'est pas (ou plus) en cours. On ferme la connexion proprement.
      setProgress(prev => ({ ...prev, done: true }));
      es.close();
    });

    es.onerror = () => es.close();

    return () => {
      es.close();
      esRef.current = null;
    };
  }, [taskId, shouldConnect]);

  return progress;
}
