import AxeBuilder from "@axe-core/playwright";
import { expect, type Page, test as base } from "@playwright/test";
import registry from "../../registry/registry.json";

export const components = registry.items.filter((item) => item.type === "registry:ui");

/** Keep browser errors fatal and examples independent of external services. */
export const test = base.extend<{ browserErrors: string[] }>({
	browserErrors: [async ({ page, context }, use) => {
		const errors: string[] = [];
		page.on("pageerror", (error) => errors.push(error.message));
		await context.route("**/*", (route) => {
			const url = new URL(route.request().url());

			if (url.hostname === "127.0.0.1") {
				return route.continue();
			}

			return route.abort();
		});
		await use(errors);
		expect(errors, "uncaught errors in the real WASM app").toEqual([]);
	}, { auto: true }],
});

export { expect };

/** Wait for the generated Rust example to mount before interacting or auditing. */
export async function preview(page: Page, name: string) {
	await page.goto(`/preview/index.html?example=${name}`);
	await expect(page.locator("#preview > *").first()).toBeAttached();
	await expect(page.locator("#preview")).not.toContainText("No example named");
}

/** Audit every WCAG A/AA rule axe supports, without exclusions or disabled rules. */
export async function audit(page: Page) {
	const result = await new AxeBuilder({ page })
		.withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
		.analyze();

	expect(result.violations.map(({ id, nodes }) => ({
		id,
		nodes: nodes.map(({ target, failureSummary }) => ({ target, failureSummary })),
	}))).toEqual([]);
}
