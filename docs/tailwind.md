# Tailwind CSS v4

Every component is headless: no styles ship, and state is exposed as `data-*` attributes so Tailwind's arbitrary variants style it directly. No plugin, no configuration, no dependency — add the crate, style with the classes you already write.

## Style with arbitrary variants

Tailwind v4 supports `data-[...]` variants out of the box. Every component takes a `class` prop, added after its built-in `leptos-aria-*` class on the element it renders. Pass a string, or a closure for classes that change:

```rust
use leptos::prelude::*;
use leptos_aria::Accordion;
use leptos_aria::AccordionItem;
use leptos_aria::AccordionPanel;
use leptos_aria::AccordionTrigger;

view! {
	<Accordion>
		<AccordionItem key="one".to_string()>
			<AccordionTrigger class="data-[state=open]:bg-slate-100 font-medium">
				"Section"
			</AccordionTrigger>
			<AccordionPanel class="data-[state=open]:animate-in">
				"Content"
			</AccordionPanel>
		</AccordionItem>
	</Accordion>
}
```

## Or name the variants once

A short `@custom-variant` block in your CSS turns every state into a one-word variant:

```css
@import "tailwindcss";

@custom-variant open (&:where([data-state="open"], [data-state="checked"], [data-state="on"]));
@custom-variant closed (&:where([data-state="closed"], [data-state="unchecked"], [data-state="off"]));
@custom-variant disabled- (*:disabled, &:where([data-disabled="true"]));
```

```html
<button class="open:bg-slate-100 closed:bg-white disabled-:opacity-50" />
```

## The attribute contract

| Attribute          | Values                    | Where                                                                                      |
| ------------------ | ------------------------- | ------------------------------------------------------------------------------------------ |
| `data-state`       | `open` / `closed`         | Modal, Popover, Tooltip, AlertDialog, HoverCard, ContextMenu, Accordion item/trigger/panel |
| `data-state`       | `checked` / `unchecked`   | Switch                                                                                     |
| `data-state`       | `on` / `off`              | ToggleGroupItem                                                                            |
| `data-state`       | `selected`                | Calendar cells, Tabs, ListBox options                                                      |
| `data-state`       | `loaded` / `fallback`     | Avatar                                                                                     |
| `data-disabled`    | `true` / `false`          | Accordion item, overlay triggers                                                           |
| `data-dragging`    | `true` / `false`          | DropZone                                                                                   |
| `data-is-dragging` | `true` / `false`          | DraggableItem                                                                              |
| `data-current`     | `true`                    | Pagination page buttons                                                                    |
| `data-orientation` | `horizontal` / `vertical` | Separator, Toolbar                                                                         |
| `data-action`      | `cancel` / `confirm`      | AlertDialog buttons                                                                        |
| `data-focused`     | `true` / `false`          | ListBox options (the active option), Tabs (the tab with keyboard focus)                    |
| `data-expanded`    | `true` / `false`          | ComboBox                                                                                   |

Alongside the ARIA attributes (`aria-expanded`, `aria-pressed`, `aria-selected`, ...), Tailwind v4's built-in `aria-*` variants cover the rest: `aria-pressed:bg-blue-500` needs no extra code.
