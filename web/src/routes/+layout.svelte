<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { initLocale, getLocale } from '$lib/i18n';
	import { isLowPowerDevice } from '$lib/motion';

	let { children } = $props();

	$effect(() => {
		initLocale();
		document.documentElement.lang = getLocale();
	});

	$effect(() => {
		document.documentElement.classList.toggle('low-power', isLowPowerDevice());
		const mq = window.matchMedia('(any-pointer: coarse)');
		const handler = () => {
			document.documentElement.classList.toggle('low-power', isLowPowerDevice());
		};
		mq.addEventListener('change', handler);
		return () => mq.removeEventListener('change', handler);
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="app-ambient" aria-hidden="true"></div>
{@render children()}
