use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub name: String,
    pub description: String,
    pub category: String,
    pub resource: Option<String>,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub default_scope: String,
    pub level: u8,
    pub is_system: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub scope_type: String,
    pub scope_value: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessCheckRequest {
    pub role: String,
    pub permission: String,
    pub scope_values: Vec<String>,
    pub required_scope: String,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessCheckResponse {
    pub allowed: bool,
    pub reason: Option<String>,
}

pub struct RbacService {
    roles: Arc<RwLock<HashMap<String, Role>>>,
    permissions: Arc<RwLock<HashMap<String, Permission>>>,
    user_roles: Arc<RwLock<HashMap<String, Vec<(String, Option<String>)>>>>,
    user_scopes: Arc<RwLock<HashMap<String, Vec<Scope>>>>,
}

impl RbacService {
    pub fn new() -> Self {
        let roles = Self::default_roles();
        let permissions = Self::default_permissions();

        Self {
            roles: Arc::new(RwLock::new(roles)),
            permissions: Arc::new(RwLock::new(permissions)),
            user_roles: Arc::new(RwLock::new(HashMap::new())),
            user_scopes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn default_roles() -> HashMap<String, Role> {
        let mut roles = HashMap::new();
        
        roles.insert("admin".to_string(), Role {
            name: "admin".to_string(),
            description: "Full system administrator".to_string(),
            permissions: vec!["*".to_string()],
            default_scope: "system".to_string(),
            level: 100,
            is_system: true,
        });

        roles.insert("yayasan".to_string(), Role {
            name: "yayasan".to_string(),
            description: "Foundation management".to_string(),
            permissions: vec![
                "user:read".to_string(),
                "user:write".to_string(),
                "payment:read".to_string(),
                "payment:write".to_string(),
                "report:generate".to_string(),
                "student:read".to_string(),
                "audit:read".to_string(),
            ],
            default_scope: "foundation".to_string(),
            level: 90,
            is_system: false,
        });

        roles.insert("finance".to_string(), Role {
            name: "finance".to_string(),
            description: "Finance department".to_string(),
            permissions: vec![
                "payment:read".to_string(),
                "payment:write".to_string(),
                "report:generate".to_string(),
            ],
            default_scope: "school_wide".to_string(),
            level: 70,
            is_system: false,
        });

        roles.insert("hr".to_string(), Role {
            name: "hr".to_string(),
            description: "Human resources".to_string(),
            permissions: vec![
                "user:read".to_string(),
                "user:write".to_string(),
                "staff:manage".to_string(),
                "report:generate".to_string(),
            ],
            default_scope: "school_wide".to_string(),
            level: 70,
            is_system: false,
        });

        roles.insert("teacher".to_string(), Role {
            name: "teacher".to_string(),
            description: "Teacher".to_string(),
            permissions: vec![
                "student:read".to_string(),
                "student:manage".to_string(),
                "attendance:manage".to_string(),
                "grade:manage".to_string(),
                "assignment:manage".to_string(),
            ],
            default_scope: "assigned_class".to_string(),
            level: 50,
            is_system: false,
        });

        roles.insert("student".to_string(), Role {
            name: "student".to_string(),
            description: "Student".to_string(),
            permissions: vec![
                "self:read".to_string(),
                "self:update".to_string(),
                "grade:read".to_string(),
                "attendance:read".to_string(),
                "assignment:read".to_string(),
            ],
            default_scope: "own".to_string(),
            level: 10,
            is_system: false,
        });

        roles.insert("parent".to_string(), Role {
            name: "parent".to_string(),
            description: "Parent/Guardian".to_string(),
            permissions: vec![
                "child:read".to_string(),
                "child:update".to_string(),
                "grade:read".to_string(),
                "attendance:read".to_string(),
                "payment:read".to_string(),
            ],
            default_scope: "own_children".to_string(),
            level: 20,
            is_system: false,
        });

        roles.insert("donor".to_string(), Role {
            name: "donor".to_string(),
            description: "Donor".to_string(),
            permissions: vec![
                "donation:read".to_string(),
                "donation:write".to_string(),
            ],
            default_scope: "own".to_string(),
            level: 5,
            is_system: false,
        });

        roles.insert("public".to_string(), Role {
            name: "public".to_string(),
            description: "Public access".to_string(),
            permissions: vec![],
            default_scope: "none".to_string(),
            level: 0,
            is_system: false,
        });

        roles
    }

    fn default_permissions() -> HashMap<String, Permission> {
        let mut perms = HashMap::new();
        
        let categories = vec![
            ("auth", vec!["auth:manage"]),
            ("user", vec!["user:read", "user:write", "user:delete", "user:impersonate"]),
            ("role", vec!["role:manage", "role:assign"]),
            ("payment", vec!["payment:read", "payment:write", "payment:approve"]),
            ("student", vec!["student:read", "student:manage"]),
            ("staff", vec!["staff:read", "staff:manage"]),
            ("attendance", vec!["attendance:read", "attendance:manage"]),
            ("grade", vec!["grade:read", "grade:manage"]),
            ("assignment", vec!["assignment:read", "assignment:manage"]),
            ("report", vec!["report:read", "report:generate"]),
            ("audit", vec!["audit:read", "audit:export"]),
            ("file", vec!["file:read", "file:upload", "file:delete"]),
            ("ai", vec!["ai:query"]),
            ("donation", vec!["donation:read", "donation:write"]),
        ];

        for (category, actions) in categories {
            for action in actions {
                let name = format!("{}:{}", category, action.split(':').nth(1).unwrap_or("read"));
                perms.insert(name.clone(), Permission {
                    name: name.clone(),
                    description: format!("{} {}", category, action.split(':').nth(1).unwrap_or("manage")),
                    category: category.to_string(),
                    resource: None,
                    action: action.to_string(),
                });
            }
        }

        perms
    }

    pub async fn has_permission(&self, role: &str, permission: &str) -> bool {
        let roles = self.roles.read().await;
        if let Some(role_data) = roles.get(role) {
            role_data.permissions.iter().any(|p| p == "*" || p == permission)
        } else {
            false
        }
    }

    pub async fn get_default_scope(&self, role: &str) -> String {
        let roles = self.roles.read().await;
        roles.get(role)
            .map(|r| r.default_scope.clone())
            .unwrap_or_else(|| "none".to_string())
    }

    pub async fn check_access(&self, request: AccessCheckRequest) -> Result<AccessCheckResponse, AppError> {
        let roles = self.roles.read().await;
        
        let role_data = roles.get(&request.role)
            .ok_or_else(|| AppError::Forbidden("Role not found".to_string()))?;

        if !role_data.permissions.iter().any(|p| p == "*" || p == &request.permission) {
            return Ok(AccessCheckResponse {
                allowed: false,
                reason: Some("Insufficient permissions".to_string()),
            });
        }

        if request.required_scope == "system" || request.required_scope.is_empty() {
            return Ok(AccessCheckResponse {
                allowed: true,
                reason: None,
            });
        }

        if request.scope_values.iter().any(|s| s == &request.required_scope || s == "*") {
            return Ok(AccessCheckResponse {
                allowed: true,
                reason: None,
            });
        }

        Ok(AccessCheckResponse {
            allowed: false,
            reason: Some(format!("Scope '{}' required but not granted", request.required_scope)),
        })
    }

    pub async fn get_role_permissions(&self, role: &str) -> Result<Vec<String>, AppError> {
        let roles = self.roles.read().await;
        let role_data = roles.get(role)
            .ok_or_else(|| AppError::NotFound("Role not found".to_string()))?;
        Ok(role_data.permissions.clone())
    }

    pub async fn get_all_roles(&self) -> Vec<Role> {
        let roles = self.roles.read().await;
        roles.values().cloned().collect()
    }

    pub async fn get_all_permissions(&self) -> Vec<Permission> {
        let perms = self.permissions.read().await;
        perms.values().cloned().collect()
    }
}

impl Default for RbacService {
    fn default() -> Self {
        Self::new()
    }
}