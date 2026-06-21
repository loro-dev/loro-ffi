use std::sync::Arc;

use loro::{ContainerTrait, LoroResult, PeerID};

use crate::{
    ContainerID, DiffEvent, LoroDoc, LoroValue, LoroValueLike, Subscriber, Subscription,
    ValueOrContainer,
};

use super::{LoroCounter, LoroList, LoroMovableList, LoroText, LoroTree};

#[derive(Debug, Clone)]
pub struct LoroMap {
    pub(crate) inner: loro::LoroMap,
}

impl LoroMap {
    pub fn new() -> Self {
        Self {
            inner: loro::LoroMap::new(),
        }
    }

    pub fn is_attached(&self) -> bool {
        self.inner.is_attached()
    }

    /// If a detached container is attached, this method will return its corresponding attached handler.
    pub fn get_attached(&self) -> Option<Arc<LoroMap>> {
        self.inner
            .get_attached()
            .map(|x| Arc::new(LoroMap { inner: x }))
    }

    /// Delete a key-value pair from the map.
    pub fn delete(&self, key: &str) -> LoroResult<()> {
        self.inner.delete(key)
    }

    /// Iterate over the key-value pairs of the map.
    // pub fn for_each<I>(&self, f: I)
    // where
    //     I: FnMut(&str, loro::ValueOrContainer),
    // {
    //     self.map.for_each(f)
    // }
    /// Insert a key-value pair into the map.
    pub fn insert(&self, key: &str, value: Arc<dyn LoroValueLike>) -> LoroResult<()> {
        self.inner.insert(key, value.as_loro_value())
    }

    /// Get the length of the map.
    pub fn len(&self) -> u32 {
        self.inner.len() as u32
    }

    /// Get the ID of the map.
    pub fn id(&self) -> ContainerID {
        self.inner.id().into()
    }

    /// Whether the map is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the value of the map with the given key.
    pub fn get(&self, key: &str) -> Option<Arc<dyn ValueOrContainer>> {
        self.inner
            .get(key)
            .map(|v| Arc::new(v) as Arc<dyn ValueOrContainer>)
    }

    pub fn get_or_create_text_container(
        &self,
        key: &str,
        text: Arc<LoroText>,
    ) -> LoroResult<Arc<LoroText>> {
        #[allow(deprecated)]
        let c = self
            .inner
            .get_or_create_container(key, text.as_ref().clone().inner)?;
        Ok(Arc::new(LoroText { inner: c }))
    }

    pub fn get_or_create_map_container(
        &self,
        key: &str,
        map: Arc<LoroMap>,
    ) -> LoroResult<Arc<LoroMap>> {
        #[allow(deprecated)]
        let c = self
            .inner
            .get_or_create_container(key, map.as_ref().clone().inner)?;
        Ok(Arc::new(LoroMap { inner: c }))
    }

    pub fn get_or_create_tree_container(
        &self,
        key: &str,
        tree: Arc<LoroTree>,
    ) -> LoroResult<Arc<LoroTree>> {
        #[allow(deprecated)]
        let c = self
            .inner
            .get_or_create_container(key, tree.as_ref().clone().inner)?;
        Ok(Arc::new(LoroTree { inner: c }))
    }

    pub fn get_or_create_list_container(
        &self,
        key: &str,
        list: Arc<LoroList>,
    ) -> LoroResult<Arc<LoroList>> {
        #[allow(deprecated)]
        let c = self
            .inner
            .get_or_create_container(key, list.as_ref().clone().inner)?;
        Ok(Arc::new(LoroList { inner: c }))
    }

    pub fn get_or_create_movable_list_container(
        &self,
        key: &str,
        list: Arc<LoroMovableList>,
    ) -> LoroResult<Arc<LoroMovableList>> {
        #[allow(deprecated)]
        let c = self
            .inner
            .get_or_create_container(key, list.as_ref().clone().inner)?;
        Ok(Arc::new(LoroMovableList { inner: c }))
    }

    pub fn get_or_create_counter_container(
        &self,
        key: &str,
        counter: Arc<LoroCounter>,
    ) -> LoroResult<Arc<LoroCounter>> {
        #[allow(deprecated)]
        let c = self
            .inner
            .get_or_create_container(key, counter.as_ref().clone().inner)?;
        Ok(Arc::new(LoroCounter { inner: c }))
    }

    pub fn ensure_mergeable_list(&self, key: &str) -> LoroResult<Arc<LoroList>> {
        let c = self.inner.ensure_mergeable_list(key)?;
        Ok(Arc::new(LoroList { inner: c }))
    }

    pub fn ensure_mergeable_map(&self, key: &str) -> LoroResult<Arc<LoroMap>> {
        let c = self.inner.ensure_mergeable_map(key)?;
        Ok(Arc::new(LoroMap { inner: c }))
    }

    pub fn ensure_mergeable_tree(&self, key: &str) -> LoroResult<Arc<LoroTree>> {
        let c = self.inner.ensure_mergeable_tree(key)?;
        Ok(Arc::new(LoroTree { inner: c }))
    }

    pub fn ensure_mergeable_movable_list(&self, key: &str) -> LoroResult<Arc<LoroMovableList>> {
        let c = self.inner.ensure_mergeable_movable_list(key)?;
        Ok(Arc::new(LoroMovableList { inner: c }))
    }

    pub fn ensure_mergeable_text(&self, key: &str) -> LoroResult<Arc<LoroText>> {
        let c = self.inner.ensure_mergeable_text(key)?;
        Ok(Arc::new(LoroText { inner: c }))
    }

    pub fn ensure_mergeable_counter(&self, key: &str) -> LoroResult<Arc<LoroCounter>> {
        let c = self.inner.ensure_mergeable_counter(key)?;
        Ok(Arc::new(LoroCounter { inner: c }))
    }

    #[inline]
    pub fn insert_list_container(
        &self,
        key: &str,
        child: Arc<LoroList>,
    ) -> LoroResult<Arc<LoroList>> {
        let c = self
            .inner
            .insert_container(key, child.as_ref().clone().inner)?;
        Ok(Arc::new(LoroList { inner: c }))
    }

    #[inline]
    pub fn insert_map_container(&self, key: &str, child: Arc<LoroMap>) -> LoroResult<Arc<LoroMap>> {
        let c = self
            .inner
            .insert_container(key, child.as_ref().clone().inner)?;
        Ok(Arc::new(LoroMap { inner: c }))
    }

    #[inline]
    pub fn insert_text_container(
        &self,
        key: &str,
        child: Arc<LoroText>,
    ) -> LoroResult<Arc<LoroText>> {
        let c = self
            .inner
            .insert_container(key, child.as_ref().clone().inner)?;
        Ok(Arc::new(LoroText { inner: c }))
    }

    #[inline]
    pub fn insert_tree_container(
        &self,
        key: &str,
        child: Arc<LoroTree>,
    ) -> LoroResult<Arc<LoroTree>> {
        let c = self
            .inner
            .insert_container(key, child.as_ref().clone().inner)?;
        Ok(Arc::new(LoroTree { inner: c }))
    }

    #[inline]
    pub fn insert_movable_list_container(
        &self,
        key: &str,
        child: Arc<LoroMovableList>,
    ) -> LoroResult<Arc<LoroMovableList>> {
        let c = self
            .inner
            .insert_container(key, child.as_ref().clone().inner)?;
        Ok(Arc::new(LoroMovableList { inner: c }))
    }

    #[inline]
    pub fn insert_counter_container(
        &self,
        key: &str,
        child: Arc<LoroCounter>,
    ) -> LoroResult<Arc<LoroCounter>> {
        let c = self
            .inner
            .insert_container(key, child.as_ref().clone().inner)?;
        Ok(Arc::new(LoroCounter { inner: c }))
    }

    /// Get the shallow value of the map.
    ///
    /// It will not convert the state of sub-containers, but represent them as [LoroValue::Container].
    pub fn get_value(&self) -> LoroValue {
        self.inner.get_value().into()
    }

    /// Get the deep value of the map.
    ///
    /// It will convert the state of sub-containers into a nested JSON value.
    pub fn get_deep_value(&self) -> LoroValue {
        self.inner.get_deep_value().into()
    }

    pub fn is_deleted(&self) -> bool {
        self.inner.is_deleted()
    }

    pub fn get_last_editor(&self, key: &str) -> Option<PeerID> {
        self.inner.get_last_editor(key)
    }

    pub fn clear(&self) -> LoroResult<()> {
        self.inner.clear()
    }

    pub fn keys(&self) -> Vec<String> {
        self.inner.keys().map(|k| k.to_string()).collect()
    }

    pub fn values(&self) -> Vec<Arc<dyn ValueOrContainer>> {
        self.inner
            .values()
            .map(|v| Arc::new(v) as Arc<dyn ValueOrContainer>)
            .collect()
    }

    pub fn doc(&self) -> Option<Arc<LoroDoc>> {
        self.inner.doc().map(|x| Arc::new(LoroDoc { doc: x }))
    }

    pub fn subscribe(&self, subscriber: Arc<dyn Subscriber>) -> Option<Arc<Subscription>> {
        self.inner
            .subscribe(Arc::new(move |e| {
                subscriber.on_diff(DiffEvent::from(e));
            }))
            .map(|x| Arc::new(x.into()))
    }
}

impl Default for LoroMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{ContainerIdLike, LoroValue};

    use super::*;

    struct TestValue(LoroValue);

    impl LoroValueLike for TestValue {
        fn as_loro_value(&self) -> LoroValue {
            self.0.clone()
        }
    }

    fn value(value: LoroValue) -> Arc<dyn LoroValueLike> {
        Arc::new(TestValue(value))
    }

    fn root_map() -> Arc<LoroMap> {
        let doc = LoroDoc::new();
        doc.get_map(Arc::new(String::from("root")) as Arc<dyn ContainerIdLike>)
    }

    #[test]
    fn ensure_mergeable_containers_are_stable_and_usable() {
        let root = root_map();

        let text = root.ensure_mergeable_text("body").unwrap();
        text.insert(0, "hello").unwrap();
        let same_text = root.ensure_mergeable_text("body").unwrap();
        assert_eq!(format!("{:?}", text.id()), format!("{:?}", same_text.id()));
        assert_eq!(same_text.slice(0, 5).unwrap(), "hello");

        let map = root.ensure_mergeable_map("meta").unwrap();
        map.insert(
            "name",
            value(LoroValue::String {
                value: String::from("Ada"),
            }),
        )
        .unwrap();
        assert!(map.get("name").unwrap().is_value());

        let list = root.ensure_mergeable_list("items").unwrap();
        list.push(value(LoroValue::I64 { value: 42 })).unwrap();
        assert_eq!(list.len(), 1);

        let counter = root.ensure_mergeable_counter("revision").unwrap();
        counter.increment(2.0).unwrap();
        assert_eq!(counter.get_value(), 2.0);
    }

    #[test]
    fn ensure_mergeable_rejects_non_mergeable_value() {
        let root = root_map();
        root.insert("plain", value(LoroValue::I64 { value: 1 }))
            .unwrap();

        assert!(root.ensure_mergeable_map("plain").is_err());
    }
}
