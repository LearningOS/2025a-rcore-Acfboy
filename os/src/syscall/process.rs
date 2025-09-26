//! Process management syscalls
use crate::{mm::{MapPermission, PTEFlags, VirtAddr}, task::{change_program_brk, exit_current_and_run_next, get_addr_flag_in_cur_space, get_addr_in_cur_space, insert_in_current, is_vpn_in_space, read_syscall_count, suspend_current_and_run_next, unmap_area_in_current}, timer::get_time};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let usec = get_time();
    let sec = usec / 1000;
    let ts_addr_0 = get_addr_in_cur_space(ts as usize);
    let ts_addr_1 = get_addr_in_cur_space((ts as usize) + 1);
    if ts_addr_0.is_none() || ts_addr_1.is_none() {
        -1
    } else {
        let addr_0 = ts_addr_0.unwrap() as *mut usize;
        let addr_1 = ts_addr_1.unwrap() as *mut usize;
        unsafe {
            *addr_0 = sec;
            *addr_1 = sec;
        }
        0
    }
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            let addr_and_flag = get_addr_flag_in_cur_space(id);
            // println!("addr flag {:?}", addr_and_flag);
            if let Some((addr, flag)) = addr_and_flag {
                if (flag & PTEFlags::R) != PTEFlags::empty() && (flag & PTEFlags::U) != PTEFlags::empty()  {
                    println!("flag addr {:?}", flag); //#
                    unsafe { *(addr as *const usize) as isize }
                } else {
                    -1      
                }
            } else {
                -1
            }
        }
        1 => {
            let addr_and_flag = get_addr_flag_in_cur_space(id);
            if let Some((addr, flag)) = addr_and_flag {
                if (flag & PTEFlags::W) != PTEFlags::empty() {
                    unsafe { *(addr as *mut usize) = data }
                    0
                } else {
                    -1
                }
            } else {
                -1
            }
        }
        2 => {
            read_syscall_count(id) as isize
        }
        _ => -1
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!("kernel: sys_mmap");
    let end = VirtAddr::from(start + len);
    // let end_c = VirtAddr::from(start + len - 1);
    let start = VirtAddr::from(start);
    if prot & !0x7 != 0 || prot & 0x7 == 0 || !start.aligned() {
        return -1;
    }
    let start_vpn = start.floor();
    let end_vpn = end.ceil();
    for i in start_vpn.0 .. end_vpn.0 {
        if is_vpn_in_space(i) {
            return -1;
        }
    }
    insert_in_current(start, end, MapPermission::from_prot(prot));
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    let end = VirtAddr::from(start + len);
    let start = VirtAddr::from(start);
    if !start.aligned() {
        println!("mmap err 1; {}", start.0);
        return -1;
    }
    let start_vpn = start.floor();
    let end_vpn = end.ceil();
    for i in start_vpn.0 .. end_vpn.0 {
        if !is_vpn_in_space(i) {
            return -1;
        }
    }
    if unmap_area_in_current(start_vpn, end_vpn) {
        0
    } else {
        -1
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
