import { useTranslation } from 'react-i18next';
import { SUPPORTED_LANGUAGES, type SupportedLanguage } from './index';

interface Props {
  value: SupportedLanguage;
  onChange: (language: SupportedLanguage) => void;
  variant?: 'sidebar' | 'select';
  ariaLabel?: string;
  className?: string;
}

export function LanguagePicker({
  value,
  onChange,
  variant = 'select',
  ariaLabel,
  className,
}: Props) {
  const { t } = useTranslation();
  const label = ariaLabel ?? t('language.label');

  if (variant === 'select') {
    return (
      <select
        value={value}
        aria-label={label}
        onChange={(event) => onChange(event.target.value as SupportedLanguage)}
        className={
          className
          ?? 'w-full border border-surface-300 rounded-lg px-3 py-2.5 text-sm focus:ring-2 focus:ring-brand-500 focus:border-brand-500 transition-shadow'
        }
      >
        {SUPPORTED_LANGUAGES.map((code) => (
          <option key={code} value={code}>
            {t(`language.native.${code}`)}
          </option>
        ))}
      </select>
    );
  }

  return (
    <div className="grid grid-cols-3 gap-1" role="radiogroup" aria-label={label}>
      {SUPPORTED_LANGUAGES.map((code) => {
        const active = value === code;
        const native = t(`language.native.${code}`);
        return (
          <button
            key={code}
            type="button"
            role="radio"
            aria-checked={active}
            aria-label={native}
            title={native}
            onClick={() => onChange(code)}
            className={`px-1.5 py-1 rounded text-[11px] font-medium transition-colors ${
              active
                ? 'bg-brand-600/20 text-brand-300'
                : 'text-surface-400 hover:bg-white/5 hover:text-white'
            }`}
          >
            {code.toUpperCase()}
          </button>
        );
      })}
    </div>
  );
}
