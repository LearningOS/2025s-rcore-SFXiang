//! Process management syscalls
use crate::{
    task::{exit_current_and_run_next, suspend_current_and_run_next, get_current_task_control_block},
    timer::get_time_us,
};

use core::ptr;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

///  TODO: implement the syscall
///  □ 如果 trace_request 为 0，则 id 应被视作 *const u8 ，表示读取当前任务 id 地址处一个字节的无符号整数值。此时应忽略 data 参数。返回值为 id 地址处的值。
///  □ 如果 trace_request 为 1，则 id 应被视作 *mut u8 ，表示写入 data （作为 u8，即只考虑最低位的一个字节）到该用户程序 id 地址处。返回值应为0。
///  □ 如果 trace_request 为 2，表示查询当前任务调用编号为 id 的系统调用的次数，返回值为这个调用次数。本次调用也计入统计 。

///  从用户空间地址读取单个字节
fn from_byte_ptr(ptr: *const u8) -> Result<u8, ()> {
    // 前置检查：空指针校验
    if ptr.is_null() {
        return Err(());
    }

    // 安全边界：通过volatile读取避免编译器优化
    unsafe {
        Ok(ptr::read_volatile(ptr))
    }
}

///  向用户空间地址写入单个字节
fn to_byte_ptr(ptr: *mut u8, value: u8) -> Result<(), ()> {
    // 前置检查：空指针和可写性校验
    if ptr.is_null() {
        return Err(());
    }

    // 安全边界：通过volatile写入确保操作原子性
    unsafe {
        ptr::write_volatile(ptr, value);
        Ok(())
    }
}
/// sys_trace
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request {
        // 读取用户空间单字节
        0 => {
            let ptr = _id as *const u8;
            if let Ok(value) = from_byte_ptr(ptr) {
                value as isize
            } else {
                -1
            }
        }

        // 写入用户空间单字节
        1 => {
            let ptr = _id as *mut u8;
            let value = _data as u8;
            if to_byte_ptr(ptr, value).is_ok() {
                0
            } else {
                -1
            }
        }

        // 统计系统调用次数
        2 => {
            if let Some(task) = get_current_task_control_block() {
                let mut task_inner = task; // 获取任务内部数据的可变独占访问
                let syscall_id = _id;
                
                // 检查系统调用号有效性
                if syscall_id >= task_inner.syscall_count.len() {
                    return -1;
                }
                
                // 递增并返回统计值（包含本次调用）
                task_inner.syscall_count[syscall_id] += 1;
                task_inner.syscall_count[syscall_id] as isize
            } else {
                -1
            }
        }

        // 无效请求类型
        _ => -1,
    }

}
