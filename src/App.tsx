import { Routes, Route, useNavigate } from "react-router-dom";
import SettingsPage from "./pages/Settings";
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { GetInvoiceTct } from "./pages/GetInvoiceTct";
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

function App() {
  const navigate = useNavigate();
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen("tray-navigate", (event) => {
      const { route, data } = event.payload as {
        route: string;
        data: any;
      };
      navigate(route, { state: data });
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [navigate]);

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

  return (
    <Routes>
      <Route path="/" element={<SplashScreen />} />
      <Route path="/blank" element={<BlankPage />} />
      <Route path="/report" element={<PreviewReport />} />
      <Route path="/settings" element={<SettingsPage />} />
      <Route path="/login" element={<LoginPage />} />
      <Route path="/get-invoice-tct" element={<GetInvoiceTct />} />
      <Route path="/update" element={<UpdatePage />} />
      <Route path="/quit" element={<QuitPage />} />
    </Routes>
  );
}

export default App;