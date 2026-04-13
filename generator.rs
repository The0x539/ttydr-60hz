use elf::{ElfBytes, endian::LittleEndian, symbol::Symbol};
use indexmap::IndexMap;
use std::{collections::HashMap, ffi::CStr, fmt::Write as _, io::Write, ops::RangeInclusive};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("no input file specified");
    let version = args.next().expect("no game version specified");

    let file_data = std::fs::read(path).unwrap();
    let file = ElfBytes::<LittleEndian>::minimal_parse(&file_data).unwrap();
    let patches = PatchFile::parse(&file);
    let text = build_text_file(&patches, &version);
    std::io::stdout().write_all(text.as_ref()).unwrap();

    eprintln!("{}", env!("CARGO_PKG_VERSION_PRE"));
}

#[derive(Debug, Clone)]
struct PatchFile<'a> {
    versions: IndexMap<&'a str, &'a str>,
    groups: Vec<PatchGroup<'a>>,
}

impl<'a> PatchFile<'a> {
    fn parse(file: &ElfBytes<'a, LittleEndian>) -> Self {
        let symbols = extract_symbols(file);
        let text_header = file.section_header_by_name(".text").unwrap().unwrap();
        let text = file.section_data(&text_header).unwrap().0;

        let versions = extract_strings(&symbols["versions"], text)
            .chunks_exact(2)
            .map(|c| (c[0], c[1]))
            .collect();

        let group_names = extract_strings(&symbols["groups"], text);

        let mut groups = Vec::new();
        for name in group_names {
            let offset = symbols[name].st_value as usize;
            let group = PatchGroup::from_bytes(name, &text[offset..]);
            groups.push(group);
        }

        Self { versions, groups }
    }
}

fn extract_symbols<'a>(file: &ElfBytes<'a, LittleEndian>) -> HashMap<&'a str, Symbol> {
    let (symtab, strings) = file.symbol_table().unwrap().unwrap();
    let mut m = HashMap::new();
    for sym in symtab.iter() {
        let name = strings.get(sym.st_name as usize).unwrap();
        m.insert(name, sym);
    }
    m
}

fn extract_strings<'a>(symbol: &Symbol, section: &'a [u8]) -> Vec<&'a str> {
    let mut data = &section[symbol.st_value as usize..];
    let mut v = Vec::new();
    while data[0] != 0 {
        let name = CStr::from_bytes_until_nul(data).unwrap().to_str().unwrap();
        v.push(name);
        data = &data[name.len() + 1..];
    }
    v
}

fn build_text_file(patch_file: &PatchFile, version: &str) -> String {
    let (version_index, _, &build_id) = patch_file
        .versions
        .get_full(version)
        .expect("unrecognized game version");

    let code_path = format!("exefs/main-{version}");
    let game_code = std::fs::read(code_path).ok().map(|nso| get_nso_text(&nso));

    let mut buf = String::new();
    _ = writeln!(buf, "@nsobid-{}", build_id.to_ascii_uppercase());
    _ = writeln!(buf, "@flag print_values");
    _ = writeln!(buf, "@flag offset_shift 0x100");
    _ = writeln!(buf);

    for group in &patch_file.groups {
        let group_text = group.to_text(version_index, game_code.as_deref());
        _ = writeln!(buf, "{group_text}");
    }

    if game_code.is_some() {
        eprintln!("✔️ All patches validated against game code");
    }

    buf
}

#[derive(Debug, Clone)]
struct PatchGroup<'a> {
    name: &'a str,
    func_offsets: [i32; 2],
    patches: Vec<Patch>,
}

impl<'a> PatchGroup<'a> {
    fn from_bytes(name: &'a str, mut data: &[u8]) -> Self {
        Self {
            name,
            func_offsets: std::array::from_fn(|_| read_i32(&mut data)),
            patches: std::iter::from_fn(|| Patch::from_bytes(&mut data)).collect(),
        }
    }

    fn to_text(&self, version_index: usize, game_code: Option<&[u8]>) -> String {
        let mut buf = String::new();
        _ = writeln!(buf, "[{}]", self.name);
        _ = writeln!(buf, "@enabled");
        let func_offset = self.func_offsets[version_index];
        for patch in &self.patches {
            let line = patch.to_text(func_offset, game_code);
            _ = writeln!(buf, "{line}");
        }
        buf
    }
}

#[derive(Debug, Copy, Clone)]
struct Patch {
    instruction_offset: i32,
    old_code: [u8; 4],
    new_code: [u8; 4],
}

impl Patch {
    fn from_bytes(data: &mut &[u8]) -> Option<Self> {
        let offset = read_i32(data);
        if offset == -1 {
            return None;
        }

        let (&old_code, rest) = data.split_first_chunk().unwrap();
        let (&new_code, rest) = rest.split_first_chunk().unwrap();
        *data = rest;
        Some(Patch {
            instruction_offset: offset,
            old_code,
            new_code,
        })
    }

    fn to_text(&self, func_offset: i32, game_code: Option<&[u8]>) -> String {
        let mut byte_offset = (func_offset + self.instruction_offset) as usize;
        assert_eq!(func_offset % 4, 0);
        assert_eq!(byte_offset % 4, 0);

        if let Some(game_code) = &game_code {
            assert_eq!(
                self.old_code,
                game_code[byte_offset as usize..][..4],
                "Patch's old code does not match game code at offset {:#x}+{:x}",
                func_offset,
                self.instruction_offset,
            );
        }

        let range = self.trim_payload();
        byte_offset += *range.start();
        assert!(!range.is_empty(), "Patch has no changed bytes");

        let mut buf = format!("{byte_offset:06x} ");
        for _ in 0..*range.start() {
            buf += "  "
        }
        for byte in &self.new_code[range.clone()] {
            _ = write!(buf, "{byte:02x}");
        }

        if let [Some(old_asm), Some(new_asm)] = [self.old_code, self.new_code].map(disassemble) {
            for _ in *range.end()..3 {
                buf += "  ";
            }
            _ = write!(buf, " // {old_asm} -> {new_asm}",);
        }

        buf
    }

    fn trim_payload(&self) -> RangeInclusive<usize> {
        let mut i = 0;
        while i < 4 && self.new_code[i] == self.old_code[i] {
            i += 1;
        }
        let mut j = 3;
        while j > i && self.new_code[j] == self.old_code[j] {
            j -= 1;
        }
        i..=j
    }
}

fn get_nso_text(nso: &[u8]) -> Vec<u8> {
    let text_offset = read_i32(&mut &nso[0x10..0x14]) as usize;
    let text_size = read_i32(&mut &nso[0x18..0x1c]) as usize;
    let text_zsize = read_i32(&mut &nso[0x60..0x64]) as usize;

    let compressed_text = &nso[text_offset..][..text_zsize];
    lz4_flex::block::decompress(compressed_text, text_size)
        .expect("Game code is present, but decompression failed")
}

fn read_i32(data: &mut &[u8]) -> i32 {
    let chunk = data.split_off(..4).unwrap();
    i32::from_le_bytes(chunk.try_into().unwrap())
}

fn disassemble(instr: [u8; 4]) -> Option<String> {
    use capstone::prelude::*;
    use std::rc::Rc;

    let cs = {
        thread_local! {
            static CS: Rc<Capstone> = Capstone::new()
                .arm64()
                .mode(arch::arm64::ArchMode::Arm)
                .build()
                .unwrap()
                .into();
        };
        CS.with(Rc::clone)
    };

    let instr = &cs.disasm_all(&instr, 0).unwrap()[0];

    let mnemonic = instr.mnemonic()?;

    let Some(mut op) = instr.op_str() else {
        return Some(mnemonic.to_owned());
    };

    if mnemonic == "fmov" && op.contains("#") {
        while op.ends_with("0") && !op.ends_with(".0") {
            op = op.strip_suffix("0").unwrap();
        }
    }

    if mnemonic == "mov" {
        if let Some(new_op) = get_float_operand(op) {
            return Some(format!("{mnemonic} {new_op}"));
        }
    }

    Some(format!("{mnemonic} {op}"))
}

fn get_float_operand(op_str: &str) -> Option<String> {
    op_str.strip_prefix("w")?;
    let (prefix, imm_str) = op_str.rsplit_once(", #0x")?;
    let imm_int = u32::from_str_radix(imm_str, 0x10).unwrap();
    let imm_float = f32::from_bits(imm_int);

    if !imm_float.is_normal() {
        return None;
    }

    let mut s = format!("{prefix}, #{imm_float}");
    if imm_float.fract() == 0.0 {
        s += ".0";
    }
    Some(s)
}
