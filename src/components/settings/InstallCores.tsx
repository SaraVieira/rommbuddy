import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { CoreInfo } from "../../types";
import { useAppToast } from "../../App";

interface InstallCoresProps {
  retroarchPath: string;
  onCoresChanged: (cores: CoreInfo[]) => void;
}

export default function InstallCores({
  retroarchPath,
  onCoresChanged,
}: InstallCoresProps) {
  const toast = useAppToast();

  const [availableCores, setAvailableCores] = useState<CoreInfo[]>([]);
  const [loadingAvailable, setLoadingAvailable] = useState(false);
  const [installingCore, setInstallingCore] = useState<string | null>(null);
  const [coreSearch, setCoreSearch] = useState("");

  const handleLoadAvailable = async () => {
    setLoadingAvailable(true);
    try {
      const available: CoreInfo[] = await invoke("get_available_cores", {
        retroarchPath,
      });
      setAvailableCores(available);
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setLoadingAvailable(false);
    }
  };

  const handleInstallCore = async (coreName: string) => {
    setInstallingCore(coreName);
    try {
      await invoke("install_core", { retroarchPath, coreName });
      toast(`Installed ${coreName}`, "success");
      const detected: CoreInfo[] = await invoke("detect_cores", {
        retroarchPath,
      });
      onCoresChanged(detected);
      setAvailableCores((prev) => prev.filter((c) => c.core_name !== coreName));
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setInstallingCore(null);
    }
  };

  const filteredCores = availableCores.filter((core) => {
    const label = core.display_name || core.core_name;
    return label.toLowerCase().includes(coreSearch.toLowerCase());
  });

  return (
    <section className="mt-3xl">
      <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">
        // Install Cores
      </h2>
      <div className="card">
        {availableCores.length === 0 ? (
          <button
            className="btn btn-secondary"
            onClick={handleLoadAvailable}
            disabled={loadingAvailable}
          >
            {loadingAvailable ? "Loading..." : "Load Available Cores"}
          </button>
        ) : (
          <>
            <div className="flex items-center gap-lg mb-lg">
              <input
                type="text"
                className="flex-1 px-[10px] py-[6px] rounded-none border border-border bg-bg-elevated text-text-primary font-mono text-body focus:border-accent outline-none"
                placeholder="Search cores..."
                value={coreSearch}
                onChange={(e) => setCoreSearch(e.target.value)}
              />
              <span className="text-text-muted text-nav whitespace-nowrap">
                {filteredCores.length} cores available
              </span>
            </div>
            <div className="max-h-[400px] overflow-y-auto flex flex-col gap-xs">
              {filteredCores.map((core) => (
                <div
                  key={core.core_name}
                  className="flex items-center justify-between py-[6px] px-lg rounded-none hover:bg-bg-elevated"
                >
                  <span className="text-body text-text-primary">
                    {core.display_name || core.core_name}
                  </span>
                  <button
                    className="btn btn-primary btn-sm"
                    onClick={() => handleInstallCore(core.core_name)}
                    disabled={installingCore !== null}
                  >
                    {installingCore === core.core_name
                      ? "Installing..."
                      : "Install"}
                  </button>
                </div>
              ))}
            </div>
          </>
        )}
      </div>
    </section>
  );
}
