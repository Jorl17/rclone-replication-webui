import { Link, NavLink } from 'react-router-dom';
import { Server, RefreshCw, Bell } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { LanguageSwitcher } from '../../i18n/LanguageSwitcher';

export function Sidebar() {
  const { t } = useTranslation();
  const links = [
    { to: '/tasks', label: t('nav.tasks'), desc: t('nav.tasksDesc'), icon: RefreshCw },
    { to: '/remotes', label: t('nav.remotes'), desc: t('nav.remotesDesc'), icon: Server },
    { to: '/notifications', label: t('nav.notifications'), desc: t('nav.notificationsDesc'), icon: Bell },
  ];

  return (
    <aside className="w-60 shrink-0 bg-surface-900 text-white flex flex-col min-h-screen">
      <Link
        to="/"
        className="block px-5 py-5 border-b border-white/10 cursor-pointer text-inherit no-underline hover:bg-white/5 transition-colors"
      >
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-lg bg-brand-600 flex items-center justify-center shadow-lg shadow-brand-600/25">
            <RefreshCw size={15} className="text-white" />
          </div>
          <div>
            <span className="font-semibold text-sm tracking-tight">{t('app.name')}</span>
            <p className="text-[10px] text-surface-500 leading-tight">{t('app.tagline')}</p>
          </div>
        </div>
      </Link>

      <nav className="flex-1 px-3 py-4">
        <p className="px-2 mb-2.5 text-[10px] font-semibold uppercase tracking-widest text-surface-500">
          {t('nav.section')}
        </p>
        <ul className="space-y-0.5">
          {links.map(({ to, label, desc, icon: Icon }) => (
            <li key={to}>
              <NavLink
                to={to}
                className={({ isActive }) =>
                  `flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm transition-all duration-150 group ${
                    isActive
                      ? 'bg-brand-600/20 text-brand-300 font-medium'
                      : 'text-surface-400 hover:bg-white/5 hover:text-white'
                  }`
                }
              >
                <Icon size={17} className="shrink-0" />
                <div className="min-w-0">
                  <span className="block leading-tight">{label}</span>
                  <span className="block text-[10px] text-surface-500 group-hover:text-surface-400 leading-tight">
                    {desc}
                  </span>
                </div>
              </NavLink>
            </li>
          ))}
        </ul>
      </nav>

      <div className="px-5 py-4 border-t border-white/10 space-y-3">
        <LanguageSwitcher />
        <p className="text-[10px] text-surface-600">
          {t('app.madeBy')}{' '}
          <a href="https://sixmon.net" target="_blank" rel="noopener noreferrer" className="text-surface-400 hover:text-brand-400 transition-colors">Sixmon</a>
          {' '}&{' '}
          <a href="https://claude.ai" target="_blank" rel="noopener noreferrer" className="text-surface-400 hover:text-brand-400 transition-colors">Claude</a>
        </p>
      </div>
    </aside>
  );
}
