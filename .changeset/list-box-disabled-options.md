---
main: patch
---

# List box options honour their disabled state

A ListBoxOption with is_disabled set can no longer be selected by a click or by Enter or Space, and arrow keys, Home, End, and type-ahead skip it. The flag is read when it is needed, so an option disabled after it gained focus is ignored too. ListBoxContext gains a disabled_options field, register_option, which records an option's key with its disabled flag, and is_key_disabled; register_key still registers an enabled option. Type-ahead without a focused option now checks the first option first. Menu relies on the list box for disabled items instead of guarding them itself.
