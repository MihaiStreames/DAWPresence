<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import StatusBox from "$lib/components/StatusBox.svelte";
  import MenuBar from "$lib/components/MenuBar.svelte";

  interface Status {
    daw_name: string;
    project_name: string;
    cpu_usage: string;
    ram_usage: string;
    is_connected: boolean;
  }

  interface Settings {
    hide_project_name: boolean;
    hide_system_usage: boolean;
    update_interval: number;
  }

  let status = $state<Status>({
    daw_name: "",
    project_name: "",
    cpu_usage: "",
    ram_usage: "",
    is_connected: false,
  });

  let settings = $state<Settings>({
    hide_project_name: false,
    hide_system_usage: false,
    update_interval: 2500,
  });

  let intervalId: ReturnType<typeof setInterval> | null = null;

  async function fetchStatus() {
    try {
      status = await invoke<Status>("get_status");
    } catch (e) {
      console.error("Failed to fetch status:", e);
    }
  }

  async function fetchSettings() {
    try {
      settings = await invoke<Settings>("get_settings");
    } catch (e) {
      console.error("Failed to fetch settings:", e);
    }
  }

  function startPolling() {
    if (intervalId) clearInterval(intervalId);
    intervalId = setInterval(fetchStatus, settings.update_interval);
  }

  async function handleSettingsChange() {
    await fetchSettings();
    startPolling();
  }

  onMount(async () => {
    await fetchSettings();
    await fetchStatus();
    startPolling();
  });

  onDestroy(() => {
    if (intervalId) clearInterval(intervalId);
  });
</script>

<main>
  <MenuBar
    hideProjectName={settings.hide_project_name}
    hideSystemUsage={settings.hide_system_usage}
    onSettingsChange={handleSettingsChange}
  />

  <div class="status-grid">
    <StatusBox
      title="Current Digital Audio Workstation"
      value={status.daw_name}
    />
    <StatusBox title="Opening Project" value={status.project_name} />
    <StatusBox title="CPU Usage" value={status.cpu_usage} large />
    <StatusBox title="RAM Usage" value={status.ram_usage} large />
  </div>
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  .status-grid {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 1fr;
    grid-template-rows: 1fr 1fr;
    gap: 16px;
    padding: 16px;
  }
</style>
