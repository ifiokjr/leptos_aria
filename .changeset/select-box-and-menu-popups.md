---
main: minor
---

# Select box and menus manage their own popups

SelectBox shows its chosen option on the trigger, hides its list until opened, and closes on a choice, escape, or a press outside, moving focus into the list and back. It takes value, default_value, on_change, and placeholder, plus trigger_class, popup_class, option_class, an indicator for each option, and trigger children. Its class now sits on a new root element. Menu runs the chosen item, by click or with Enter or Space on the highlighted one, through on_action on the menu and on each MenuItem, then closes, and skips disabled items. MenuTrigger takes an is_open signal, shares it with the Menu inside, and closes on an outside press. ListBox sets aria-activedescendant to the highlighted option and drops aria-labelledby when it has no label. The overlay crate adds use_popup_focus, which moves focus into a popup on open and back to its trigger on close.
