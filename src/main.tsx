import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./App.css";
import { HashRouter } from "react-router-dom";
import { GlobalDialog } from "./components/GlobalDialog";
import { GlobalLoading } from "./components/GlobalLoading";
ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <HashRouter>
      <>
        <GlobalLoading />
        <GlobalDialog />
        <App />
      </>
    </HashRouter>
  </React.StrictMode>,
);
