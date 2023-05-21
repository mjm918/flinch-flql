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

```javascript

//Create collection
new({}); 

// TTL
ttl(60).if('').into('');

//Drop collection
drop(''); 

//Check if pointer exists in collection
exists('').into(''); 

//Length of collection
length(''); 

//Update or Insert into collection
put({}).into(''); 

//Conditional Update or Insert into collection
put({}).when('gjson_expression').into(''); 

//Update or Insert into collection to a Pointer
put({}).pointer('').into(''); 

//Get from collection
get.from(''); 

//Conditional Get from collection
get.when('gjson_expression').from(''); 

//Get Pointer from collection
get.pointer('').from(''); 

//Get View from collection
get.view('').from(''); 

//Get Clip from collection
get.clip('').from(''); 

//Delete from collection
delete.from(''); 

//Conditional Delete from collection
delete.when('gjson_expression').from(''); 

//Delete Pointer from collection
delete.pointer('').from(''); 

//Delete View from collection
delete.view('').from(''); 

//Delete Clip from collection
delete.clip('').from(''); 
```
<br><br>
# Example
 ```rust
         use flql::{parse, Flql};
         let commands = vec![
             "new({});",
             "drop('');",
             "exists('').into('');",
             "length('');",
             "put({}).into('');",
             "put({}).when('gjson_expression').into('');",
             "put({}).pointer('').into('');",
             "get.from('');",
             "get.when('gjson_expression').from('');",
             "get.pointer('').from('');",
             "get.view('').from('');",
             "get.clip('').from('');",
             "delete.from('');",
             "delete.when('gjson_expression').from('');",
             "delete.pointer('').from('');",
             "delete.clip('').from('');"
         ];
         for command in commands {
             let chk = parse(command);
             assert!(chk.is_ok(),"{:?}",chk.err());
             if chk.is_ok() {
                 let parsed = chk.unwrap();
                 match parsed {
                     Flql::New(_) => {}
                     Flql::Drop(_) => {}
                     Flql::Exists(_,_) => {}
                     Flql::Length(_) => {}
                     Flql::Flush(_) => {}
                     Flql::Put(_, _) => {}
                     Flql::PutWhen(_, _, _) => {}
                     Flql::PutPointer(_, _, _) => {}
                     Flql::Search(_,_) => {}
                     Flql::Get(_) => {}
                     Flql::GetWhen(_, _) => {}
                     Flql::GetPointer(_, _) => {}
                     Flql::GetView(_, _) => {}
                     Flql::GetClip(_, _) => {}
                     Flql::GetIndex(_,_) => {}
                     Flql::GetRange(_,_,_,_) => {}
                     Flql::Delete(_) => {}
                     Flql::DeleteWhen(_, _) => {}
                     Flql::DeletePointer(_, _) => {}
                     Flql::DeleteClip(_, _) => {}
                     Flql::None => {}
                 }
             }
         }
 ```

In `when` function you can use any expression from (https://github.com/tidwall/gjson.rs) to manipulate data.

Documentation is from gjson repo:
This document is designed to explain the structure of a GJSON Path through examples.

- [Path structure](#path-structure)
- [Basic](#basic)
- [Wildcards](#wildcards)
- [Escape Character](#escape-character)
- [Arrays](#arrays)
- [Queries](#queries)
- [Dot vs Pipe](#dot-vs-pipe)
- [Modifiers](#modifiers)
- [Multipaths](#multipaths)
- [Literals](#literals)

The definitive implemenation is [github.com/tidwall/gjson](https://github.com/tidwall/gjson).  
Use the [GJSON Playground](https://gjson.dev) to experiment with the syntax online.

## Path structure

A GJSON Path is intended to be easily expressed as a series of components seperated by a `.` character.

Along with `.` character, there are a few more that have special meaning, including `|`, `#`, `@`, `\`, `*`, `!`, and `?`.

## Example

Given this JSON

 ```json
 {
   "name": {"first": "Tom", "last": "Anderson"},
   "age":37,
   "children": ["Sara","Alex","Jack"],
   "fav.movie": "Deer Hunter",
   "friends": [
     {"first": "Dale", "last": "Murphy", "age": 44, "nets": ["ig", "fb", "tw"]},
     {"first": "Roger", "last": "Craig", "age": 68, "nets": ["fb", "tw"]},
     {"first": "Jane", "last": "Murphy", "age": 47, "nets": ["ig", "tw"]}
   ]
 }
 ```

The following GJSON Paths evaluate to the accompanying values.

### Basic

In many cases you'll just want to retreive values by object name or array index.

 ```go
 name.last              "Anderson"
 name.first             "Tom"
 age                    37
 children               ["Sara","Alex","Jack"]
 children.0             "Sara"
 children.1             "Alex"
 friends.1              {"first": "Roger", "last": "Craig", "age": 68}
 friends.1.first        "Roger"
 ```

### Wildcards

A key may contain the special wildcard characters `*` and `?`.
The `*` will match on any zero+ characters, and `?` matches on any one character.

 ```go
 child*.2               "Jack"
 c?ildren.0             "Sara"
 ```

### Escape character

Special purpose characters, such as `.`, `*`, and `?` can be escaped with `\`.

 ```go
 fav\.movie             "Deer Hunter"
 ```
### Arrays

The `#` character allows for digging into JSON Arrays.

To get the length of an array you'll just use the `#` all by itself.

 ```go
 friends.#              3
 friends.#.age         [44,68,47]
 ```

### Queries

You can also query an array for the first match by  using `#(...)`, or find all matches with `#(...)#`.
Queries support the `==`, `!=`, `<`, `<=`, `>`, `>=` comparison operators,
and the simple pattern matching `%` (like) and `!%` (not like) operators.

 ```go
 friends.#(last=="Murphy").first     "Dale"
 friends.#(last=="Murphy")#.first    ["Dale","Jane"]
 friends.#(age>45)#.last             ["Craig","Murphy"]
 friends.#(first%"D*").last          "Murphy"
 friends.#(first!%"D*").last         "Craig"
 ```

To query for a non-object value in an array, you can forgo the string to the right of the operator.

 ```go
 children.#(!%"*a*")                 "Alex"
 children.#(%"*a*")#                 ["Sara","Jack"]
 ```

Nested queries are allowed.

 ```go
 friends.#(nets.#(=="fb"))#.first  >> ["Dale","Roger"]
 ```

*Please note that prior to v1.3.0, queries used the `#[...]` brackets. This was
changed in v1.3.0 as to avoid confusion with the new [multipath](#multipaths)
syntax. For backwards compatibility, `#[...]` will continue to work until the
next major release.*

The `~` (tilde) operator will convert a value to a boolean before comparison.

For example, using the following JSON:

 ```json
 {
   "vals": [
     { "a": 1, "b": true },
     { "a": 2, "b": true },
     { "a": 3, "b": false },
     { "a": 4, "b": "0" },
     { "a": 5, "b": 0 },
     { "a": 6, "b": "1" },
     { "a": 7, "b": 1 },
     { "a": 8, "b": "true" },
     { "a": 9, "b": false },
     { "a": 10, "b": null },
     { "a": 11 }
   ]
 }
 ```

You can now query for all true(ish) or false(ish) values:

 ```flql
 vals.#(b==~true)#.a    >> [1,2,6,7,8]
 vals.#(b==~false)#.a   >> [3,4,5,9,10,11]
 ```

The last value which was non-existent is treated as `false`

### Dot vs Pipe

The `.` is standard separator, but it's also possible to use a `|`.
In most cases they both end up returning the same results.
The cases where`|` differs from `.` is when it's used after the `#` for [Arrays](#arrays) and [Queries](#queries).

Here are some examples

 ```go
 friends.0.first                     "Dale"
 friends|0.first                     "Dale"
 friends.0|first                     "Dale"
 friends|0|first                     "Dale"
 friends|#                           3
 friends.#                           3
 friends.#(last="Murphy")#           [{"first": "Dale", "last": "Murphy", "age": 44},{"first": "Jane", "last": "Murphy", "age": 47}]
 friends.#(last="Murphy")#.first     ["Dale","Jane"]
 friends.#(last="Murphy")#|first     <non-existent>
 friends.#(last="Murphy")#.0         []
 friends.#(last="Murphy")#|0         {"first": "Dale", "last": "Murphy", "age": 44}
 friends.#(last="Murphy")#.#         []
 friends.#(last="Murphy")#|#         2
 ```

Let's break down a few of these.

The path `friends.#(last="Murphy")#` all by itself results in

 ```json
 [{"first": "Dale", "last": "Murphy", "age": 44},{"first": "Jane", "last": "Murphy", "age": 47}]
 ```

The `.first` suffix will process the `first` path on each array element *before* returning the results. Which becomes

 ```json
 ["Dale","Jane"]
 ```

But the `|first` suffix actually processes the `first` path *after* the previous result.
Since the previous result is an array, not an object, it's not possible to process
because `first` does not exist.

Yet, `|0` suffix returns

 ```json
 {"first": "Dale", "last": "Murphy", "age": 44}
 ```

Because `0` is the first index of the previous result.

### Modifiers

A modifier is a path component that performs custom processing on the JSON.

For example, using the built-in `@reverse` modifier on the above JSON payload will reverse the `children` array:

 ```go
 children.@reverse                   ["Jack","Alex","Sara"]
 children.@reverse.0                 "Jack"
 ```

There are currently the following built-in modifiers:

- `@reverse`: Reverse an array or the members of an object.
- `@ugly`: Remove all whitespace from JSON.
- `@pretty`: Make the JSON more human readable.
- `@this`: Returns the current element. It can be used to retrieve the root element.
- `@valid`: Ensure the json document is valid.
- `@flatten`: Flattens an array.
- `@join`: Joins multiple objects into a single object.
- `@keys`: Returns an array of keys for an object.
- `@values`: Returns an array of values for an object.
- `@tostr`: Converts json to a string. Wraps a json string.
- `@fromstr`: Converts a string from json. Unwraps a json string.
- `@group`: Groups arrays of objects. See [e4fc67c](https://github.com/tidwall/gjson/commit/e4fc67c92aeebf2089fabc7872f010e340d105db).

#### Modifier arguments

A modifier may accept an optional argument. The argument can be a valid JSON payload or just characters.

For example, the `@pretty` modifier takes a json object as its argument.

 ```json
 @pretty:{"sortKeys":true}
 ```

Which makes the json pretty and orders all of its keys.

 ```json
 {
   "age":37,
   "children": ["Sara","Alex","Jack"],
   "fav.movie": "Deer Hunter",
   "friends": [
     {"age": 44, "first": "Dale", "last": "Murphy"},
     {"age": 68, "first": "Roger", "last": "Craig"},
     {"age": 47, "first": "Jane", "last": "Murphy"}
   ],
   "name": {"first": "Tom", "last": "Anderson"}
 }
 ```

*The full list of `@pretty` options are `sortKeys`, `indent`, `prefix`, and `width`.
Please see [Pretty Options](https://github.com/tidwall/pretty#customized-output) for more information.*

### Multipaths

Starting with v1.3.0, GJSON added the ability to join multiple paths together
to form new documents. Wrapping comma-separated paths between `[...]` or
`{...}` will result in a new array or object, respectively.

For example, using the given multipath:

 ```flql
 {name.first,age,"the_murphys":friends.#(last="Murphy")#.first}
 ```

Here we selected the first name, age, and the first name for friends with the
last name "Murphy".

You'll notice that an optional key can be provided, in this case
"the_murphys", to force assign a key to a value. Otherwise, the name of the
actual field will be used, in this case "first". If a name cannot be
determined, then "_" is used.

This results in

 ```json
 {"first":"Tom","age":37,"the_murphys":["Dale","Jane"]}
 ```

### Literals

Starting with v1.12.0, GJSON added support of json literals, which provides a way for constructing static blocks of json. This is can be particularly useful when constructing a new json document using [multipaths](#multipaths).

A json literal begins with the '!' declaration character.

For example, using the given multipath:

 ```json
 {name.first,age,"company":!"Happysoft","employed":!true}
 ```

Here we selected the first name and age. Then add two new fields, "company" and "employed".

This results in

 ```json
 {"first":"Tom","age":37,"company":"Happysoft","employed":true}
 ```

# Credits:
This library would not possible without the following:

**KSQL**
https://crates.io/crates/ksql

**gjson.rs**
https://github.com/tidwall/gjson.rs

Purpose of using the codes from the libraries (instead of using them as a library) is to add
additional functionalities and remove unnecessary functions.