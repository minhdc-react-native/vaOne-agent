import { Routes, Route, useNavigate } from "react-router-dom";
import SettingsPage from "./pages/Settings";
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { GetInvoices } from "./pages/GetInvoices";
import { SplashScreen } from "./pages/Splash";
import LoginPage from "./pages/LoginPage";
import { BlankPage } from "./pages/BlankPage";
import { PreviewReport } from "./pages/PreviewReport";
import QuitPage from "./pages/QuitPage";
import { UpdatePage } from "./pages/Update";
import {
  enable,
  isEnabled,
} from "@tauri-apps/plugin-autostart";
import { useAppStore } from "./stores/app.store";
import { invoke } from "@tauri-apps/api/core";
import { TaxConfig } from "./pages/TaxConfig";
import { TaxInvoiceManager } from "./features/TaxInvoiceManager";
import ProgressTask from "./pages/ProgressTask";

function App() {
  const navigate = useNavigate();
  const setOpenOverlay = useAppStore(store => store.setOpenOverlay);
  const setSyncProgress = useAppStore(store => store.setSyncProgress);

  useEffect(() => {
    let unlisteners: (() => void)[] = [];

    const setupListeners = async () => {
      const listeners = await Promise.all([
        listen("tray-navigate", (event) => {
          const { route, data } = event.payload as {
            route: string;
            data: any;
          };

          navigate(route, { state: data });
        }),

        listen("modal_closed", () => {
          setOpenOverlay(false);
        }),

        listen<{
          completed: number;
          total: number;
          item: any;
        }>("sync-progress", (event) => {
          console.log(event.payload);
          setSyncProgress(event.payload);
        }),
      ]);

      unlisteners = listeners;
    };

    setupListeners();

    return () => {
      unlisteners.forEach(unlisten => unlisten());
    };
  }, [navigate, setOpenOverlay, setSyncProgress]);

  // useUpdater();
  const autostartInitialized = useAppStore(store => store.autostartInitialized);
  const setAutostartInitialized = useAppStore(store => store.setAutostartInitialized);
  useEffect(() => {
    if (autostartInitialized) return;

    (async () => {
      try {
        const enabled = await isEnabled();

        if (!enabled) {
          await enable();
        }

        setAutostartInitialized(true);
      } catch (err) {
        console.error("Failed to initialize autostart:", err);
      }
    })();
  }, [autostartInitialized, setAutostartInitialized]);

  const login = useAppStore(s => s.login);

  useEffect(() => {
    if (!login) return;
    invoke("connect_invoice", {
      login,
    });
  }, [login]);
  return (
    <Routes>
      <Route path="/" element={<SplashScreen />} />
      <Route path="/blank" element={<BlankPage />} />
      <Route path="/report" element={<PreviewReport />} />
      <Route path="/settings" element={<SettingsPage />} />
      <Route path="/tax-config/:taxCode?" element={<TaxConfig />} />
      <Route path="/history-invoice/:taxCode" element={<TaxInvoiceManager />} />
      <Route path="/login" element={<LoginPage />} />
      <Route path="/get-invoices" element={<GetInvoices />} />
      <Route path="/progress-task" element={<ProgressTask />} />
      <Route path="/update" element={<UpdatePage />} />
      <Route path="/quit" element={<QuitPage />} />
    </Routes>
  );
}

export default App;