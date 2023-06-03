use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

fn generate_static_atoms() {
    let static_atoms_file_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("static_atoms.txt");
    let generated_file_path = Path::new(&env::var("OUT_DIR").unwrap()).join("atoms.rs");

    println!("cargo:rerun-if-changed={}", static_atoms_file_path.display());

    let mut atom_ty = string_cache_codegen::AtomType::new("atoms::Atom", "atom!");
    for line in BufReader::new(File::open(&static_atoms_file_path).unwrap()).lines() {
        let name = line.unwrap();
        let name = name.trim();
        if !name.is_empty() {
            atom_ty.atom(name);
        }
        //atom_ty.atom(&local_name.to_ascii_lowercase());
    }
    atom_ty
        .with_macro_doc("Takes a name as a string and returns its key in the string cache.")
        .write_to_file(&generated_file_path)
        .unwrap();
}

fn main() {
    generate_static_atoms()
}
