use leptos::js_sys::Array;
use leptos::js_sys::Reflect;
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

pub fn is_android() -> bool {
  test_user_agent("android")
}

pub fn is_chrome() -> bool {
  test_user_agent("chrome")
}

pub fn is_mac() -> bool {
  test_platform("mac")
}

pub fn is_apple_device() -> bool {
  is_ios() || is_mac()
}

pub fn is_webkit() -> bool {
  test_user_agent("applewebkit") && !is_chrome()
}

fn test_user_agent(search_text: impl AsRef<str>) -> bool {
  match get_user_agent() {
    Some(user_agent) => user_agent.to_lowercase().contains(search_text.as_ref()),
    None => false,
  }
}

fn get_user_agent() -> Option<String> {
  Reflect::get(&window().navigator(), &"userAgentData".into())
    .ok()
    .and_then(|data| Reflect::get(&data, &"brands".into()).ok())
    .and_then(|v| {
      Array::from(&v)
        .map(&mut |value, _, _| Reflect::get(&value, &"brand".into()).unwrap_or_default())
        .join("\n")
        .as_string()
    })
    .or_else(|| window().navigator().platform().ok())
}

fn test_platform(search_text: impl AsRef<str>) -> bool {
  match get_platform() {
    Some(platform) => platform.to_lowercase().contains(search_text.as_ref()),
    None => false,
  }
}

fn get_platform() -> Option<String> {
  Reflect::get(&window().navigator(), &"userAgentData".into())
    .ok()
    .and_then(|data| Reflect::get(&data, &"platform".into()).ok())
    .and_then(|platform| platform.as_string())
    .or_else(|| window().navigator().platform().ok())
}
