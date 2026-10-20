---
main: minor
---

# Every component takes a class prop

Headless components now accept `class` directly (`<AccordionTrigger class="data-[state=open]:font-bold">`) instead of requiring Leptos' `attr:class`. The class is a reactive `Signal<String>`, so strings and closures both work, and it is appended after the built-in `leptos-aria-*` class. Also fixes Pagination's next button, which never fired because an unbraced `>=` in its `disabled` attribute ended the tag early.
