import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./App.css";
import { HashRouter } from "react-router-dom";
import { GlobalDialog } from "./components/GlobalDialog";
import { GlobalLoading } from "./components/GlobalLoading";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 1000 * 60 * 5,
      refetchOnWindowFocus: false,
    },
    mutations: {
      retry: 0,
    },
  },
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <HashRouter>
      <QueryClientProvider client={queryClient}>
        <GlobalLoading />
        <GlobalDialog />
        <App />
      </QueryClientProvider>
    </HashRouter>
  </React.StrictMode>,
);
