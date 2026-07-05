import { HashRouter, Routes, Route } from "react-router-dom";
import { Layout } from "./components/Layout";
import { MainPage } from "./pages/MainPage";
import { HistoryPage } from "./pages/HistoryPage";
import { HealthPage } from "./pages/HealthPage";
import { SettingsPage } from "./pages/SettingsPage";
import { ROUTES } from "./config/app";

// デスクトップアプリのためURLルーティングではなくハッシュルータを用いる。
export default function App() {
  return (
    <HashRouter>
      <Layout>
        <Routes>
          <Route path={ROUTES.main} element={<MainPage />} />
          <Route path={ROUTES.history} element={<HistoryPage />} />
          <Route path={ROUTES.health} element={<HealthPage />} />
          <Route path={ROUTES.settings} element={<SettingsPage />} />
        </Routes>
      </Layout>
    </HashRouter>
  );
}
