
use serde::{Deserialize, Serialize};
use weil_macros::{constructor, mutate, query, secured, smart_contract, WeilType};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize,WeilType,Clone)]
pub struct ProofInfo {
    pub proof_hash: String,
    pub proof_type: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize,WeilType,Clone)]
pub struct ComplaintInfo {
    pub user_id: String,
    pub complaint_hash: String,
    pub timestamp: String,
    pub status: String,
    pub last_status_update: String,
    pub proofs: Vec<ProofInfo>,
}

trait AuditLayer {
    fn new() -> Result<Self, String>
    where
        Self: Sized;
    async fn complaint_register(&mut self, complaint_id: String, complaint_hash: String, user_id: String, timestamp: String) -> bool;
    async fn register_proof(&mut self, complaint_id: String, proof_hash: String, proof_type: String, timestamp: String) -> bool;
    async fn update_complaint_status(&mut self, complaint_id: String, status: String, timestamp: String) -> bool;
    async fn get_complaints(&self) -> std::collections::BTreeMap<String, ComplaintInfo>;
    async fn get_complaint(&self, complaint_id: String) -> ComplaintInfo;
    fn tools(&self) -> String;
    fn prompts(&self) -> String;
}

#[derive(Serialize, Deserialize, WeilType)]
pub struct AuditLayerContractState {
    // define your contract state here!
    complaints: BTreeMap<String, ComplaintInfo>,
}

#[smart_contract]
impl AuditLayer for AuditLayerContractState {
    #[constructor]
    fn new() -> Result<Self, String>
    where
        Self: Sized,
    {
        Ok(Self {
          complaints: BTreeMap::new(),
        })
    }


    #[mutate]
    async fn complaint_register(&mut self, complaint_id: String, complaint_hash: String, user_id: String, timestamp: String) -> bool {
        if self.complaints.contains_key(&complaint_id) {
          return false;
        }
        let new_complaint = ComplaintInfo {
          user_id,
          complaint_hash,
          timestamp: timestamp.clone(),
          status: "FILED".to_string(),
          last_status_update: timestamp.clone(),
          proofs: Vec::new(),
        };
        self.complaints.insert(complaint_id, new_complaint);
        true
    }

    #[mutate]
    async fn register_proof(&mut self, complaint_id: String, proof_hash: String, proof_type: String, timestamp: String) -> bool {
        let complaint = match self.complaints.get_mut(&complaint_id) {
          Some(c) => c,
          None => return false,
        };
        if complaint.status == "REJECTED" || complaint.status == "RESOLVED" {
          // the complaint has already been rejected or resloved so no need add proofs to it
          return false;
        }
        let proof = ProofInfo {
          proof_hash,
          proof_type,
          timestamp
        };
        complaint.proofs.push(proof);
        true
    }

    #[mutate]
    async fn update_complaint_status(&mut self, complaint_id: String, status: String, timestamp: String) -> bool {
        let complaint = match self.complaints.get_mut(&complaint_id) {
          Some(c) => c,
          None => return false,
        };
        if complaint.status == "REJECTED" || complaint.status == "RESOLVED" {
          // the complaint has already been rejected or resloved so no need to update or change status
          return false;
        };
        complaint.status = status;
        complaint.last_status_update = timestamp;
        true

    }

    #[query]
    async fn get_complaints(&self) -> std::collections::BTreeMap<String, ComplaintInfo> {
        self.complaints.clone()
    }

    #[query]
    async fn get_complaint(&self, complaint_id: String) -> ComplaintInfo {
        self.complaints.get(&complaint_id).cloned().unwrap_or(ComplaintInfo {
          user_id: "".to_string(),
          complaint_hash: "".to_string(),
          timestamp: "".to_string(),
          status: "".to_string(),
          proofs: Vec::new(),
          last_status_update: "".to_string(),
        })
    }


    #[query]
    fn tools(&self) -> String {
        r#"[
  {
    "type": "function",
    "function": {
      "name": "complaint_register",
      "description": "register a new complaint\n",
      "parameters": {
        "type": "object",
        "properties": {
          "complaint_id": {
            "type": "string",
            "description": "unique complaint id\n"
          },
          "complaint_hash": {
            "type": "string",
            "description": "SHA256 hash of the complaint text\n"
          },
          "user_id": {
            "type": "string",
            "description": "unique user id\n"
          },
          "timestamp": {
            "type": "string",
            "description": "time at which the complaint is registered\n"
          }
        },
        "required": [
          "complaint_id",
          "complaint_hash",
          "user_id",
          "timestamp"
        ]
      }
    }
  },
  {
    "type": "function",
    "function": {
      "name": "register_proof",
      "description": "we will store the hash of the proof, what type of proof and timestamp of proof submission using this funtion",
      "parameters": {
        "type": "object",
        "properties": {
          "complaint_id": {
            "type": "string",
            "description": "complaint id to which the proof belong\n"
          },
          "proof_hash": {
            "type": "string",
            "description": "SHA256 hash of the proof\n"
          },
          "proof_type": {
            "type": "string",
            "description": "type of the proof which has been submitted\n"
          },
          "timestamp": {
            "type": "string",
            "description": "time at which the proof submitted\n"
          }
        },
        "required": [
          "complaint_id",
          "proof_hash",
          "proof_type",
          "timestamp"
        ]
      }
    }
  },
  {
    "type": "function",
    "function": {
      "name": "update_complaint_status",
      "description": "used to update the complaint status\n",
      "parameters": {
        "type": "object",
        "properties": {
          "complaint_id": {
            "type": "string",
            "description": "complaint_id in which the status needs to be updated if the current status if REJECTED or RESOLVED we skip\n"
          },
          "status": {
            "type": "string",
            "description": "status of the complaint like FILED, UNDER_INVESTIGATION, RESOLVED, REJECTED\n"
          },
          "timestamp": {
          "type": "string",
          "description":"timestamp of the which the complaint status is updated\n"
          }
        },
        "required": [
          "complaint_id",
          "status",
          "timestamp"
        ]
      }
    }
  },
  {
    "type": "function",
    "function": {
      "name": "get_complaints",
      "description": "to fetch a all complaints\n",
      "parameters": {
        "type": "object",
        "properties": {},
        "required": []
      }
    }
  },
  {
    "type": "function",
    "function": {
      "name": "get_complaint",
      "description": "to fetch a single complaint with complaint id\n",
      "parameters": {
        "type": "object",
        "properties": {
          "complaint_id": {
            "type": "string",
            "description": "complaint id which we want to retrieve\n"
          }
        },
        "required": [
          "complaint_id"
        ]
      }
    }
  }
]"#.to_string()
    }


    #[query]
    fn prompts(&self) -> String {
        r#"{
  "prompts": []
}"#.to_string()
    }
}

