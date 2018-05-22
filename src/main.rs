extern crate xml;
extern crate clap;

extern crate tixml2svd;

use tixml2svd::{Args, process_peripheral, process_device};

use std::fs::File;
use std::io::{Error, ErrorKind};


fn main() {
    ::std::process::exit(match main_() {
       Ok(_) => 0,
       Err(err) => {
           eprintln!("error: {:?}", err);
           1
       }
    });
}


fn main_() -> std::io::Result<()> {
    let matches = clap::App::new("tixml2svd")
        .version("0.1")
        .about("Convert Texas-Instruments device xml data into SVD format.\n\
Remove any byte-order-mark (BOM) from your file before processing.")
        .arg(clap::Arg::with_name("input")
             .short("i")
             .long("input")
             .value_name("FILE")
             .required(true)
             .help("Input xml file"))
        .arg(clap::Arg::with_name("cpunum")
             .short("c")
             .long("cpunum")
             .value_name("INTEGER")
             .help("Select cpu number with an integer, starting with 0"))
        .arg(clap::Arg::with_name("peripheral")
             .short("p")
             .long("peripheral")
             .help("Compile single peripheral file"))
        .arg(clap::Arg::with_name("verbose")
             .short("v")
             .long("verbose")
             .multiple(true)
             .help("Be more verbose"))
        .arg(clap::Arg::with_name("silent")
             .short("s")
             .long("silent")
             .help("Be silent"))
        .get_matches();

    let fname_in = matches.value_of("input").unwrap();

    let requested_cpunum = matches.value_of("cpunum").unwrap_or("0").parse::<u32>()
        .map_err(|_| Error::new(ErrorKind::Other, format!("invalid cpunum, must be a valid non-negative integer.")))?;
    
    let args = Args::new(matches.is_present("silent"),
                           matches.occurrences_of("verbose") as u32,
                           matches.is_present("peripheral"),
                           requested_cpunum);

    if !matches.is_present("silent") {
        eprintln!("Processing file: {}", fname_in);
    }

    //let fd_in = std::io::stdin();
    let fd_in = File::open(fname_in)?;

    //let mut fd_out = File::create(fname_out).unwrap();
    let stdout = std::io::stdout();
    let mut fd_out = stdout.lock();

    if matches.is_present("peripheral") {
        process_peripheral(&args, fd_in, &mut fd_out)
    } else {
        process_device(&args, fd_in, &fname_in, &mut fd_out)
    }
}
