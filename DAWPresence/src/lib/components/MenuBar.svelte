<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";

  interface Props {
    hideProjectName: boolean;
    hideSystemUsage: boolean;
    updateInterval: number;
    onSettingsChange: () => void | Promise<void>;
  }

  let { hideProjectName, hideSystemUsage, updateInterval, onSettingsChange }: Props = $props();

  let unlistenSettingsChanged: UnlistenFn | null = null;
  let unlistenOpenUpdateInterval: UnlistenFn | null = null;

  async function toggleHideProjectName() {
    try {
      await invoke("toggle_hide_project_name");
      await onSettingsChange();
    } catch (e) {
      console.error("Failed to toggle hide project name:", e);
    }
  }

  async function toggleHideSystemUsage() {
    try {
      await invoke("toggle_hide_system_usage");
      await onSettingsChange();
    } catch (e) {
      console.error("Failed to toggle hide system usage:", e);
    }
  }

  async function setUpdateInterval() {
    const input = prompt(
      "Type the presence update interval (in milliseconds):",
      `${updateInterval}`
    );
    if (input) {
      const interval = parseInt(input, 10);
      if (interval >= 1000 && interval <= 100000000) {
        try {
          await invoke("set_update_interval", { interval });
          await onSettingsChange();
        } catch (e) {
          console.error("Failed to set update interval:", e);
        }
      } else {
        alert("Update interval must be between 1000ms and 100,000,000ms");
      }
    }
  }

  onMount(async () => {
    unlistenSettingsChanged = await listen("settings-changed", () => {
      void onSettingsChange();
    });
    unlistenOpenUpdateInterval = await listen("open-update-interval", () => {
      void (async () => {
        await onSettingsChange();
        await setUpdateInterval();
      })();
    });
  });

  onDestroy(() => {
    if (unlistenSettingsChanged) unlistenSettingsChanged();
    if (unlistenOpenUpdateInterval) unlistenOpenUpdateInterval();
  });
</script>

<div class="menu-bar">
  <button class="menu-item" onclick={toggleHideProjectName}>
    [{hideProjectName ? "ON" : "OFF"}] Hide Project Name
  </button>
  <button class="menu-item" onclick={toggleHideSystemUsage}>
    [{hideSystemUsage ? "ON" : "OFF"}] Hide System Usage
  </button>
  <button class="menu-item" onclick={setUpdateInterval}>
    Set Update Interval
  </button>
</div>

<style>
  .menu-bar {
    display: flex;
    gap: 4px;
    padding: 6px 10px;
    background: var(--menu-bg);
    border-bottom: 1px solid var(--border-color);
  }

  .menu-item {
    background: transparent;
    border: none;
    color: var(--text-color);
    padding: 4px 12px;
    font-size: 0.85rem;
    cursor: pointer;
    border-radius: 4px;
    transition: background 0.15s;
  }

  .menu-item:hover {
    background: var(--hover-bg);
  }
</style>
