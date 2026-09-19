import { CronExpressionParser } from 'cron-parser';
import { Calendar, CheckCircle2, XCircle } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { formatCronHuman } from '../../i18n';
import { formatDateTime } from '../../i18n/format';

interface Props {
  expression: string;
}

/**
 * Détecte le format d'une expression cron :
 * - 5 champs : format Unix standard (minute heure jour mois j.semaine)
 * - 6 champs : format étendu avec secondes en premier
 * - @macro   : raccourcis comme @hourly, @daily, @weekly...
 */
function detectFormat(expr: string): '5-fields' | '6-fields' | 'macro' | 'unknown' {
  const trimmed = expr.trim();
  if (trimmed.startsWith('@')) return 'macro';
  const count = trimmed.split(/\s+/).length;
  if (count === 5) return '5-fields';
  if (count === 6) return '6-fields';
  return 'unknown';
}

/**
 * Affiche un aperçu d'une expression cron :
 * - Validation (valide / invalide)
 * - Traduction localisée
 * - 3 prochaines exécutions prévues
 *
 * Supporte les formats 5 champs (standard Unix) et 6 champs (avec secondes).
 */
export function CronPreview({ expression }: Props) {
  const { t, i18n } = useTranslation();
  const trimmed = expression.trim();
  if (!trimmed) return null;

  const format = detectFormat(trimmed);
  const language = i18n.resolvedLanguage ?? i18n.language;

  let humanReadable: string | null = null;
  let error: string | null = null;
  try {
    humanReadable = formatCronHuman(trimmed, language);
  } catch (e) {
    error = e instanceof Error ? e.message : t('cron.invalidFallback');
  }

  let nextRuns: Date[] = [];
  if (!error) {
    try {
      const it = CronExpressionParser.parse(trimmed, { tz: 'Europe/Paris' });
      for (let i = 0; i < 3; i++) {
        nextRuns.push(it.next().toDate());
      }
    } catch (e) {
      error = e instanceof Error ? e.message : t('cron.invalidFallback');
      nextRuns = [];
    }
  }

  if (error) {
    return (
      <div className="mt-2 flex items-start gap-2 text-xs bg-red-50 border border-red-200 rounded-lg px-3 py-2">
        <XCircle size={14} className="shrink-0 mt-0.5 text-red-500" />
        <div>
          <p className="font-medium text-red-700">{t('cron.invalidTitle')}</p>
          <p className="text-red-600 mt-0.5">{error}</p>
        </div>
      </div>
    );
  }

  const formatLabel =
    format === '5-fields' ? t('cron.format5') :
    format === '6-fields' ? t('cron.format6') :
    format === 'macro' ? t('cron.formatMacro') : '';

  return (
    <div className="mt-2 bg-emerald-50 border border-emerald-200 rounded-lg px-3 py-2 space-y-1.5">
      <div className="flex items-start gap-2 text-xs">
        <CheckCircle2 size={14} className="shrink-0 mt-0.5 text-emerald-600" />
        <p className="text-emerald-800">
          <span className="font-medium">{t('cron.valid')}</span>
          {formatLabel && (
            <span className="ml-1.5 px-1.5 py-0.5 rounded bg-emerald-100 text-emerald-700 text-[10px] font-mono">
              {formatLabel}
            </span>
          )}
          {' — '}{humanReadable}
        </p>
      </div>
      {nextRuns.length > 0 && (
        <div className="flex items-start gap-2 text-xs pt-1 border-t border-emerald-200/70">
          <Calendar size={14} className="shrink-0 mt-0.5 text-emerald-600" />
          <div className="min-w-0">
            <p className="font-medium text-emerald-800 mb-0.5">{t('cron.nextRuns')}</p>
            <ul className="text-emerald-700 space-y-0.5">
              {nextRuns.map((d, i) => (
                <li key={i} className="font-mono">
                  {formatDateTime(d, language, { dateStyle: 'medium', timeStyle: 'short' })}
                </li>
              ))}
            </ul>
          </div>
        </div>
      )}
    </div>
  );
}
