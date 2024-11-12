use core::panic;
use std::io::Write;
use std::cell::RefCell;

thread_local! {static LOGGER: RefCell<Option<IOManager>> = RefCell::new(None);}

// -------------------------------------------- Traits -------------------------------------------- //
#[allow(unused)]
pub trait ExtString: AsRef<str> + Sized {

    fn unwrap_pat(&self, spat: &str, epat: &str) -> Option<&str> {

        let this = self.as_ref();

        if ! ((&this).starts_with(spat) && (&this).ends_with(epat)) {return None;}

        return Some(&this[spat.len()..this.len()-epat.len()]);

    }

    fn send_to_stdout<'a>(self) -> Self {

        return LOGGER.with(|mng| {

            if let Some(mng) = mng.borrow_mut().as_mut() {
                
                return mng.push_stdout(self);
            
            }

            panic!("ExtString::send_to_stdout - No IOManager found in thread local storage.");

        });
    
    }

    fn send_to_stderr<'a>(self) -> Self {
        
        return LOGGER.with(|mng| {

            if let Some(mng) = mng.borrow_mut().as_mut() {
                
                return mng.push_stdout(self);
            
            }

            panic!("ExtString::send_to_stdout - No IOManager found in thread local storage.");

        });

    }

}
pub trait ExtResult<T> {fn unwrap_or_stderr(self) -> T;}
pub trait MapOption<'a,T>: Sized {
    
    fn attempt(self, msg: impl Into<Fail<'a>>) -> Attempt<'a,T> {self.attempt(msg.into())}

}

// -------------------------------------- Structures & Types -------------------------------------- //

type STR<'a> = std::borrow::Cow<'a, str>;
pub type Attempt<'a,T> = Result<T, Fail<'a>>;

pub struct Fail<'a> {places: Vec<failure::ErrDetails<'a>>, msg: STR<'a>}
pub struct JSON {root: serde_json::Value}
pub struct Connection {stream: std::net::TcpStream}
pub struct IOManager {stderr: std::fs::File, stdout: std::fs::File}

// ------------------------------------------- Functions ------------------------------------------- //

#[track_caller]
pub fn fail<'a,T>(msg: impl Into<std::borrow::Cow<'a,str>>) -> Attempt<'a,T> {
    
    return Err(Fail::new(std::panic::Location::caller(), msg));

}

// -------------------------------------------- Modules -------------------------------------------- //

mod extend_string {

    use crate::tools::ExtString;

    impl<T> ExtString for T where T: AsRef<str> {}

}

mod json_io {

    use crate::tools::*;
    use serde_json::Value;
    use base64::{Engine as _, engine::general_purpose::STANDARD};


    impl JSON {

        pub fn new<'a>() -> Attempt<'a,Self> {

            let raw = match std::env::args().nth(1) {

                Some(raw) => raw,
                None => return fail("No JSON provided."),

            };

            // Decode base64 to string
            let json_str = match STANDARD.decode(&raw) {

                Ok(bytes) => String::from_utf8(bytes)?,

                _ => raw, // If it's not base64, just use the raw string

            };

            Ok(JSON {root: match serde_json::from_str(&json_str) {

                Ok(val) => val,
                Err(err) => return fail(format!("{:?}. \n\tInput: {}", err, json_str)),

            }})

        }

        pub fn get<'a,T>(&'a self, keys: &[&str]) -> Attempt<'a,T> where Value: Convert<'a,T> {

            match self.root.get(keys[0]) {

                Some(val) => {

                    if keys.len() == 1 {return val.make();}

                    let slice = &keys[1..keys.len()];

                    return self.get(slice);

                }

                None => return fail(format!("JSON::get - Key not found: {:?}", keys)),

            }
        
        }

        pub fn get_or<'a,T>(&'a self, keys: &[&str], default: T) -> T where Value: Convert<'a,T> {self.get(keys).unwrap_or(default)}

    }

    impl std::ops::Deref for JSON {type Target = Value; fn deref(&self) -> &Value {&self.root}}

    pub trait Convert<'a,T> {fn make(&'a self) -> Attempt<'a,T>;}

    impl<'a> Convert<'a,&'a str> for Value {
        
        fn make(&'a self) -> Attempt<'a,&str> {
        
            self.as_str().attempt(format!("json_io::Convert - Value is not a string: {:?}", self))
    
        }

    }

    impl Convert<'_,u16> for Value {
        
        fn make(&self) -> Attempt<'_,u16> {
        
            self.as_u64().map(|x| x as u16).attempt(format!("json_io::Convert - Value is not a u16: {:?}", self))
    
        }

    }

    impl Convert<'_,u64> for Value {
        
        fn make(&self) -> Attempt<'_,u64> {
        
            self.as_u64().attempt(format!("json_io::Convert - Value is not a u64: {:?}", self))
    
        }

    }

}

mod failure {

    use crate::tools::{Fail, Attempt, STR};

    pub trait ExtLocation {fn as_place<'a>(&'a self) -> ErrDetails<'a>;}
    pub struct ErrDetails<'a> {file: STR<'a>, line: u32, function: String}
    
    #[allow(unused)]
    pub mod fail {

        use crate::ExtString;
        use super::*;
        use core::panic;
        use std::panic::Location;
        use std::fmt::Debug;

        impl<'a> Fail<'a> {

            pub fn new(place: &'a Location<'a>, msg: impl Into<STR<'a>>) -> Self {

                return Fail {places: vec![place.as_place()], msg: msg.into()};

            }

            #[track_caller] 
            pub fn panic<T>(msg: impl Into<STR<'a>>) -> T {

                <Attempt<'a,T>>::Err(Fail::new(std::panic::Location::caller(), msg)).unwrap();

                unreachable!("tools::panic - Unreachable code reached.");

            }

            pub fn from_debug(place: &'a Location<'a>, err: impl Debug) -> Self {

                return Fail {places: vec![place.as_place()], msg: format!("{:?}", err).into()};

            }

            pub fn show(&self) -> String {

                let len = self.places.len();

                if len == 0 {panic!("Fail::show - No places in Fail struct");}

                let detail = &self.places[0];

                let mut out = format!("\n\tError: {}", self.msg);

                for i in 0..len {
                    
                    out += &format!("\n\t{} - {}, {}, {}\n", 
                
                        i, self.places[i].file, self.places[i].line, self.places[i].function

                    );
                }

                return out.send_to_stderr();

            }
    
        }

        impl std::fmt::Debug for Fail<'_> {fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {write!(f, "{}", self.show())}}
        impl std::fmt::Display for Fail<'_> {fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {write!(f, "{}", self.show())}}

        impl<'a> From<&'a str> for Fail<'a> {#[track_caller] fn from(msg: &'a str) -> Self {Fail::new(Location::caller(), msg)}}
        impl From<String> for Fail<'_> {#[track_caller] fn from(msg: String) -> Self {Fail::new(Location::caller(), msg)}}

        impl From<std::net::AddrParseError> for Fail<'_> {#[track_caller] fn from(err: std::net::AddrParseError) -> Self {Fail::from_debug(Location::caller(), err)}}

        impl From<std::io::Error> for Fail<'_> {#[track_caller] fn from(err: std::io::Error) -> Self {Fail::from_debug(Location::caller(), err)}}

        impl From<serde_json::Error> for Fail<'_> {#[track_caller] fn from(err: serde_json::Error) -> Self {Fail::from_debug(Location::caller(), err)}}

        impl From<std::string::FromUtf8Error> for Fail<'_> {#[track_caller] fn from(err: std::string::FromUtf8Error) -> Self {Fail::from_debug(Location::caller(), err)}}

    }

    mod caller {

        use super::{ExtLocation, ErrDetails, STR};

        use std::collections::HashMap;
        use std::cell::RefCell;
        use std::panic::Location;
        use Type::*;

        const SRC: [(&str,&str); 2] = [
            ("src/tools.rs", include_str!("tools.rs")),
            ("src/main.rs", include_str!("main.rs")),
        ];

        // The path is the file name
        // The map is a hashmap of line numbers to function names and their kind
        
        
        enum Type {Fn, Impl, Trait, Mod}
        struct CallMap<'a> {map: HashMap<u32, (&'a str, Type)>}

        thread_local! {static FILE_CACHE: RefCell<HashMap<STR<'static>, CallMap<'static>>> = RefCell::new(HashMap::new());}

        impl ExtLocation for Location<'_> {
            
            fn as_place<'a>(&'a self) -> ErrDetails<'a> {

                let file: STR = self.file().to_string().into();
                let line: u32 = self.line();
                let mut function = String::new();

                FILE_CACHE.with(|cache| {
    
                    let mut cache = cache.borrow_mut();
        
                    let map: &CallMap<'_> = match cache.get(file.as_ref()) {
        
                        Some(file) => file,
        
                        None => {

                            let src = SRC.iter()
                                .find(|(name, _)| *name == file)
                                .map(|(_, src)| src)
                                .expect(&format!("caller::Location::as_place - Failed to find file: {}", &file));
        
                            cache.insert(file.clone(), CallMap::new(src));

                            cache.get(&file)
                                .expect(&format!("caller::Location::as_place - Failed to get inserted file back out. File: {}", &file))

                        }
        
                    };
        
                    let mut l: u32 = line;
                    let mut func_name: Option<&str> = None;
                    let mut impl_trait_name: Option<&str> = None;
                    let mut mod_name: Option<&str> = None;
    
                    while let Some((name, kind)) = next(&map, &mut l) {
    
                        if l > 0 {l -= 1;} else {break;}

                        if matches!(kind, Fn) && func_name.is_none() {func_name = Some(name); continue;}

                        if (matches!(kind, Trait) || matches!(kind, Impl)) && impl_trait_name.is_none() {
                            
                            impl_trait_name = Some(name); continue;
                        
                        }

                        if matches!(kind, Mod) {mod_name = Some(name); break;}
    
                    }
    
                    if let Some(mod_name) = mod_name {function += &format!("{}::", mod_name);}
    
                    if let Some(impl_trait_name) = impl_trait_name {function += &format!("{}::", impl_trait_name);}
    
                    if let Some(total) = func_name {function += total;}
    
                });

                return ErrDetails {file, line, function};
    
            }
        
        }

        fn next<'a>(map: &'a CallMap, i: &mut u32) -> Option<&'a (&'a str, Type)> {
    
            while map.get(&i).is_none() && *i > 0 {*i -= 1;} return map.get(&i);
        
        }
        
        mod caller_mappings {

            use super::{CallMap, Type, Type::*};
            use std::collections::HashMap;

            impl<'a> CallMap<'a> {

                pub fn new(src: &'a str) -> Self {
    
                    let mut map: HashMap<u32, (&str, Type)> = HashMap::new();
    
                    for (i, line) in src.split('\n').enumerate() {
    
                        let num = i as u32 + 1; // Line numbers start at 1
            
                        if let Some((_, closest)) = Type::find_closest(&line) {
            
                            match closest {

                                Fn => {extract_fn(&line).map(|s| map.insert(num, (s, Type::Fn)));},
                                Impl => {extract_impl(&line).map(|s| map.insert(num, (s, Type::Impl)));},
                                Trait => {extract_trait(&line).map(|s| map.insert(num, (s, Type::Trait)));},
                                Mod => {extract_mod(&line).map(|s| map.insert(num, (s, Type::Mod)));},
            
                            }
                        }
    
                    }
    
                    return CallMap {map};
    
                }
    
            }

            impl<'a> std::ops::Deref for CallMap<'a> {
                
                type Target = HashMap<u32, (&'a str, Type)>; 
                
                fn deref(&self) -> &HashMap<u32, (&'a str, Type)> {&self.map}
            
            }

            fn extract_impl(line: &str) -> Option<&str> {

                if !(line.contains(Impl.as_str()) && line.contains("{")) {return None;}
        
                let line = line.split(Impl.as_str()).nth(1)?;
        
                let start = line.find(">")
                    .or_else(|| Some(0))?;
        
                let end = closet_str(&line, ["for", "{"])?.0;
        
                if start >= end || end >= line.len() {return None;}
        
                Some(line[start..end].trim())
        
            }
        
            fn extract_trait(line: &str) -> Option<&str> {
        
                if !(line.contains(Trait.as_str()) && line.contains("{")) {return None;}
        
                let line = line.split(Trait.as_str()).nth(1)?;

                let end = closet_str(&line, ["<", "{", ":"])?.0;
        
                Some(line[..end].trim())
        
            }
            
            fn extract_mod(line: &str) -> Option<&str> {
        
                if !line.contains(Mod.as_str()) {return None;}
        
                let line = line.split(Mod.as_str()).nth(1)?;
        
                let end = line.find("{")?;
        
                Some(line[..end].trim())
        
            }
        
            fn extract_fn(line: &str) -> Option<&str> {
        
                if !line.contains(Fn.as_str()) {return None;}
        
                let line = line.split(Fn.as_str()).nth(1)?;

                let end = closet_str(line, ["<", "("])?.0;
            
                Some(line[..end].trim())
            
            }

            fn closet_str<'a,const N: usize>(line: &'a str, ary: [&'a str; N]) -> Option<(usize, &'a str)> {

                let mut best: Option<(usize, &str)> = None;
        
                for string in ary.iter() {
        
                    match (best, line.find(string)) {
        
                        (Some((min, _)), Some(val)) => {
        
                            if val < min {best = Some((val, string))}
                        
                        },
        
                        (None, Some(val)) => {best = Some((val, string))},
        
                        (_, _) => {},
        
                    }
                }
        
                return best;
            
            }
            
        }

        mod code_types {

            use super::{Type, Type::*};

            const FN: &str = "fn ";
            const IMPL: &str = "impl ";
            const TRAIT: &str = "trait ";
            const MOD: &str = "mod ";

            const ORDERED: [Type; 4] = [Fn, Impl, Trait, Mod];

            impl Type {

                pub const fn as_str<'a>(&self) -> &'a str {

                    match self {

                        Type::Fn => &FN,
                        Type::Impl => &IMPL,
                        Type::Trait => &TRAIT,
                        Type::Mod => &MOD,

                    }

                }

                pub fn find_closest<'a>(line: &'a str) -> Option<(usize, Type)> {

                    let mut best: Option<(usize, Type)> = None;
            
                    for ftype in ORDERED.iter() {
            
                        match (best, line.find(ftype.as_str())) {
            
                            (Some((min, _)), Some(val)) => {
            
                                if val < min {best = Some((val, *ftype))}
                            
                            },
            
                            (None, Some(val)) => {best = Some((val, *ftype))},
            
                            (_, _) => {},
            
                        }
                    }
            
                    return best;
                
                }

            }

            impl Copy for Type {}
            impl Clone for Type {fn clone(&self) -> Self {return *self;}}

            impl std::ops::Deref for Type {type Target = str; fn deref(&self) -> &str {self.as_str()}}

        }

    }

    mod extend {

        use super::*;
        use crate::tools::{MapOption, ExtResult, ExtString};

        impl<'a,T> ExtResult<T> for Attempt<'a,T> {

            fn unwrap_or_stderr(self) -> T {

                self.unwrap_or_else(|e| {e.show().send_to_stderr(); std::process::exit(1);})

            }

        }

        impl<'a,T> MapOption<'a,T> for Option<T> {}

    }

}

mod connection {

    use crate::tools::*;
    use std::net::{SocketAddr, TcpStream, IpAddr};

    impl Connection {

        pub fn new<'a>(ip: &'a str, port: u16, timeout: u64) -> Attempt<'a,Self> {
        
            use rayon::prelude::{IntoParallelIterator, ParallelIterator};
        
            const BASE_IP: &str = "192.168.1.";
            const MAX_IP: u8 = 254;
        
            // Try to connect to the given IP address
            if let Ok(stream) = try_connect(ip, port, timeout) {return Ok(Connection {stream});}
        
            // Search all possible IP addresses in the LAN
            let tcp = (1..=MAX_IP).into_par_iter()
                .map(|i| {
        
                    try_connect(format!("{}{}", BASE_IP, i), port, timeout)
        
                })
                .find_any(|stream| stream.is_ok())
                .unwrap_or_else(|| {
                    
                    fail(format!("No reachable server found for LAN at address: 192.168.1.X:{}", port))
                
                })?;

            return Ok(Connection {stream: tcp});
        
            fn try_connect<'a>(ip: impl AsRef<str>, port: u16, timeout: u64) -> Attempt<'a,TcpStream> {
        
                return Ok(TcpStream::connect_timeout(
                    &SocketAddr::new(ip.as_ref().parse::<IpAddr>()?, port),
                    std::time::Duration::from_millis(timeout)
                )?);
        
            }
        
        }

    }

    impl std::ops::Deref for Connection {type Target = TcpStream; fn deref(&self) -> &TcpStream {&self.stream}}

}

mod io_manager {

    use super::{IOManager, Attempt, ExtString};
    use std::fs::OpenOptions;
    use std::path::Path;
    use fs2::FileExt;
    use std::io::Write;
    use std::time::SystemTime;
    use std::process;

    const LOG: &str = "StdErr.log";
    const OUT: &str = "StdOut.log";

    impl IOManager {

        pub fn new<'a>(root_folder: &str) -> Self {

            let err = OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(Path::new(root_folder).join(LOG))
                .unwrap_or_else(|err| {

                    println!("IOManager::new - Failed to access Error Log. {:?}", err);

                    std::process::exit(1);

                });

            let out = OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(Path::new(root_folder).join(OUT))
                .unwrap_or_else(|err| {

                    println!("IOManager::new - Failed to access Output Log. {:?}", err);

                    std::process::exit(1);

                });

            err.lock_exclusive().expect("IOManager::new - Failed to lock Error Log.");
            out.lock_exclusive().expect("IOManager::new - Failed to lock Output Log.");

            return IOManager {stderr: err, stdout: out};
        
        }

        fn format_log<'a>(content: &str) -> String {

            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            return format!("\nTime: {}, PID: {}{}", timestamp, process::id(), content);

        }

        pub fn push_stdout<'a,T: ExtString>(&mut self, content: T) -> T {

            writeln!(self.stdout, "{}", Self::format_log(content.as_ref())).unwrap();
            self.stdout.sync_all().unwrap();  return content;

        }

        pub fn push_stderr<'a,T: ExtString>(&mut self, content: T) -> T {

            writeln!(self.stderr, "{}", Self::format_log(content.as_ref())).unwrap();
            self.stderr.sync_all().unwrap();  return content;

        }

    }

    // Automatically release the lock when LockedFile is dropped
    impl Drop for IOManager {

        fn drop(&mut self) {

            let _ = self.stderr.unlock();
            let _ = self.stdout.unlock();

        }
    
    }

}

mod old {

    // mod python;

    // use pyo3::{Python, PyResult, PyObject, types::PyModule};
    // use util::{*, fallible::Attempt};
    // use python::*;

    // const START: &str = "www.acnc.gov.au/charity/charities";

    // const URLS: [&str; 1] = [
        
    //     "https://www.acnc.gov.au/charity/charities/8b53c1d4-39af-e811-a961-000d3ad24182/profile",

    // ];

    // fn main() {

    //     let json: Attempt<String> = Python::with_gil(|py| {

    //         // Create a Python module from our SQL script
    //         let scraper_module = PyModule::from_code(py, SCRAPPER, "scraper.py", "scraper")?;

    //         let get_dataset = scraper_module.getattr("get_dataset_resources")?;

    //         let json = get_dataset.call1((SQL,))?.extract::<String>()?;

    //         return Ok(json);

    //     });

    //     json.unwrap().print();

    //     // let html_data: Attempt<T2Vec<String, Option<i32>>> = Python::with_gil(|py| {

    //     //     // Create a Python module from our SCRAPPER script
    //     //     let scraper_module = PyModule::from_code(py, SCRAPPER.get(), "scraper.py", "scraper")?;

    //     //     let mut html_data: T2Vec<String, Option<i32>> = T2Vec::new();

    //     //     for url in URLS.iter() {

    //     //         // Get HTML content
    //     //         let get_html = scraper_module.getattr("get_html")?;
    //     //         let result: (String, Option<i32>) = get_html.call1((*url,))?.extract()?;

    //     //         html_data.push(result);

    //     //     }

    //     //     return Ok(html_data);

    //     // });

    //     // html_data.unwrap().print_vec();

    // }

}