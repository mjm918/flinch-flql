#[macro_use]
extern crate pest_derive;
extern crate pest;

use anyhow::{Result, anyhow, Error};
use pest::Parser;
use serde_json::Value;
use pest::iterators::{Pair, Pairs};

#[derive(Parser)]
#[grammar = "./dql.pest"]
pub struct DqlParser;

#[derive(PartialEq, Debug, Clone, Default)]
pub struct Clause {
    and: Option<Value>,
    or: Option<Value>
}

#[derive(PartialEq, Debug, Clone)]
pub enum DataTypes {
    Str(String),
    Json(Value)
}

#[derive(PartialEq, Debug, Clone)]
pub enum Formatted {
    Create(DataTypes, DataTypes),
    Open(DataTypes),
    Drop(DataTypes),
    Len(DataTypes),
    Upsert(DataTypes,Option<Clause>),
    Put(DataTypes,DataTypes),
    Exists(DataTypes),
    Search(DataTypes),
    Find(Option<Clause>),
    Get(DataTypes),
    Delete(Option<DataTypes>,Option<Clause>),
    None
}

fn recur(pair: Pair<Rule>) -> Formatted {
    let ret = match pair.as_rule() {
        Rule::expr => recur(pair.into_inner().next().unwrap()),
        Rule::create => {
            let kv = kv(pair);
            Formatted::Create(
                DataTypes::Str(kv.0),
                DataTypes::Json(
                    serde_json::from_str(
                        &kv.1
                    ).unwrap()
                )
            )
        }
        Rule::open => {
            Formatted::Open(
                DataTypes::Str(k(pair))
            )
        }
        Rule::drop => {
            Formatted::Drop(
                DataTypes::Str(k(pair))
            )
        }
        Rule::len => {
            Formatted::Len(
                DataTypes::Str(k(pair))
            )
        }
        Rule::upsert => {
            let k = k(pair);
            Formatted::Upsert(
                DataTypes::Json(
                    serde_json::from_str(
                        &k
                    ).unwrap()
                ),
                None
            )
        }
        Rule::upsert_where => {
            let mut p = pair.into_inner();
            let lh_pair = p.next().unwrap();
            let doc: Value = serde_json::from_str(
                &str(lh_pair)
            ).unwrap();

            let mut clause = Clause::default();
            cl(&mut clause, p);

            Formatted::Upsert(DataTypes::Json(doc), Some(clause))
        }
        Rule::put => {
            let kv = kv(pair);
            Formatted::Put(
                DataTypes::Str(kv.0),
                DataTypes::Json(
                    serde_json::from_str(&kv.1.as_str()).unwrap()
                )
            )
        }
        Rule::exi => {
            Formatted::Exists(DataTypes::Str(k(pair)))
        }
        Rule::search => {
            Formatted::Search(DataTypes::Str(k(pair)))
        }
        Rule::find => {
            let mut p = pair.clone().into_inner();
            let empty = str(p.next().unwrap());
            if empty.eq("{}") {
                Formatted::Find(None)
            } else {
                p = pair.into_inner();
                let mut clause = Clause::default();
                cl(&mut clause, p);

                Formatted::Find(Some(clause))
            }
        }
        Rule::get => {
            Formatted::Get(DataTypes::Str(k(pair)))
        }
        Rule::delete => {
            Formatted::Delete(Some(DataTypes::Str(k(pair))), None)
        }
        Rule::delete_where => {
            let mut p = pair.clone().into_inner();
            let empty = str(p.next().unwrap());
            if empty.eq("{}") {
                Formatted::Delete(None, None)
            } else {
                p = pair.into_inner();
                let mut clause = Clause::default();
                cl(&mut clause, p);

                Formatted::Delete(None,Some(clause))
            }
        }
        _=> Formatted::None
    };
    ret
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
                        _=> unreachable!()
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
                _ => unreachable!()
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
    opt.as_str().to_string().trim_matches('\'').to_string()
}

pub fn parse(dql: &str) -> Result<Vec<Formatted>, Error> {
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
        let parsed = parse("CREATE 'collection' -> {};");
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert!(!parsed.is_empty());
        println!("Time taken to parse {:?}",ttk.elapsed());

        println!("{:?}",parsed);
    }
}
