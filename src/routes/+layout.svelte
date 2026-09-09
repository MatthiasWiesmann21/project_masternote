<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { initSettings } from '$lib/stores/settings';

  let { children } = $props();

  onMount(() => {
    initSettings();
  });
</script>

<div id="app">
  {@render children()}
</div>
