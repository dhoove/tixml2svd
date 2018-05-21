/// This utility creates
/// [SVD](https://www.keil.com/pack/doc/CMSIS/SVD/html/svd_Format_pg.html)
/// files from the Texas-Instruments XML (called TIXML from now on) device
/// and peripheral descriptor files.

extern crate xml;

use xml::reader;
use xml::writer;
use xml::writer::EmitterConfig;
use std::error::Error;

use std::io;

use std::fs::File;
use std::path::Path;
use std::str::FromStr;

use xml::reader::EventReader;
use xml::name::OwnedName;
use reader::XmlEvent::{StartElement, EndElement};

/// This structure contains arguments used to customize the behavior of tixml2svd.
pub struct Args {
    /// Produce no output other than the SVD data
    silent: bool,
    /// Produce additional output, given 0, 1, 2, etc.
    verbose: u32,
    // Expect a peripheral file instead of a device file.
    peripheral_only: bool,
    // If there are several CPUs, read peripherals from CPU 0, 1, or 2, for example.
    cpunum: u32,
}

impl Args {
    pub fn new(silent: bool, verbose: u32, peripheral_only: bool, cpunum: u32) -> Args {
        let a = Args { silent,
                        verbose,
                        peripheral_only,
                        cpunum,
        };
        a
    }
}

fn write_access<O>(args: &Args, xml_out: &mut xml::EventWriter<&mut O>, ti_access: &str) -> io::Result<()> where
    O: io::Write,
{
    let access = match ti_access {
        "RO" => "read",
        "WO" => "write",
        "RW" => "read-write",
        unknown => {
            if !args.silent {
                eprintln!("Ignoring unknown access key '{}'", unknown);
            }
            return Ok(());
        }
    };

    write_tag(args, xml_out, "access", access)
}

fn write_start<O>(args: &Args, xml_out: &mut xml::EventWriter<&mut O>, element: &str) -> io::Result<()> where
    O: io::Write,
{
    let event: writer::XmlEvent = writer::XmlEvent::start_element(element).into();
    if args.verbose > 2 {
        eprintln!("Writing start-tag: {:?}", event);
    }
    match xml_out.write(event) {
        Ok(x) => Ok(x),
        Err(x) => Err(io::Error::new(io::ErrorKind::Other, x.description())),
    }
}

fn write_content<O>(args: &Args, xml_out: &mut xml::EventWriter<&mut O>, content: &str) -> io::Result<()> where
    O: io::Write,
{
    let event: writer::XmlEvent = writer::XmlEvent::characters(content).into();
    if args.verbose > 2 {
        eprintln!("Writing content: {:?}", event);
    }
    match xml_out.write(event) {
        Ok(x) => Ok(x),
        Err(x) => Err(io::Error::new(io::ErrorKind::Other, x.description())),
    }
}

fn write_end<O>(args: &Args, xml_out: &mut xml::EventWriter<&mut O>) -> io::Result<()> where
    O: io::Write,
{
    let event: writer::XmlEvent = writer::XmlEvent::end_element().into();
    if args.verbose > 2 {
        eprintln!("Writing end-tag: {:?}", event);
    }
    match xml_out.write(event) {
        Ok(x) => Ok(x),
        Err(x) => Err(io::Error::new(io::ErrorKind::Other, x.description())),
    }
}


fn write_tag<O>(args: &Args, xml_out: &mut xml::EventWriter<&mut O>, element: &str, content: &str) -> io::Result<()> where
    O: io::Write,
{
    write_start(args, xml_out, element)?;
    write_content(args, xml_out, content)?;
    write_end(args, xml_out)?;
    Ok(())
}

/// Used by process_device_base to open each peripheral file and
/// provide a xml parser for the file. It only makes sense to replace
/// this if you wish to run this code without file-based storage.
pub fn get_parser_from_filename(root: &str, filename: &str) -> io::Result<xml::EventReader<std::fs::File>> {
    let root_path = Path::new(root);
    let concat_path = root_path.with_file_name(filename);
    let fd_periph = File::open(&concat_path)?;
    Ok(EventReader::new(fd_periph))
}

/// Convert a TIXML device to SVD.
pub fn process_device<I, O>(args: &Args, fin: I, root_path: &str, fout: &mut O) -> io::Result<()> where
    I: io::Read,
    O: io::Write,
{
    let mut xml_out = EmitterConfig::new().perform_indent(true).create_writer(fout);
    let parser = EventReader::new(fin);

    process_device_base(args, parser, &mut xml_out, root_path, &get_parser_from_filename)
}

/// Convert a TIXML device to SVD.
pub fn process_device_base<I, O>(
    args: &Args,
    parser: xml::EventReader<I>,
    mut xml_out: &mut xml::EventWriter<&mut O>,
    root_path: &str,
    fname2parser: &Fn(&str, &str) -> io::Result<xml::EventReader<std::fs::File>>
) -> io::Result<()> where
    I: io::Read,
    O: io::Write,
{
    let mut printed_peripherals_tag = true;
    let mut in_cpu_tag = false;
    let mut cpunum = 0;

    for e in parser {
        match e {
            Ok(StartElement { name, attributes, namespace: _namespace }) => {
                if args.verbose > 0 {
                    eprintln!("Processing StartElement: {}", name);
                }
                let OwnedName { local_name, namespace: _namespace, prefix: _prefix } = name;
                match local_name.as_ref() {
                    "device" => {
                        write_start(args, &mut xml_out, "device")?;
                    },
                    "cpu" => {
                        in_cpu_tag = true;
                        if cpunum != args.cpunum {
                            continue;
                        }
                        printed_peripherals_tag = false;
                    },
                    "instance" => {
                        if !in_cpu_tag | (cpunum != args.cpunum) {
                            continue;
                        }
                        
                        let mut f_baseaddr: Option<String> = None;
                        let mut _f_endaddr: Option<String> = None;
                        let mut f_size: Option<String> = None;
                        let mut f_id: Option<String> = None;
                        let mut f_href: Option<String> = None;
                        
                        for attr in attributes {
                            let xml::attribute::OwnedAttribute { name, value } = attr;
                            let OwnedName { local_name: attr_name, .. } = name;
                            match attr_name.as_ref() {
                                "baseaddr" => if value.len() > 0 { f_baseaddr = Some(value) },
                                "endaddr" => if value.len() > 0 { _f_endaddr = Some(value) },
                                "size" => if value.len() > 0 { f_size = Some(value) },
                                "id" => if value.len() > 0 { f_id = Some(value) },
                                "href" => if value.len() > 0 { f_href = Some(value) },
                                unknown => {
                                    if args.verbose > 0 {
                                        eprintln!("Ignoring unknown key '{}' for '{}'", unknown, local_name);
                                    };
                                },
                            };
                        }
                        
                        if let Some(id) = f_id {
                            // If no ID present, ignore the module (TI-internal?)
                            if id.len() > 0 {
                                if !printed_peripherals_tag {
                                    write_start(args, &mut xml_out, "peripherals")?;
                                    printed_peripherals_tag = true;
                                }
                                
                                write_start(args, &mut xml_out, "peripheral")?;
                                write_tag(args, &mut xml_out, "name", &id)?;

                                if let Some(baseaddr) = f_baseaddr {
                                    write_tag(args, &mut xml_out, "baseAddress", &baseaddr)?;
                                }
                                
                                match f_size {
                                    Some(size) => {
                                        write_start(args, &mut xml_out, "addressBlock")?;
                                        write_tag(args, &mut xml_out, "offset", "0")?;
                                        write_tag(args, &mut xml_out, "size", &size)?;
                                        write_tag(args, &mut xml_out, "usage", "registers")?;
                                        write_end(args, &mut xml_out)?;
                                    },
                                    None => {
                                        if !args.silent {
                                            eprintln!("Peripheral has no size for {}", local_name);
                                        }
                                    }
                                    
                                }
                                
                                if let Some(href) = f_href {
                                    if !args.silent {
                                        eprintln!("Processing peripheral file: {:?}", &href);
                                    }
                                    let parser = fname2parser(root_path, &href)?;
                                    process_peripheral_base(&args, parser, &mut xml_out)?;
                                }

                                write_end(args, &mut xml_out)?;
                            }
                        }
                        
                    },
                    unknown => {
                        if args.verbose > 0 {
                            eprintln!("Ignoring unknown start element key '{}'", unknown);
                        }
                    },
                }
            },

            Ok(EndElement { name }) => {
                if args.verbose > 0 {
                    eprintln!("Processing EndElement: {}", name);
                }
                let OwnedName { local_name, .. } = name;
                match local_name.as_ref() {
                    "device" => {
                        write_end(args, &mut xml_out)?;
                    },
                    "cpu" => {
                        if cpunum == args.cpunum {
                            if printed_peripherals_tag {
                                write_end(args, &mut xml_out)?;
                            }
                            
                            printed_peripherals_tag = true;
                        }
                        
                        in_cpu_tag = false;
                        cpunum += 1;
                    },
                    "instance" => {
                    },
                    unknown => {
                        if args.verbose > 0 {
                            eprintln!("Ignoring unknown end element key '{}'", unknown);
                        }
                    },
                }
            },

            Err(e) => {
                return Err(io::Error::new(io::ErrorKind::Other, e.description()));
            },
            _ => {}
        }
    }
    Ok(())
}

/// Convert a TIXML peripheral to SVD.
pub fn process_peripheral<I, O>(args: &Args, fin: I, fout: &mut O) -> io::Result<()> where
    I: io::Read,
    O: io::Write,
{
    let mut xml_out = EmitterConfig::new().perform_indent(true).create_writer(fout);
    let parser = EventReader::new(fin);

    process_peripheral_base(args, parser, &mut xml_out)
}

/// Convert a TIXML peripheral to SVD.
pub fn process_peripheral_base<I, O>(
    args: &Args,
    parser: xml::EventReader<I>,
    mut xml_out: &mut xml::EventWriter<&mut O>
) -> io::Result<()> where
    I: io::Read,
    O: io::Write,
{
    let mut printed_registers_tag = false;

    let mut printed_fields_tag = false;

    #[allow(non_snake_case)]
    let mut printed_enumeratedValues_tag = false;

    // Temporary storage to check for resetval overflow
    let mut register_width = None;

    let mut register_reset_value = None;

    for e in parser {
        match e {
            Ok(StartElement { name, attributes, namespace: _ }) => {
                if args.verbose > 0 {
                    eprintln!("Processing StartElement: {}", name);
                }
                let OwnedName { local_name, .. } = name;
                match local_name.as_ref() {
                    "module" => {
                        if args.peripheral_only {
                            write_start(args, &mut xml_out, "peripheral")?;
                        }
                        printed_registers_tag = false;
                        for attr in attributes {
                            let xml::attribute::OwnedAttribute { name, value } = attr;
                            let OwnedName { local_name: attr_name, .. }  = name;
                            match attr_name.as_ref() {
                                "HW_revision" => (),
                                "XML_version" => (),
                                "noNamespaceSchemaLocation" => (),
                                "id" => {
                                    if args.peripheral_only {
                                        write_tag(args, &mut xml_out, "name", &value)?;
                                    }
                                },
                                "value" => {
                                    if args.peripheral_only {
                                        write_tag(args, &mut xml_out, "value", &value)?;
                                    }
                                },
                                "token" => (),
                                "description" => { write_tag(args, &mut xml_out, "description", &value)?; },
                                unknown => {
                                    if args.verbose > 0 {
                                        eprintln!("Ignoring unknown key '{}' for '{}'", unknown, local_name);
                                    };
                                },
                            };
                        }
                    },
                    
                    "register" => {
                        if !printed_registers_tag {
                            printed_registers_tag = true;
                            write_start(args, &mut xml_out, "registers")?;
                        }
                        
                        write_start(args, &mut xml_out, "register")?;
                        printed_fields_tag = false;
                        register_reset_value = None;
                        
                        for attr in attributes {
                            let xml::attribute::OwnedAttribute { name, value } = attr;
                            let OwnedName { local_name: attr_name, .. } = name;
                            match attr_name.as_ref() {
                                "id" => { write_tag(args, &mut xml_out, "name", &value)?; },
                                "value" => { write_tag(args, &mut xml_out, "value", &value)?; },
                                "width" => {
                                    let w: u32 = value.parse().unwrap();
                                    register_width = Some(w);
                                    write_tag(args, &mut xml_out, "size", &value)?;
                                },
                                "acronym" => (),
                                "description" => { write_tag(args, &mut xml_out, "description", &value)?; },
                                "rwaccess" => { write_access(args, &mut xml_out, &value)?; },
                                "offset" => { write_tag(args, &mut xml_out, "addressOffset", &value)?; },
                                "resetval" => {
                                    let resetval: u64 = value.parse().unwrap();
                                    register_reset_value = Some(resetval);
                                },
                                unknown => {
                                    if args.verbose > 0 {
                                        eprintln!("Ignoring unknown key '{}' for '{}'", unknown, local_name);
                                    };
                                },
                            };
                        }
                    },
                    
                    "bitfield" => {
                        if !printed_fields_tag {
                            printed_fields_tag = true;
                            write_start(args, &mut xml_out, "fields")?;
                        }
                        
                        write_start(args, &mut xml_out, "field")?;
                        printed_enumeratedValues_tag = false;
                        
                        let mut f_name: Option<String> = None;
                        let mut f_range: Option<String> = None;
                        let mut f_begin: Option<String> = None;
                        let mut f_width: Option<String> = None;
                        let mut f_end: Option<String> = None;
                        let mut f_rwaccess: Option<String> = None;
                        let mut f_description: Option<String> = None;
                        let mut f_reset_value: Option<u64> = None;
                        
                        for attr in attributes {
                            let xml::attribute::OwnedAttribute { name, value } = attr;
                            let OwnedName { local_name: attr_name, .. } = name;
                            match attr_name.as_ref() {
                                "id" => if value.len() > 0 { f_name = Some(value) },
                                "range" => if value.len() > 0 { f_range = Some(value) },
                                "begin" => if value.len() > 0 { f_begin = Some(value) },
                                "width" => if value.len() > 0 { f_width = Some(value) },
                                "end" => if value.len() > 0 { f_end = Some(value) },
                                "rwaccess" => if value.len() > 0 { f_rwaccess = Some(value) },
                                "description" => if value.len() > 0 { f_description = Some(value) },
                                "resetval" => {
                                    let resetval: Result<u64, std::num::ParseIntError>;
                                    if value.starts_with("0x") {
                                        resetval = u64::from_str_radix(&value[2..], 16);
                                    } else {
                                        resetval = u64::from_str(&value);
                                    }
                                    f_reset_value = Some(resetval.unwrap());
                                },
                                unknown => {
                                    if args.verbose > 0 {
                                        eprintln!("Ignoring unknown key '{}' for '{}'", unknown, local_name);
                                    };
                                },
                            };
                        }
                        if let Some(reset_value) = f_reset_value {
                            if let Some(shift) = f_end.clone() {
                                let shift_int = u32::from_str(&shift).unwrap();
                                let reg_width: u32 = register_width.unwrap_or(32);
                                
                                if let Some(width) = f_width.clone() {
                                    let width_int = u32::from_str(&width).unwrap();
                                    if shift_int + width_int > reg_width {
                                        return Err(io::Error::new(io::ErrorKind::Other, format!("Field {:?} with offset {} and width {} too big for register of width {}.", f_name, shift_int, width_int, reg_width)));
                                    }
                                }

                                if shift_int < reg_width {
                                    let overflow = reset_value >> (reg_width - shift_int);
                                    if overflow != 0 {
                                        return Err(io::Error::new(io::ErrorKind::Other, format!("Resetval {} too big for field {:?}.", reset_value, f_name)));
                                    }

                                    let shifted_reset_value = reset_value << shift_int;
                                    if let Some(rrv) = register_reset_value {
                                        register_reset_value = Some(rrv | shifted_reset_value)
                                    } else {
                                        register_reset_value = Some(shifted_reset_value);
                                    }
                                }
                            }
                        }

                        if let Some(name) = f_name {
                            write_tag(args, &mut xml_out, "name", &name)?;
                        }
                        if let Some(description) = f_description {
                            if (f_begin != None) && (f_end != None) {
                                let desc = format!("[{}:{}] {}", f_begin.clone().unwrap(), f_end.clone().unwrap(), description);
                                write_tag(args, &mut xml_out, "description", &desc)?;
                            } else {
                                write_tag(args, &mut xml_out, "description", &description)?;
                            }
                        }
                        if let Some(width) = f_width {
                            write_tag(args, &mut xml_out, "bitWidth", &width)?;
                        }
                        if let Some(end) = f_end {
                            write_tag(args, &mut xml_out, "bitOffset", &end)?;
                        }
                        if let Some(range) = f_range {
                            write_tag(args, &mut xml_out, "bitRange", &range)?;
                        }
                        if let Some(_rwaccess) = f_rwaccess {
                            // NOTE: This is a workaround for svd2rust not handling "read" access.
                            //write_tag(args, &mut xml_out, "{}", process_access(rwaccess.as_ref()));
                        }
                    },
                    
                    "bitenum" => {
                        if !printed_enumeratedValues_tag {
                            printed_enumeratedValues_tag = true;
                            write_start(args, &mut xml_out, "enumeratedValues")?;
                        }
                        
                        write_start(args, &mut xml_out, "enumeratedValue")?;
                        for attr in attributes {
                            let xml::attribute::OwnedAttribute { name, value } = attr;
                            let OwnedName { local_name: attr_name, .. } = name;
                            match attr_name.as_ref() {
                                "id" => { write_tag(args, &mut xml_out, "name", &value)?; },
                                "value" => { write_tag(args, &mut xml_out, "value", &value)?; },
                                "token" => (),
                                "description" => { write_tag(args, &mut xml_out, "description", &value)?; },
                                unknown => {
                                    if args.verbose > 0 {
                                        eprintln!("Ignoring unknown key '{}' for '{}'", unknown, local_name);
                                    };
                                },
                            };
                        }
                    },
                    unknown =>  {
                        if args.verbose > 0 {
                            eprintln!("Ignoring unknown start element key '{}'", unknown);
                        }
                    },
                };
            }
            Ok(EndElement { name }) => {
                if args.verbose > 0 {
                    eprintln!("Processing EndElement: {}", name);
                }
                let OwnedName { local_name, prefix: _, namespace: _ } = name;
                match local_name.as_ref() {
                    
                    "module" => {
                        if printed_registers_tag {
                            printed_registers_tag = false;
                            write_end(args, &mut xml_out)?;
                        }
                        if args.peripheral_only {
                            write_end(args, &mut xml_out)?;
                        }
                    },
                    
                    "register" => {
                        if printed_fields_tag {
                            printed_fields_tag = false;
                            write_end(args, &mut xml_out)?;
                        }
                        
                        if let Some(value) = register_reset_value {
                            let hex_reset = format!("0x{:X}", value);
                            write_tag(args, &mut xml_out, "resetValue", &hex_reset )?;
                        } else {
                            // For svd2rust
                            let rv = "0";
                            write_tag(args, &mut xml_out, "resetValue", &rv )?;
                        }
                        
                        register_width = None;
                        write_end(args, &mut xml_out)?;
                    },
                    
                    "bitfield" => {
                        if printed_enumeratedValues_tag {
                            printed_enumeratedValues_tag = false;
                            write_end(args, &mut xml_out)?;
                        }
                        write_end(args, &mut xml_out)?;
                    },
                    
                    "bitenum" => {
                        write_end(args, &mut xml_out)?;
                    },
                    unknown => {
                        if args.verbose > 0 {
                            eprintln!("Ignoring unknown end element key '{}'", unknown);
                        }
                    },
                };
            }
            Err(e) => {
                return Err(io::Error::new(io::ErrorKind::Other, e.description()));
            }
            _ => {}
        }
    }
    Ok(())
}
