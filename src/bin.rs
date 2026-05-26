use fasb::fasb::start_fasb;

pub fn main() {
    let mut input = std::env::args().skip(1);
    let arg = match input.next() {
        Some(s) => s,
        _ => {
            println!("error: expected input logic program");
            std::process::exit(-1)
        }
    };

    let mut args = input.collect::<Vec<_>>();
    let mut facets_at_startup = true;
    let mut learned_that_at_startup = false;
    if args.contains(&"--f".to_owned()) {
        facets_at_startup = false;
        let i = unsafe { args.iter().position(|x| *x == "--f").unwrap_unchecked() };
        args.remove(i);
    }
    if args.contains(&"--l".to_owned()) {
        learned_that_at_startup = true;
        let i = unsafe { args.iter().position(|x| *x == "--l").unwrap_unchecked() };
        args.remove(i);
    }

    let _ = start_fasb(args, arg, facets_at_startup, learned_that_at_startup);
}
