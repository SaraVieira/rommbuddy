import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import SectionHeading from "../SectionHeading";

type ConnectionStatus = "unchecked" | "ok" | "error" | "testing";

interface CredentialField {
  label: string;
  key: string;
  placeholder: string;
  type?: "text" | "password";
}

interface CredentialsSectionProps {
  title: string;
  description: string;
  fields: CredentialField[];
  getCommand: string;
  setCommand: string;
  testCommand: string;
  /** Maps backend credential keys to field keys */
  fieldMapping: Record<string, string>;
  /** Maps field keys to invoke param names for save */
  saveParamMapping: Record<string, string>;
  /** Maps field keys to invoke param names for test */
  testParamMapping: Record<string, string>;
  /** Build the status message from loaded credentials */
  loadedMessage?: (creds: Record<string, string>) => string;
}

export default function CredentialsSection({
  title,
  description,
  fields,
  getCommand,
  setCommand,
  testCommand,
  fieldMapping,
  saveParamMapping,
  testParamMapping,
  loadedMessage,
}: CredentialsSectionProps) {
  const [values, setValues] = useState<Record<string, string>>(
    () => Object.fromEntries(fields.map((f) => [f.key, ""])),
  );
  const [status, setStatus] = useState<ConnectionStatus>("unchecked");
  const [statusMessage, setStatusMessage] = useState("");

  const loadCredentials = useCallback(async () => {
    try {
      const creds = await invoke<Record<string, string> | null>(getCommand);
      if (creds) {
        const mapped: Record<string, string> = {};
        for (const [backendKey, fieldKey] of Object.entries(fieldMapping)) {
          mapped[fieldKey] = creds[backendKey] ?? "";
        }
        setValues(mapped);
        setStatus("ok");
        setStatusMessage(
          loadedMessage ? loadedMessage(mapped) : "Credentials saved",
        );
      }
    } catch (e) {
      console.error(`Failed to load credentials for ${title}:`, e);
    }
  }, [getCommand, fieldMapping, loadedMessage, title]);

  useEffect(() => {
    loadCredentials();
  }, [loadCredentials]);

  const buildParams = (mapping: Record<string, string>) => {
    const params: Record<string, string> = {};
    for (const [fieldKey, paramName] of Object.entries(mapping)) {
      params[paramName] = values[fieldKey] ?? "";
    }
    return params;
  };

  const handleSave = async () => {
    try {
      await invoke(setCommand, buildParams(saveParamMapping));
      toast.success(`${title} credentials saved`);
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleTest = async () => {
    setStatus("testing");
    try {
      const result = await invoke<{ success: boolean; message: string }>(
        testCommand,
        buildParams(testParamMapping),
      );
      setStatus(result.success ? "ok" : "error");
      setStatusMessage(result.message);
      if (result.success) {
        await invoke(setCommand, buildParams(saveParamMapping));
      }
    } catch (e) {
      setStatus("error");
      setStatusMessage(String(e));
    }
  };

  const allFilled = fields.every((f) => values[f.key]?.trim());

  return (
    <section className="mt-3xl">
      <SectionHeading className="mb-lg">{title}</SectionHeading>
      <div className="card">
        <p className="text-body text-text-muted mb-xl">{description}</p>
        <div className="flex flex-col gap-lg">
          {fields.map((field) => (
            <div key={field.key} className="form-group">
              <label>{field.label}</label>
              <input
                type={field.type ?? "text"}
                value={values[field.key] ?? ""}
                onChange={(e) =>
                  setValues((prev) => ({
                    ...prev,
                    [field.key]: e.target.value,
                  }))
                }
                placeholder={field.placeholder}
              />
            </div>
          ))}
          <div className="flex items-center gap-md">
            <button className="btn btn-primary" onClick={handleSave}>
              Save Credentials
            </button>
            <button
              className="btn btn-secondary"
              onClick={handleTest}
              disabled={status === "testing" || !allFilled}
            >
              {status === "testing" ? "Testing..." : "Test Connection"}
            </button>
            {status !== "unchecked" && status !== "testing" && (
              <span
                className={`text-body font-mono font-semibold uppercase ${
                  status === "ok" ? "text-accent" : "text-error"
                }`}
              >
                [{status === "ok" ? "ok" : "error"}] {statusMessage}
              </span>
            )}
          </div>
        </div>
      </div>
    </section>
  );
}
