use std::{fmt::Display, rc::Rc};

use super::clip::Clip;

#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub enum OrderOrder {
    Desc,
    Asc,
}

impl OrderOrder {
    pub fn to_bool(&self) -> bool {
        match self {
            OrderOrder::Desc => false,
            OrderOrder::Asc => true,
        }
    }
}

impl Display for OrderOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderOrder::Desc => f.write_str("desc"),
            OrderOrder::Asc => f.write_str("asc"),
        }
    }
}

pub fn sort_search_res(
    res: Vec<(i64, Clip)>,
    method: Rc<String>,
    // true for asc, false for desc
    order: bool,
) -> Vec<(i64, Clip)> {
    let mut res = res;
    match method.as_str() {
        "time" => {
            res.sort_by(|a, b| {
                let res = a.1.timestamp.cmp(&b.1.timestamp);
                if order {
                    res
                } else {
                    res.reverse()
                }
            });
        }
        "score" => {
            res.sort_by(|a, b| {
                let res = a.1.score.cmp(&b.1.score);
                if order {
                    res
                } else {
                    res.reverse()
                }
            });
        }
        "id" => {
            res.sort_by(|a, b| {
                let res = a.1.id.cmp(&b.1.id);
                if order {
                    res
                } else {
                    res.reverse()
                }
            });
        }
        "text" => {
            res.sort_by(|a, b| {
                let res = a.1.text.cmp(&b.1.text);
                if order {
                    res
                } else {
                    res.reverse()
                }
            });
        }
        "len" => {
            res.sort_by(|a, b| {
                let res = a.1.len.cmp(&b.1.len);
                if order {
                    res
                } else {
                    res.reverse()
                }
            });
        }
        _ => {}
    }

    res
}
