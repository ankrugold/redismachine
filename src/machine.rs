use std::borrow::Borrow;
use std::collections::HashMap;
use redis_module::raw;
use std::os::raw::c_void;
use std::os::raw::c_int;
use std::convert::TryInto;

pub type State = String;
pub type Action = String;
pub type Event = String;

#[derive(Debug)]
pub struct Transition {
    pub state: State,
    pub action: Action,
}

#[derive(Debug)]
pub struct SMachine {
    pub state: State,
    pub name: String,
    pub transitions: HashMap<String, Transition>,
}

impl SMachine {
    pub fn new(state: State, name: String) -> SMachine {
        SMachine {
            state,
            name,
            transitions: HashMap::new(),
        }
    }

    pub fn des(state: State, name: String, trans: HashMap<String, Transition>) -> SMachine {
        SMachine {
            state,
            name,
            transitions: trans,
        }
    }

    pub fn get_state(&mut self) -> String {
        self.state.clone()
    }

    pub fn get_name(&mut self) -> String {
        self.name.clone()
    }

    pub fn add_transition(&mut self, start: State, event: Event, end: State, action: Action) {
        let transition = Transition { state: end, action };
        self.transitions.insert(SMachine::key(start, event).to_string(), transition);
    }

    pub fn on_event(&mut self, event: Event) -> Action {
        let option = self.transitions.get(SMachine::key(self.state.clone(), event).as_str());
        match option {
            Some(trans) => {
                let trans = option.unwrap().clone();
                self.state = trans.state.clone();
                trans.action.clone()
            }
            None => {
                "".to_string()
            }
        }
    }

    fn key<'a>(start: State, event: Event) -> String {
        [start.as_str(), event.as_str()].concat()
    }
}

//redis specific code to save and load the machine

#[no_mangle]
pub unsafe extern "C" fn RedisMachineRdbSave(rdb: *mut raw::RedisModuleIO, value: *mut c_void) {
    if value.is_null() {
        return;
    }
    let m = &*(value as *mut SMachine);
    raw::save_string(rdb, &m.name.to_string());
    raw::save_string(rdb, &m.state.to_string());
    let mut trans = &m.transitions;
    let i = trans.len() as u64;
    raw::save_unsigned(rdb, i);
    for (k, v) in trans {
        raw::save_string(rdb, k);
        raw::save_string(rdb, &v.state.clone());
        raw::save_string(rdb, &v.action.clone());
    }
}

#[no_mangle]
pub unsafe extern "C" fn RedisMachineRdbLoad(rdb: *mut raw::RedisModuleIO, encver: c_int) -> *mut c_void {
    let name = raw::load_string(rdb);
    let state = raw::load_string(rdb);
    let num_trans = raw::load_unsigned(rdb);
    let mut map = HashMap::new();
    for _ in 0..num_trans {
        let key = raw::load_string(rdb);
        let state = raw::load_string(rdb);
        let action = raw::load_string(rdb);
        map.insert(key, Transition { state, action });
    }
    let mut omachine = SMachine::des(state, name, map);
    Box::into_raw(Box::new(omachine)) as *mut c_void
}


pub unsafe extern "C" fn Free(value: *mut c_void) {
    Box::from_raw(value as *mut SMachine);
}