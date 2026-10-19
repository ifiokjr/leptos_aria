// Progressive enhancements for the leptos_aria website. Every page works
// without this file; it only marks the current nav link, wires the copy
// buttons, and drives the landing-page switch specimen.

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
