use std::sync::Mutex;

pub type Backtrace = Vec<(&'static str, u64)>;

pub type ReturnStack = Vec<u64>;

pub struct BacktraceState {
    pub backtrace: Backtrace,
    pub return_stack: ReturnStack,
    pub branch_flag: bool,
}

impl Default for BacktraceState {
    fn default() -> Self {
        Self {
            backtrace: Vec::new(),
            return_stack: Vec::new(),
            branch_flag: false,
        }
    }
}

pub struct PidBacktraceCtx(pub Mutex<BacktraceState>);

impl PidBacktraceCtx {
    pub fn push_backtrace(&self, symbol: &'static str, addr: u64) {
        let backtrace_state: &mut BacktraceState = &mut self.0.lock().unwrap();
        backtrace_state    
            .backtrace
            .push((
                symbol,
                addr
            ));
    }
    pub fn pop_backtrace(&self) -> Option<(&'static str, u64)> {
        let backtrace_state = &mut self.0.lock().unwrap();
        backtrace_state
            .backtrace.pop()
        // println!("Returning from function at {:08x}, func {} at {:08x}", 
        //    pc.addr(), curr_func.0, curr_func.1);
    }

    pub fn is_unlinked_branch(&self, ra: u64) -> bool {
        // check if this is b or bl
        // if just branch, ra will match top of stack
        let backtrace_state = &mut self.0.lock().unwrap();
        let len = backtrace_state
            .return_stack
            .len();
        if (len > 0) &&
            (*backtrace_state
            .return_stack
            .last()
            .unwrap() == ra) {
            return true
        }
        false
    }

    pub fn push_return_stack(&self, ra: u64) {
        let backtrace_state = &mut self.0.lock().unwrap();
        // push link register to the return stack
        backtrace_state
            .return_stack
            .push(ra);
        // println!("{}", pid.0.lock().unwrap().backtrace);
        // println!("{}", pid.0.lock().unwrap().return_stack);
    }

    pub fn pop_return_stack(&self) -> Option<u64> {
        let backtrace_state = &mut self.0.lock().unwrap();
        backtrace_state
            .return_stack.pop()
    }

    pub fn is_return(&self, pc: u64) -> bool {
        let backtrace_state = &mut self.0.lock().unwrap();
        let last: Option<&u64> = 
            backtrace_state.return_stack.last();
        match last {
            Some(&addr) => {
                if addr == pc {
                    return true
                } else {
                    return false
                }
            },
            None => false,
        }
    }
    
    pub fn is_branch(&self) -> bool {
        self.0.lock().unwrap().branch_flag
    }

    pub fn set_branch_flag(&self) {
        self.0.lock().unwrap().branch_flag = true;
    }

    pub fn unset_branch_flag(&self) {
        self.0.lock().unwrap().branch_flag = false;
    }
}
