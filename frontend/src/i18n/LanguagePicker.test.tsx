import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { describe, expect, it, vi } from 'vitest';
import i18n from './index';
import { LanguagePicker } from './LanguagePicker';

describe('LanguagePicker', () => {
  it('notifies the parent when the select value changes', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(
      <I18nextProvider i18n={i18n}>
        <LanguagePicker value="fr" onChange={onChange} variant="select" ariaLabel="Langue" />
      </I18nextProvider>,
    );

    await user.selectOptions(screen.getByRole('combobox', { name: 'Langue' }), 'pt');
    expect(onChange).toHaveBeenCalledWith('pt');
  });

  it('notifies the parent when a sidebar radio is chosen', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(
      <I18nextProvider i18n={i18n}>
        <LanguagePicker value="fr" onChange={onChange} variant="sidebar" />
      </I18nextProvider>,
    );

    await user.click(screen.getByRole('radio', { name: 'English' }));
    expect(onChange).toHaveBeenCalledWith('en');
  });
});
