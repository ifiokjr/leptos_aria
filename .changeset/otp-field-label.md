---
main: patch
---

# One-time password field labels its group

`OneTimePasswordField` now renders its label before the cells, as a `span` that names the `role="group"` through `aria-labelledby`. Before, it rendered a `<label for>` after the group, pointing at an id no element had. Attributes spread onto the field (`attr:data-slot`, …) now reach only the group, not the label. `use_labels` also stops emitting an empty `aria-labelledby=""` on unlabelled fields: text fields without a visible label, and progress bars.
