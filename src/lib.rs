#[macro_use]
extern crate pest_derive;
extern crate pest;

use pest::Parser;
use pest::iterators::{Pair, Pairs};


/// **Create collection** <br>
/// `new({});` <br>
///
/// **Drop collection** <br>
/// `drop('');` <br>
///
/// **Check if pointer exists in collection** <br>
/// `exists('').into('');` <br>
///
/// **Length of collection** <br>
/// `length('');` <br>
///
/// **Update or Insert into collection** <br>
/// `upsert({}).into('');` <br>
///
/// **Conditional Update or Insert into collection** <br>
/// `upsert({}).when(:includes(array_filter('e.f$.g'),2):).into('');` <br>
///
/// **Update or Insert into collection to a Pointer** <br>
/// `upsert({}).pointer('').into('');` <br>
///
/// **Get from collection** <br>
/// `get.from('');` <br>
///
/// **Conditional Get from collection** <br>
/// `get.when(:includes(array_filter('e.f$.g'),2):).from('');` <br>
///
/// **Get Pointer from collection** <br>
/// `get.pointer('').from('');` <br>
///
/// **Get View from collection** <br>
/// `get.view('').from('');` <br>
///
/// **Get Clip from collection** <br>
/// `get.clip('').from('');` <br>
///
/// **Delete from collection** <br>
/// `delete.from('');` <br>
///
/// **Conditional Delete from collection** <br>
/// `delete.when(:includes(array_filter('e.f$.g'),2):).from('');` <br>
///
/// **Delete Pointer from collection** <br>
/// `delete.pointer('').from('');` <br>
///
/// **Delete View from collection** <br>
/// `delete.view('').from('');` <br>
///
/// **Delete Clip from collection** <br>
/// `delete.clip('').from('');` <br>
/// <br><br>
/// # Example
/// <br><br>
/// ```rust
///         use dql::{parse, Dql};
///         let commands = vec![
///             "new({});",
///             "drop('');",
///             "exists('').into('');",
///             "length('');",
///             "upsert({}).into('');",
///             "upsert({}).when(:includes(array_filter('e.f$.g'),2):).into('');",
///             "upsert({}).pointer('').into('');",
///             "get.from('');",
///             "get.when(:includes(array_filter('e.f$.g'),2):).from('');",
///             "get.pointer('').from('');",
///             "get.view('').from('');",
///             "get.clip('').from('');",
///             "delete.from('');",
///             "delete.when(:includes(array_filter('e.f$.g'),2):).from('');",
///             "delete.pointer('').from('');",
///             "delete.view('').from('');",
///             "delete.clip('').from('');"
///         ];
///         for command in commands {
///             let chk = parse(command);
///             assert!(chk.is_ok(),"{:?}",chk.err());
///             if chk.is_ok() {
///                 let parsed = chk.unwrap();
///                 match parsed {
///                     Dql::New(_) => {}
///                     Dql::Drop(_) => {}
///                     Dql::Exists(_) => {}
///                     Dql::Length(_) => {}
///                     Dql::Upsert(_, _) => {}
///                     Dql::UpsertWhen(_, _, _) => {}
///                     Dql::UpsertPointer(_, _, _) => {}
///                     Dql::Get(_) => {}
///                     Dql::GetWhen(_, _) => {}
///                     Dql::GetPointer(_, _) => {}
///                     Dql::GetView(_, _) => {}
///                     Dql::GetClip(_, _) => {}
///                     Dql::Delete(_) => {}
///                     Dql::DeleteWhen(_, _) => {}
///                     Dql::DeletePointer(_, _) => {}
///                     Dql::DeleteView(_, _) => {}
///                     Dql::DeleteClip(_, _) => {}
///                     Dql::None => {}
///                 }
///             }
///         }
/// ```

#[derive(Parser)]
#[grammar = "/Users/julfikar/Documents/Personal.nosync/dql/dql.pest"]
struct DqlParser;

#[derive(PartialEq, Debug, Clone)]
pub enum Dql {
    New(String),
    Drop(String),
    Exists(String),
    Length(String),
    Upsert(String,String),
    UpsertWhen(String, String, String),
    UpsertPointer(String, String, String),
    Get(String),
    GetWhen(String, String),
    GetPointer(String, String),
    GetView(String, String),
    GetClip(String, String),
    Delete(String),
    DeleteWhen(String, String),
    DeletePointer(String, String),
    DeleteView(String, String),
    DeleteClip(String, String),
    None
}

fn pair_parser(pair: Pair<Rule>) -> Dql  {
    match pair.as_rule() {
        Rule::expr => pair_parser(pair.into_inner().next().unwrap()),
        Rule::new => {
            Dql::New(one(pair).to_string())
        }
        Rule::drop => {
            Dql::Drop(one(pair).to_string())
        }
        Rule::exists => {
            Dql::Exists(one(pair).to_string())
        }
        Rule::length => {
            Dql::Length(one(pair).to_string())
        }
        Rule::upsert => {
            let two = two(pair);
            Dql::Upsert(
                two[0].to_string(),
                two[1].to_string()
            )
        }
        Rule::upsert_when => {
            let three = three(pair);
            Dql::UpsertWhen(
                three[0].to_string(),
                three[1].to_string(),
                three[2].to_string()
            )
        }
        Rule::upsert_pointer => {
            let three = three(pair);
            Dql::UpsertPointer(
                three[0].to_string(),
                three[1].to_string(),
                three[2].to_string()
            )
        }
        Rule::get => {
            Dql::Get(one(pair).to_string())
        }
        Rule::get_when => {
            let two = two(pair);
            Dql::GetWhen(
                two[0].to_string(),
                two[1].to_string()
            )
        }
        Rule::get_pointer => {
            let two = two(pair);
            Dql::GetPointer(
                two[0].to_string(),
                two[1].to_string()
            )
        }
        Rule::get_view => {
            let two = two(pair);
            Dql::GetView(
                two[0].to_string(),
                two[1].to_string()
            )
        }
        Rule::get_clip => {
            let two = two(pair);
            Dql::GetClip(
                two[0].to_string(),
                two[1].to_string()
            )
        }
        Rule::delete => {
            Dql::Delete(one(pair).to_string())
        }
        Rule::delete_when => {
            let two = two(pair);
            Dql::DeleteWhen(
                two[0].to_string(),
                two[1].to_string()
            )
        }
        Rule::delete_pointer => {
            let two = two(pair);
            Dql::DeletePointer(
                two[0].to_string(),
                two[1].to_string()
            )
        }
        Rule::delete_view => {
            let two = two(pair);
            Dql::DeleteView(
                two[0].to_string(),
                two[1].to_string()
            )
        }
        Rule::delete_clip => {
            let two = two(pair);
            Dql::DeleteClip(
                two[0].to_string(),
                two[1].to_string()
            )
        }
        _ => Dql::None
    }
}

fn one(opt: Pair<Rule>) -> String {
    let mut pair = opt.into_inner();
    str(pair.next().unwrap())
}

fn two(opt: Pair<Rule>) -> [String; 2] {
    let mut pair = opt.into_inner();
    let index_id = str(pair.next().unwrap());
    let doc = str(pair.next().unwrap());
    [index_id, doc]
}

fn three(opt:Pair<Rule>) -> [String; 3] {
    let mut pair = opt.into_inner();
    let index_id = str(pair.next().unwrap());
    let doc = str(pair.next().unwrap());
    let cond = str(pair.next().unwrap());
    [index_id, doc, cond]
}

fn str(opt: Pair<Rule>) -> String {
    opt.as_str().to_string()
}

pub fn parse(dql: &str) -> Result<Dql, String> {
    let pairs = DqlParser::parse(Rule::program,dql);
    return if pairs.is_ok() {
        let mut node = None;
        let pairs = pairs.unwrap();
        for pair in pairs {
            node = match pair.as_rule() {
                Rule::expr => Some(pair_parser(pair)),
                _ => None
            };
            if node.is_some() {
                break;
            }
        }
        if node.is_some() {
            Ok(node.unwrap())
        } else {
            Err("failed to parse".to_owned())
        }
    } else {
        Err(format!("{}", pairs.err().unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Dql, parse};

    #[test]
    fn test() {
        let commands = vec![
            "new({});",
            "drop('');",
            "exists('').into('');",
            "length('');",
            "upsert({}).into('');",
            "upsert({}).when(:includes(array_filter('e.f$.g'),2):).into('');",
            "upsert({}).pointer('').into('');",
            "get.from('');",
            "get.when(:includes(array_filter('e.f$.g'),2):).from('');",
            "get.pointer('').from('');",
            "get.view('').from('');",
            "get.clip('').from('');",
            "delete.from('');",
            "delete.when(:includes(array_filter('e.f$.g'),2):).from('');",
            "delete.pointer('').from('');",
            "delete.view('').from('');",
            "delete.clip('').from('');"
        ];
        for command in commands {
            let chk = parse(command);
            assert!(chk.is_ok(),"{:?}",chk.err());
            if chk.is_ok() {
                let parsed = chk.unwrap();
                match parsed {
                    Dql::New(_) => {}
                    Dql::Drop(_) => {}
                    Dql::Exists(_) => {}
                    Dql::Length(_) => {}
                    Dql::Upsert(_, _) => {}
                    Dql::UpsertWhen(_, _, _) => {}
                    Dql::UpsertPointer(_, _, _) => {}
                    Dql::Get(_) => {}
                    Dql::GetWhen(_, _) => {}
                    Dql::GetPointer(_, _) => {}
                    Dql::GetView(_, _) => {}
                    Dql::GetClip(_, _) => {}
                    Dql::Delete(_) => {}
                    Dql::DeleteWhen(_, _) => {}
                    Dql::DeletePointer(_, _) => {}
                    Dql::DeleteView(_, _) => {}
                    Dql::DeleteClip(_, _) => {}
                    Dql::None => {}
                }
            }
        }
    }
}