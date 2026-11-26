import React from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import { applyTheme, getInitialTheme } from "./hooks/useTheme";
import "./i18n";
import "./styles.css";
import "./theme.css";
import { ToastProvider } from "./components/ToastProvider";

applyTheme(getInitialTheme());

createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ToastProvider><App /></ToastProvider>
  </React.StrictMode>,
);