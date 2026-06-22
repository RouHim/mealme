<script lang="ts">
	type IconName = 'search' | 'empty-meals' | 'alert' | 'calendar' | 'chevron-left' | 'chevron-right' | 'plus' | 'trash' | 'x' | 'check' | 'edit-3';
	let { name, class: className = '', size = 24 }: { name: IconName; class?: string; size?: number } = $props();
</script>

{#if name === 'search'}
	<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class={className} aria-hidden="true">
		<circle cx="11" cy="11" r="7"></circle>
		<line x1="21" y1="21" x2="16.65" y2="16.65"></line>
	</svg>
{:else if name === 'empty-meals'}
	<!-- custom: not from Feather -->
	<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 96 96" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" class={className} aria-hidden="true">
		<circle cx="48" cy="48" r="28"></circle>
		<circle cx="48" cy="48" r="20" stroke-dasharray="2 4" opacity="0.4"></circle>
		<line x1="14" y1="20" x2="14" y2="76"></line>
		<line x1="82" y1="20" x2="82" y2="76"></line>
		<line x1="11" y1="20" x2="17" y2="20"></line>
		<line x1="11" y1="76" x2="17" y2="76"></line>
		<line x1="79" y1="20" x2="85" y2="20"></line>
		<line x1="79" y1="76" x2="85" y2="76"></line>
	</svg>
{:else if name === 'alert'}
	<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class={className} aria-hidden="true">
		<circle cx="12" cy="12" r="10"></circle>
		<line x1="12" y1="8" x2="12" y2="12"></line>
		<line x1="12" y1="16" x2="12.01" y2="16"></line>
	</svg>
{:else if name === 'calendar'}
	<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class={className} aria-hidden="true">
		<rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
		<line x1="16" y1="2" x2="16" y2="6"></line>
		<line x1="8" y1="2" x2="8" y2="6"></line>
		<line x1="3" y1="10" x2="21" y2="10"></line>
	</svg>
{:else if name === 'chevron-left'}
	<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class={className} aria-hidden="true">
		<polyline points="15 18 9 12 15 6"></polyline>
	</svg>
{:else if name === 'chevron-right'}
	<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class={className} aria-hidden="true">
		<polyline points="9 18 15 12 9 6"></polyline>
	</svg>
{:else if name === 'plus'}
	<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class={className} aria-hidden="true">
		<line x1="12" y1="5" x2="12" y2="19"></line>
		<line x1="5" y1="12" x2="19" y2="12"></line>
	</svg>
{:else if name === 'trash'}
	<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class={className} aria-hidden="true">
		<polyline points="3 6 5 6 21 6"></polyline>
		<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
	</svg>
{:else if name === 'x'}
	<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class={className} aria-hidden="true">
		<line x1="18" y1="6" x2="6" y2="18"></line>
		<line x1="6" y1="6" x2="18" y2="18"></line>
	</svg>
{:else if name === 'check'}
	<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class={className} aria-hidden="true">
		<polyline points="20 6 9 17 4 12"></polyline>
	</svg>
{:else if name === 'edit-3'}
	<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class={className} aria-hidden="true">
		<path d="M12 20h9"></path>
		<path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path>
	</svg>
{/if}
