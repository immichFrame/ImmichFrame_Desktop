const { invoke } = window.__TAURI__.core;


window.addEventListener("DOMContentLoaded", async () => {
  const btnQuit = document.getElementById("btnQuit");
  const btnSettings = document.getElementById("btnSettings");
  const urlModal = document.getElementById("urlModal");
  const closeModal = document.getElementById("closeModal");
  const urlInput = document.getElementById("urlInput");
  const saveUrlBtn = document.getElementById("saveUrl");
  const iframe = document.getElementById("webview");
  const defaultUrl = "https://demo.immichframe.online/";
  let currentUrl = defaultUrl;

  const parsePositiveInt = (value) => {
    const n = Number.parseInt(String(value ?? ""), 10);
    return Number.isFinite(n) && n > 0 ? n : null;
  };

  const hardReloadIframe = () => {
    if (!iframe) return;
    const url = currentUrl;
    try {
      iframe.src = "about:blank";
      window.setTimeout(() => {
        try {
          iframe.src = url;
        } catch (e) {
          console.error("Failed to restore iframe src after blanking:", e);
        }
      }, 250);
    } catch (e) {
      console.error("Failed to hard reload iframe:", e);
    }
  };

  const formatKb = (kb) => {
    if (kb == null) return "—";
    const mib = kb / 1024;
    if (mib < 1024) return `${mib.toFixed(0)} MiB`;
    const gib = mib / 1024;
    return `${gib.toFixed(2)} GiB`;
  };

  const truthyEnv = (value) => {
    const v = String(value ?? "").trim().toLowerCase();
    return v === "1" || v === "true" || v === "yes" || v === "on";
  };

  const startDebugOverlay = () => {
    const el = document.createElement("div");
    el.id = "immichframe-debug-overlay";
    el.style.position = "fixed";
    el.style.top = "10px";
    el.style.left = "10px";
    el.style.zIndex = "999999";
    el.style.padding = "10px 12px";
    el.style.borderRadius = "10px";
    el.style.background = "rgba(0, 0, 0, 0.65)";
    el.style.color = "#fff";
    el.style.fontFamily = "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace";
    el.style.fontSize = "12px";
    el.style.lineHeight = "1.35";
    el.style.whiteSpace = "pre";
    el.style.userSelect = "none";
    el.style.cursor = "pointer";
    el.title = "ImmichFrame debug overlay (click to hide)";
    el.textContent = "Loading memory stats…";

    el.addEventListener("click", () => {
      el.remove();
    });

    document.body.appendChild(el);

    const update = async () => {
      try {
        const stats = await invoke("get_linux_resource_stats");
        if (!stats) {
          el.textContent = "Debug overlay: stats unavailable";
          return;
        }

        const now = new Date();
        const ts = now.toLocaleTimeString();

        const lines = [];
        lines.push(`ImmichFrame debug  ${ts}`);
        lines.push(`URL: ${currentUrl}`);
        lines.push("");

        lines.push(
          `MemAvail: ${formatKb(stats.mem_available_kb)} / Total: ${formatKb(stats.mem_total_kb)}`
        );
        lines.push(`MemFree:  ${formatKb(stats.mem_free_kb)}`);
        lines.push(`SwapFree: ${formatKb(stats.swap_free_kb)} / Total: ${formatKb(stats.swap_total_kb)}`);
        lines.push(`CMAFree:  ${formatKb(stats.cma_free_kb)} / Total: ${formatKb(stats.cma_total_kb)}`);
        if (stats.webkit_rss_kb != null) {
          const n = stats.webkit_process_count != null ? ` (${stats.webkit_process_count})` : "";
          lines.push(`WebKit RSS:${formatKb(stats.webkit_rss_kb)}${n}`);
        }
        lines.push("");
        lines.push(`Self RSS: ${formatKb(stats.self_rss_kb)}   VmSize: ${formatKb(stats.self_vmsize_kb)}`);

        el.textContent = lines.join("\n");
      } catch (e) {
        el.textContent = `Debug overlay error: ${String(e)}`;
      }
    };

    update();
    window.setInterval(update, 5000);
  };
  
  try {
    const savedUrl = await invoke("read_url_from_file") || "";
    const urlToLoad = savedUrl.trim() ? savedUrl : defaultUrl;
    currentUrl = urlToLoad;
    if (iframe) {
      iframe.src = urlToLoad;
    }
  } catch (error) {
    console.error("Error loading saved URL:", error);
    currentUrl = defaultUrl;
    if (iframe) {
      iframe.src = defaultUrl;
    }
  }

  // Optional low-memory watchdogs (off by default):
  // - IMMICHFRAME_AUTO_RELOAD_MINUTES: periodically hard-reload the iframe.
  // - IMMICHFRAME_AUTO_RESTART_MINUTES: restart the entire app.
  try {
    const overlayEnabled = truthyEnv(
      await invoke("read_immichframe_env", { suffix: "DEBUG_OVERLAY" })
    );
    if (overlayEnabled) {
      console.log("Debug overlay enabled");
      startDebugOverlay();
    }

    const reloadMinutes = parsePositiveInt(
      await invoke("read_immichframe_env", { suffix: "AUTO_RELOAD_MINUTES" })
    );
    if (reloadMinutes) {
      console.log(`Auto iframe reload enabled: every ${reloadMinutes} minutes`);
      window.setInterval(hardReloadIframe, reloadMinutes * 60 * 1000);
    }

    const restartMinutes = parsePositiveInt(
      await invoke("read_immichframe_env", { suffix: "AUTO_RESTART_MINUTES" })
    );
    if (restartMinutes) {
      console.log(`Auto restart enabled: after ${restartMinutes} minutes`);
      window.setTimeout(async () => {
        try {
          await invoke("restart_app");
        } catch (e) {
          console.error("Auto restart failed:", e);
        }
      }, restartMinutes * 60 * 1000);
    }
  } catch (e) {
    // Keep fully silent behavior if the new command isn't available.
    console.warn("Watchdog env config not available:", e);
  }

  if (btnQuit) {
    btnQuit.addEventListener("click", async () => {
      console.log("Attempting to quit app");
      try {
        await invoke("exit_app");
      } catch (error) {
        console.error("Error invoking exit_app:", error);
      }
    });
  }
  if (btnSettings) {
    btnSettings.addEventListener("click", async () => {
      console.log("Opening settings modal");
      try {
        const savedUrl = await invoke("read_url_from_file") || "";
        
        if (urlInput) {
          urlInput.value = savedUrl.trim() ? savedUrl : defaultUrl;
        }

        urlModal.style.display = "flex";
      } catch (error) {
        console.error("Error loading saved URL:", error);
        
        if (urlInput) {
          urlInput.value = defaultUrl;
        }

        urlModal.style.display = "flex";
      }
    });
  }
  if (closeModal) {
    closeModal.addEventListener("click", () => {
      urlModal.style.display = "none";
    });
  }
  if (saveUrlBtn) {
    saveUrlBtn.addEventListener("click", async () => {
      const url = urlInput.value;
      console.log("URL to show:", url);
      
      try {
        await invoke("save_url_to_file", { url });
        console.log("URL saved successfully.");
      } catch (error) {
        console.error("Error saving URL:", error);
      }

      if (iframe) {
        currentUrl = url;
        iframe.src = url;
      }

      urlModal.style.display = "none";
    });
  }
});