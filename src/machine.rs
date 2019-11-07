use std::borrow::Borrow;
use std::collections::HashMap;

pub type State = String;
pub type Action = String;
pub type Event = String;

#[derive(Debug)]
struct Transition {
    state: State,
    action: Action,
}

#[derive(Debug)]
pub struct SMachine {
    state: State,
    name: String,
    transitions: HashMap<String, Transition>,
}

impl SMachine {
    pub fn new(state: State, name: String) -> SMachine{
        SMachine {
            state,
            name,
            transitions: HashMap::new(),
        }
    }

    pub fn get_state(&mut self) -> String {
        self.state.clone()
    }

    pub fn add_transition(&mut self, start: State, event: Event, end: State, action: Action) {
        let transition = Transition { state: end, action };
        self.transitions.insert(SMachine::key(start, event).to_string(), transition);
    }

    pub fn on_event(&mut self, event: Event) -> Action {
        let option = self.transitions.get(SMachine::key(self.state.clone(), event).as_str());
        match option {
            Some(trans)=>{
                let trans = option.unwrap().clone();
                self.state=trans.state.clone();
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