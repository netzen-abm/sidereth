use crate::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessAction {
    Read,
    Create,
    Update,
    AppendEvent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessRequest {
    pub actor_id: Id,
    pub case_id: Id,
    pub action: AccessAction,
}

pub trait AuthorizationPolicy {
    fn authorize(&self, request: &AccessRequest) -> Result<(), &'static str>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseAccessPolicy {
    pub owner_id: Id,
}

impl AuthorizationPolicy for CaseAccessPolicy {
    fn authorize(&self, request: &AccessRequest) -> Result<(), &'static str> {
        if request.actor_id != self.owner_id {
            return Err("case access denied");
        }
        if request.case_id.is_empty() {
            return Err("case id is required");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(actor_id: &str) -> AccessRequest {
        AccessRequest {
            actor_id: actor_id.into(),
            case_id: "case-1".into(),
            action: AccessAction::Read,
        }
    }

    #[test]
    fn owner_is_authorized() {
        let policy = CaseAccessPolicy {
            owner_id: "user-1".into(),
        };
        assert!(policy.authorize(&request("user-1")).is_ok());
    }

    #[test]
    fn other_actor_is_denied() {
        let policy = CaseAccessPolicy {
            owner_id: "user-1".into(),
        };
        assert_eq!(
            policy.authorize(&request("user-2")),
            Err("case access denied")
        );
    }
}
