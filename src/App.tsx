import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SanitizationResult } from "./types/sanitizer";
import { HUD } from "./components/HUD";

const SAMPLE_RAW_LOG = `2026-08-14T16:28:46.123Z [ERROR] 0x7ffee1234567 Failed connection to postgres://admin:P@ssword123!@db.internal:5432/prod
OPENAI_KEY=sk-proj-9876543210fedcba9876543210fedcba
AWS_KEY=AKIAIOSFODNN7EXAMPLE
node_modules/express/lib/router/layer.js:95:5
node_modules/express/lib/router/route.js:137:13
Error downloading resource (Repeated 4x)
Error downloading resource (Repeated 4x)
Error downloading resource (Repeated 4x)
data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==`;

export function App() {
  const [result, setResult] = useState<SanitizationResult | null>(null);

  useEffect(() => {
    // Listen for Tauri backend sanitization events
    const unlistenPromise = listen<SanitizationResult>("sanitization-complete", (event) => {
      setResult(event.payload);
    });

    // Keyboard shortcut Esc to hide HUD
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        hideHUD();
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  const hideHUD = async () => {
    try {
      await invoke("hide_hud");
    } catch (err) {
      console.log("Hide HUD call:", err);
    }
  };

  const runTestPrompt = async () => {
    try {
      const res = await invoke<SanitizationResult>("sanitize_text", {
        text: SAMPLE_RAW_LOG,
      });
      setResult(res);
    } catch (err) {
      console.error("Test prompt error:", err);
    }
  };

  return (
    <div className="w-screen h-screen bg-transparent flex flex-col items-center justify-start overflow-hidden">
      <HUD
        result={result}
        onHide={hideHUD}
        onRunTestPrompt={runTestPrompt}
      />
    </div>
  );
}

export default App;
