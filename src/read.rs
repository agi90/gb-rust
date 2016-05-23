use std::fs::File;
use std::io::Read;

pub mod gb_proc;

use self::gb_proc::opcodes::OpCode;

pub fn main() {
    println!("YEE");
    let mut f = File::open("rom.gb").unwrap();
    let mut s = vec![];

    f.read_to_end(&mut s);

    let mut i = 0x100;
    while i < 0x5400 {
        print!("[{:x}] ", i);
        let op = if s[i] == 0xCB {
            i += 1;
            OpCode::from_byte(s[i], true)
        } else if s[i] == 0xE3 || s[i] == 0xE4 || s[i] == 0xF4 || s[i] == 0xFD ||
            s[i] == 0xEC || s[i] == 0xDD || s[i] == 0xDB || s[i] == 0xFC || s[i] == 0xED ||
            s[i] == 0xD3 || s[i] == 0xEB {
            OpCode::Nop
        } else {
            OpCode::from_byte(s[i], false)
        };

        i += 1;

        print!("{}", op.to_string());

        match op {
            OpCode::LdBn | OpCode::LdCn | OpCode::LdDn | OpCode::LdEn | OpCode::LdHn | OpCode::LdLn |
            OpCode::LdHLn | OpCode::LdAx | OpCode::LdhAn | OpCode::AddAx | OpCode::AdcAx |
            OpCode::SubAx | OpCode::AndAx | OpCode::OrAx | OpCode::XorAx | OpCode::CpAx |
            OpCode::LdhnA | OpCode::LdhlSPn | OpCode::AddSPx | OpCode::Jrn | OpCode::JrNZn |
            OpCode::JrZn | OpCode::JrNCn | OpCode::JrCn
            => {
                println!(" 0x{:x}", s[i]);
                i += 1;
            },
            OpCode::LdAnn | OpCode::LdBCnn | OpCode::LdDEnn | OpCode::LdHLnn |
            OpCode::LdSPnn | OpCode::LdnnSP | OpCode::Jpnn | OpCode::Callnn | OpCode::CallNZnn |
            OpCode::CallZnn | OpCode::CallNCnn | OpCode::CallCnn | OpCode::LdnnA | OpCode::JpCnn |
            OpCode::JpNCnn | OpCode::JpZnn | OpCode::JpNZnn
            => {
                println!(" ${:02X}{:02X}", s[i], s[i+1]);
                i += 2;
            },
            _ => { println!("") }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
