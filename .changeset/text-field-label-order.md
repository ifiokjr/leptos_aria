---
main: patch
---

# Text fields render their label first

`TextField` and `TextArea` now render their `<label>` before the field, and attributes spread onto them (`attr:data-slot`, `attr:autocomplete`, …) reach only the `<input>` or `<textarea>`, matching upstream `useTextField`. Previously the label came after the field and received every spread attribute too, so the registry's `Input` and `Textarea` showed their label below the field. `TextField` also no longer renders an empty `<label>` when it has no label.
