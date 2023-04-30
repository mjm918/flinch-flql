# DQL - Document Query Language

[![Rust](https://github.com/mjm918/dql/actions/workflows/rust.yml/badge.svg)](https://github.com/mjm918/dql/actions/workflows/rust.yml)

DQL (Document Query Language) is a lightweight query language designed to retrieve data from an in-memory database called Flinch. Flinch is a real-time unstructured document database that is capable of storing and retrieving documents in JSON format.

DQL was created to simplify the querying process in Flinch and make it easier for developers to retrieve data from the database. It is a simple, intuitive, and expressive language that is easy to learn and use.

# Features:

- DQL supports basic CRUD (Create, Read, Update, Delete) operations.
- It supports querying documents based on a variety of criteria, including text matching, date range, and numerical range.
- It uses a simple and intuitive syntax that is easy to read and write.
- DQL is optimized for performance and can handle large datasets with ease.

# Getting Started:
To use DQL, you will need to have Flinch installed and running on your system. Once Flinch is up and running, you can start querying your data using DQL.

**CREATE a collection**

Here `'collection'` is the name of your collection. Followed by pointer operator `->` 
and then collection options as `json` object

`CREATE collection_name -> {};`


**DROP**

Drop or delete a collection

`DROP collection_name;`

**LENGTH**

Length of a collection / number of elements in a collection

`LEN collection_name;`

**UPSERT**

Update or Insert (if not exists) documents.

`UPSERT collection_name [{"avc":"1123"}];`

Update or Insert document with condition

`UPSERT collection_name {"avc":"1123"} WHERE $or:[{"$eq":["a.b",1]}] $and:[{"$lt":["a",3]}];`

**PUT**

Put with a pointer to a document ID.

`PUT collection_name -> 'id' -> {};`

**EXISTS**

Exists works with a pointer.

`EXISTS collection_name -> 'id';`

**SEARCH**

Works only with collection option `content_opt`. Useful when you use flinch as a search engine.

`SEARCH collection_name -> 'your random query';`

**GET**

`GET` is similar to `SELECT` 

1. To get all documents 

`GET collection_name WHERE {};`

2. Get all documents that matches condition

`GET collection_name WHERE $or:[{"$eq":["a->b",1]},{"$lt":["a",3]}];`

`GET` also works with pointer.

`GET collection_name -> 'id';`

**DELETE**

`DELETE` works with both pointer and condition

1. Delete a pointer with ID

`DELETE collection_name -> 'id';`

2. Delete documents that matches condition

`DELETE collection_name WHERE $or:[{"$eq":["a->b",1]}] $and:[{"$lt":["a",3]}];`

# Sort, Offset, Limit

`SORT prop_name DESC/ASC OFFSET u64 LIMIT u64`

# Available Operators

| Keyword | Description             |
|---------|-------------------------|
| `$eq`   | Equal `=`               |
| `$neq`  | Not Equal `<>`          |
| `$lt`   | Lower than `<`          |
| `$lte`  | Lower than Equal `<=`   |
| `$gt`   | Greater than `>`        |
| `$gte`  | Greater than Equal `>=` |
| `$like` | Like `%%`               |
| `$inc`  | Includes / Contains     |
| `$ninc` | Not Includes / Contains |


# Conjunctions

| Keyword | Usage                                                                                                                             |
|---------|-----------------------------------------------------------------------------------------------------------------------------------|
| `$or`   | Expects an array of strict type object.<br/>Object can be only like<br/>`{<Operator>:[<field_name OR pointer>,<expected_value>]}` |
| `$and`  | ”                                                                                                                                 |

NOTE: Pointers in a condition, is used to access nested document regardless the type is array or object.
Pointer can be defined as `doc_property->nested_doc_property`

# Example

```
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

for ql in parsed {
    match ql {
        Dql::Create(name, option) => {}
        Dql::Open(name) => {}
        Dql::Drop(name) => {}
        Dql::Len(name) => {}
        Dql::Upsert(name, doc, clause) => {}
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
```
