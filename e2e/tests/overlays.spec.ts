import { audit, expect, preview, test } from "./fixtures";

test("dialog: keyboard opens, traps focus, closes, and restores the trigger", async ({ page }) => {
	await preview(page, "dialog");
	const trigger = page.getByRole("button", { name: "Edit profile", exact: true });
	await page.keyboard.press("Tab");
	await expect(trigger).toBeFocused();
	await page.keyboard.press("Enter");
	const dialog = page.getByRole("dialog", { name: "Edit profile" });
	await expect(dialog).toBeVisible();
	await expect(dialog).toHaveAttribute("aria-modal", "true");
	await expect(dialog).toHaveAccessibleDescription("Make changes to your profile here.");
	await expect(dialog.locator(":focus")).toHaveCount(1);
	await audit(page);

	for (let step = 0; step < 8; step++) {
		await page.keyboard.press(step < 4 ? "Tab" : "Shift+Tab");
		await expect(dialog.locator(":focus")).toHaveCount(1);
	}

	await page.keyboard.press("Escape");
	await expect(dialog).toBeHidden();
	await expect(trigger).toBeFocused();
	await page.keyboard.press("Enter");
	await dialog.getByRole("button", { name: "Cancel", exact: true }).click();
	await expect(dialog).toBeHidden();
	await expect(trigger).toBeFocused();
});

test("alert dialog: cancel receives focus and dismissal restores the trigger", async ({ page }) => {
	await preview(page, "alert-dialog");
	const trigger = page.getByRole("button", { name: "Delete account" });
	await trigger.focus();
	await page.keyboard.press("Enter");
	const dialog = page.getByRole("alertdialog", { name: "Are you absolutely sure?" });
	await expect(dialog).toBeVisible();
	await expect(dialog).toHaveAccessibleDescription("This permanently deletes your account and its data.");
	await expect(dialog.getByRole("button", { name: "Cancel" })).toBeFocused();
	await audit(page);
	await page.keyboard.press("Enter");
	await expect(dialog).toBeHidden();
	await expect(trigger).toBeFocused();
});

test("dropdown menu: keyboard reaches enabled items, skips disabled, and Escape restores focus", async ({ page }) => {
	await preview(page, "dropdown-menu");
	const trigger = page.getByRole("button", { name: "Open", exact: true });
	await trigger.focus();
	await page.keyboard.press("Enter");
	const menu = page.getByRole("menu", { name: "Account" });
	await expect(menu).toBeVisible();
	await expect(trigger).toHaveAttribute("aria-expanded", "true");
	await expect(page.getByRole("menuitem", { name: "Billing" })).toHaveAttribute("aria-disabled", "true");
	await page.keyboard.press("ArrowDown");
	await expect(menu).toBeFocused();
	await expect(page.getByRole("menuitem", { name: "Profile" })).toHaveAttribute("data-focused", "true");
	await page.keyboard.press("ArrowDown");
	await expect(page.getByRole("menuitem", { name: "Profile" })).toHaveAttribute("data-focused", "true");
	await audit(page);
	await page.keyboard.press("Escape");
	await expect(menu).toBeHidden();
	await expect(trigger).toBeFocused();
});

test("select: keyboard chooses an option and restores the combobox", async ({ page }) => {
	await preview(page, "select");
	const trigger = page.getByRole("combobox", { name: "Fruit" });
	await trigger.focus();
	await page.keyboard.press("Enter");
	await expect(trigger).toHaveAttribute("aria-expanded", "true");
	const list = page.getByRole("listbox");
	await expect(list).toBeFocused();
	await page.keyboard.press("End");
	await expect(page.getByRole("option", { name: "Banana" })).toHaveAttribute("data-focused", "true");
	await audit(page);
	await page.keyboard.press("Enter");
	await expect(trigger).toContainText("Banana");
	await expect(trigger).toHaveAttribute("aria-expanded", "false");
	await expect(trigger).toBeFocused();
	await page.keyboard.press("Enter");
	await page.keyboard.press("Escape");
	await expect(trigger).toBeFocused();
	await expect(trigger).toContainText("Banana");
});

test("popover: keyboard opens and Escape closes with focus restored", async ({ page }) => {
	await preview(page, "popover");
	const trigger = page.getByRole("button", { name: "Dimensions" });
	await trigger.focus();
	await page.keyboard.press("Enter");
	await expect(trigger).toHaveAttribute("aria-expanded", "true");
	await expect(page.getByText("Set the dimensions for the layer.")).toBeVisible();
	await audit(page);
	await page.keyboard.press("Escape");
	await expect(page.getByText("Set the dimensions for the layer.")).toBeHidden();
	await expect(trigger).toBeFocused();
});

test("tooltip: keyboard focus exposes its accessible description", async ({ page }) => {
	await preview(page, "tooltip");
	const trigger = page.getByRole("button", { name: "Add", exact: true });
	await page.keyboard.press("Tab");
	await expect(trigger).toBeFocused();
	await expect(page.getByRole("tooltip")).toHaveText("Add to library");
	await expect(trigger).toHaveAccessibleDescription("Add to library");
	await audit(page);
	await page.keyboard.press("Escape");
	await expect(page.getByRole("tooltip")).toBeHidden();
	await expect(trigger).toBeFocused();
	await page.keyboard.press("Tab");
	await expect(page.getByRole("tooltip")).toBeHidden();
});

test("tooltip: pointer hover opens and leaving closes the bubble", async ({ page }) => {
	await preview(page, "tooltip");
	await page.getByRole("button", { name: "Add", exact: true }).hover();
	await expect(page.getByRole("tooltip")).toBeVisible();
	await page.mouse.move(0, 0);
	await expect(page.getByRole("tooltip")).toBeHidden();
});

test("hover card: pointer reveals supplementary content", async ({ page }) => {
	await preview(page, "hover-card");
	await page.getByRole("link", { name: "@leptos" }).hover();
	await expect(page.getByText("The Rust framework for the web.")).toBeVisible();
	await audit(page);
});
