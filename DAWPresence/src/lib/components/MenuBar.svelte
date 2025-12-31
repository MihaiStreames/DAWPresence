<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface Props {
    hideProjectName: boolean;
    hideSystemUsage: boolean;
    onSettingsChange: () => void;
  }

  let { hideProjectName, hideSystemUsage, onSettingsChange }: Props = $props();

  async function toggleHideProjectName() {
    try {
      await invoke("toggle_hide_project_name");
      onSettingsChange();
    } catch (e) {
      console.error("Failed to toggle hide project name:", e);
    }
  }

  async function toggleHideSystemUsage() {
    try {
      await invoke("toggle_hide_system_usage");
      onSettingsChange();
    } catch (e) {
      console.error("Failed to toggle hide system usage:", e);
    }
  }

  async function setUpdateInterval() {
    const input = prompt(
      "Enter update interval in milliseconds (minimum 1000):",
      "2500"
    );
    if (input) {
      const interval = parseInt(input, 10);
      if (interval >= 1000) {
        try {
          await invoke("set_update_interval", { interval });
          onSettingsChange();
        } catch (e) {
          console.error("Failed to set update interval:", e);
        }
      } else {
        alert("Interval must be at least 1000ms");
      }
    }
  }
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
