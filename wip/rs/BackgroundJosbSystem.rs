// crates/infrastructure/queue/src/lib.rs
//! Background Jobs System with integrated handlers

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

#[derive(Error, Debug)]
pub enum JobError {
    #[error("Job execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Job not found: {0}")]
    NotFound(String),

    #[error("Queue is full")]
    QueueFull,
}

// ============================================================================
// JOB TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobType {
    // Email jobs
    SendWelcomeEmail {
        domain: String,
        email: String,
        password: String,
    },
    SendPaymentSuccessEmail {
        domain: String,
        invoice_id: String,
    },
    SendPaymentFailedEmail {
        domain: String,
        amount: String,
    },
    SendSubscriptionExpiringEmail {
        domain: String,
        days_until: u8,
    },

    // Stripe webhook processing
    ProcessStripeWebhook {
        event_id: String,
        payload: String,
    },

    // White label operations
    ProvisionWhiteLabel {
        domain: String,
        white_label_id: String,
    },
    DeleteWhiteLabel {
        domain: String,
        white_label_id: String,
    },

    // Scheduled maintenance
    CleanupExpiredSessions,
    BackupDatabases,
    UpdateStorageMetrics,
    CheckSubscriptionRenewals,
    CheckOverduePayments,
}

// ============================================================================
// JOB STRUCT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub job_type: JobType,
    pub retry_count: u8,
    pub max_retries: u8,
    pub created_at: i64,
}

impl Job {
    pub fn new(job_type: JobType) -> Self {
        use time::OffsetDateTime;

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            job_type,
            retry_count: 0,
            max_retries: 3,
            created_at: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }

    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }
}

// ============================================================================
// JOB QUEUE
// ============================================================================

pub struct JobQueue {
    sender: mpsc::UnboundedSender<Job>,
    failed_jobs: Arc<RwLock<Vec<Job>>>,
}

impl JobQueue {
    pub fn new(worker_count: usize) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<Job>();
        let failed_jobs = Arc::new(RwLock::new(Vec::new()));

        // Spawn workers
        for worker_id in 0..worker_count {
            let mut worker_receiver = receiver;
            let failed = Arc::clone(&failed_jobs);
            let worker_sender = sender.clone();

            tokio::spawn(async move {
                Self::worker(worker_id, &mut worker_receiver, failed, worker_sender).await;
            });

            // Create new receiver for next worker
            receiver = sender.subscribe();
        }

        Self {
            sender,
            failed_jobs,
        }
    }

    pub fn enqueue(&self, job: Job) -> Result<(), JobError> {
        self.sender
            .send(job)
            .map_err(|_| JobError::QueueFull)
    }

    async fn worker(
        worker_id: usize,
        receiver: &mut mpsc::UnboundedReceiver<Job>,
        failed_jobs: Arc<RwLock<Vec<Job>>>,
        sender: mpsc::UnboundedSender<Job>,
    ) {
        info!("Worker {} started", worker_id);

        while let Some(mut job) = receiver.recv().await {
            info!("Worker {} processing job {}: {:?}", worker_id, job.id, job.job_type);

            match Self::execute_job(&job).await {
                Ok(_) => {
                    info!("Worker {} completed job {}", worker_id, job.id);
                }
                Err(e) => {
                    error!("Worker {} job {} failed: {}", worker_id, job.id, e);

                    if job.can_retry() {
                        warn!("Retrying job {} (attempt {}/{})", job.id, job.retry_count + 1, job.max_retries);
                        job.retry_count += 1;

                        // Exponential backoff
                        let delay = 2_u64.pow(job.retry_count as u32);
                        sleep(Duration::from_secs(delay)).await;

                        // Re-enqueue for retry
                        if let Err(e) = sender.send(job.clone()) {
                            error!("Failed to re-enqueue job {}: {}", job.id, e);
                        }
                    } else {
                        error!("Job {} exhausted retries, moving to failed queue", job.id);
                        failed_jobs.write().await.push(job);
                    }
                }
            }
        }

        warn!("Worker {} stopped", worker_id);
    }

    async fn execute_job(job: &Job) -> Result<(), JobError> {
        match &job.job_type {
            JobType::SendWelcomeEmail { domain, email, password } => {
                info!("Sending welcome email to {} for domain {}", email, domain);
                // TODO: Integrate with actual email service
                sleep(Duration::from_millis(100)).await;
                Ok(())
            }

            JobType::SendPaymentSuccessEmail { domain, invoice_id } => {
                info!("Sending payment success email for {} (invoice: {})", domain, invoice_id);
                // TODO: Integrate with actual email service
                sleep(Duration::from_millis(100)).await;
                Ok(())
            }

            JobType::SendPaymentFailedEmail { domain, amount } => {
                info!("Sending payment failed email for {} (amount: {})", domain, amount);
                // TODO: Integrate with actual email service
                sleep(Duration::from_millis(100)).await;
                Ok(())
            }

            JobType::SendSubscriptionExpiringEmail { domain, days_until } => {
                info!("Sending subscription expiring email for {} ({} days)", domain, days_until);
                // TODO: Integrate with actual email service
                sleep(Duration::from_millis(100)).await;
                Ok(())
            }

            JobType::ProcessStripeWebhook { event_id, payload } => {
                info!("Processing Stripe webhook event: {}", event_id);
                // TODO: Integrate with actual Stripe handler
                sleep(Duration::from_millis(200)).await;
                Ok(())
            }

            JobType::ProvisionWhiteLabel { domain, white_label_id } => {
                info!("Provisioning white label: {} (ID: {})", domain, white_label_id);
                // TODO: Integrate with actual provisioner
                sleep(Duration::from_secs(1)).await;
                Ok(())
            }

            JobType::DeleteWhiteLabel { domain, white_label_id } => {
                info!("Deleting white label: {} (ID: {})", domain, white_label_id);
                // TODO: Integrate with actual deprovisioner
                sleep(Duration::from_millis(500)).await;
                Ok(())
            }

            JobType::CleanupExpiredSessions => {
                info!("Cleaning up expired sessions");
                // TODO: Query database and delete expired sessions
                sleep(Duration::from_millis(300)).await;
                Ok(())
            }

            JobType::BackupDatabases => {
                info!("Backing up databases");
                // TODO: Execute backup script
                sleep(Duration::from_secs(2)).await;
                Ok(())
            }

            JobType::UpdateStorageMetrics => {
                info!("Updating storage metrics for all white labels");
                // TODO: Calculate and update storage usage
                sleep(Duration::from_millis(500)).await;
                Ok(())
            }

            JobType::CheckSubscriptionRenewals => {
                info!("Checking upcoming subscription renewals");
                // TODO: Query database for renewals in next 3 days, send emails
                sleep(Duration::from_millis(400)).await;
                Ok(())
            }

            JobType::CheckOverduePayments => {
                info!("Checking for overdue payments");
                // TODO: Find gray labels >7 days overdue, move to black
                sleep(Duration::from_millis(400)).await;
                Ok(())
            }
        }
    }

    pub async fn get_failed_jobs(&self) -> Vec<Job> {
        self.failed_jobs.read().await.clone()
    }

    pub async fn retry_failed_job(&self, job_id: &str) -> Result<(), JobError> {
        let mut failed = self.failed_jobs.write().await;

        if let Some(pos) = failed.iter().position(|j| j.id == job_id) {
            let mut job = failed.remove(pos);
            job.retry_count = 0;
            self.enqueue(job)?;
            Ok(())
        } else {
            Err(JobError::NotFound(job_id.to_string()))
        }
    }
}

// ============================================================================
// JOB SCHEDULER
// ============================================================================

pub struct JobScheduler {
    queue: Arc<JobQueue>,
}

impl JobScheduler {
    pub fn new(queue: Arc<JobQueue>) -> Self {
        Self { queue }
    }

    pub async fn start(&self) {
        info!("Starting job scheduler");

        // Cleanup expired sessions every 1 hour
        self.schedule_recurring(
            Duration::from_secs(3600),
            JobType::CleanupExpiredSessions,
            "cleanup_sessions",
        );

        // Backup databases every 6 hours
        self.schedule_recurring(
            Duration::from_secs(21600),
            JobType::BackupDatabases,
            "backup_databases",
        );

        // Update storage metrics every 1 hour
        self.schedule_recurring(
            Duration::from_secs(3600),
            JobType::UpdateStorageMetrics,
            "update_storage",
        );

        // Check subscription renewals every 12 hours
        self.schedule_recurring(
            Duration::from_secs(43200),
            JobType::CheckSubscriptionRenewals,
            "check_renewals",
        );

        // Check overdue payments every 6 hours
        self.schedule_recurring(
            Duration::from_secs(21600),
            JobType::CheckOverduePayments,
            "check_overdue",
        );

        info!("Job scheduler started with 5 recurring tasks");
    }

    fn schedule_recurring(&self, interval: Duration, job_type: JobType, name: &str) {
        let queue = Arc::clone(&self.queue);
        let job_type = job_type.clone();
        let name = name.to_string();

        tokio::spawn(async move {
            loop {
                sleep(interval).await;

                let job = Job::new(job_type.clone());
                if let Err(e) = queue.enqueue(job) {
                    error!("Failed to enqueue {} job: {}", name, e);
                } else {
                    info!("Scheduled {} job", name);
                }
            }
        });
    }
}

// ============================================================================
// JOB EXECUTOR WITH DEPENDENCIES
// ============================================================================

use infrastructure_db::Database;
use infrastructure_email::EmailService;

pub struct JobExecutor {
    db: Database,
    email_service: EmailService,
    // Add provisioner when ready
}

impl JobExecutor {
    pub fn new(db: Database, email_service: EmailService) -> Self {
        Self { db, email_service }
    }

    pub async fn execute(&self, job: &Job) -> Result<(), JobError> {
        match &job.job_type {
            JobType::SendWelcomeEmail { domain, email, password } => {
                infrastructure_email::send_welcome_email_job(
                    &self.email_service,
                    &self.db,
                    domain,
                    email,
                    password,
                )
                    .await
                    .map_err(|e| JobError::ExecutionFailed(e.to_string()))
            }

            JobType::SendPaymentSuccessEmail { domain, invoice_id } => {
                infrastructure_email::send_payment_success_email_job(
                    &self.email_service,
                    &self.db,
                    domain,
                    invoice_id,
                )
                    .await
                    .map_err(|e| JobError::ExecutionFailed(e.to_string()))
            }

            JobType::SendPaymentFailedEmail { domain, amount } => {
                infrastructure_email::send_payment_failed_email_job(
                    &self.email_service,
                    &self.db,
                    domain,
                    amount,
                )
                    .await
                    .map_err(|e| JobError::ExecutionFailed(e.to_string()))
            }

            _ => {
                // Other job types handled by default executor
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_job_queue() {
        let queue = Arc::new(JobQueue::new(2));

        let job = Job::new(JobType::SendWelcomeEmail {
            domain: "example.com".to_string(),
            email: "user@example.com".to_string(),
            password: "temp123".to_string(),
        });

        assert!(queue.enqueue(job).is_ok());

        sleep(Duration::from_secs(1)).await;
    }

    #[tokio::test]
    async fn test_scheduler() {
        let queue = Arc::new(JobQueue::new(2));
        let scheduler = JobScheduler::new(Arc::clone(&queue));

        scheduler.start().await;

        sleep(Duration::from_secs(2)).await;
    }
}