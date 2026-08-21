<script lang="ts">
	export let variant: 'primary' | 'secondary' | 'danger' | 'ghost' = 'primary';
	export let size: 'sm' | 'md' | 'lg' = 'md';
	export let disabled = false;
	export let ariaLabel: string | undefined = undefined;
</script>

<button
	class="eel-btn eel-btn--{variant} eel-btn--{size}"
	{disabled}
	aria-label={ariaLabel}
	on:click
>
	<slot />
</button>

<style>
	.eel-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		font-weight: 600;
		border-radius: 4px;
		border: 2px solid transparent;
		cursor: pointer;
		transition: all 200ms;
	}
	.eel-btn:focus-visible { outline: none; box-shadow: 0 0 0 3px rgba(26, 86, 219, 0.5); }
	.eel-btn--sm { padding: 0.25rem 0.75rem; font-size: 0.875rem; }
	.eel-btn--md { padding: 0.5rem 1rem; font-size: 1rem; }
	.eel-btn--lg { padding: 0.75rem 1.5rem; font-size: 1.125rem; }
	.eel-btn--primary { background: #1a56db; color: #fff; border-color: #1a56db; }
	.eel-btn--secondary { background: #f3f4f6; color: #374151; border-color: #d1d5db; }
	.eel-btn--danger { background: #b91c1c; color: #fff; }
	.eel-btn--ghost { background: transparent; color: #1a56db; }
	button:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
