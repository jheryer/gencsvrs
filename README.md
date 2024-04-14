# gencsv CLI
`gencsv` is a command-line tool written in Rust that lets users quickly generate fake CSV data. 
## Table of Contents

- [gencsv CLI](#gencsv-cli)
  - [Table of Contents](#table-of-contents)
  - [Installation](#installation)
    - [Cargo](#cargo)
  - [Usage](#usage)
  - [Example](#example)
    - [Generate 10 rows in csv format write to std out](#generate-10-rows-in-csv-format-write-to-std-out)
    - [Generate 10 rows in parquet format write test.parquet file](#generate-10-rows-in-parquet-format-write-testparquet-file)
    - [Generate two new rows, write to out.parquet, append data from test.parquet, hard delete rows 0,1,2](#generate-two-new-rows-write-to-outparquet-append-data-from-testparquet-hard-delete-rows-012)
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
  -s, --schema <SCHEMA>                Data Schema "col:STRING, col2:INT, col3:TIME"
  -f, --file-target <FILE_TARGET>      Output file name (rquired for parquet file output)
  -r, --rows <ROWS>                    Generate number of rows [default: 10]
  -c, --csv                            CSV output
  -p, --parquet                        Parquet output
  -a, --append-target <APPEND_TARGET>  Parquet append target
  -h, --help                           Print help
  -V, --version                        Print version
```

## Example

### Generate 10 rows in csv format write to std out
```
$ gencsv -s "id:INT_INC,name:NAME,phone:PHONE,zip:ZIP_CODE,date:DATE,guid:UUID,range:INT_RNG:(-15-23)" -r 10 -c

id,name,phone,zip,date,guid,range
0,Horacio Lueilwitz,915.671.3404,56824-2040,1122-07-08,dba48ed4-6ff0-45b6-9b22-19ff186bb62a,-15
1,Samson Hilll,(212) 549-5505,49705,2365-03-26,44723f63-3a56-4e57-8e39-576e1497fbd6,-14
2,Nettie Will,628-206-5589,21869-9397,2658-02-17,58261ccd-a0b6-4dea-97c7-8a27e502abbc,-13
3,Erna Boyer,999-245-2157,04756,0375-04-27,99711069-f918-4e92-be07-ad54d4e97c7a,-12
4,Krystel O'Conner,210-398-0062,72381-0103,0416-02-05,9f63d3de-9470-4a4b-adc9-f3a06397549b,-11
5,Prince Brakus,415.468.0038,62410,2237-06-03,3d1c059d-77d1-4eef-8f2d-0546f8f7abeb,-10
6,Geovany Corwin,459.414.3759,75505,2789-10-23,9f3b4f86-f557-4394-9fb4-b4e068e8faf8,-9
7,Cecilia Prosacco,(798) 403-3534,31349-6648,2018-05-31,cf054cf0-473a-4e9b-be02-710d76c85daf,-8
8,Pietro Veum,826-794-2448,73861,1804-07-08,906ffe37-c7ae-4cf7-a446-da6d0e32f9d0,-7
9,Mateo Nikolaus,(594) 138-8262,43276,1576-11-07,d98c9514-7d07-42d3-8092-b2af2663d35d,-6
```

### Generate 10 rows in parquet format write test.parquet file
```
$ gencsv -s 'id:INT_INC,name:NAME,phone:PHONE,zip:ZIP_CODE,date:DATE,price:PRICE,gid:UUID' -r 10 -p -f test.parquet
```

### Generate two new rows, write to out.parquet, append data from test.parquet, hard delete rows 0,1,2
```
$ gencsv -s 'id:INT_RNG:(10-19),name:NAME,phone:PHONE,zip:ZIP_CODE,date:DATE,price:PRICE,gid:UUID' -r 2 -p -f out.parquet -a test.parquet -d 0-2
```

## Available Data Types

* STRING
* INT
* INT_INC
* INT_RNG
* DIGIT
* DECIMAL
* PRICE (0.00 - 9999.00)
* DATE
* TIME
* DATE_TIME
* NAME
* FIRST_NAME
* LAST_NAME
* SSN
* ZIP_CODE
* COUNTRY_CODE
* STATE_NAME
* STATE_ABBR
* LAT
* LON
* PHONE
* LOREM_WORD
* LOREM_TITLE
* LOREM_SENTENCE
* LOREM_PARAGRAPH
* UUID

