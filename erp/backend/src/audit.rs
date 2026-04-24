use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, TimeZone};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub user_id: Option<String>,
    pub user_email: Option<String>,
    pub action: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub trace_id: Uuid,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
    pub severity: AuditSeverity,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQueryRequest {
    pub user_id: Option<String>,
    pub action: Option<String>,
    pub entity_type: Option<String>,
    pub start_date: Option<chrono::DateTime<Utc>>,
    pub end_date: Option<chrono::DateTime<Utc>>,
    pub severity: Option<AuditSeverity>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_actions: u64,
    pub actions_by_type: Vec<ActionCount>,
    pub actions_by_user: Vec<UserActionCount>,
    pub actions_by_severity: Vec<SeverityCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCount {
    pub action: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActionCount {
    pub user_id: String,
    pub user_email: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityCount {
    pub severity: String,
    pub count: u64,
}

pub struct AuditService {
    logs: Arc<RwLock<Vec<AuditEntry>>>,
    max_logs: usize,
}

impl AuditService {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            max_logs: 100000,
        }
    }

    pub async fn log(&self, entry: AuditEntry) {
        let mut logs = self.logs.write().await;
        
        if logs.len() >= self.max_logs {
            logs.drain(0..1000);
        }
        
        logs.push(entry);
    }

    pub async fn log_action(
        &self,
        user_id: Option<String>,
        user_email: Option<String>,
        action: String,
        entity_type: Option<String>,
        entity_id: Option<String>,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        session_id: Option<String>,
        severity: AuditSeverity,
    ) -> Uuid {
        let trace_id = Uuid::new_v4();
        
        let entry = AuditEntry {
            id: Uuid::new_v4(),
            user_id,
            user_email,
            action,
            entity_type,
            entity_id,
            old_value,
            new_value,
            ip_address,
            user_agent,
            trace_id,
            session_id,
            request_id: None,
            severity,
            created_at: Utc::now(),
        };
        
        self.log(entry).await;
        trace_id
    }

    pub fn log_action_sync(
        &self,
        user_id: Option<String>,
        user_email: Option<String>,
        action: String,
        entity_type: Option<String>,
        entity_id: Option<String>,
        old_value: Option<serde_json::Value>,
        new_value: Option<serde_json::Value>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        session_id: Option<String>,
        severity: AuditSeverity,
    ) -> Uuid {
        let trace_id = Uuid::new_v4();
        
        let entry = AuditEntry {
            id: Uuid::new_v4(),
            user_id,
            user_email,
            action,
            entity_type,
            entity_id,
            old_value,
            new_value,
            ip_address,
            user_agent,
            trace_id,
            session_id,
            request_id: None,
            severity,
            created_at: Utc::now(),
        };
        
        let mut logs = self.logs.blocking_write();
        logs.push(entry);
        trace_id
    }

    pub async fn get_logs(&self) -> Vec<AuditEntry> {
        let logs = self.logs.read().await;
        logs.clone()
    }

    pub async fn query(&self, request: AuditQueryRequest) -> Result<Vec<AuditEntry>, AppError> {
        let logs = self.logs.read().await;
        
        let mut filtered: Vec<&AuditEntry> = logs.iter().collect();
        
        if let Some(user_id) = &request.user_id {
            filtered.retain(|l| l.user_id.as_ref() == Some(user_id));
        }
        
        if let Some(action) = &request.action {
            filtered.retain(|l| l.action == *action);
        }
        
        if let Some(entity_type) = &request.entity_type {
            filtered.retain(|l| l.entity_type.as_ref() == Some(entity_type));
        }
        
        if let Some(start) = request.start_date {
            filtered.retain(|l| l.created_at >= start);
        }
        
        if let Some(end) = request.end_date {
            filtered.retain(|l| l.created_at <= end);
        }
        
        if let Some(severity) = &request.severity {
            filtered.retain(|l| l.severity == *severity);
        }
        
        let limit = request.limit.unwrap_or(100);
        let offset = request.offset.unwrap_or(0);
        
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        let result: Vec<AuditEntry> = filtered
            .into_iter()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();
        
        Ok(result)
    }

    pub async fn get_stats(&self) -> AuditStats {
        let logs = self.logs.read().await;
        
        let mut actions_count: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        let mut user_count: std::collections::HashMap<String, (String, u64)> = std::collections::HashMap::new();
        let mut severity_count: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        
        for log in logs.iter() {
            *actions_count.entry(log.action.clone()).or_insert(0) += 1;
            
            if let (Some(uid), Some(email)) = (&log.user_id, &log.user_email) {
                let entry = user_count.entry(uid.clone()).or_insert((email.clone(), 0));
                entry.1 += 1;
            }
            
            let severity = format!("{:?}", log.severity);
            *severity_count.entry(severity).or_insert(0) += 1;
        }
        
        let actions_by_type: Vec<ActionCount> = actions_count
            .into_iter()
            .map(|(action, count)| ActionCount { action, count })
            .collect();
        
        let actions_by_user: Vec<UserActionCount> = user_count
            .into_iter()
            .map(|(user_id, (user_email, count))| UserActionCount { user_id, user_email, count })
            .collect();
        
        let actions_by_severity: Vec<SeverityCount> = severity_count
            .into_iter()
            .map(|(severity, count)| SeverityCount { severity, count })
            .collect();
        
        AuditStats {
            total_actions: logs.len() as u64,
            actions_by_type,
            actions_by_user,
            actions_by_severity,
        }
    }

    pub async fn get_logs_by_trace(&self, trace_id: &Uuid) -> Option<Vec<AuditEntry>> {
        let logs = self.logs.read().await;
        let found: Vec<AuditEntry> = logs
            .iter()
            .filter(|l| l.trace_id == *trace_id)
            .cloned()
            .collect();
        
        if found.is_empty() {
            None
        } else {
            Some(found)
        }
    }

    pub async fn clear_old_logs(&self, days: i64) -> usize {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        let mut logs = self.logs.write().await;
        
        let before = logs.len();
        logs.retain(|l| l.created_at >= cutoff);
        
        before - logs.len()
    }
}

impl Default for AuditService {
    fn default() -> Self {
        Self::new()
    }
}