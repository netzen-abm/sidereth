# SIDERETH Authorization Matrix

Status: DESIGN BASELINE

| Capability | User | Agent | Professional | System |
|---|---:|---:|---:|---:|
| Create case | Allow | Allow within workflow | Allow | Allow |
| Read own case | Allow | Scoped | Scoped by assignment | Allow for execution |
| Capture evidence | Allow | Allow if explicitly enabled | Allow | N/A |
| Retrieve public legal source | Allow | Allow | Allow | Allow |
| Draft response | Allow | Allow | Allow | N/A |
| Submit legal response | Explicit approval | Never autonomous | Allow when authorized | Never |
| File appeal | Explicit approval | Never autonomous | Allow when authorized | Never |
| Change legal conclusion | User/professional review | Recommend only | Allow | Never |
| Delete evidence | Explicit policy-controlled | Never | Policy-controlled | Never |
| Export case | Allow | Never autonomously | Allow | N/A |
| Access unrelated cases | Deny | Deny | Deny | Deny unless policy explicitly grants |

## Non-negotiable controls
- least privilege
- purpose limitation
- case-scoped access
- explicit consent for external sharing
- no autonomous high-impact legal action
- auditable policy decisions
- deny by default
