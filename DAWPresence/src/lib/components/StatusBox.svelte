<script lang="ts">
  interface Props {
    title: string;
    value: string;
    large?: boolean;
  }

  let { title, value, large = false }: Props = $props();
</script>

<div class="status-box">
  <div class="title">{title}</div>
  <div class="value" class:large>{value}</div>
</div>

<style>
  .status-box {
    background: var(--box-bg);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .title {
    font-size: 0.85rem;
    color: var(--text-muted);
    margin-bottom: 8px;
    font-weight: 500;
  }

  .value {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1rem;
    text-align: center;
    word-break: break-word;
  }

  .value.large {
    font-family: "Consolas", "Monaco", monospace;
    font-size: 1.25rem;
    font-weight: 500;
  }
</style>
