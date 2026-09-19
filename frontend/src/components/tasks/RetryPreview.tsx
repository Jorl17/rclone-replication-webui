import { Trans, useTranslation } from 'react-i18next';

interface Props {
  attempts: number;
  delaySeconds: number;
}

export function RetryPreview({ attempts, delaySeconds }: Props) {
  const { t } = useTranslation();
  if (attempts <= 0) return null;

  const schedule = Array.from({ length: Math.min(attempts, 3) }, (_, i) =>
    t('tasks.retry.attempt', { n: i + 2, seconds: delaySeconds * (i + 1) }),
  ).join(', ') + (attempts > 3 ? ', ...' : '');

  return (
    <p className="text-xs text-surface-500 bg-surface-50 rounded-lg px-3 py-2">
      <Trans
        i18nKey="tasks.retry.preview"
        count={attempts}
        values={{ schedule }}
        components={{ bold: <strong /> }}
      />
    </p>
  );
}
