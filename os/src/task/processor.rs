//!Implementation of [`Processor`] and Intersection of control flow
//!
//! Here, the continuous operation of user apps in CPU is maintained,
//! the current running state of CPU is recorded,
//! and the replacement and transfer of control flow of different applications are executed.

use super::__switch;
use super::{fetch_task, TaskStatus};
use super::{TaskContext, TaskControlBlock};
use crate::config::PAGE_SIZE_BITS;
use crate::mm::{ MapPermission, VirtAddr, VirtPageNum};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use alloc::sync::Arc;
use lazy_static::*;

/// Processor management structure
pub struct Processor {
    ///The task currently executing on the current processor
    current: Option<Arc<TaskControlBlock>>,

    ///The basic control flow of each core, helping to select and switch process
    idle_task_cx: TaskContext,
}

impl Processor {
    ///Create an empty Processor
    pub fn new() -> Self {
        Self {
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }

    ///Get mutable reference to `idle_task_cx`
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }

    ///Get current task in moving semanteme
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }

    ///Get current task in cloning semanteme
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
}

lazy_static! {
    pub static ref PROCESSOR: UPSafeCell<Processor> = unsafe { UPSafeCell::new(Processor::new()) };
}

///The main part of process execution and scheduling
///Loop `fetch_task` to get the process that needs to run, and switch the process through `__switch`
pub fn run_tasks() {
    loop {
        let mut processor = PROCESSOR.exclusive_access();
        if let Some(task) = fetch_task() {
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            task.add_stride();
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            // release coming task_inner manually
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processor);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            warn!("no tasks available in run_tasks");
        }
    }
}

/// Get current task through take, leaving a None in its place
pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().take_current()
}

/// Get a copy of the current task
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.exclusive_access().current()
}

/// Get the physical address of an virtual address in current app's space
pub fn get_phy_addr_in_cur_space(addr: VirtAddr) -> Option<usize> {
    let current = PROCESSOR.exclusive_access().current();
    let vpn = addr.floor();
    let offset = addr.page_offset();
    if let Some(app) = current {
        let inner = app.inner_exclusive_access();
        let ppe_opt = inner
            .memory_set
            .translate(vpn);
        if let Some(ppe) = ppe_opt {
            if ppe.is_valid() && ppe.writable() {
                Some((ppe.ppn().0 << PAGE_SIZE_BITS) + offset)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

/// Insert area to current address space
pub fn insert_in_current(start: VirtAddr, end: VirtAddr, permission: MapPermission) {
    let current = current_task().unwrap();
    current.inner_exclusive_access().memory_set.insert_framed_area(start, end, permission);
}

/// Check wether a virtual page number in this space.
pub fn is_vpn_in_space(vpn: usize) -> bool {
    get_phy_addr_in_cur_space(VirtAddr::from(vpn << PAGE_SIZE_BITS)).is_some()
}

/// Unmap a whole area. Assumed that start to end as a whole area.
pub fn unmap_area_in_current(start: VirtPageNum, _end: VirtPageNum) {
    let current = current_task().unwrap();
    current.inner_exclusive_access().memory_set.remove_area_with_start_vpn(start);

}

/// Get the current user token(addr of page table)
pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    task.get_user_token()
}

///Get the mutable reference to trap context of current task
pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

///Return to idle control flow for new scheduling
pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut processor = PROCESSOR.exclusive_access();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}
