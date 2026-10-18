use js_sys::Array;
use js_sys::Reflect;

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

fn window() -> web_sys::Window {
	web_sys::window().expect("the `window` global is available in the browser")
}

fn test_user_agent(search_text: impl AsRef<str>) -> bool {
	match get_user_agent() {
		Some(user_agent) => user_agent.to_lowercase().contains(search_text.as_ref()),
		None => false,
	}
}

fn get_user_agent() -> Option<String> {
	let navigator = window().navigator();

	Reflect::get(navigator.as_ref(), &"userAgentData".into())
		.ok()
		.and_then(|data| Reflect::get(&data, &"brands".into()).ok())
		.and_then(|brands| {
			Array::from(&brands)
				.map(&mut |value, _, _| Reflect::get(&value, &"brand".into()).unwrap_or_default())
				.join("\n")
				.as_string()
		})
		.or_else(|| navigator.platform().ok())
}

fn test_platform(search_text: impl AsRef<str>) -> bool {
	match get_platform() {
		Some(platform) => platform.to_lowercase().contains(search_text.as_ref()),
		None => false,
	}
}

fn get_platform() -> Option<String> {
	let navigator = window().navigator();

	Reflect::get(navigator.as_ref(), &"userAgentData".into())
		.ok()
		.and_then(|data| Reflect::get(&data, &"platform".into()).ok())
		.and_then(|platform| platform.as_string())
		.or_else(|| navigator.platform().ok())
}
