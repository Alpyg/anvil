use crate::services::{BoxError, Services};
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct CronJob {
    pub build: fn(&Services) -> Result<Job, BoxError>,
}
inventory::collect!(CronJob);

pub fn parse_timezone(name: &str) -> Result<chrono_tz::Tz, BoxError> {
    name.parse::<chrono_tz::Tz>().or_else(|_| {
        if name.eq_ignore_ascii_case("utc") {
            Ok(chrono_tz::UTC)
        } else {
            Err(format!("invalid timezone `{name}`").into())
        }
    })
}

pub async fn start_cron(services: &Services) -> Result<JobScheduler, BoxError> {
    let scheduler = JobScheduler::new().await?;
    for job in inventory::iter::<CronJob> {
        scheduler.add((job.build)(services)?).await?;
    }
    scheduler.start().await?;
    Ok(scheduler)
}
