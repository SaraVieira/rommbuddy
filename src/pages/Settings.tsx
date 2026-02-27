import { useState } from "react";
import RetroArchTab from "../components/settings/RetroArchTab";
import EmulatorsTab from "../components/settings/EmulatorsTab";
import IntegrationsTab from "../components/settings/IntegrationsTab";
import DatFilesTab from "../components/settings/DatFilesTab";

const TABS = ["retroarch", "emulators", "integrations", "dat"] as const;
type Tab = (typeof TABS)[number];

const TAB_LABELS: Record<Tab, string> = {
  retroarch: "RetroArch",
  emulators: "Emulators",
  integrations: "Integrations",
  dat: "Dat Files",
};

export default function Settings() {
  const [activeTab, setActiveTab] = useState<Tab>("retroarch");

  return (
    <div className="page">
      <h1 className="font-display text-page-title font-bold text-text-primary mb-md uppercase">
        Settings
      </h1>
      <p className="text-body text-text-muted mb-xl">
        Configure emulators, integrations, and metadata.
      </p>

      <div className="flex items-center gap-0 border-b border-border mb-xl">
        {TABS.map((tab) => (
          <button
            key={tab}
            className={`px-xl py-md text-label font-mono uppercase tracking-wide border-b-2 transition-colors ${
              activeTab === tab
                ? "border-accent text-accent"
                : "border-transparent text-text-muted hover:text-text-secondary"
            }`}
            onClick={() => setActiveTab(tab)}
          >
            {TAB_LABELS[tab]}
          </button>
        ))}
      </div>

      {activeTab === "retroarch" && <RetroArchTab />}
      {activeTab === "emulators" && <EmulatorsTab />}
      {activeTab === "integrations" && <IntegrationsTab />}
      {activeTab === "dat" && <DatFilesTab />}
    </div>
  );
}
