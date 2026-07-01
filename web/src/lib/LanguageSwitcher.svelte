<script lang="ts">
	import { getLocale, getLocalePreference, setLocale, t } from '$lib/i18n';
	import Icon from '$lib/Icon.svelte';
	import type { LocalePreference } from '$lib/i18n/types';

	interface Option {
		pref: LocalePreference;
		flag: string;
		label: string;
	}

	const options: Option[] = [
		{ pref: 'system', flag: '\u{1F5A5}\uFE0F', label: t('languageSystem') },
		{ pref: 'en', flag: '\u{1F1EC}\u{1F1E7}', label: t('languageEnglish') },
		{ pref: 'de', flag: '\u{1F1E9}\u{1F1EA}', label: t('languageGerman') },
	];

	let open = $state(false);
	let focusedIndex = $state(0);
	let triggerEl = $state<HTMLButtonElement>();

	const effectiveFlag = $derived(getLocale() === 'de' ? '\u{1F1E9}\u{1F1EA}' : '\u{1F1EC}\u{1F1E7}');
	const effectiveLabel = $derived(getLocale() === 'de' ? t('languageGerman') : t('languageEnglish'));

	function select(index: number): void {
		setLocale(options[index].pref);
		open = false;
		triggerEl?.focus();
	}

	function handleTriggerKeydown(e: KeyboardEvent): void {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			open = true;
			focusedIndex = options.findIndex((o) => o.pref === getLocalePreference());
			if (focusedIndex < 0) focusedIndex = 0;
		}
	}

	function handleListboxKeydown(e: KeyboardEvent): void {
		switch (e.key) {
			case 'ArrowDown':
				e.preventDefault();
				focusedIndex = (focusedIndex + 1) % options.length;
				break;
			case 'ArrowUp':
				e.preventDefault();
				focusedIndex = (focusedIndex + options.length - 1) % options.length;
				break;
			case 'Enter':
			case ' ':
				e.preventDefault();
				select(focusedIndex);
				break;
			case 'Escape':
				e.preventDefault();
				open = false;
				triggerEl?.focus();
				break;
		}
	}

	$effect(() => {
		if (!open) return;
		const handler = (e: MouseEvent) => {
			const root = document.querySelector('.lang-switcher');
			if (root && !root.contains(e.target as Node)) {
				open = false;
			}
		};
		document.addEventListener('click', handler, { capture: true });
		return () => document.removeEventListener('click', handler, { capture: true });
	});
</script>

<div class="lang-switcher">
	<button
		class="lang-switcher__trigger"
		type="button"
		aria-haspopup="listbox"
		aria-expanded={open}
		aria-label={t('languageLabel')}
		bind:this={triggerEl}
		onkeydown={handleTriggerKeydown}
		onclick={() => { open = !open; if (open) focusedIndex = 0; }}
	>
		<span>{effectiveFlag}</span>
		<span class="lang-switcher__label">{effectiveLabel}</span>
		<Icon name="chevron-down" size={16} />
	</button>

	{#if open}
		<ul
			role="listbox"
			class="lang-switcher__menu"
		>
			{#each options as opt, i}
				<li
					role="option"
					aria-selected={getLocalePreference() === opt.pref}
					tabindex={i === focusedIndex ? 0 : -1}
					class:lang-switcher__option--active={getLocalePreference() === opt.pref}
					onclick={() => select(i)}
					onkeydown={handleListboxKeydown}
				>
					<span>{opt.flag}</span>
					<span>{opt.label}</span>
				</li>
			{/each}
		</ul>
	{/if}
</div>
