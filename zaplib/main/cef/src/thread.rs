use crate::ptr::{wrap_ptr, BaseRefCountedExt, WrapperFor};
use crate::ThreadId;
use zaplib_cef_sys::{cef_currently_on, cef_post_task, cef_task_t};

pub fn currently_on(id: ThreadId) -> bool {
    unsafe { cef_currently_on(id) > 0 }
}

pub fn post_task<F: FnOnce()>(id: ThreadId, func: F) -> Result<(), bool> {
    if currently_on(id) {
        // Execute it now
        func();
        return Ok(());
    }

    let task = wrap_ptr(move |base| TaskWrapper {
        _base: cef_task_t { base, execute: Some(TaskWrapper::<F>::execute) },
        func: Some(func),
    });

    let ok = unsafe { cef_post_task(id, task) };
    if ok > 0 {
        Ok(())
    } else {
        Err(false)
    }
}

pub struct TaskWrapper<F: FnOnce()> {
    _base: cef_task_t,
    func: Option<F>,
}
unsafe impl<F: FnOnce()> WrapperFor<cef_task_t> for TaskWrapper<F> {}
impl<F: FnOnce()> TaskWrapper<F> {
    fn from_ptr<'a>(ptr: *mut cef_task_t) -> &'a mut BaseRefCountedExt<cef_task_t, TaskWrapper<F>> {
        unsafe { &mut *(ptr as *mut _) }
    }

    extern "C" fn execute(task: *mut cef_task_t) {
        let task = Self::from_ptr(task);

        if let Some(func) = task.func.take() {
            (func)();
        }
    }
}
