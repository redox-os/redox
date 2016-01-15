use collections::string::String;
use collections::BTreeMap;

use scheduler::context::context_switch;

pub struct Scheme {
    next_id: i64,
    todo: BTreeMap<i64, (i64, i64, i64, i64)>,
    done: BTreeMap<i64, (i64, i64, i64, i64)>,
}

impl Scheme {
    pub fn new() -> Scheme {
        Scheme {
            next_id: 1,
            todo: BTreeMap::new(),
            done: BTreeMap::new(),
        }
    }

    fn call(&mut self, regs: &mut (i64, i64, i64, i64)) {
        let id = self.next_id;

        //TODO: What should be done about collisions in self.todo or self.done?
        self.next_id += 1;
        if self.next_id <= 0 {
            self.next_id = 1;
        }

        self.todo.insert(id, *regs);

        loop {
            if let Some(new_regs) = self.done.remove(&id) {
                *regs = new_regs;
                return
            }

            unsafe { context_switch(false) } ;
        }
    }
}
