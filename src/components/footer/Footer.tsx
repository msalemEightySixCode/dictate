import React, { useState, useEffect } from "react";
import { getVersion } from "@tauri-apps/api/app";

import ModelSelector from "../model-selector";
import UpdateChecker from "../update-checker";
import { commands, type PhoneServerStatus } from "@/bindings";

const Footer: React.FC = () => {
  const [version, setVersion] = useState("");
  const [phone, setPhone] = useState<PhoneServerStatus | null>(null);

  useEffect(() => {
    const fetchVersion = async () => {
      try {
        const appVersion = await getVersion();
        setVersion(appVersion);
      } catch (error) {
        console.error("Failed to get app version:", error);
        setVersion("0.1.2");
      }
    };

    fetchVersion();
  }, []);

  useEffect(() => {
    const refresh = () => commands.getPhoneServerStatus().then(setPhone);
    refresh();
    const id = setInterval(refresh, 5000);
    return () => clearInterval(id);
  }, []);

  const phoneReady = !!phone?.listening && !!phone.tailscale_ip;
  const phoneLabel = !phone?.listening
    ? "iPhone server off"
    : phone.tailscale_ip
      ? "iPhone server on"
      : "Tailscale off";

  return (
    <div className="w-full border-t border-mid-gray/20 pt-3">
      <div className="flex justify-between items-center text-xs px-4 pb-3 text-text/60">
        <div className="flex items-center gap-4">
          <ModelSelector />
          <span
            className="flex items-center gap-1.5"
            title={phone?.tailscale_ip ?? undefined}
          >
            <span
              className={`w-2 h-2 rounded-full ${
                phoneReady ? "bg-green-500" : "bg-red-500"
              }`}
            />
            {phoneLabel}
          </span>
        </div>

        {/* Update Status */}
        <div className="flex items-center gap-1">
          <UpdateChecker />
          <span>•</span>
          {/* eslint-disable-next-line i18next/no-literal-string */}
          <span>v{version}</span>
        </div>
      </div>
    </div>
  );
};

export default Footer;
