import { useState, useCallback } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import type { ScanProgress } from "../../types";
import { toast } from "sonner";
import ProgressBar from "./ProgressBar";
import CredentialsSection from "./CredentialsSection";

export default function IntegrationsTab() {
  const [updatingMetadataDb, setUpdatingMetadataDb] = useState(false);
  const [metadataDbProgress, setMetadataDbProgress] =
    useState<ScanProgress | null>(null);

  const handleUpdateMetadataDb = useCallback(async () => {
    if (updatingMetadataDb) return;
    setUpdatingMetadataDb(true);
    setMetadataDbProgress(null);
    try {
      const channel = new Channel<ScanProgress>();
      channel.onmessage = (p) => setMetadataDbProgress(p);
      await invoke("update_launchbox_db", { channel });
      toast.success("Metadata database updated!");
    } catch (e) {
      toast.error(String(e));
    } finally {
      setUpdatingMetadataDb(false);
      setMetadataDbProgress(null);
    }
  }, [updatingMetadataDb]);

  return (
    <>
      <section>
        <h2 className="font-mono text-section font-semibold text-accent uppercase tracking-wide mb-lg">
          // Metadata Database
        </h2>
        <div className="card">
          <p className="text-body text-text-muted mb-lg">
            LaunchBox metadata database provides game descriptions, ratings,
            release dates, and cover art. Update periodically to get the latest
            metadata.
          </p>
          <button
            className="btn btn-secondary"
            disabled={updatingMetadataDb}
            onClick={handleUpdateMetadataDb}
          >
            {updatingMetadataDb ? "Updating..." : "Update Metadata DB"}
          </button>
          {metadataDbProgress && <ProgressBar progress={metadataDbProgress} />}
        </div>
      </section>

      <CredentialsSection
        title="RetroAchievements"
        description="Connect your RetroAchievements account to view achievement progress for your games. Get your Web API Key from retroachievements.org/controlpanel.php"
        fields={[
          { label: "Username", key: "username", placeholder: "YourUsername" },
          {
            label: "Web API Key",
            key: "apiKey",
            placeholder: "Your API key",
            type: "password",
          },
        ]}
        getCommand="get_ra_credentials"
        setCommand="set_ra_credentials"
        testCommand="test_ra_connection"
        fieldMapping={{ username: "username", api_key: "apiKey" }}
        saveParamMapping={{ username: "username", apiKey: "apiKey" }}
        testParamMapping={{ username: "username", apiKey: "apiKey" }}
        loadedMessage={(creds) => `Connected as ${creds.username}`}
      />

      <CredentialsSection
        title="IGDB / Twitch"
        description="Connect to IGDB for richer game metadata including descriptions, screenshots, themes, and ratings. Create a Twitch developer application at dev.twitch.tv/console to get your Client ID and Secret."
        fields={[
          {
            label: "Client ID",
            key: "clientId",
            placeholder: "Your Twitch Client ID",
          },
          {
            label: "Client Secret",
            key: "clientSecret",
            placeholder: "Your Twitch Client Secret",
            type: "password",
          },
        ]}
        getCommand="get_igdb_credentials"
        setCommand="set_igdb_credentials"
        testCommand="test_igdb_connection"
        fieldMapping={{ client_id: "clientId", client_secret: "clientSecret" }}
        saveParamMapping={{
          clientId: "clientId",
          clientSecret: "clientSecret",
        }}
        testParamMapping={{
          clientId: "clientId",
          clientSecret: "clientSecret",
        }}
      />

      <CredentialsSection
        title="ScreenScraper"
        description="Connect to ScreenScraper for high-quality retro game artwork and metadata including box art, screenshots, and detailed game info. Create a free account at screenscraper.fr for higher rate limits."
        fields={[
          {
            label: "Username",
            key: "username",
            placeholder: "Your ScreenScraper username",
          },
          {
            label: "Password",
            key: "password",
            placeholder: "Your ScreenScraper password",
            type: "password",
          },
        ]}
        getCommand="get_ss_credentials"
        setCommand="set_ss_credentials"
        testCommand="test_ss_connection"
        fieldMapping={{ username: "username", password: "password" }}
        saveParamMapping={{ username: "username", password: "password" }}
        testParamMapping={{ username: "username", password: "password" }}
        loadedMessage={(creds) => `Credentials saved for ${creds.username}`}
      />
    </>
  );
}
