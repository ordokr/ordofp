#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use ordofp_core::pfds::{Queue, Stack};
use std::collections::VecDeque;

#[derive(Arbitrary, Debug)]
enum StackOp {
    Push(u8),
    Pop,
    Peek,
}

#[derive(Arbitrary, Debug)]
enum QueueOp {
    Enqueue(u8),
    Dequeue,
    Peek,
}

#[derive(Arbitrary, Debug)]
enum PfdsOp {
    StackOps(Vec<StackOp>),
    QueueOps(Vec<QueueOp>),
}

fuzz_target!(|op: PfdsOp| {
    match op {
        PfdsOp::StackOps(ops) => {
            let mut stack = Stack::new();
            let mut vec = Vec::new();

            for op in ops {
                match op {
                    StackOp::Push(v) => {
                        stack = stack.push(v);
                        vec.push(v);
                    }
                    StackOp::Pop => {
                        // pop() consumes; clone (Rc bump) so `stack` stays usable
                        // when the pop returns None or the differential check panics.
                        let s_res = stack.clone().pop();
                        let v_res = vec.pop();
                        match (s_res, v_res) {
                            (Some((s_val, new_stack)), Some(v_val)) => {
                                assert_eq!(s_val, v_val);
                                stack = new_stack;
                            }
                            (None, None) => {}
                            _ => panic!(
                                "Stack mismatch: stack {:?}, vec {:?}",
                                stack.peek(),
                                vec.last()
                            ),
                        }
                    }
                    StackOp::Peek => {
                        assert_eq!(stack.peek(), vec.last());
                    }
                }
            }
        }
        PfdsOp::QueueOps(ops) => {
            let mut queue = Queue::new();
            let mut deque = VecDeque::new();

            for op in ops {
                match op {
                    QueueOp::Enqueue(v) => {
                        queue = queue.enqueue(v);
                        deque.push_back(v);
                    }
                    QueueOp::Dequeue => {
                        // dequeue() consumes; clone so `queue` stays usable on the
                        // None / mismatch paths.
                        let q_res = queue.clone().dequeue();
                        let d_res = deque.pop_front();
                        match (q_res, d_res) {
                            (Some((q_val, new_queue)), Some(d_val)) => {
                                assert_eq!(q_val, d_val);
                                queue = new_queue;
                            }
                            (None, None) => {}
                            _ => panic!(
                                "Queue mismatch: queue {:?}, deque {:?}",
                                queue.peek(),
                                deque.front()
                            ),
                        }
                    }
                    QueueOp::Peek => {
                        assert_eq!(queue.peek(), deque.front());
                    }
                }
            }
        }
    }
});
