import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
	testDir: "./tests",
	fullyParallel: true,
	forbidOnly: !!process.env.CI,
	retries: 0,
	workers: process.env.CI ? 2 : 4,
	reporter: [["list"], ["html", { open: "never" }]],
	use: {
		baseURL: "http://127.0.0.1:4173",
		trace: "retain-on-failure",
		screenshot: "only-on-failure",
	},
	webServer: {
		command: "npm run serve",
		url: "http://127.0.0.1:4173/preview/index.html",
		reuseExistingServer: false,
	},
	projects: [
		{ name: "chromium", use: { ...devices["Desktop Chrome"] } },
		{ name: "firefox", use: { ...devices["Desktop Firefox"] } },
		{ name: "webkit", use: { ...devices["Desktop Safari"] } },
		{ name: "dark", use: { ...devices["Desktop Chrome"], colorScheme: "dark" } },
		{
			name: "mobile",
			use: { ...devices["iPhone 13"], defaultBrowserType: "chromium", reducedMotion: "reduce" },
		},
	],
});
