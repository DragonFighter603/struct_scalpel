#![feature(ptr_internals)]
#![feature(allocator_api)]
#![feature(auto_traits)]
#![feature(negative_impls)]

extern crate self as struct_scalpel;

use std::sync::Once;

pub mod impls;

pub use struct_scalpel_proc_macro::Dissectible;

pub auto trait Unsized {}

impl<T> !Unsized for T where T: Sized {}

pub struct FieldInfo {
    pub type_name: &'static str,
    pub size: usize,
    pub align: usize,
    pub offset: usize
}

impl FieldInfo {
    pub fn from_val_and_base<T>(base: usize, v: &T) -> Self {
        Self {
            type_name: std::any::type_name::<T>(), 
            size: std::mem::size_of::<T>(), 
            align: std::mem::align_of::<T>(), 
            offset: v as *const _ as usize - base
        }
    }
}

pub enum StructFields {
    Named(Vec<(&'static str, FieldInfo)>),
    Tuple(Vec<FieldInfo>),
    Unit,
}

pub enum Structure {
    Struct(StructFields),
    Enum(Vec<(&'static str, StructFields)>),
    Union(Vec<(&'static str, FieldInfo)>),
    Tuple(Vec<FieldInfo>),
    Reference(bool)
}

pub struct LayoutInfo {
    pub attrs: Vec<&'static str>,
    pub name: &'static str,
    pub size: usize,
    pub align: usize,
    pub structure: Structure
}

pub trait Dissectible {
    fn field_info() -> LayoutInfo;
}

static ANSI_INIT: Once = Once::new();

pub fn print_dissection_info<T: Dissectible>() {
    ANSI_INIT.call_once(|| {
        enable_ansi_support::enable_ansi_support().expect("ansi not supported!");
    });

    let info = T::field_info();

    for attr in info.attrs {
        println!("{attr}")
    }
    println!("\x1b[38;5;245m/* size={:2}, align={:2} */\x1b[0m", info.size, info.align);

    match info.structure {
        Structure::Struct(fields) => {
            print!("struct \x1b[1;31m{}\x1b[0m ", info.name);
            match fields {
                StructFields::Named(fields) => {
                    if fields.is_empty() { println!("{{ }}"); return }
                    println!("{{");
                    let n: u8 = 255 / fields.len() as u8;
                    for (f, (fname, field)) in fields.iter().enumerate() {
                        let (r, g, b) = hsv_to_rgb(f as u8 * n, 200, 255);
                        print!("    ");
                        print_named_field(r, g, b, fname, field);
                        println!();
                    }
                    println!("}}");
                    if info.size == 0 { return }
                    'outer: for i in 0..info.size {
                        for (f, (_fname, field)) in fields.iter().enumerate() {
                            if i >= field.offset && i < field.offset + field.size {
                                let (r, g, b) = hsv_to_rgb(f as u8 * n, 200, 255);
                                print!("\x1b[48;2;{r};{g};{b}m.");
                                continue 'outer;
                            }
                        }
                        print!("\x1b[48;2;100;100;100m.");
                    }
                    println!("\x1b[0m");
                },
                StructFields::Tuple(fields) => {
                    if fields.is_empty() { println!("();"); return }
                    if fields.len() == 1 { print!("("); }
                    else { println!("("); }
                    let n: u8 = 255 / fields.len() as u8;
                    for (f,  field) in fields.iter().enumerate() {
                        let (r, g, b) = hsv_to_rgb(f as u8 * n, 200, 255);
                        if fields.len() > 1 { print!("    "); }
                        print_field(r, g, b, field);
                        if fields.len() > 1 { println!(); }
                    }
                    println!(");");
                    if info.size == 0 { return }
                    'outer: for i in 0..info.size {
                        for (f,  field) in fields.iter().enumerate() {
                            if i >= field.offset && i < field.offset + field.size {
                                let (r, g, b) = hsv_to_rgb(f as u8 * n, 200, 255);
                                print!("\x1b[48;2;{r};{g};{b}m.");
                                continue 'outer;
                            }
                        }
                        print!("\x1b[48;2;100;100;100m.");
                    }
                    println!("\x1b[0m");
                },
                StructFields::Unit => println!(";"),
            }
        },
        Structure::Enum(variants) => {
            print!("enum \x1b[1;31m{}\x1b[0m {{", info.name);
            if variants.is_empty() { println!(" }}"); return }
            println!();
            for (name, fields) in &variants {
                print!("    {name}");
                match fields {
                    StructFields::Named(fields) => {
                        if fields.is_empty() { println!(" {{ }},"); return }
                        println!(" {{");
                        let n: u8 = 255 / fields.len() as u8;
                        for (f, (ident, field)) in fields.iter().enumerate() {
                            let (r, g, b) = hsv_to_rgb(f as u8 * n, 200, 255);
                            print!("        ");
                            print_named_field(r, g, b, ident, field);
                            println!();
                        }
                        println!("    }},");
                    },
                    StructFields::Tuple(fields) => {
                        if fields.is_empty() { println!("(),"); return }
                        if fields.len() == 1 { print!("("); }
                        else { println!("("); }
                        let n: u8 = 255 / fields.len() as u8;
                        for (f, field) in fields.iter().enumerate() {
                            let (r, g, b) = hsv_to_rgb(f as u8 * n, 200, 255);
                            if fields.len() > 1 { print!("        "); }
                            print_field(r, g, b, field);
                            if fields.len() > 1 { println!(); }
                        }
                        if fields.len() == 1 { println!("),"); }
                        else { println!("    ),"); }
                    },
                    StructFields::Unit => println!(","),
                }
            }
            println!("}}");
            if info.size == 0 { return }
            for (_, fields) in &variants {
                match fields {
                    StructFields::Named(fields) => {
                        if fields.is_empty() { println!(); return }
                        let n: u8 = 255 / fields.len() as u8;
                        'outer: for i in 0..info.size {
                            for (f,  (_, field)) in fields.iter().enumerate() {
                                if i >= field.offset && i < field.offset + field.size {
                                    let (r, g, b) = hsv_to_rgb(f as u8 * n, 200, 255);
                                    print!("\x1b[48;2;{r};{g};{b}m.");
                                    continue 'outer;
                                }
                            }
                            print!("\x1b[48;2;100;100;100m.");
                        }
                        print!("\x1b[0m");
                        println!()
                    },
                    StructFields::Tuple(fields) => {
                        if fields.is_empty() { println!(); return }
                        let n: u8 = 255 / fields.len() as u8;
                        'outer: for i in 0..info.size {
                            for (f,  field) in fields.iter().enumerate() {
                                if i >= field.offset && i < field.offset + field.size {
                                    let (r, g, b) = hsv_to_rgb(f as u8 * n, 200, 255);
                                    print!("\x1b[48;2;{r};{g};{b}m.");
                                    continue 'outer;
                                }
                            }
                            print!("\x1b[48;2;100;100;100m.");
                        }
                        println!("\x1b[0m");
                    },
                    StructFields::Unit => {
                        for _ in 0..info.size {
                            print!("\x1b[48;2;100;100;100m.");
                        }
                        println!("\x1b[0m");
                    },
                }
            }
        },
        Structure::Union(fields) => {
            print!("union \x1b[1;31m{}\x1b[0m ", info.name);
            if fields.is_empty() { println!("{{ }}"); return }
            println!("{{");
            let n: u8 = 255 / fields.len() as u8;
            for (f, (fname, field)) in fields.iter().enumerate() {
                let (r, g, b) = hsv_to_rgb(f as u8 * n, 200, 255);
                print!("    ");
                print_named_field(r, g, b, fname, field);
                println!();
            }
            println!("}}");
            if info.size == 0 { return }
            for (f, (_fname, field)) in fields.iter().enumerate() {
                for i in 0..info.size {
                    if i < field.size {
                        let (r, g, b) = hsv_to_rgb(f as u8 * n, 200, 255);
                        print!("\x1b[48;2;{r};{g};{b}m.");
                    } else {
                        print!("\x1b[48;2;100;100;100m.");
                    }
                }
                println!("\x1b[0m");
            }
        },
        Structure::Reference(mutable) => {
            if mutable {
                print!("\x1b[1;31m&mut\x1b[0m {}", info.name)
            } else {
                print!("\x1b[1;31m&\x1b[0m{}", info.name)
            }
            if info.size == 8 {
                println!()
            } else {
                println!(" \x1b[48;2;100;100;160m<+unsized data>\x1b[0m");
            }
            for i in 0..info.size {
                if i < 8 {
                    let (r, g, b) = hsv_to_rgb(0, 200, 255);
                    print!("\x1b[48;2;{r};{g};{b}m.");
                } else {
                    print!("\x1b[48;2;100;100;160m.");
                }
            }
            println!("\x1b[0m");
        },
        Structure::Tuple(fields) => {
            if fields.is_empty() { println!("()"); return }
            if fields.len() == 1 { print!("("); }
            else { println!("("); }
            let n: u8 = 255 / fields.len() as u8;
            for (f,  field) in fields.iter().enumerate() {
                let (r, g, b) = hsv_to_rgb(f as u8 * n, 200, 255);
                if fields.len() > 1 { print!("    "); }
                print_field(r, g, b, field);
                if fields.len() > 1 { println!(); }
            }
            println!(")");
            if info.size == 0 { return }
            'outer: for i in 0..info.size {
                for (f,  field) in fields.iter().enumerate() {
                    if i >= field.offset && i < field.offset + field.size {
                        let (r, g, b) = hsv_to_rgb(f as u8 * n, 200, 255);
                        print!("\x1b[48;2;{r};{g};{b}m.");
                        continue 'outer;
                    }
                }
                print!("\x1b[48;2;100;100;100m.");
            }
            println!("\x1b[0m");
        },
    }
}

fn print_field(r: u8, g: u8, b:u8, field: &FieldInfo) {
    print!("\x1b[48;2;{r};{g};{b}m{}\x1b[0m,\t\x1b[38;5;245m/* offset={:2}, size={:2}, align={:2} */\x1b[0m", field.type_name, field.offset, field.size, field.align);
}

fn print_named_field(r: u8, g: u8, b:u8, name: &str, field: &FieldInfo) {
    print!("{}: \x1b[48;2;{r};{g};{b}m{}\x1b[0m,\t\x1b[38;5;245m/* offset={:2}, size={:2}, align={:2} */\x1b[0m", name, field.type_name, field.offset, field.size, field.align);
}

fn hsv_to_rgb(h: u8, s: u8, v: u8) -> (u8, u8, u8) {
    let h = h as u32;
    let s = s as u32;
    let v = v as u32;

    if s == 0 {
        return (v as u8, v as u8, v as u8);
    }
    
    let region = h / 43;
    let remainder = (h - (region * 43)) * 6; 
    
    let p = (v * (255 - s)) >> 8;
    let q = (v * (255 - ((s * remainder) >> 8))) >> 8;
    let t = (v * (255 - ((s * (255 - remainder)) >> 8))) >> 8;
    
    match region {
        0 => (v as u8, t as u8, p as u8),
        1 => (q as u8, v as u8, p as u8),
        2 => (p as u8, v as u8, t as u8),
        3 => (p as u8, q as u8, v as u8),
        4 => (t as u8, p as u8, v as u8),
        _ => (v as u8, p as u8, q as u8),
    }  
}

/// Safety:
/// The purpose of this function is to create a nonzero dummy value.
/// (nonzero to satisfy Option<T> where T != 0 or similar null pointer optimizations)
/// Obviously, values created by this function are most 
/// likely not usable and will cause crashes if attempted to be used.
/// Zira will complain.
/// However, values created here are never intended to behave at all. 
/// It is required that they are not dropped, instead use `std::mem::forget`,
/// since destructors/dropping also run code.
/// The only purpose of these dummys is to provide pointer offsets for the macro.
pub unsafe fn dummy_nonzero<T>() -> T {
    let mut dummy = std::mem::MaybeUninit::zeroed();
    let ptr = &mut dummy as *mut _ as *mut u8;
    for i in 0..std::mem::size_of::<T>() {
        *ptr.add(i) = 0xFF;
    }
    dummy.assume_init()
}
