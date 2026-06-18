use  serde::Deserialize;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum Steps {
    Match { 
        #[serde(rename = "match")]
        criteria: MatchCriteria 
    },
  
    MoveTo { 
        move_to: String 
    },

    Notify { 
        notify: String 
    }
}


#[derive(Debug, Deserialize)]
pub struct MatchCriteria {
    pub ext: String,
}