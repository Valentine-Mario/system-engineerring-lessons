// The Base64 encoding process is to:

// Divid the input bytes stream into blocks of 3 bytes.

// Divid 24 bits of each 3-byte block into 4 groups of 6 bits.

// Map each group of 6 bits to 1 printable character, based on the 6-bit value using the Base64 character set map.

// If the last 3-byte block has only 1 byte of input data, pad 2 bytes of zero (\x0000). After encoding it as a normal block, override the last 2 characters with 2 equal signs (==), so the decoding process knows 2 bytes of zero were padded.

// If the last 3-byte block has only 2 bytes of input data, pad 1 byte of zero (\x00). After encoding it as a normal block, override the last 1 character with 1 equal signs (=), so the decoding process knows 1 byte of zero was padded.

// Carriage return (\r) and new line (\n) are inserted into the output character stream. They will be ignored by the decoding process.

//eg
// Input Data          A
// Input Bits   01000001
// Padding      01000001 00000000 00000000
//                    \      \      \
// Bit Groups   010000 010000 000000 000000
// Mapping           Q      Q      A      A
// Overriding        Q      Q      =      =

//eg
// Input Data          A        B
// Input Bits   01000001 01000010
// Padding      01000001 01000010 00000000
//                    \      \      \
// Bit Groups   010000 010100 001000 000000
// Mapping           Q      U      I      A
// Overriding        Q      U      I      =

//eg
// Input Data          A        B        C
// Input Bits   01000001 01000010 01000011
//                    \      \      \
// Bit Groups   010000 010100 001001 000011
// Mapping           Q      U      J      D

// There are basically three operations that our alphabet should be able to perform;
// Going from an index to a character, going from a character back to the original 6-bit index
// and getting the character used for padding.
use std::iter::FromIterator;

pub trait Alphabet {
    fn get_char_for_index(&self, index: u8) -> Option<char>;
    fn get_index_for_char(&self, character: char) -> Option<u8>;
    fn get_padding_char(&self) -> char;
}

pub struct Classic;

const UPPERCASEOFFSET: i8 = 65;
const LOWERCASEOFFSET: i8 = 71;
const DIGITOFFSET: i8 = -4;

impl Alphabet for Classic {
    fn get_char_for_index(&self, index: u8) -> Option<char> {
        let index = index as i8;

        let ascii_index = match index {
            0..=25 => index + UPPERCASEOFFSET,  //A-Z
            26..=51 => index + LOWERCASEOFFSET, //a-z
            52..=61 => index + DIGITOFFSET,     //0-9
            62 => 43,                           //+
            63 => 47,                           // /
            _ => return None,
        } as u8;
        Some(ascii_index as char)
    }

    fn get_index_for_char(&self, character: char) -> Option<u8> {
        let character = character as i8;

        let base64_index = match character {
            65..=90 => character - UPPERCASEOFFSET,  // A-Z
            97..=122 => character - LOWERCASEOFFSET, // a-z
            48..=57 => character - DIGITOFFSET,      // 0-9
            43 => 62,                                // +
            47 => 63,                                // /

            _ => return None,
        } as u8;
        Some(base64_index)
    }
    fn get_padding_char(&self) -> char {
        '='
    }
}

// Divid the input bytes stream into blocks of 3 bytes (24 bits)
// It converts the input of up-to 3 bytes into an output of up-to 4 bytes.
// Essentially converting the 8-bit unsigned integers into 6-bit.
fn split(chunk: &[u8]) -> Vec<u8> {

    match chunk.len() {
        1 => vec![&chunk[0] >> 2, (&chunk[0] & 0b00000011) << 4],
        2 => vec![
            &chunk[0] >> 2,
            (&chunk[0] & 0b00000011) << 4 | &chunk[1] >> 4,
            (&chunk[1] & 0b00001111) << 2,
        ],
        3 => vec![
            &chunk[0] >> 2,
            (&chunk[0] & 0b00000011) << 4 | &chunk[1] >> 4,
            (&chunk[1] & 0b00001111) << 2 | &chunk[2] >> 6,
            &chunk[2] & 0b00111111,
        ],
        _ => unreachable!(),
    }
}

// slice the input data into 3-byte chunks and run them through our split function. Once they're split,
//  we can convert each chunk by looking up the 6-bit number in our alphabet
fn encode_using_alphabet<T: Alphabet>(alphabet: &T, data: &[u8]) -> String {
    let encoded = data
        .chunks(3)
        .map(split)
        .flat_map(|chunk| encode_chunk(alphabet, chunk));

    String::from_iter(encoded)
}

fn encode_chunk<T: Alphabet>(alphabet: &T, chunk: Vec<u8>) -> Vec<char> {
    //pre filling the buffer with 4 padding characters
    let mut out = vec![alphabet.get_padding_char(); 4];
    //iterate over the chunk
    for i in 0..chunk.len() {
        //map index to char an d replace if exist
        if let Some(chr) = alphabet.get_char_for_index(chunk[i]) {
            out[i] = chr;
        }
    }
    out
}

pub fn encode(data: &[u8]) -> String {
    let classic_alphabet = &Classic {};
    encode_using_alphabet(classic_alphabet, data)
}


//decoding
pub fn decode_using_alphabet<T:Alphabet>(alphabet:T, data:&String)->Result<Vec<u8>, std::io::Error>{
    // if data is not multiple of four bytes, data is invalid
    if data.chars().count() % 4 != 0 {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput))
    }

    // we split the string into its chars and slice it in chunks of 4 char's.
    // Each slice is fed through the original function that will fetch the original 
    // char from the alphabet which is flat_map'ed through the stitch function
    let result = data
        .chars()
        .collect::<Vec<char>>()
        .chunks(4)
        .map(|chunk| original(&alphabet, chunk) )
        .flat_map(stitch)
        .collect();

    Ok(result)
}

fn original<T: Alphabet>(alphabet: &T, chunk: &[char]) -> Vec<u8> {
    //It filters the padding characters and uses the looks up the left-over characters in our alphabet
    chunk
        .iter()
        .filter(|character| *character != &alphabet.get_padding_char())
        .map(|character| { 
            alphabet
                .get_index_for_char(*character)
                .expect("unable to find character in alphabet")
        })
        .collect()
}

//It takes a Vec of bytes and returns another Vec of bytes, containing a maximum of three 8-bit numbers.
fn stitch(bytes: Vec<u8>) -> Vec<u8> {
    let out = match bytes.len() {
        2 => vec![
            (bytes[0] & 0b00111111) << 2 | bytes[1] >> 4,
            (bytes[1] & 0b00001111) << 4,
        ],

        3 => vec![
            (bytes[0] & 0b00111111) << 2 | bytes[1] >> 4,
            (bytes[1] & 0b00001111) << 4 | bytes[2] >> 2,
            (bytes[2] & 0b00000011) << 6,
        ],

        4 => vec![
            (bytes[0] & 0b00111111) << 2 | bytes[1] >> 4,
            (bytes[1] & 0b00001111) << 4 | bytes[2] >> 2,
            (bytes[2] & 0b00000011) << 6 | bytes[3] & 0b00111111,
        ],

        _ => unreachable!()
    };

    out.into_iter().filter(|&x| x > 0).collect()
}

pub fn decode(bytes: &String) -> Result<Vec<u8>, std::io::Error> {
    let alphabet = Classic {};
    decode_using_alphabet(alphabet, bytes)
}