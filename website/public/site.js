// Progressive enhancements for the leptos_aria website. Every page works
// without this file; it only marks the current nav link, wires the copy
// buttons, drives the landing-page switch specimen, and switches and sizes
// the component pages' live previews.

const page = document.body.dataset.page;

for (const link of document.querySelectorAll("[data-nav]")) {
	if (link.dataset.nav === page) {
		link.setAttribute("aria-current", "page");
	}
}

for (const button of document.querySelectorAll("[data-copy]")) {
	button.addEventListener("click", async () => {
		try {
			await navigator.clipboard.writeText(button.dataset.copy);
			button.dataset.state = "copied";
			button.textContent = "Copied";
		} catch {
			button.textContent = "Select to copy";
			return;
		}

		setTimeout(() => {
			delete button.dataset.state;
			button.textContent = "Copy";
		}, 1600);
	});
}

// The switch follows the same contract leptos_aria components do: behaviour
// sets `aria-checked` and `data-state`, CSS reads them. The inspector mirrors
// the live attributes and flashes whichever changed.
const demo = document.querySelector(".demo-switch");

if (demo) {
	const mirrors = document.querySelectorAll("[data-mirror]");

	demo.addEventListener("click", () => {
		const checked = demo.getAttribute("aria-checked") !== "true";
		demo.setAttribute("aria-checked", String(checked));
		demo.dataset.state = checked ? "checked" : "unchecked";

		for (const mirror of mirrors) {
			mirror.textContent = demo.getAttribute(mirror.dataset.mirror);
			mirror.removeAttribute("data-flash");
			// Force a reflow so the flash animation restarts on every toggle.
			void mirror.offsetWidth;
			mirror.setAttribute("data-flash", "");
		}
	});
}

// Component previews: Preview and Code tabs over one framed example. The tabs
// follow the ARIA tabs pattern, with arrow keys, Home, and End moving between
// them and selecting as they go.
for (const preview of document.querySelectorAll("[data-preview]")) {
	const tabs = [...preview.querySelectorAll("[role=\"tab\"]")];

	const select = (selected) => {
		for (const tab of tabs) {
			const isSelected = tab === selected;
			tab.setAttribute("aria-selected", String(isSelected));
			tab.tabIndex = isSelected ? 0 : -1;
			document.getElementById(tab.getAttribute("aria-controls")).hidden = !isSelected;
		}
	};

	for (const [index, tab] of tabs.entries()) {
		tab.addEventListener("click", () => select(tab));
		tab.addEventListener("keydown", (event) => {
			const next = {
				ArrowLeft: tabs.at(index - 1),
				ArrowRight: tabs[(index + 1) % tabs.length],
				Home: tabs[0],
				End: tabs.at(-1),
			}[event.key];

			if (next) {
				event.preventDefault();
				select(next);
				next.focus();
			}
		});
	}
}

// The preview page reports its example's height; grow or shrink its frame to
// match, so nothing scrolls inside it. Only same-origin preview frames count,
// and a frame hidden behind the Code tab reports zero, which is ignored.
addEventListener("message", (event) => {
	const { type, height } = event.data ?? {};

	if (event.origin !== location.origin || type !== "leptos-aria-preview:height" || !(height > 0)) {
		return;
	}

	for (const frame of document.querySelectorAll(".preview-stage iframe")) {
		if (frame.contentWindow === event.source) {
			frame.style.height = `${height}px`;
		}
	}
});
