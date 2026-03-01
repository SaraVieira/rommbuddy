import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  PlatformWithCount,
  CoreInfo,
  CoreMapping,
  EmulatorDef,
} from "../../types";
import { useAppToast } from "../../App";
import CoreSelect from "./CoreSelect";

const DEFAULT_CORES: Record<string, string> = {
  gb: "gambatte_libretro",
  gbc: "gambatte_libretro",
  gba: "mgba_libretro",
  nes: "mesen_libretro",
  snes: "snes9x_libretro",
  n64: "mupen64plus_next_libretro",
  nds: "melonds_libretro",
  psx: "swanstation_libretro",
  genesis: "genesis_plus_gx_libretro",
  arcade: "fbneo_libretro",
};

interface CoreMappingsProps {
  platforms: PlatformWithCount[];
  cores: CoreInfo[];
  mappings: CoreMapping[];
  emulators: EmulatorDef[];
  emulatorPaths: Record<string, string>;
  pathValid: boolean;
  onRefresh: () => void;
}

export default function CoreMappings({
  platforms,
  cores,
  mappings,
  emulators,
  emulatorPaths,
  pathValid,
  onRefresh,
}: CoreMappingsProps) {
  const toast = useAppToast();
  const [hideMapped, setHideMapped] = useState(false);

  if (platforms.length === 0) return null;

  const mappedCount = platforms.filter((p) =>
    mappings.some((m) => m.platform_id === p.id),
  ).length;
  const unmappedCount = platforms.length - mappedCount;

  const getMappingValue = (mapping: CoreMapping | undefined): string => {
    if (!mapping) return "";
    if (mapping.emulator_type !== "retroarch")
      return `emu:${mapping.emulator_type}`;
    return `retroarch:${mapping.core_name}`;
  };

  const getEmulatorsForPlatform = (slug: string) =>
    emulators.filter((e) => e.platforms.includes(slug) && emulatorPaths[e.id]);

  const handleCoreChange = async (platformId: number, value: string) => {
    if (value.startsWith("emu:")) {
      const emulatorId = value.slice(4);
      try {
        await invoke("set_core_mapping", {
          platformId,
          coreName: emulatorId,
          corePath: "",
          emulatorType: emulatorId,
        });
        toast("Emulator mapping saved", "success");
        onRefresh();
      } catch (e) {
        toast(String(e), "error");
      }
    } else if (value.startsWith("retroarch:")) {
      const coreName = value.slice(10);
      const core = cores.find((c) => c.core_name === coreName);
      if (!core) return;
      try {
        await invoke("set_core_mapping", {
          platformId,
          coreName: core.core_name,
          corePath: core.core_path,
          emulatorType: "retroarch",
        });
        toast("Core mapping saved", "success");
        onRefresh();
      } catch (e) {
        toast(String(e), "error");
      }
    }
  };

  const hasRetroarchCores = pathValid && cores.length > 0;

  return (
    <section className="mt-3xl">
      <div className="flex items-center justify-between mb-lg">
        <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide">
          // Core Mappings
        </h2>
        <button
          className={`px-xl py-sm font-mono text-badge uppercase border ${
            hideMapped
              ? "border-accent text-accent bg-accent/10"
              : "border-border text-text-muted bg-bg-elevated hover:border-border-light"
          }`}
          onClick={() => setHideMapped((v) => !v)}
        >
          {hideMapped
            ? `Showing unmapped (${unmappedCount})`
            : `All (${platforms.length})`}
        </button>
      </div>
      <div className="card">
        <table className="w-full border-collapse">
          <thead>
            <tr>
              <th className="text-left p-md px-lg text-nav font-medium text-text-muted uppercase tracking-wide border-b border-border">
                Platform
              </th>
              <th className="text-left p-md px-lg text-nav font-medium text-text-muted uppercase tracking-wide border-b border-border">
                Emulator / Core
              </th>
              <th className="text-left p-md px-lg text-nav font-medium text-text-muted uppercase tracking-wide border-b border-border">
                Status
              </th>
            </tr>
          </thead>
          <tbody>
            {platforms
              .filter((p) =>
                hideMapped
                  ? !mappings.some((m) => m.platform_id === p.id)
                  : true,
              )
              .map((platform) => {
              const mapping = mappings.find(
                (m) => m.platform_id === platform.id,
              );
              const defaultCore = DEFAULT_CORES[platform.slug];
              const platformEmulators = getEmulatorsForPlatform(platform.slug);
              return (
                <tr key={platform.id}>
                  <td className="p-md px-lg text-body text-text-primary border-b border-border align-middle">
                    {platform.name}{" "}
                    <span className="text-text-dim text-nav">
                      ({platform.rom_count})
                    </span>
                  </td>
                  <td className="p-md px-lg text-body text-text-primary border-b border-border align-middle">
                    <CoreSelect
                      value={getMappingValue(mapping)}
                      cores={cores}
                      emulators={platformEmulators}
                      defaultCore={defaultCore}
                      hasRetroarchCores={hasRetroarchCores}
                      onChange={(value) =>
                        handleCoreChange(platform.id, value)
                      }
                    />
                  </td>
                  <td className="p-md px-lg text-body text-text-primary border-b border-border align-middle">
                    {mapping ? (
                      <span className="text-accent font-mono font-semibold uppercase">
                        [ok]
                      </span>
                    ) : (
                      <span className="text-error font-mono font-semibold uppercase">
                        [missing]
                      </span>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </section>
  );
}
