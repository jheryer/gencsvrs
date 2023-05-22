use fake::faker::address::raw::*;
use fake::faker::chrono::raw::*;
use fake::faker::lorem::raw::*;
use fake::faker::name::raw::*;
use fake::faker::number::raw::*;
use fake::faker::phone_number::raw::*;
use fake::locales::*;
use fake::{Fake, Faker};
use uuid::Uuid;
//STRING
pub fn fake_string() -> String {
    Faker.fake::<String>()
}
//INT
//Digit
pub fn fake_int() -> String {
    let x = (0..4294967295).fake::<u32>().to_string();
    return x;
}
pub fn fake_digit() -> String {
    Digit(EN).fake()
}
// DECIMAL
pub fn fake_decimal() -> String {
    let x = (0.0..100000.0).fake::<f32>().to_string();
    return x;
}
//DATE
pub fn fake_date() -> String {
    Date(EN).fake()
}
//TIME
pub fn fake_time() -> String {
    Time(EN).fake()
}

//DATE_TIME
pub fn fake_date_time() -> String {
    DateTime(EN).fake()
}
//NAME
pub fn fake_name() -> String {
    Name(EN).fake()
}

//ZIP_CODE
pub fn fake_zipcode() -> String {
    PostCode(EN).fake()
}
//COUNTRY_CODE
pub fn fake_country_code() -> String {
    CountryCode(EN).fake()
}
//STATE_NAME
pub fn fake_state_name() -> String {
    StateName(EN).fake()
}
//STATE_ABBR
pub fn fake_state_abbr() -> String {
    StateAbbr(EN).fake()
}
//LAT
pub fn fake_lat() -> String {
    Latitude(EN).fake()
}
//LON
pub fn fake_lon() -> String {
    Longitude(EN).fake()
}
//PHONE
pub fn fake_phone() -> String {
    CellNumber(EN).fake()
}

//LOREM_WORD
pub fn fake_lorem_word() -> String {
    Word(EN).fake()
}

//LOREM_TITLE
pub fn fake_lorem_title() -> String {
    let words: Vec<String> = Words(EN, 1..4).fake();
    let mut title: String = words.join(" ");
    title.pop(); // return the string without the ending period
    return title;
}

//LOREM_SENTENCE
pub fn fake_lorem_sentence() -> String {
    Sentence(EN, 1..15).fake()
}

//LOREM_PARAGRAPH
pub fn fake_lorem_paragraph() -> String {
    Paragraph(EN, 1..100).fake()
}

//UUID
pub fn fake_uuid() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string()
}

//default
pub fn value_string() -> String {
    String::from("value")
}
pub fn unknown_string() -> String {
    String::from("unknown")
}
