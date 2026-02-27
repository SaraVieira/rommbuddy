import React from "react";
import ReactDOM from "react-dom/client";
import { createHashRouter, RouterProvider } from "react-router-dom";
import App from "./App";
import Library from "./pages/Library";
import Platforms from "./pages/Platforms";
import Search from "./pages/Search";
import Sources from "./pages/Sources";
import Settings from "./pages/Settings";
import RomDetailPage from "./pages/RomDetailPage";
import "./index.css";

const router = createHashRouter([
  {
    path: "/",
    element: <App />,
    children: [
      { index: true, element: <Library /> },
      { path: "platforms", element: <Platforms /> },
      { path: "search", element: <Search /> },
      { path: "sources", element: <Sources /> },
      { path: "settings", element: <Settings /> },
      { path: "rom/:id", element: <RomDetailPage /> },
    ],
  },
]);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>
);
