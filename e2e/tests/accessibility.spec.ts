import { audit, components, expect, preview, test } from "./fixtures";

for (const component of components) {
	test(`${component.name}: rendered example has no WCAG violations`, async ({ page }) => {
		await preview(page, component.name);
		await audit(page);
	});

	test(`${component.name}: documentation tabs and embedded example`, async ({ page }) => {
		await page.goto(`/components/${component.name}.html`);
		const tabs = page.getByRole("tablist", { name: `${component.title} example` });
		const live = tabs.getByRole("tab", { name: "Preview", exact: true });
		const code = tabs.getByRole("tab", { name: "Code", exact: true });
		const frame = page.frameLocator("iframe");
		await live.scrollIntoViewIfNeeded();
		await expect(frame.locator("#preview > *").first()).toBeAttached();
		await live.focus();
		await page.keyboard.press("ArrowRight");
		await expect(code).toBeFocused();
		await expect(code).toHaveAttribute("aria-selected", "true");
		await expect(live).toHaveAttribute("tabindex", "-1");
		await expect(page.getByRole("tabpanel")).toHaveCount(1);
		await audit(page);
		await page.keyboard.press("ArrowRight");
		await expect(live).toBeFocused();
		await expect(live).toHaveAttribute("aria-selected", "true");
		await expect(page.getByRole("tabpanel")).toHaveCount(1);
		await audit(page);
	});
}

for (const path of ["/", "/components.html", "/cli.html", "/tailwind.html"]) {
	test(`${path}: site landmarks, skip link, and WCAG audit`, async ({ page, browserName }) => {
		await page.goto(path);
		await expect(page.getByRole("main")).toHaveCount(1);
		// Safari's default link navigation uses Option-Tab on macOS.
		await page.keyboard.press(browserName === "webkit" && process.platform === "darwin" ? "Alt+Tab" : "Tab");
		await expect(page.getByRole("link", { name: /skip/i })).toBeFocused();
		await page.keyboard.press("Enter");
		await expect(page).toHaveURL(/#main$/);
		await audit(page);
	});
}
