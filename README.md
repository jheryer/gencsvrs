# gencsv CLI
`gencsv` is a command-line tool written in Rust that lets users quickly generate fake CSV data. 
## Table of Contents

- [gencsv CLI](#gencsv-cli)
  - [Table of Contents](#table-of-contents)
  - [Installation](#installation)
    - [Cargo](#cargo)
  - [Usage](#usage)
    - [example](#example)
  - [Available Data Types](#available-data-types)



## Installation


### Cargo

To install `gencsv`, ensure you have [Rust and Cargo installed](https://www.rust-lang.org/tools/install) on your system. Then, follow these steps:

1. Clone the repository:

```sh
git clone git@github.com:jheryer/gencsvrs.git
```

2. Change to the  directory:
```sh
cd gencsvrs
```
3. Build and Install
```sh
cargo build --release
cargo install --path .
```

## Usage
```
 $ gencsv -h

 Usage: gencsv [OPTIONS]

Options:
  -s, --schema <SCHEMA>        Data Schema "col:STRING, col2:INT, col3:TIME"
  -r, --rows <ROWS>            Generate number of rows [default: 10]
  -d, --delimiter <DELIMITER>  csv delimiter character [default: ,]
  -n, --no-header              include headers
  -h, --help                   Print help
  -V, --version                Print version
```

### example
```
$ gencsv -s "name:NAME,phone:PHONE,zip:ZIP_CODE,date:DATE,id:UUID" -r 10

name,phone,zip,date,id
Kris Crona,793-343-0082,49547,2558-10-04,a137a194-c713-4187-ab1f-493dfc9f3249
Syble Lynch,234-189-3986,08928-1043,2881-09-27,e3d48e03-ce3c-4dcf-849e-733ec6155fdc
Earnestine Sauer,1-382-803-0639,61345-4826,1442-08-06,7f1c8760-fd76-4772-8a5f-17e677aa75ff
Anais White,1-580-859-5676,71317,2917-04-08,e4ef8d78-c207-4507-b1d6-5dcbb2393581
Isom Mohr,686-159-2491,45045,1122-05-06,d3477fa1-1783-4dee-87a3-416a1db7278e
Maggie Gislason,(547) 638-0124,29008-5133,0124-05-06,9079d928-5854-4ec7-a3e8-802e76bb0e6a
Alda Baumbach,(711) 241-4567,54712,1005-01-30,44df682c-a718-46ef-b350-ad00cf74decd
Marie Pacocha,1-355-314-1594,50297,2441-03-05,2fdc523e-84bb-40b5-969e-21fa5cf33181
Shanel Sauer,296.675.5884,22878-7098,1940-06-05,7ac974e2-e3fe-4ec2-ba46-7f8a9928900a
Norris Kulas,1-212-742-0074,61902,0174-07-29,db5c5cb7-92ba-4570-9ab9-79ea83146379
```

## Available Data Types

* STRING
* INT
* DIGIT
* DECIMAL
* DATE
* TIME
* DATE_TIME
* NAME
* ZIP_CODE
* COUNTRY_CODE
* LAT
* LON
* PHONE
* LOREM_WORD
* LOREM_SENTENCE
* LOREM_PARAGRAPH
* UUID