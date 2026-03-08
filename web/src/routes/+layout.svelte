<script lang="ts">
	import '../app.css';
	let { children } = $props();
</script>

<div class="root">
	{@render children()}
</div>

<style>
	.root {
		height: 100vh;
		display: flex;
	}
</style>
