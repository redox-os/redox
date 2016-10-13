use alloc::arc::{Arc, Weak};
use collections::BTreeMap;
use spin::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

use context;
use sync::WaitQueue;
use syscall::data::Event;

type EventList = Weak<WaitQueue<Event>>;

type Registry = BTreeMap<(usize, usize), BTreeMap<(usize, usize), EventList>>;

static REGISTRY: Once<RwLock<Registry>> = Once::new();

/// Initialize registry, called if needed
fn init_registry() -> RwLock<Registry> {
    RwLock::new(Registry::new())
}

/// Get the global schemes list, const
fn registry() -> RwLockReadGuard<'static, Registry> {
    REGISTRY.call_once(init_registry).read()
}

/// Get the global schemes list, mutable
pub fn registry_mut() -> RwLockWriteGuard<'static, Registry> {
    REGISTRY.call_once(init_registry).write()
}

pub fn register(fd: usize, scheme_id: usize, id: usize) -> bool {
    let (context_id, events) = {
        let contexts = context::contexts();
        let context_lock = contexts.current().expect("event::register: No context");
        let context = context_lock.read();
        (context.id, Arc::downgrade(&context.events))
    };

    let mut registry = registry_mut();
    let entry = registry.entry((scheme_id, id)).or_insert_with(|| {
        BTreeMap::new()
    });
    if entry.contains_key(&(context_id, fd)) {
        false
    } else {
        entry.insert((context_id, fd), events);
        true
    }
}

pub fn unregister(fd: usize, scheme_id: usize, id: usize) {
    let mut registry = registry_mut();

    let mut remove = false;
    if let Some(entry) = registry.get_mut(&(scheme_id, id)) {
        entry.remove(&(context::context_id(), fd));

        if entry.is_empty() {
            remove = true;
        }
    }

    if remove {
        registry.remove(&(scheme_id, id));
    }
}

pub fn trigger(scheme_id: usize, id: usize, flags: usize, data: usize) {
    let registry = registry();
    if let Some(event_lists) = registry.get(&(scheme_id, id)) {
        for entry in event_lists.iter() {
            if let Some(event_list) = entry.1.upgrade() {
                event_list.send(Event {
                    id: (entry.0).1,
                    flags: flags,
                    data: data
                });
            }
        }
    }
}
