import React from "react";
import ReactDOM from "react-dom/client";
import { Kryton } from "@azrtydxb/core";
import App from "./App";

console.log("Kryton class:", Kryton);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
