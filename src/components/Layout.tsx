import type { ReactNode } from "react";
import { NavLink } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import type { IconProp } from "@fortawesome/fontawesome-svg-core";
import { ROUTES } from "@/config/app";
import { LanguageSwitcher } from "./LanguageSwitcher";

interface NavEntry {
  to: string;
  icon: IconProp;
  labelKey: string;
}

const NAV: NavEntry[] = [
  { to: ROUTES.main, icon: "pen-to-square", labelKey: "nav.main" },
  { to: ROUTES.history, icon: "clock-rotate-left", labelKey: "nav.history" },
  { to: ROUTES.health, icon: "heart-pulse", labelKey: "nav.health" },
  { to: ROUTES.settings, icon: "gear", labelKey: "nav.settings" },
];

export function Layout({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-name">{t("app.name")}</span>
        </div>
        <nav className="nav">
          {NAV.map((entry) => (
            <NavLink
              key={entry.to}
              to={entry.to}
              className={({ isActive }) => (isActive ? "nav-item active" : "nav-item")}
              end={entry.to === ROUTES.main}
            >
              <FontAwesomeIcon icon={entry.icon} fixedWidth />
              <span>{t(entry.labelKey)}</span>
            </NavLink>
          ))}
        </nav>
        <div className="sidebar-footer">
          <LanguageSwitcher />
        </div>
      </aside>
      <main className="content">{children}</main>
    </div>
  );
}
