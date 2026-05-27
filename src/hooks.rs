use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

type ActionFn = Arc<dyn Fn() + Send + Sync>;
type FilterFn = Arc<dyn Fn(String) -> String + Send + Sync>;

pub struct HookRegistry {
    actions: HashMap<String, Vec<(i32, ActionFn)>>,
    filters: HashMap<String, Vec<(i32, FilterFn)>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
            filters: HashMap::new(),
        }
    }

    pub fn add_action(
        &mut self,
        hook: &str,
        cb: impl Fn() + Send + Sync + 'static,
        priority: i32,
    ) {
        let list = self.actions.entry(hook.to_string()).or_default();
        list.push((priority, Arc::new(cb)));
        list.sort_by_key(|(p, _)| *p);
    }

    pub fn do_action(&self, hook: &str) {
        if let Some(callbacks) = self.actions.get(hook) {
            for (_, cb) in callbacks {
                cb();
            }
        }
    }

    pub fn add_filter(
        &mut self,
        hook: &str,
        cb: impl Fn(String) -> String + Send + Sync + 'static,
        priority: i32,
    ) {
        let list = self.filters.entry(hook.to_string()).or_default();
        list.push((priority, Arc::new(cb)));
        list.sort_by_key(|(p, _)| *p);
    }

    pub fn apply_filters(&self, hook: &str, value: String) -> String {
        match self.filters.get(hook) {
            Some(cbs) => cbs.iter().fold(value, |v, (_, cb)| cb(v)),
            None => value,
        }
    }
}

static INSTANCE: OnceLock<Mutex<HookRegistry>> = OnceLock::new();

fn registry() -> &'static Mutex<HookRegistry> {
    INSTANCE.get_or_init(|| Mutex::new(HookRegistry::new()))
}

/// Register a callback to run when `hook` is triggered.
pub fn add_action(hook: &str, cb: impl Fn() + Send + Sync + 'static, priority: i32) {
    registry().lock().unwrap().add_action(hook, cb, priority);
}

/// Trigger all callbacks registered for `hook`.
pub fn do_action(hook: &str) {
    registry().lock().unwrap().do_action(hook);
}

/// Register a string-transformation callback for `hook`.
pub fn add_filter(
    hook: &str,
    cb: impl Fn(String) -> String + Send + Sync + 'static,
    priority: i32,
) {
    registry().lock().unwrap().add_filter(hook, cb, priority);
}

/// Run `value` through all filter callbacks registered for `hook`.
pub fn apply_filters(hook: &str, value: String) -> String {
    registry().lock().unwrap().apply_filters(hook, value)
}
