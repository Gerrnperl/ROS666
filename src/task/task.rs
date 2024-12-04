use super::context::TaskCtx;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Create,
    Ready,
    Running,
    Wait,
    Stopped,
}

pub type TaskId = usize;

#[derive(Copy, Clone, Debug)]
pub struct TaskControlBlock {
    pub id: TaskId,
    pub status: TaskStatus,
    pub ctx: TaskCtx,
}
