use common::raf::Raf;

const JOB_INIT_NAME: &'static str = "INITIALISIERUNG";

pub struct JobInfo {
    id: String,
    sgbd: String,
    name: String,
    fixed_function_struct_id: String,
    args_first: String,
    args: String,
    results: String,
    arg_limit: String,
}

pub struct FiniteStateMachine {
    request_init: bool,
    job_running: bool,
    job_running_old: bool,
    job_std: bool,
    job_std_old: bool,
}

impl FiniteStateMachine {
    pub fn execute_job(&mut self, bytes: &mut Raf, info: JobInfo, is_recersive: bool) {
        if !self.request_init && !is_recersive {
            self.execute_init_job();
        }
        let mut buffer: [u8; 2] = [0, 0];
    }

    pub fn execute_job_by_name(&mut self, name: &str, is_recersive: bool) {}

    pub fn execute_init_job(&mut self) {
        let job_running = self.job_running;
        let job_std_old = self.job_std;
        {
            self.execute_job_by_name(JOB_INIT_NAME, true);
        }
    }
}
