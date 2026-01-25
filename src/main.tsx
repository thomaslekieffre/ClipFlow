import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { RegionOverlay } from "./components/overlay/RegionOverlay";
import "./styles.css";

function Root() {
  const path = window.location.pathname;

  if (path === "/overlay") {
    return <RegionOverlay />;
  }

  return <App />;
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
);
