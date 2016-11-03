use alloc::arc::{Arc, Weak};
use collections::BTreeMap;
use spin::{Once, RwLock, RwLockReadGuard, RwLockWriteGuard};

use context;
use sync::WaitQueue;
use syscall::data::Event;

type EventList = Weak<WaitQueue<Event>>;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct RegKey {
    scheme_id: usize,
    event_id: usize,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessKey {
    context_id: usize,
    fd: usize,
}

type Registry = BTreeMap<RegKey, BTreeMap<ProcessKey, EventList>>;

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

pub fn register(fd: usize, scheme_id: usize, event_id: usize) -> bool {
    let (context_id, events) = {
        let contexts = context::contexts();
        let context_lock = contexts.current().expect("event::register: No context");
        let context = context_lock.read();
        (context.id, Arc::downgrade(&context.events))
    };

    let mut registry = registry_mut();
    let entry = registry.entry(RegKey {
        scheme_id: scheme_id,
        event_id: event_id
    }).or_insert_with(|| {
        BTreeMap::new()
    });
    let process_key = ProcessKey {
        context_id: context_id,
        fd: fd
    };
    if entry.contains_key(&process_key) {
        false
    } else {
        entry.insert(process_key, events);
        true
    }
}

pub fn unregister(fd: usize, scheme_id: usize, event_id: usize) {
    let mut registry = registry_mut();

    let mut remove = false;
    let key = RegKey {
        scheme_id: scheme_id,
        event_id: event_id
    };
    if let Some(entry) = registry.get_mut(&key) {
        let process_key = ProcessKey {
            context_id: context::context_id(),
            fd: fd,
        };
        entry.remove(&process_key);

        if entry.is_empty() {
            remove = true;
        }
    }

    if remove {
        registry.remove(&key);
    }
}

pub fn trigger(scheme_id: usize, event_id: usize, flags: usize, data: usize) {
    let registry = registry();
    let key = RegKey {
        scheme_id: scheme_id,
        event_id: event_id
    };
    if let Some(event_lists) = registry.get(&key) {
        for entry in event_lists.iter() {
            if let Some(event_list) = entry.1.upgrade() {
                event_list.send(Event {
                    id: (entry.0).fd,
                    flags: flags,
                    data: data
                });
            }
        }
    }
}
