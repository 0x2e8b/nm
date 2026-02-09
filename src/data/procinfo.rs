use super::model::Process;

/// Enrich processes with full executable paths using libproc.
pub fn enrich_process_paths(processes: &mut [Process]) {
    for proc in processes.iter_mut() {
        if proc.pid == 0 {
            continue;
        }
        match libproc::libproc::proc_pid::pidpath(proc.pid as i32) {
            Ok(path) => proc.path = Some(path),
            Err(_) => {}
        }
    }
}
