<script lang="ts">
	// A font combo box: free text like before, but opening it always lists every font,
	// with the current one highlighted. Arrow keys move through the list and apply each
	// font immediately, so keys update live; Escape restores the font it opened with.
	import { tick } from "svelte";

	export let value: string;
	export let bundled: string[] = [];
	export let system: string[] = [];
	export let id: string;
	export let placeholder = "";

	type Option = { name: string; divider?: false } | { divider: true };

	let open = false;
	// null while browsing: the whole list shows. Set once the user types, to filter.
	let query: string | null = null;
	let original = value;
	let highlighted = -1;
	let listbox: HTMLUListElement;
	let input: HTMLInputElement;

	$: systemFonts = [...new Set(system)].filter((font) => !bundled.includes(font)).sort((a, b) => a.localeCompare(b, undefined, { sensitivity: "base" }));
	$: options = buildOptions(query, bundled, systemFonts);

	function buildOptions(query: string | null, bundled: string[], systemFonts: string[]): Option[] {
		const matches = (font: string) => query == null || font.toLowerCase().includes(query.trim().toLowerCase());
		const top = bundled.filter(matches).map((name) => ({ name }));
		const rest = systemFonts.filter(matches).map((name) => ({ name }));
		return top.length && rest.length ? [...top, { divider: true }, ...rest] : [...top, ...rest];
	}

	function optionId(index: number) {
		return `${id}-option-${index}`;
	}

	async function scrollToHighlighted() {
		await tick();
		listbox?.querySelector(`#${CSS.escape(optionId(highlighted))}`)?.scrollIntoView({ block: "nearest" });
	}

	function openList() {
		if (open) return;
		original = value;
		query = null;
		open = true;
		highlighted = options.findIndex((option) => !option.divider && option.name.toLowerCase() == value?.toLowerCase());
		scrollToHighlighted();
	}

	function close() {
		open = false;
		query = null;
	}

	function choose(index: number) {
		const option = options[index];
		if (!option || option.divider) return;
		highlighted = index;
		value = option.name;
		scrollToHighlighted();
	}

	function move(step: number) {
		if (!options.length) return;
		let index = highlighted;
		for (let i = 0; i < options.length; i++) {
			index = Math.max(0, Math.min(options.length - 1, index + step));
			if (!options[index].divider) break;
			if (index == 0 || index == options.length - 1) step = -step;
		}
		choose(index);
	}

	function onKeydown(event: KeyboardEvent) {
		switch (event.key) {
			case "ArrowDown":
			case "ArrowUp":
				event.preventDefault();
				if (!open) openList();
				else move(event.key == "ArrowDown" ? 1 : -1);
				break;
			case "PageDown":
			case "PageUp":
				if (!open) return;
				event.preventDefault();
				move(event.key == "PageDown" ? 10 : -10);
				break;
			case "Home":
			case "End":
				if (!open || query != null) return;
				event.preventDefault();
				highlighted = event.key == "Home" ? -1 : options.length;
				move(event.key == "Home" ? 1 : -1);
				break;
			case "Enter":
				if (!open) return;
				event.preventDefault();
				if (highlighted >= 0) choose(highlighted);
				close();
				break;
			case "Escape":
				if (!open) return;
				event.preventDefault();
				event.stopPropagation();
				value = original;
				close();
				break;
			case "Tab":
				close();
				break;
		}
	}

	function onInput() {
		// Typing still sets the font directly, as the plain text box did.
		query = value;
		if (!open) {
			original = value;
			open = true;
		}
		highlighted = options.findIndex((option) => !option.divider);
	}
</script>

<div class="relative w-full">
	<input
		bind:this={input}
		bind:value
		{id}
		{placeholder}
		role="combobox"
		aria-autocomplete="list"
		aria-expanded={open}
		aria-controls="{id}-listbox"
		aria-activedescendant={open && highlighted >= 0 ? optionId(highlighted) : undefined}
		autocomplete="off"
		spellcheck="false"
		class="w-full pl-1 pr-6 text-neutral-300 bg-neutral-600 border border-neutral-500 rounded-lg"
		on:focus={openList}
		on:click={openList}
		on:blur={close}
		on:keydown={onKeydown}
		on:input={onInput}
	/>
	<button
		type="button"
		tabindex="-1"
		aria-label="Show fonts"
		class="absolute right-1 top-1/2 -translate-y-1/2 px-1 text-xs text-neutral-400"
		on:mousedown|preventDefault={() => {
			if (open) close();
			else {
				input.focus();
				openList();
			}
		}}>▾</button
	>
	{#if open && options.length}
		<ul
			bind:this={listbox}
			id="{id}-listbox"
			role="listbox"
			class="absolute left-0 right-0 mt-1 max-h-60 overflow-y-auto py-1 bg-neutral-700 border border-neutral-500 rounded-lg shadow-lg z-20"
		>
			{#each options as option, index}
				{#if option.divider}
					<li role="separator" class="my-1 border-t border-neutral-600"></li>
				{:else}
					<!-- svelte-ignore a11y-click-events-have-key-events -->
					<li
						id={optionId(index)}
						role="option"
						aria-selected={index == highlighted}
						class="px-2 py-0.5 truncate cursor-pointer {index == highlighted ? 'bg-blue-600 text-white' : 'hover:bg-neutral-600'}"
						style="font-family: '{option.name.replace(/'/g, "\\'")}', sans-serif"
						title={option.name}
						on:mousedown|preventDefault
						on:click={() => {
							choose(index);
							close();
						}}
					>
						{option.name}
					</li>
				{/if}
			{/each}
		</ul>
	{/if}
</div>
