import { useTranslation } from 'react-i18next';
import { LanguagePicker } from './LanguagePicker';
import { persistLanguage, resolveLanguage } from './index';

export function LanguageSwitcher() {
  const { i18n, t } = useTranslation();
  const current = resolveLanguage(i18n.resolvedLanguage ?? i18n.language);

  return (
    <div className="space-y-1.5">
      <p className="text-[10px] font-semibold uppercase tracking-widest text-surface-500">
        {t('language.label')}
      </p>
      <LanguagePicker
        value={current}
        onChange={(code) => {
          void persistLanguage(code);
        }}
        variant="sidebar"
        ariaLabel={t('language.label')}
      />
    </div>
  );
}
