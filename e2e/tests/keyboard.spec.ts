import { audit, expect, preview, test } from "./fixtures";

test("tabs: focus and selection travel together, wrap, and restore with Home/End", async ({ page }) => {
	await preview(page, "tabs");
	const account = page.getByRole("tab", { name: "Account" });
	const password = page.getByRole("tab", { name: "Password" });
	await expect(page.getByRole("tab", { name: "Billing" })).toBeDisabled();
	await page.keyboard.press("Tab");
	await expect(account).toBeFocused();

	for (
		const [key, selected, other, content] of [
			["ArrowRight", password, account, "Change your password."],
			["ArrowRight", account, password, "Change your account details."],
			["ArrowLeft", password, account, "Change your password."],
			["Home", account, password, "Change your account details."],
			["End", password, account, "Change your password."],
		] as const
	) {
		await page.keyboard.press(key);
		await expect(selected).toBeFocused();
		await expect(selected).toHaveAttribute("aria-selected", "true");
		await expect(selected).toHaveAttribute("tabindex", "0");
		await expect(other).toHaveAttribute("aria-selected", "false");
		await expect(other).toHaveAttribute("tabindex", "-1");
		await expect(page.getByRole("tabpanel")).toHaveText(content);
	}

	await account.click();
	await expect(account).toBeFocused();
	await expect(account).toHaveAttribute("aria-selected", "true");
	await audit(page);
});

test("accordion: keyboard activation updates expansion and its labelled panel", async ({ page }) => {
	await preview(page, "accordion");
	const shipping = page.getByRole("button", { name: "Shipping" });
	const returns = page.getByRole("button", { name: "Returns" });
	await expect(shipping).toHaveAttribute("aria-expanded", "true");
	await shipping.focus();
	await page.keyboard.press("Space");
	await expect(shipping).toHaveAttribute("aria-expanded", "false");
	await expect(page.getByText("Orders ship within two working days.")).toBeHidden();
	await returns.focus();
	await page.keyboard.press("Enter");
	await expect(returns).toHaveAttribute("aria-expanded", "true");
	await expect(page.getByRole("region", { name: "Returns" })).toBeVisible();
	await audit(page);
});

for (
	const [name, role, label, attribute] of [
		["checkbox", "checkbox", "Accept terms and conditions", "aria-checked"],
		["switch", "switch", "Airplane mode", "aria-checked"],
		["toggle", "button", "Bold", "aria-pressed"],
	] as const
) {
	test(`${name}: keyboard toggles the accessible state twice`, async ({ page }) => {
		await preview(page, name);
		const control = page.getByRole(role, { name: label });
		await page.keyboard.press("Tab");
		await expect(control).toBeFocused();

		for (const selected of [true, false]) {
			await page.keyboard.press("Space");

			if (role === "checkbox" || role === "switch") {
				await expect(control).toBeChecked({ checked: selected });
			} else {
				await expect(control).toHaveAttribute(attribute, String(selected));
			}
		}

		await audit(page);
	});
}

test("button: disabled actions are skipped by keyboard focus", async ({ page }) => {
	await preview(page, "button");
	await page.keyboard.press("Tab");
	await expect(page.getByRole("button", { name: "Save", exact: true })).toBeFocused();
	await page.keyboard.press("Enter");
	await page.keyboard.press("Tab");
	await expect(page.getByRole("button", { name: "Cancel" })).toBeFocused();
	await expect(page.getByRole("button", { name: "Delete" })).toBeDisabled();
	await page.keyboard.press("Tab");
	await expect(page.getByRole("button", { name: "Delete" })).not.toBeFocused();
});

for (
	const [name, label, value] of [
		["input", "Email", "reader@example.test"],
		["label", "Email", "reader@example.test"],
		["textarea", "Message", "A keyboard accessible message.\nWith a second line."],
	] as const
) {
	test(`${name}: the visible label names an editable field`, async ({ page }) => {
		await preview(page, name);
		const field = page.getByRole("textbox", { name: label });
		await page.keyboard.press("Tab");
		await expect(field).toBeFocused();
		await field.fill(value);
		await expect(field).toHaveValue(value);
		await audit(page);
	});
}

test("radio group: arrows change selection and focus without extra Tab stops", async ({ page }) => {
	await preview(page, "radio-group");
	const comfortable = page.getByRole("radio", { name: "Comfortable", exact: true });
	const compact = page.getByRole("radio", { name: "Compact", exact: true });
	await expect(page.getByRole("radiogroup", { name: "Density" })).toBeVisible();
	await page.keyboard.press("Tab");
	await expect(comfortable).toBeFocused();
	await page.keyboard.press("ArrowDown");
	await expect(compact).toBeFocused();
	await expect(compact).toBeChecked();
	await expect(comfortable).not.toBeChecked();
	await page.keyboard.press("ArrowDown");
	await expect(page.getByRole("radio", { name: "Default", exact: true })).toBeChecked();
	await audit(page);
});

test("slider: arrows, Home, and End expose the current value", async ({ page }) => {
	await preview(page, "slider");
	const slider = page.getByRole("slider", { name: "Volume" });
	await slider.focus();
	await expect(slider).toHaveValue("40");

	for (const [key, value] of [["ArrowRight", "45"], ["ArrowLeft", "40"], ["Home", "0"], ["End", "100"]] as const) {
		await page.keyboard.press(key);
		await expect(slider).toHaveValue(value);
	}

	await audit(page);
});

test("toggle group: changing alignment leaves only one item pressed", async ({ page }) => {
	await preview(page, "toggle-group");
	const left = page.getByRole("button", { name: "Align left" });
	const center = page.getByRole("button", { name: "Align center" });
	await expect(left).toHaveAttribute("aria-pressed", "true");
	await center.focus();
	await page.keyboard.press("Space");
	await expect(center).toHaveAttribute("aria-pressed", "true");
	await expect(left).toHaveAttribute("aria-pressed", "false");
	await audit(page);
});

test("pagination: keyboard navigation updates the current page and boundaries", async ({ page }) => {
	await preview(page, "pagination");
	await expect(page.getByRole("button", { name: "Previous page" })).toBeDisabled();
	const next = page.getByRole("button", { name: "Next page" });
	await next.focus();
	await page.keyboard.press("Enter");
	await expect(page.locator("[aria-current=\"page\"]")).toHaveText("2");
	const last = page.getByRole("button", { name: "Page 5", exact: true });
	await last.focus();
	await page.keyboard.press("Enter");
	await expect(next).toBeDisabled();
	await audit(page);
});
