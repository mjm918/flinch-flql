#[macro_use]
extern crate pest_derive;
extern crate pest;

use anyhow::{Result, anyhow, Error};
use pest::Parser;
use serde_json::Value;
use pest::iterators::{Pair, Pairs};

#[derive(Parser)]
#[grammar = "./dql.pest"]
struct DqlParser;

#[derive(PartialEq, Debug, Clone)]
pub enum Direction {
    Asc,
    Desc
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct  OffsetLimit {
    offset: Option<u64>,
    limit: Option<u64>
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct SortDirection {
    field: Option<String>,
    direction: Option<Direction>
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct Clause {
    and: Option<Value>,
    or: Option<Value>
}

#[derive(PartialEq, Debug, Clone)]
pub enum Dql {
    Create(String, Value),
    Drop(String),
    Len(String),
    Upsert(String,Value,Option<Clause>),
    UpsertWithoutClause(String,Value),
    Put(String,String,Value),
    Exists(String,String),
    Search(String,String,Option<SortDirection>,Option<OffsetLimit>),
    GetIndex(String,String),
    GetWithoutClause(String,Option<SortDirection>,Option<OffsetLimit>),
    Get(String,Option<Clause>,Option<SortDirection>,Option<OffsetLimit>),
    DeleteIndex(String,String),
    DeleteWithoutClause(String),
    Delete(String,Option<Clause>),
    None
}

fn recur(pair: Pair<Rule>) -> Dql {
    let ret = match pair.as_rule() {
        Rule::expr => recur(pair.into_inner().next().unwrap()),
        Rule::create => {
            let kv = kv(pair);
            Dql::Create(
                kv.0,
                serde_json::from_str(
                    &kv.1
                ).unwrap()
            )
        }
        Rule::drop => {
            Dql::Drop(
                k(pair)
            )
        }
        Rule::len => {
            Dql::Len(
                k(pair)
            )
        }
        Rule::upsert => {
            let kv = kv(pair);
            Dql::UpsertWithoutClause(
                kv.0,
                serde_json::from_str(
                    &kv.1
                ).unwrap()
            )
        }
        Rule::upsert_where => {
            let mut p = pair.clone().into_inner();
            let col = k(pair);

            p.next();

            let lh_pair = p.next().unwrap();
            let doc: Value = serde_json::from_str(
                &str(lh_pair)
            ).unwrap();

            let mut clause = Clause::default();
            cl(&mut clause, p);

            Dql::Upsert(col, doc, Some(clause))
        }
        Rule::put => {
            let kv = kv(pair.clone());
            let mut p = pair.into_inner();
            p.next();
            p.next();
            let obj = str(p.next().unwrap());
            Dql::Put(
                kv.0,
                kv.1,
                serde_json::from_str(&obj.as_str()).unwrap()
            )
        }
        Rule::exi => {
            let kv = kv(pair);
            Dql::Exists(kv.0, kv.1)
        }
        Rule::search => {
            let kv = kv(pair.clone());
            let p = pair.into_inner();
            Dql::Search(kv.0, kv.1,sr(p.clone()),ol(p))
        }
        Rule::select => {
            let mut p = pair.clone().into_inner();
            let col = p.next().unwrap().as_str().to_string();
            let empty = p.next().unwrap().as_str().to_string();
            if empty.eq("{}") {
                let sort = sr(p.clone());
                let ofsl = ol(p.clone());

                Dql::GetWithoutClause(col, sort,ofsl)
            } else {
                p = pair.into_inner();
                p.next();

                let mut clause = Clause::default();
                cl(&mut clause, p.clone());

                let sort = sr(p.clone());
                let ofsl = ol(p.clone());

                Dql::Get(col,  Some(clause),sort,ofsl)
            }
        }
        Rule::get => {
            let kv = kv(pair);
            Dql::GetIndex(kv.0, kv.1)
        }
        Rule::delete => {
            let kv = kv(pair);
            Dql::DeleteIndex(kv.0, kv.1)
        }
        Rule::delete_where => {
            let mut p = pair.clone().into_inner();
            let col = p.next().unwrap().as_str().to_string();
            let empty = p.next().unwrap().as_str().to_string();
            if empty.eq("{}") {
                Dql::DeleteWithoutClause(col)
            } else {
                p = pair.into_inner();
                p.next();

                let mut clause = Clause::default();
                cl(&mut clause, p);

                Dql::Delete(col, Some(clause))
            }
        }
        _=> Dql::None
    };
    ret
}
fn ol(mut p: Pairs<Rule>) -> Option<OffsetLimit> {
    let mut offset: Option<u64> = None;
    let mut limit: Option<u64> = None;
    loop {
        let next = p.next();
        if next.is_none() {
            break;
        } else {
            let next = next.unwrap();
            match next.as_rule() {
                Rule::offset => {
                    let ofs = next.as_str().to_uppercase()
                        .replace("OFFSET","")
                        .trim()
                        .parse::<u64>();
                    if ofs.is_ok() {
                        offset = Some(ofs.unwrap());
                    }
                }
                Rule::limit => {
                    let lmt = next.as_str().to_uppercase()
                        .replace("LIMIT","")
                        .trim()
                        .parse::<u64>();
                    if lmt.is_ok() {
                        limit = Some(lmt.unwrap());
                    }
                }
                _=>{}
            }
        }
    }
    if offset.is_none() && limit.is_none() {
        None
    } else {
        Some(OffsetLimit {
            offset,
            limit,
        })
    }
}
fn sr(mut p: Pairs<Rule>) -> Option<SortDirection> {
    let mut field: Option<String> = None;
    let mut direction: Option<Direction> = None;
    loop {
        let next = p.next();
        if next.is_none() {
            break;
        } else {
            let next = next.unwrap();
            match next.as_rule() {
                Rule::sort => {
                    field = Some(next.as_str()
                        .replace("SORT","")
                        .replace("sort","")
                        .trim().to_string());
                }
                Rule::direction => {
                    let kwr = next.as_str().trim().to_uppercase();
                    direction = Some(
                        match kwr.as_str() {
                            "ASC"=> Direction::Asc,
                            "DESC"=> Direction::Desc,
                            _=> Direction::Asc
                        }
                    );
                }
                _=>{}
            }
        }
    }
    if field.is_none() && direction.is_none() {
        None
    } else {
        Some(SortDirection{ field, direction })
    }
}
fn cl(clause: &mut Clause, mut p: Pairs<Rule>) {
    let mut conjunction = format!("");
    loop {
        let next = p.next();
        if next.is_none() {
            break;
        } else {
            let next = next.unwrap().clone();
            match next.as_rule() {
                Rule::conjunction => {
                    conjunction = str(next);
                    match conjunction.as_str() {
                        "$or" => {
                            clause.or = Some(Value::Array(vec![]));
                        },
                        "$and" => {
                            clause.and = Some(Value::Array(vec![]));
                        },
                        _=> {}
                    }
                }
                Rule::clause_array => {
                    let v: Option<Value> = Some(serde_json::from_str(&str(next).as_str()).unwrap());
                    match conjunction.as_str() {
                        "$or" => {
                            clause.or = v;
                        },
                        "$and" => {
                            clause.and = v;
                        },
                        _=> {
                            println!("what is it ? {}",conjunction);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
fn k(opt: Pair<Rule>) -> String {
    let mut pair = opt.into_inner();
    str(pair.next().unwrap())
}
fn kv(opt: Pair<Rule>) -> (String, String) {
    let mut pair = opt.into_inner();
    let index_id = str(pair.next().unwrap());
    let doc = str(pair.next().unwrap());
    (index_id, doc)
}
fn str(opt: Pair<Rule>) -> String {
    opt.as_str().to_string()
}

pub fn parse(dql: &str) -> Result<Vec<Dql>, Error> {
    let pairs = DqlParser::parse(Rule::program,dql);
    if pairs.is_err() {
        Err(anyhow!("{}",pairs.err().unwrap()))
    } else {
        let mut nodes = vec![];
        let pairs = pairs.unwrap();
        for pair in pairs {
            match pair.as_rule() {
                Rule::expr => nodes.push(recur(pair)),
                _=>{}
            }
        }
        Ok(nodes)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use super::*;

    #[test]
    fn it_works() {
        let ttk = Instant::now();
        let ql = r#"
            CREATE collection -> {};

            DROP collection;

            LEN collection;

            UPSERT collection [{"avc":"1123"}];

            UPSERT collection {"avc":"1123"} WHERE $or:[{"$eq":{"a.b":1}}] $and:[{"$lt":{"a":3}}];

            PUT collection -> id -> {};

            EXISTS collection -> id;

            SEARCH collection -> 'your random query' OFFSET 0 LIMIT 1000000;

            GET collection WHERE {} SORT id DESC OFFSET 0 LIMIT 1000000;

            GET collection WHERE $or:[{"$eq":{"a.b":3}},{"$lt":{"b":3}}] OFFSET 0 LIMIT 1000000;

            GET collection -> id;

            DELETE collection -> id;

            DELETE collection WHERE $or:[{"$eq":{"a.b":1}}] $and:[{"$lt":{"a":3}}];
        "#;
        let parsed = parse(ql);
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert!(!parsed.is_empty());
        println!("Time taken to parse {:?}",ttk.elapsed());

        for ql in &parsed {
            match ql {
                Dql::Create(name, option) => {}
                Dql::Drop(name) => {}
                Dql::Len(name) => {}
                Dql::Upsert(name, doc, clause) => {}
                Dql::UpsertWithoutClause(name, doc) =>{}
                Dql::Put(name, index, doc) => {}
                Dql::Exists(name, index) => {}
                Dql::Search(name, query, sort, limit) => {}
                Dql::GetIndex(name, index) => {}
                Dql::GetWithoutClause(name, sort, limit) => {}
                Dql::Get(name, clause, sort, limit) => {}
                Dql::DeleteIndex(name, index) => {}
                Dql::DeleteWithoutClause(name) => {}
                Dql::Delete(name, clause) => {}
                Dql::None => {}
            }
        }

        println!("{:?}",parsed);
    }
}
