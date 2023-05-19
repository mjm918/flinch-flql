# FLQL - Flinch Query Language

[![Rust](https://github.com/mjm918/dql/actions/workflows/rust.yml/badge.svg)](https://github.com/mjm918/dql/actions/workflows/rust.yml)

FLQL - Flinch Query Language is a lightweight query language designed to retrieve data from an in-memory database called Flinch. Flinch is a real-time unstructured document database that is capable of storing and retrieving documents in JSON format.

FLQL was created to simplify the querying process in Flinch and make it easier for developers to retrieve data from the database. It is a simple, intuitive, and expressive language that is easy to learn and use.

# Features:

- FLQL supports basic CRUD (Create, Read, Update, Delete) operations.
- It supports querying documents based on a variety of criteria, including text matching, date range, and numerical range.
- It uses a simple and intuitive syntax that is easy to read and write.
- FLQL is optimized for performance and can handle large datasets with ease.

# Getting Started:
To use FLQL, you will need to have Flinch installed and running on your system. Once Flinch is up and running, you can start querying your data using FLQL.

> Create collection <br>
`new({});` <br>

> Drop collection <br>
`drop('');` <br>

> Check if pointer exists in collection <br>
`exists('').into('');` <br>

> Length of collection <br>
`length('');` <br>

> Update or Insert into collection <br>
`put({}).into('');` <br>

> Conditional Update or Insert into collection <br>
`put({}).when(:includes(array_filter('e.f$.g'),2):).into('');` <br>

> Update or Insert into collection to a Pointer <br>
`put({}).pointer('').into('');` <br>

> Get from collection <br>
`get.from('');` <br>

> Conditional Get from collection <br>
`get.when(:includes(array_filter('e.f$.g'),2):).from('');` <br>

> Get Pointer from collection <br>
`get.pointer('').from('');` <br>

> Get View from collection <br>
`get.view('').from('');` <br>

> Get Clip from collection <br>
`get.clip('').from('');` <br>

> Delete from collection <br>
`delete.from('');` <br>

> Conditional Delete from collection <br>
`delete.when(:includes(array_filter('e.f$.g'),2):).from('');` <br>

> Delete Pointer from collection <br>
`delete.pointer('').from('');` <br>

> Delete View from collection <br>
`delete.view('').from('');` <br>

>Delete Clip from collection <br>
`delete.clip('').from('');` <br>

