use leptos::window;

pub fn is_ios() -> bool {
  is_iphone() || is_ipad()
}

pub fn is_iphone() -> bool {
  test_platform("iphone")
}

pub fn is_ipad() -> bool {
  test_platform("ipad")
}

pub fn test_platform(search_text: impl AsRef<str>) -> bool {
  window()
    .navigator()
    .platform()
    .unwrap()
    .to_lowercase()
    .contains(search_text.as_ref())
}
