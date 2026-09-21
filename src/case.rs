#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaseState {
    Draft,
    Active,
    WaitingUser,
    WaitingAuthority,
    ResponseDue,
    Resolved,
    Closed,
}

impl CaseState {
    pub fn can_transition_to(&self, next: &Self) -> bool {
        matches!(
            (self, next),
            (Self::Draft, Self::Active)
                | (Self::Active, Self::WaitingUser)
                | (Self::Active, Self::WaitingAuthority)
                | (Self::Active, Self::ResponseDue)
                | (Self::Active, Self::Resolved)
                | (Self::WaitingUser, Self::Active)
                | (Self::WaitingAuthority, Self::Active)
                | (Self::ResponseDue, Self::Active)
                | (Self::ResponseDue, Self::Resolved)
                | (Self::Resolved, Self::Closed)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Case {
    pub case_id: Id,
    pub state: CaseState,
}

impl Case {
    pub fn new(case_id: Id) -> Result<Self, &'static str> {
        if case_id.is_empty() {
            return Err("case id is required");
        }
        Ok(Self {
            case_id,
            state: CaseState::Draft,
        })
    }

    pub fn transition(&mut self, next: CaseState) -> Result<(), &'static str> {
        if !self.state.can_transition_to(&next) {
            return Err("invalid case state transition");
        }
        self.state = next;
        Ok(())
    }
}
