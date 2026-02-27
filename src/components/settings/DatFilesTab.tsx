import { useState, useEffect, useCallback } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  PlatformWithCount,
  ScanProgress,
  DatFileInfo,
  DatDetectResult,
  VerificationStats,
} from "../../types";
import { useAppToast } from "../../App";
import ProgressBar from "./ProgressBar";
import PlatformDialog from "./PlatformDialog";

export default function DatFilesTab() {
  const toast = useAppToast();

  const [platforms, setPlatforms] = useState<PlatformWithCount[]>([]);
  const [datFiles, setDatFiles] = useState<DatFileInfo[]>([]);
  const [importingDat, setImportingDat] = useState(false);
  const [datProgress, setDatProgress] = useState<ScanProgress | null>(null);
  const [verifying, setVerifying] = useState(false);
  const [verifyProgress, setVerifyProgress] = useState<ScanProgress | null>(
    null,
  );

  const [showPlatformDialog, setShowPlatformDialog] = useState(false);
  const [pendingDatPath, setPendingDatPath] = useState<string | null>(null);
  const [pendingDatHeaderName, setPendingDatHeaderName] = useState("");

  const loadPlatforms = useCallback(async () => {
    try {
      const p: PlatformWithCount[] = await invoke("get_platforms_with_counts");
      setPlatforms(p);
    } catch (e) {
      console.error("Failed to load platforms:", e);
    }
  }, []);

  const loadDatFiles = useCallback(async () => {
    try {
      const files: DatFileInfo[] = await invoke("get_dat_files");
      setDatFiles(files);
    } catch (e) {
      console.error("Failed to load DAT files:", e);
    }
  }, []);

  useEffect(() => {
    loadPlatforms();
    loadDatFiles();
  }, [loadPlatforms, loadDatFiles]);

  const doImportDat = async (filePath: string, platformSlug: string) => {
    setImportingDat(true);
    setDatProgress(null);
    try {
      const channel = new Channel<ScanProgress>();
      channel.onmessage = (p) => setDatProgress(p);
      const datType = filePath.toLowerCase().includes("redump")
        ? "redump"
        : "no-intro";
      await invoke("import_dat_file", {
        filePath,
        datType,
        platformSlug,
        channel,
      });
      toast("DAT file imported!", "success");
      loadDatFiles();
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setImportingDat(false);
      setDatProgress(null);
    }
  };

  const handleImportDat = async () => {
    const selected = await open({
      directory: false,
      multiple: false,
      title: "Select DAT file",
      filters: [{ name: "DAT Files", extensions: ["dat", "xml"] }],
    });
    if (!selected) return;

    try {
      const result: DatDetectResult = await invoke("detect_dat_platform", {
        filePath: selected as string,
      });
      if (!result.detected_slug) {
        setPendingDatPath(selected as string);
        setPendingDatHeaderName(result.header_name);
        setShowPlatformDialog(true);
        return;
      }
      await doImportDat(selected as string, result.detected_slug);
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const handlePlatformSelect = async (slug: string) => {
    setShowPlatformDialog(false);
    if (pendingDatPath) {
      await doImportDat(pendingDatPath, slug);
      setPendingDatPath(null);
    }
  };

  const handleRemoveDat = async (id: number) => {
    try {
      await invoke("remove_dat_file", { datFileId: id });
      setDatFiles((prev) => prev.filter((d) => d.id !== id));
      toast("DAT file removed", "success");
    } catch (e) {
      toast(String(e), "error");
    }
  };

  const handleVerify = async () => {
    setVerifying(true);
    setVerifyProgress(null);
    try {
      const channel = new Channel<ScanProgress>();
      channel.onmessage = (p) => setVerifyProgress(p);
      const stats: VerificationStats = await invoke("verify_library", {
        platformId: null,
        channel,
      });
      toast(
        `Verified ${stats.verified}, Unverified ${stats.unverified}, Bad Dumps ${stats.bad_dump}`,
        "success",
      );
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setVerifying(false);
      setVerifyProgress(null);
    }
  };

  return (
    <>
      <section>
        <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">
          // Dat File Management
        </h2>
        <div className="card">
          <p className="text-body text-text-muted mb-lg">
            Import No-Intro and Redump DAT files to verify your ROMs against
            known-good databases. When a platform cannot be auto-detected, you
            will be asked to select it manually.
          </p>
          <div className="flex items-center gap-lg mb-lg">
            <button
              className="btn btn-primary"
              disabled={importingDat}
              onClick={handleImportDat}
            >
              {importingDat ? "Importing..." : "Import DAT File"}
            </button>
            <span className="text-text-muted text-nav">
              {datFiles.length} DAT file{datFiles.length !== 1 ? "s" : ""}{" "}
              imported
            </span>
          </div>
          {datProgress && <ProgressBar progress={datProgress} />}
          {datFiles.length > 0 && (
            <table className="w-full border-collapse">
              <thead>
                <tr>
                  <th className="text-left p-md px-lg text-nav font-medium text-text-muted uppercase tracking-wide border-b border-border">
                    Name
                  </th>
                  <th className="text-left p-md px-lg text-nav font-medium text-text-muted uppercase tracking-wide border-b border-border">
                    Platform
                  </th>
                  <th className="text-right p-md px-lg text-nav font-medium text-text-muted uppercase tracking-wide border-b border-border">
                    Entries
                  </th>
                  <th
                    className="p-md px-lg border-b border-border"
                    style={{ width: 60 }}
                  ></th>
                </tr>
              </thead>
              <tbody>
                {datFiles.map((dat) => (
                  <tr key={dat.id}>
                    <td className="p-md px-lg text-body text-text-primary border-b border-border">
                      {dat.name}
                    </td>
                    <td className="p-md px-lg text-body text-text-muted border-b border-border font-mono">
                      {dat.platform_slug}
                    </td>
                    <td className="p-md px-lg text-body text-text-muted border-b border-border text-right font-mono">
                      {dat.entry_count.toLocaleString()}
                    </td>
                    <td className="p-md px-lg border-b border-border text-right">
                      <button
                        className="text-error font-mono text-badge hover:underline cursor-pointer bg-transparent border-none uppercase"
                        onClick={() => handleRemoveDat(dat.id)}
                      >
                        Remove
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </section>

      <section className="mt-3xl">
        <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">
          // Verify library
        </h2>
        <div className="card">
          <p className="text-body text-text-muted mb-lg">
            Compare ROM hashes against imported DAT files to verify integrity.
            Verified ROMs show a green badge, bad dumps show a warning.
          </p>
          <button
            className="btn btn-secondary"
            disabled={datFiles.length === 0 || verifying}
            onClick={handleVerify}
          >
            {verifying ? "Verifying..." : "Verify Library"}
          </button>
          {verifyProgress && <ProgressBar progress={verifyProgress} />}
        </div>
      </section>

      {showPlatformDialog && (
        <PlatformDialog
          platforms={platforms}
          headerName={pendingDatHeaderName}
          onSelect={handlePlatformSelect}
          onCancel={() => {
            setShowPlatformDialog(false);
            setPendingDatPath(null);
          }}
        />
      )}
    </>
  );
}
