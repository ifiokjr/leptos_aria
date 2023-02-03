use std::marker::PhantomData;

use leptos::js_sys;
use leptos::wasm_bindgen::JsValue;

/// `web_sys::Element` is not hashable meaning it's not possible to us it as a
/// key in a `HashMap`. This is a silly map implementation with no regard for
/// performance that allow us to use elements as keys.
#[derive(Clone, PartialEq, Eq)]
pub struct Map<K, V>(js_sys::Map, PhantomData<(K, V)>)
where
  K: AsRef<JsValue> + From<JsValue>,
  V: AsRef<JsValue> + From<JsValue>;

impl<K, V> Default for Map<K, V>
where
  K: AsRef<JsValue> + From<JsValue>,
  V: AsRef<JsValue> + From<JsValue>,
{
  fn default() -> Self {
    Self(Default::default(), PhantomData)
  }
}

impl<K, V> AsRef<JsValue> for Map<K, V>
where
  K: AsRef<JsValue> + From<JsValue>,
  V: AsRef<JsValue> + From<JsValue>,
{
  fn as_ref(&self) -> &JsValue {
    self.0.as_ref()
  }
}

impl<K, V> From<JsValue> for Map<K, V>
where
  K: AsRef<JsValue> + From<JsValue>,
  V: AsRef<JsValue> + From<JsValue>,
{
  fn from(value: JsValue) -> Self {
    Self(value.into(), PhantomData)
  }
}

impl<K, V> Map<K, V>
where
  K: AsRef<JsValue> + From<JsValue>,
  V: AsRef<JsValue> + From<JsValue>,
{
  /// The Map object holds key-value pairs. Any value (both objects and
  /// primitive values) maybe used as either a key or a value.
  pub fn new() -> Self {
    Default::default()
  }

  /// The `clear()` method removes all elements from a Map object.
  ///
  /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/clear)

  pub fn clear(&self) {
    self.0.clear();
  }

  /// The `delete()` method removes the specified element from a Map object.
  ///
  /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/delete)

  pub fn delete(&self, key: &K) -> bool {
    self.0.delete(key.as_ref())
  }

  /// The `get()` method returns a specified element from a Map object.
  ///
  /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/get)
  pub fn get(&self, key: &K) -> Option<V> {
    if self.has(key) {
      Some(self.0.get(key.as_ref()).into())
    } else {
      None
    }
  }

  /// The set() method adds or updates an element with a specified key and value
  /// to a Map object.
  pub fn set(&self, key: &K, value: &V) {
    self.0.set(key.as_ref(), value.as_ref());
  }

  /// The `has()` method returns a boolean indicating whether an element with
  /// the specified key exists or not.
  pub fn has(&self, key: &K) -> bool {
    self.0.has(key.as_ref())
  }

  /// The `forEach()` method executes a provided function once per each
  /// key/value pair in the Map object, in insertion order.
  /// Note that in Javascript land the `Key` and `Value` are reversed compared
  /// to normal expectations: # Examples
  /// ```
  /// let js_map = Map::new();
  /// js_map.for_each(&mut |value, key| {
  ///   // Do something here...
  /// })
  /// ```
  /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map/forEach)

  pub fn for_each(&self, callback: &mut dyn FnMut(K, V)) {
    self.0.for_each(&mut |value, key| {
      callback(key.into(), value.into());
    });
  }

  pub fn size(&self) -> u32 {
    self.0.size()
  }

  pub fn is_empty(&self) -> bool {
    self.size() == 0
  }

  // pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
  //   self.0.iter_mut().map(|(k, v)| (k, v))
  // }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Set<T>(js_sys::Set, PhantomData<T>)
where
  T: AsRef<JsValue> + From<JsValue>;

impl<T> Default for Set<T>
where
  T: AsRef<JsValue> + From<JsValue>,
{
  fn default() -> Self {
    Self(Default::default(), PhantomData)
  }
}

impl<T> AsRef<JsValue> for Set<T>
where
  T: AsRef<JsValue> + From<JsValue>,
{
  fn as_ref(&self) -> &JsValue {
    &self.0.as_ref()
  }
}

impl<T> From<JsValue> for Set<T>
where
  T: AsRef<JsValue> + From<JsValue>,
{
  fn from(value: JsValue) -> Self {
    Self(value.into(), PhantomData)
  }
}

impl<T> Set<T>
where
  T: AsRef<JsValue> + From<JsValue>,
{
  /// The [`Set`] object lets you store unique values of any type, whether
  /// primitive values or object references.
  ///
  /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set)
  pub fn new() -> Self {
    Default::default()
  }

  /// The `add()` method appends a new element with a specified value to the
  /// end of a [`Set`] object.
  ///
  /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/add)
  pub fn add(&self, value: &T) {
    self.0.add(value.as_ref());
  }

  /// The `clear()` method removes all elements from a [`Set`] object.
  ///
  /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/clear)
  pub fn clear(&self) {
    self.0.clear();
  }

  /// The `delete()` method removes the specified element from a [`Set`]
  /// object.
  ///
  /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/delete)
  pub fn delete(&self, value: &T) -> bool {
    self.0.delete(value.as_ref())
  }

  /// The `forEach()` method executes a provided function once for each value
  /// in the Set object, in insertion order.
  ///
  /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/forEach)
  pub fn for_each(&self, callback: &mut dyn FnMut(T, u32)) {
    self.0.for_each(&mut |value: JsValue, index: JsValue, _| {
      // callback(value.into(), key.into());
    });
  }

  /// The `has()` method returns a boolean indicating whether an element with
  /// the specified value exists in a [`Set`] object or not.
  ///
  /// [MDN documentation](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set/has)
  pub fn has(&self, value: &T) -> bool {
    self.0.has(value.as_ref())
  }

  /// The size accessor property returns the number of elements in a [`Set`]
  /// object.
  ///
  /// [MDN documentation](https://developer.mozilla.org/de/docs/Web/JavaScript/Reference/Global_Objects/Set/size)
  pub fn size(&self) -> u32 {
    self.0.size()
  }

  pub fn is_empty(&self) -> bool {
    self.0.size() == 0
  }
}
