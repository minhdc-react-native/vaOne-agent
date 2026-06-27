import { Routes, Route, Navigate, useNavigate } from "react-router-dom";
import SettingsPage from "./pages/Settings";
import LoginPage from "./pages/LoginPage";
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { GetInvoiceTct } from "./pages/GetInvoiceTct";

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

  return (
    <Routes>
      <Route path="/" element={<Navigate to="/settings" replace />} />
      <Route path="/settings" element={<SettingsPage />} />
      <Route path="/login" element={<LoginPage />} />
      <Route path="/get-invoice-tct" element={<GetInvoiceTct />} />
    </Routes>
  );
}

export default App;