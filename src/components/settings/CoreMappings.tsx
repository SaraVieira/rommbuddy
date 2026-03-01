import { useState, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  PlatformWithCount,
  CoreInfo,
  CoreMapping,
  EmulatorDef,
} from "../../types";
import { toast } from "sonner";
import CoreSelect, { type CoreSelectValue, encodeMapping, decodeMapping } from "./CoreSelect";
import { DEFAULT_CORES } from "../../utils/defaultCores";

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
  const [hideMapped, setHideMapped] = useState(false);

  const mappingsByPlatformId = useMemo(
    () => new Map(mappings.map((m) => [m.platform_id, m])),
    [mappings],
  );

  if (platforms.length === 0) return null;

  const mappedCount = platforms.filter((p) =>
    mappingsByPlatformId.has(p.id),
  ).length;
  const unmappedCount = platforms.length - mappedCount;

  const getMappingValue = (mapping: CoreMapping | undefined): CoreSelectValue => {
    if (!mapping) return "";
    return encodeMapping(mapping.emulator_type, mapping.core_name);
  };

  const getEmulatorsForPlatform = (slug: string) =>
    emulators.filter((e) => e.platforms.includes(slug) && emulatorPaths[e.id]);

  const handleCoreChange = async (platformId: number, value: CoreSelectValue) => {
    const decoded = decodeMapping(value);
    if (!decoded) return;

    try {
      if (decoded.type === "emulator") {
        await invoke("set_core_mapping", {
          platformId,
          coreName: decoded.id,
          corePath: "",
          emulatorType: decoded.id,
        });
        toast.success("Emulator mapping saved");
      } else {
        const core = cores.find((c) => c.core_name === decoded.coreName);
        if (!core) return;
        await invoke("set_core_mapping", {
          platformId,
          coreName: core.core_name,
          corePath: core.core_path,
          emulatorType: "retroarch",
        });
        toast.success("Core mapping saved");
      }
      onRefresh();
    } catch (e) {
      toast.error(String(e));
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
                hideMapped ? !mappingsByPlatformId.has(p.id) : true,
              )
              .map((platform) => {
                const mapping = mappingsByPlatformId.get(platform.id);
                const defaultCore = DEFAULT_CORES[platform.slug];
                const platformEmulators = getEmulatorsForPlatform(
                  platform.slug,
                );
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
