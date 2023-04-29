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

`CREATE 'collection' -> {};`

**OPEN**

Opens a collection to read/write

`OPEN 'collection';`

**DROP**

Drop or delete a collection

`DROP 'collection';`

**LENGTH**

Length of a collection / number of elements in a collection

`LEN 'collection';`

**UPSERT**

Update or Insert (if not exists) documents.

`UPSERT [{"avc":"1123"}];`

Update or Insert document with condition

`UPSERT {"avc":"1123"} WHERE $or:[{"$eq":["a.b",1]}] $and:[{"$lt":["a",3]}];`

**PUT**

Put with a pointer to a document ID.

`PUT -> 'id' -> {};`

**EXISTS**

Exists works with a pointer.

`EXISTS -> 'id';`

**SEARCH**

Works only with collection option `content_opt`. Useful when you use flinch as a search engine.

`SEARCH -> 'your random query';`

**FIND**

`FIND` is similar to `SELECT` 

1. To get all documents 

`FIND WHERE {};`

2. Get all documents that matches condition

`FIND WHERE $or:[{"$eq":["a->b",1]},{"$lt":["a",3]}];`

**GET**

`GET` works only with pointer.

`GET -> 'id';`

**DELETE**

`DELETE` works with both pointer and condition

1. Delete a pointer with ID

`DELETE -> 'id';`

2. Delete documents that matches condition

`DELETE WHERE $or:[{"$eq":["a->b",1]}] $and:[{"$lt":["a",3]}];`

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
