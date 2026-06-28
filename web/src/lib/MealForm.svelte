<script lang="ts">
	import { validateMeal } from '$lib/validation';
	import { t } from '$lib/i18n';
	import Icon from '$lib/Icon.svelte';
	import type { Meal, NewIngredientLine } from '$lib/types';
	import { mealImageUrl } from '$lib/api';

	let {
		initialName = '',
		initialIngredients = [{ name: '', quantity: null }],
		initialInstructions = '',
		initialImage = null,
		editMode = false,
		editingMeal = null,
		submitting = false,
		onsubmit,
		oncancel,
	}: {
		initialName?: string;
		initialIngredients?: NewIngredientLine[];
		initialInstructions?: string;
		initialImage?: File | null;
		editMode?: boolean;
		editingMeal?: Meal | null;
		submitting?: boolean;
		onsubmit: (payload: {
			name: string;
			ingredients: NewIngredientLine[];
			instructions: string;
			image: File | null;
			removeImage: boolean;
		}) => void | Promise<void>;
		oncancel?: () => void;
	} = $props();

	let formName = $state(initialName);
	let formIngredients = $state<NewIngredientLine[]>(
		initialIngredients.length > 0
			? initialIngredients.map((i) => ({ name: i.name, quantity: i.quantity }))
			: [{ name: '', quantity: null }]
	);
	let formInstructions = $state(initialInstructions);
	let formImage = $state<File | null>(initialImage);
	let removeImage = $state(false);
	let formError = $state<string | null>(null);

	function validIngredientLines(): NewIngredientLine[] {
		return formIngredients.filter((r) => r.name.trim().length > 0);
	}

	function addIngredientRow() {
		formIngredients = [...formIngredients, { name: '', quantity: null }];
	}

	function removeIngredientRow(idx: number) {
		formIngredients = formIngredients.filter((_, i) => i !== idx);
	}

	function onImageChange(e: Event) {
		const file = (e.target as HTMLInputElement).files?.[0] ?? null;
		formImage = file;
		removeImage = false;
	}

	function onRemoveImageClick() {
		removeImage = true;
		formImage = null;
	}

	async function onSubmit() {
		formError = null;
		const valid = validIngredientLines();
		const result = validateMeal(formName, valid, formInstructions);
		if (!result.ok) {
			formError = t(result.messageKey);
			return;
		}
		try {
			await onsubmit({
				name: formName.trim(),
				ingredients: valid,
				instructions: formInstructions.trim(),
				image: formImage,
				removeImage,
			});
		} catch (err) {
			const raw = err instanceof Error ? err.message : String(err ?? '');
			formError = raw === '__REQUEST_FAILED__' ? t('errorSaveFailed') : raw;
		}
	}
</script>

<section class="form-card">
	<h2>
		{#if editMode}
			{t('formEditHeading', { name: formName || t('formUntitled') })}
		{:else}
			{t('formAddHeading')}
		{/if}
	</h2>
	<form onsubmit={(e) => { e.preventDefault(); onSubmit(); }} class="form-card__form">
		<label class="field">
			<span class="field__label">{t('fieldNameLabel')}</span>
			<input
				type="text"
				bind:value={formName}
				placeholder={t('fieldNamePlaceholder')}
				maxlength={200}
			/>
		</label>
		<fieldset class="field">
			<legend class="field__label">{t('fieldIngredientsLabel')}</legend>
			<div class="ingredient-rows">
				{#each formIngredients as ing, i (i)}
					<div class="ingredient-row">
						<input
							type="text"
							bind:value={ing.name}
							placeholder={t('fieldIngredientName')}
							maxlength={100}
							aria-label="{t('fieldIngredientName')} {i + 1}"
						/>
						<input
							type="text"
							value={ing.quantity ?? ''}
							oninput={(e) => { ing.quantity = (e.target as HTMLInputElement).value || null; }}
							placeholder={t('fieldIngredientQuantity')}
							maxlength={50}
							class="ingredient-row__quantity"
						/>
						<button type="button" class="btn btn--ghost btn--icon"
							onclick={() => removeIngredientRow(i)}
							aria-label={t('fieldIngredientRemove')}
							disabled={formIngredients.length <= 1}
						>
							<Icon name="trash-2" size={16} />
						</button>
					</div>
				{/each}
			</div>
			<button type="button" class="btn btn--ghost"
				onclick={addIngredientRow}
				disabled={formIngredients.length >= 100}
			>
				<Icon name="plus" size={14} /> {t('fieldIngredientAdd')}
			</button>
		</fieldset>

		<label class="field">
			<span class="field__label">{t('fieldInstructionsLabel')}</span>
			<textarea
				bind:value={formInstructions}
				placeholder={t('fieldInstructionsPlaceholder')}
				maxlength={20000}
				rows="6"
			></textarea>
		</label>

		<!-- Image field -->
		<div class="field">
			<span class="field__label">{t('fieldImageLabel')}</span>
			{#if editMode && editingMeal?.has_image && !removeImage}
				<div class="meal-image-controls">
					<img src={mealImageUrl(editingMeal.id)} alt="" class="meal-image-preview" />
					<button type="button" class="btn btn--ghost" onclick={onRemoveImageClick}>
						<Icon name="trash-2" size={14} /> {t('fieldImageRemove')}
					</button>
				</div>
			{/if}
			<input
				type="file"
				accept="image/*"
				onchange={onImageChange}
			/>
			{#if formImage}
				<span class="image-status">{t('imageStaged')}</span>
			{:else if removeImage}
				<span class="image-status">{t('imageStagedRemove')}</span>
			{/if}
		</div>
		{#if formError}
			<p class="form-error" role="alert">
				<Icon name="circle-alert" size={18} />
				<span>{formError}</span>
			</p>
		{/if}
		<div class="form-card__actions">
			<button type="submit" class="btn btn--primary" disabled={submitting}>
				{#if editMode}
					<Icon name="check" size={16} />
					{t('buttonSave')}
				{:else}
					<Icon name="plus" size={16} />
					{t('buttonAdd')}
				{/if}
			</button>
			{#if editMode}
				<button type="button" class="btn btn--ghost" onclick={oncancel}>
					<Icon name="x" size={16} />
					{t('buttonCancel')}
				</button>
			{/if}
		</div>
	</form>
</section>

<style>
	.form-card {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-top: 2px solid rgb(124 45 18 / 0.15);
		border-radius: var(--radius-lg);
		padding: var(--space-6);
		box-shadow: var(--shadow-sm);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}
	.form-card h2 {
		font-family: var(--font-display);
	}
	.form-card__form {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
	}
	fieldset.field {
		border: none;
		padding: 0;
		margin: 0;
	}
	.ingredient-rows {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.ingredient-row {
		display: flex;
		gap: var(--space-2);
		align-items: center;
	}
	.ingredient-row input {
		flex: 1;
		min-width: 0;
	}
	.ingredient-row__quantity {
		max-width: 140px;
	}
	.btn--icon {
		padding: var(--space-1);
		min-width: 36px;
		min-height: 36px;
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.form-card__actions {
		display: flex;
		gap: var(--space-2);
		flex-wrap: wrap;
	}
	.meal-image-controls {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		margin-bottom: var(--space-2);
	}
	.meal-image-preview {
		max-width: 100px;
		max-height: 100px;
		object-fit: cover;
		border-radius: var(--radius-sm);
	}
	.image-status {
		display: block;
		margin-top: var(--space-1);
		font-size: var(--text-xs);
		color: var(--color-text-secondary);
	}
	@keyframes shake {
		0%, 100% { transform: translateX(0); }
		20% { transform: translateX(-6px); }
		40% { transform: translateX(6px); }
		60% { transform: translateX(-4px); }
		80% { transform: translateX(4px); }
	}
	.form-error {
		animation: shake var(--motion-exit);
	}
</style>
