mod cpu;
mod float_type;

use crate::cpu::*;
use crate::float_type::*;

use std::fmt;
use std::mem;

#[derive(Debug, PartialEq, Clone)]
enum FileState {
    Open,
    Closed,
}
#[derive(Debug, Clone)]
struct File {
    name: String,
    data: Vec<u8>,
    state: FileState,
}

impl fmt::Display for FileState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileState::Open => write!(f, "OPEN"),
            FileState::Closed => write!(f, "CLOSED"),
        }
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{} ({})>", self.name, self.state)
    }
}

trait Read {
    fn read(self: &Self, save_to: &mut Vec<u8>) -> Result<usize, String>;
    fn write(self: &mut Self, data: Vec<u8>) -> Result<usize, String>;
}

impl Read for File {
    fn read(&self, save_to: &mut Vec<u8>) -> Result<usize, String> {
        if self.state != FileState::Open {
            return Err(String::from("File must be open for reading"));
        }
        let mut tmp = self.data.clone();
        let read_length = tmp.len();
        save_to.reserve(read_length);
        save_to.append(&mut tmp);
        Ok(read_length)
    }
    fn write(&mut self, data: Vec<u8>) -> Result<usize, String> {
        if self.state != FileState::Open {
            return Err(String::from("File must be open for Writing"));
        }
        self.data = data;
        let length = self.data.len();
        Ok(length)
    }
}

impl File {
    fn new(name: &str) -> Self {
        File {
            name: String::from(name),
            data: Vec::new(),
            state: FileState::Closed,
        }
    }

    fn open(&mut self) -> Result<File, String> {
        self.state = FileState::Open;
        let a = self.clone();
        Ok(a)
    }
    fn close(&mut self) -> Result<File, String> {
        self.state = FileState::Closed;
        let a = self.clone();
        Ok(a)
    }
}

fn receives_closure<F>(closure: F)
where
    F: Fn(i32, i32) -> i32,
{
    let result = closure(1, 2);
    println!("closure(1, 2) => {}", result);
}

fn return_closues() -> impl Fn(i32) -> i32 {
    |x| x + 4
}
fn main() {
    let add = |x, y| x + y;
    receives_closure(add);

    let closure = return_closues();
    println!("{}", closure(34));

    let mut f5 = File::new("5.txt");
    let mut buffer: Vec<u8> = vec![];

    if f5.read(&mut buffer).is_err() {
        println!("Error reading file as file is still closed");
    }
    let f6 = f5.open().unwrap();
    let f6_length = f6.read(&mut buffer).unwrap();
    f5.close().unwrap();
    let text = String::from_utf8_lossy(&buffer);

    println!("{:?}", &f6);
    println!("{} is {} bytes long", &f6.name, f6_length);
    println!("{}", text);
    f5.open().unwrap();
    f5.write(vec![1, 4, 3, 2]).unwrap();
    f5.close().unwrap();
    println!("{}", &f5);

    let a: f32 = 42.42;
    //transmute Reinterprets the bits of a value of one type as another type.
    let frankentype: u32 = unsafe { std::mem::transmute(a) };
    println!("{}", frankentype);
    println!("{:032b}", frankentype);
    let b: f32 = unsafe { std::mem::transmute(frankentype) };
    println!("{}", b);

    let num: u8 = 0b1111_1111;
    println!("{}", num);

    let big_endian: [u8; 4] = [0xAA, 0xBB, 0xCC, 0xDD];
    let little_endian: [u8; 4] = [0xDD, 0xCC, 0xBB, 0xAA];

    let a: i32 = unsafe { mem::transmute(big_endian) };
    let b: i32 = unsafe { mem::transmute(little_endian) };

    println!("a {} b {}", a, b);

    let (signbit, exponent, fraction) = deconstruct_f32(23.908);
    let (sign, exponent, mantissa) = decode_f32_parts(signbit, exponent, fraction);
    let reconstituted_n = f32_from_parts(sign, exponent, mantissa);
    println!(
        "{} -> [sign:{}, exponent:{}, mantissa:{:?}] -> {}",
        23.908, signbit, exponent, mantissa, reconstituted_n
    );

    println!("{:?}", Q7::from(0.1234));
    println!("{:?}", f64::from(Q7::from(0.1234)));
    println!("{:?}", f64::from(Q7::from(127.)));

    println!("{:032b}", (18 as u32) << 10);
    println!("{}", 0b0_01111110_00000000000000000000000);
    println!("{:05b}", (00100 | 01100));

    println!("{}", generate_f32(200));

    let mut cpu = CPU {
        register: [0; 16],
        memory: [0; 4096],
        position_in_memory: 0,
    };
    cpu.register[0] = 5;
    cpu.register[1] = 10;
    cpu.register[2] = 10;
    cpu.register[3] = 10;
    cpu.register[4] = 30;

    //Load the opcode 0x8014 to memory. 0x8014 means “add the value in register 1 to register 0”
    //op_byte1
    cpu.memory[0] = 0x80;
    //op_byte2
    cpu.memory[1] = 0x14;

    //Load the opcode 0x8024 to memory, whuch means “add the value in register 2 to register 0”
    //op_byte1
    cpu.memory[2] = 0x80;
    //op_byte2
    cpu.memory[3] = 0x24;

    //Load the opcode 0x8034 to memory, whuch means “add the value in register 3 to register 0”
    //op_byte1
    cpu.memory[4] = 0x80;
    //op_byte2
    cpu.memory[5] = 0x34;

    //Load the opcode 0x8044 to memory, whuch means “add the value in register 4 to register 0”
    //op_byte1
    cpu.memory[6] = 0x80;
    //op_byte2
    cpu.memory[7] = 0x44;
    cpu.run();
    assert_eq!(cpu.register[0], 65);
    println!("5 + 10 + 10 + 10 + 30 = {}", cpu.register[0]);
}
