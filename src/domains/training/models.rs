//! Training Models — نماذج التدريب

use f64;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingProgram {
    pub id: Option<Thing>,
    pub title: String,
    pub program_type: Option<String>, // internal, external, online, on_job, conference
    pub category: Option<String>,     // technical, safety, management, soft_skills, regulatory
    pub provider: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub total_hours: Option<f64>,
    pub max_participants: Option<i64>,
    pub cost: Option<f64>,
    pub currency: Option<String>,
    pub status: Option<String>, // planned, active, completed, cancelled
    pub certificate_provided: Option<bool>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTrainingProgramRequest {
    pub title: String,
    pub program_type: Option<String>,
    pub category: Option<String>,
    pub provider: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub total_hours: Option<f64>,
    pub max_participants: Option<i64>,
    pub cost: Option<f64>,
    pub certificate_provided: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingEnrollment {
    pub id: Option<Thing>,
    pub employee: Thing,
    pub program: Thing,
    pub enrolled_at: Option<String>,
    pub status: Option<String>, // enrolled, in_progress, completed, dropped
    pub score: Option<f64>,
    pub certificate_number: Option<String>,
    pub completed_at: Option<String>,
    pub notes: Option<String>,
    pub is_archived: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollRequest {
    pub employee_id: String,
    pub program_id: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteEnrollmentRequest {
    pub score: Option<f64>,
    pub certificate_number: Option<String>,
    pub notes: Option<String>,
}
