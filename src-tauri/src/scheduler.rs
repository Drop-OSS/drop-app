use std::time::Duration;

use async_trait::async_trait;
use log::{info, warn};
use tokio::time;

#[async_trait]
trait ScheduleTask {
    /// Returns how many minutes between calls
    fn timeframe(&mut self) -> usize;
    async fn call(&mut self) -> Result<(), anyhow::Error>;
}

struct Test;

#[async_trait]
impl ScheduleTask for Test {
    fn timeframe(&mut self) -> usize {
        1
    }

    async fn call(&mut self) -> Result<(), anyhow::Error> {
        info!("ran background task");
        Ok(())
    }
}

struct TaskData {
    task: Box<dyn ScheduleTask + Send + Sync>,
    updates_since_call: usize,
}

pub async fn scheduler_task() -> ! {
    let mut interval = time::interval(Duration::from_mins(1));
    interval.tick().await;

    let mut tasks = vec![TaskData {
        task: Box::new(Test {}),
        updates_since_call: 0,
    }];

    loop {
        for task in &mut tasks {
            task.updates_since_call += 1;
            if task.task.timeframe() <= task.updates_since_call {
                let result = task.task.call().await;
                if let Err(err) = result {
                    warn!("background task returned error: {err:?}");
                }
                task.updates_since_call = 0;
            }
        }
        interval.tick().await;
    }
}
