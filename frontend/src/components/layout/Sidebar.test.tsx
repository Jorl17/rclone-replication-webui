import { cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import i18n from '../../i18n';
import { Sidebar } from './Sidebar';

afterEach(() => {
  cleanup();
});

describe('Sidebar brand', () => {
  beforeEach(async () => {
    await i18n.changeLanguage('en');
  });

  it('takes the logo block to the main page with a pointer cursor', async () => {
    const user = userEvent.setup();
    render(
      <I18nextProvider i18n={i18n}>
        <MemoryRouter initialEntries={['/remotes']}>
          <Routes>
            <Route path="/" element={<div>home-page</div>} />
            <Route path="/remotes" element={<Sidebar />} />
          </Routes>
        </MemoryRouter>
      </I18nextProvider>,
    );

    const brand = screen.getByRole('link', { name: /rclone-ui/i });
    expect(brand).toHaveAttribute('href', '/');
    expect(brand).toHaveClass('cursor-pointer');
    expect(brand).toHaveTextContent('File replication');

    await user.click(brand);
    expect(screen.getByText('home-page')).toBeInTheDocument();
  });
});
