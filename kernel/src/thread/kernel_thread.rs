// SPDX-License-Identifier: MPL-2.0

use ostd::{
    cpu::CpuSet,
    task::{Task, TaskOptions},
};

use super::{oops, AsThread, Thread};
use crate::{
    prelude::*,
    sched::{Nice, SchedPolicy},
};

/// The inner data of a kernel thread.
struct KernelThread;

/// Options to create or spawn a new kernel thread.
pub struct ThreadOptions {
    func: Option<Box<dyn FnOnce() + Send>>,
    cpu_affinity: CpuSet,
    sched_policy: SchedPolicy,
}

impl ThreadOptions {
    /// Creates the thread options with the thread function.
    pub fn new<F>(func: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        let cpu_affinity = CpuSet::new_full();
        let sched_policy = SchedPolicy::Fair(Nice::default());
        Self {
            func: Some(Box::new(func)),
            cpu_affinity,
            sched_policy,
        }
    }

    /// Sets the CPU affinity of the new thread.
    pub fn cpu_affinity(mut self, cpu_affinity: CpuSet) -> Self {
        self.cpu_affinity = cpu_affinity;
        self
    }

    /// Sets the scheduling policy.
    pub fn sched_policy(mut self, sched_policy: SchedPolicy) -> Self {
        self.sched_policy = sched_policy;
        self
    }
}

impl ThreadOptions {
    /// Builds a new kernel thread without running it immediately.
    pub fn build(mut self) -> Arc<Task> {
        let task_fn = self.func.take().unwrap();
        let thread_fn = move || {
            let _ = oops::catch_panics_as_oops(task_fn);
            // Ensure that the thread exits.
            current_thread!().exit();
        };

        Arc::new_cyclic(|weak_task| {
            let thread = {
                let kernel_thread = KernelThread;
                let cpu_affinity = self.cpu_affinity;
                let sched_policy = self.sched_policy;
                Arc::new(Thread::new(
                    weak_task.clone(),
                    kernel_thread,
                    cpu_affinity,
                    sched_policy,
                ))
            };

            TaskOptions::new(thread_fn).data(thread).build().unwrap()
        })
    }

    /// Builds a new kernel thread and runs it immediately.
    #[track_caller]
    pub fn spawn(self) -> Arc<Thread> {
        let task = self.build();
        let thread = task.as_thread().unwrap().clone();
        thread.run();
        thread
    }
}
