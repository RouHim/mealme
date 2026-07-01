export type { Locale, LocalePreference, TranslationKey } from './types';
export {
	dictionaries,
	getLocale,
	getLocalePreference,
	t,
	formatNumber,
	formatDate,
	detectInitialLocale,
	initLocale,
	setLocale,
} from './state.svelte';
