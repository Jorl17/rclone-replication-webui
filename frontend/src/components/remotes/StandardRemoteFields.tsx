import { UseFormRegister, UseFormWatch, UseFormSetValue, FieldErrors } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import type { RemoteFieldDef } from '../../config/remoteFieldSchemas';
import { OAuthButton } from './OAuthButton';

interface Props {
  fields: RemoteFieldDef[];
  /** Type rclone (drive, dropbox, onedrive...) pour piloter l'affichage du bouton OAuth */
  remoteType: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  register: UseFormRegister<any>;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  watch: UseFormWatch<any>;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  setValue: UseFormSetValue<any>;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  errors: FieldErrors<any>;
  /** En édition, les champs sensibles peuvent être laissés vides pour conserver la valeur stockée */
  isEdit?: boolean;
}

const inputCls = "w-full border border-surface-300 rounded-lg px-3 py-2.5 text-sm focus:ring-2 focus:ring-brand-500 focus:border-brand-500 transition-shadow";

function translatedLabel(exists: boolean, value: unknown, fallback: string): string {
  if (!exists || typeof value !== 'string' || value.length === 0) return fallback;
  return value;
}

/** Mapping des types rclone supportant le flow OAuth intégré. */
const OAUTH_PROVIDERS: Record<string, string> = {
  dropbox: 'Dropbox',
  drive: 'Google Drive',
  onedrive: 'OneDrive',
};

export function StandardRemoteFields({ fields, remoteType, register, watch, setValue, errors, isEdit }: Props) {
  const { t, i18n } = useTranslation();
  const oauthLabel = OAUTH_PROVIDERS[remoteType];
  const clientId = (watch('config.client_id') ?? '') as string;
  const clientSecret = (watch('config.client_secret') ?? '') as string;

  return (
    <fieldset className="border border-surface-200 rounded-lg p-4 space-y-4">
      <legend className="text-xs font-semibold text-surface-500 uppercase tracking-wider px-1">
        {t('remotes.form.connectionParams')}
      </legend>

      {oauthLabel && (
        <OAuthButton
          provider={remoteType}
          label={oauthLabel}
          clientId={clientId}
          clientSecret={clientSecret}
          onSuccess={(tokenJson) => {
            setValue('config.token', tokenJson, { shouldDirty: true, shouldValidate: true });
          }}
        />
      )}

      {fields.map((field) => {
        const fieldError = (errors.config as Record<string, { message?: string }> | undefined)?.[field.key];
        const isPassword = field.type === 'password';
        const labelKey = `remotes.types.${remoteType}.fields.${field.key}.label`;
        const helpKey = `remotes.types.${remoteType}.fields.${field.key}.help`;
        const placeholderKey = `remotes.types.${remoteType}.fields.${field.key}.placeholder`;
        const label = translatedLabel(i18n.exists(labelKey), t(labelKey), field.label);
        const help = translatedLabel(i18n.exists(helpKey), t(helpKey), field.helpText ?? '');
        const placeholder = translatedLabel(i18n.exists(placeholderKey), t(placeholderKey), field.placeholder ?? '');
        const editPlaceholder = isPassword && isEdit ? t('remotes.form.keepSecret') : placeholder || undefined;

        return (
          <div key={field.key}>
            <label className="block text-sm font-medium text-surface-700 mb-1">
              {label}
              {field.required && !(isEdit && isPassword) && <span className="text-red-400 ml-0.5">*</span>}
            </label>

            {field.type === 'select' ? (
              <select {...register(`config.${field.key}`)} className={inputCls}>
                {field.options?.map((opt) => (
                  <option key={opt.value} value={opt.value}>
                    {translatedLabel(
                      i18n.exists(`remotes.types.${remoteType}.fields.${field.key}.options.${opt.value}`),
                      t(`remotes.types.${remoteType}.fields.${field.key}.options.${opt.value}`),
                      opt.label,
                    )}
                  </option>
                ))}
              </select>
            ) : (
              <input
                {...register(`config.${field.key}`)}
                type={isPassword ? 'password' : 'text'}
                inputMode={field.type === 'number' ? 'numeric' : undefined}
                placeholder={editPlaceholder}
                className={inputCls}
                autoComplete={isPassword ? 'new-password' : undefined}
              />
            )}

            {help && <p className="text-xs text-surface-400 mt-1">{help}</p>}
            {isEdit && isPassword && (
              <p className="text-xs text-amber-600 mt-1">
                {t('remotes.form.keepSecretHelp')}
              </p>
            )}
            {fieldError?.message && <p className="text-red-500 text-xs mt-1">{fieldError.message}</p>}
          </div>
        );
      })}
    </fieldset>
  );
}
