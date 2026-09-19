import { CheckCircle2, XCircle, Loader2, MinusCircle, HelpCircle, SkipForward } from 'lucide-react';
import { useTranslation } from 'react-i18next';

type Status = 'success' | 'failure' | 'running' | 'disabled' | 'skipped' | 'unknown';

const config: Record<Status, { bg: string; text: string; labelKey: string; Icon: typeof CheckCircle2 }> = {
  success:  { bg: 'bg-emerald-50', text: 'text-emerald-700', labelKey: 'status.success', Icon: CheckCircle2 },
  failure:  { bg: 'bg-red-50', text: 'text-red-700', labelKey: 'status.failure', Icon: XCircle },
  running:  { bg: 'bg-brand-50', text: 'text-brand-700', labelKey: 'status.running', Icon: Loader2 },
  skipped:  { bg: 'bg-amber-50', text: 'text-amber-700', labelKey: 'status.skipped', Icon: SkipForward },
  disabled: { bg: 'bg-surface-100', text: 'text-surface-500', labelKey: 'status.disabled', Icon: MinusCircle },
  unknown:  { bg: 'bg-surface-100', text: 'text-surface-500', labelKey: 'status.unknown', Icon: HelpCircle },
};

export function StatusBadge({ status }: { status: Status }) {
  const { t } = useTranslation();
  const { bg, text, labelKey, Icon } = config[status] ?? config.unknown;
  return (
    <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium ${bg} ${text}`}>
      <Icon size={12} className={status === 'running' ? 'animate-spin' : ''} />
      {t(labelKey)}
    </span>
  );
}
