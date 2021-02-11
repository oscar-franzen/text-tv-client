/*
 * An extremely simple Swedish Television Text TV Client [1,2] for the
 * Linux command line.  Written as a toy project to learn Rust.
 *
 * Compile instructions at:
 * https://github.com/oscar-franzen/text-tv-client
 *
 * For feedback:
 * OF; <p.oscar.franzen@gmail.com>
 *
 * [1] https://sv.wikipedia.org/wiki/Text-TV
 * [2] https://en.wikipedia.org/wiki/Teletext
 */

use std::io::prelude::*;
use std::io::Write;
use std::fs::File;
use std::path::Path;
use std::net::TcpStream;
use std::env;
use std::process;
use std::collections::HashMap;

use rustls;
use webpki;
use regex::Regex;
use getopts::Options;
use home;

const MESSAGE_SIZE : usize  = 1024;
const HOSTNAME: &str = "www.svt.se";
const VERSION: &str = "0.1";
const CACHE_FILE: &str = "topics";

struct Texttv {
    useragent : String,
    conf_dir : String,
    stream : Option<rustls::StreamOwned<rustls::ClientSession, TcpStream>>
}

impl Texttv {
    fn http_connect(&mut self, hostname : &str) { //-> rustls::StreamOwned<rustls::ClientSession, TcpStream> {
	let socket = TcpStream::connect(hostname.to_owned() + ":443").unwrap();
	let mut config = rustls::ClientConfig::new();

	config.root_store.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

	/*
	To write TLS keys, make sure the below environment variable is set
	in the shell:

	export SSLKEYLOGFILE=/home/foobar/sslkeylog.log

	Then uncomment the below line:
	 */
	
	//config.key_log = Arc::new(rustls::KeyLogFile::new());
	
	let arc = std::sync::Arc::new(config);
	let dns_name = webpki::DNSNameRef::try_from_ascii_str(HOSTNAME).unwrap();
	let mut client = rustls::ClientSession::new(&arc, dns_name);
	self.stream = Some(rustls::StreamOwned::new(client, socket));
    }

    fn http_format(&self, page: u32) -> String {
	let req_str = format!(
	    "GET /svttext/web/pages/{}.html HTTP/1.1\r\n\
	     User-Agent: {}\r\n\
	     Host: {}\r\n\
	     Accept-Language: *\r\n\
	     Connection: Keep-Alive\r\n\r\n", page, self.useragent, HOSTNAME);

	req_str
    }

    fn http_get(&mut self, path : u32) {
	let s = self.http_format(path);
	match self.stream.as_mut().unwrap().write(s.as_bytes()) {
	    Err(e) => println!("{:?}", e),
	    _ => ()
	}
    }

    fn http_recv(&mut self) -> String {
	let mut msg : Vec<u8> = Vec::new();  
	
	loop {
	    let mut buf = [0; MESSAGE_SIZE];
	    let nbytes = self.stream.as_mut().unwrap().read(&mut buf).unwrap();
	    let buf_sliced = &buf[0..nbytes];
	    
	    msg.append(&mut buf_sliced.to_vec());

	    if nbytes < MESSAGE_SIZE {
		break;
	    }
	}

	let str = &std::str::from_utf8(&msg).unwrap();

	str.to_string()
    }

    fn parse_topics(&mut self, contents : &str,
		    save : &mut Vec<String>,
		    old : &HashMap<String, u32>) {
	// remove the header
	let i = contents.find("\r\n\r\n").unwrap() + 4;
	let contents = contents.get(i..).unwrap();

	// remove the first tag (causes problems for html_parser::Dom)
	//let i = contents.find(">").unwrap() + 1;
	//let contents = contents.get(i..).unwrap();

	let lines : Vec<&str> = contents.split('\n').collect();
	let re_news = Regex::new("<span class=\"W\">").unwrap();
	let re_parts = Regex::new("<span class=\"W\">(.*)<a href=\"([0-9]+).html\">[0-9]+</a></span>").unwrap();
	let re_html = Regex::new("<.*?>").unwrap();

	println!("\n");
	
	for l in lines {
	    if re_news.is_match(l) {
		for q in re_parts.captures_iter(l) {
		    let title = &q[1];
		    let index = &q[2];

		    // there might be html tags in the headline
		    let title = re_html.replace_all(title, "");
		    let title = title.trim_start();

		    let pr = (*old).get(title);

		    save.push((&title).to_string());

		    if pr == None {
			println!("\u{1b}[92m{} [{}]\u{1b}[0m", title, index);
		    } else {
			println!("{} [{}]", title, index);
		    }
		}
	    }
	}
    }

    fn parse_story(&mut self, contents : &str) {
	// remove the header
	let i = contents.find("\r\n\r\n").unwrap() + 4;
	let contents = contents.get(i..).unwrap();

	let lines : Vec<&str> = contents.split('\n').collect();
	let re_news = Regex::new("<span class=\"W\">").unwrap();
	let re_parts = Regex::new("<span class=\"W\">(.*)</span>$").unwrap();
	let re_html = Regex::new("<.*?>").unwrap();

	println!("\n");
	
	for l in lines {
	    if re_news.is_match(l) {
		for q in re_parts.captures_iter(l) {
		    let text = &q[1];

		    // remove tags
		    let text = re_html.replace_all(text, "");
		    let text = text.trim_start();

		    println!("{}", text);
		}
	    }
	}
    }

    fn write_cache(&mut self, save : &mut Vec<String>) {
	let mut file = File::create(format!("{}/{}",
					    self.conf_dir,
					    CACHE_FILE)).unwrap();
	
	for topic in save {
	    match file.write_all(topic.as_bytes()) {
		Err(e) => println!("{:?}", e),
		_ => ()
	    }
	    
	    match file.write_all("\n".as_bytes()) {
		Err(e) => println!("{:?}", e),
		_ => ()
	    }
	}
    }

    fn load_cache(&mut self, old : &mut HashMap<String, u32>) {
	let c = std::fs::read_to_string(format!("{}/{}",
						self.conf_dir,
						CACHE_FILE)).unwrap();

	for q in c.split("\n") {
	    old.insert(q.to_string(), 1);
	}
    }

    fn show_menu(&mut self) {
	let pages = [101, 102, 103, 104, 105];
	let mut topics = Vec::new();
	let mut old = HashMap::new();

	if Path::new(&format!("{}/{}", self.conf_dir, CACHE_FILE)).exists() {
	    self.load_cache(&mut old);
	}
	
	for page in &pages {
	    self.http_get(*page);
	    let d = self.http_recv();
	    self.parse_topics(&d, &mut topics, &old);
	}

	self.write_cache(&mut topics);
    }

    fn help(&mut self) {
	println!("This is an extremely simple tele text client for Swedish Text TV.");
	println!("Project lives at https://github.com/oscar-franzen/text-tv-client");
	println!("\nversion: {}\n", VERSION);
	println!("Feedback to: <p.oscar.franzen@gmail.com>");
    }

    fn init(&mut self) {
	let s = format!("{}{}",
			home::home_dir().unwrap().display(),
			"/.text_tv_client");
	match std::fs::create_dir(&s) {
	    Err(e) => (),
	    _ => ()
	}
	
	self.conf_dir = s;
    }
}

fn usage() {
    println!("Usage: text_tv_cli [OPTION]\n");
    println!(" -u, --useragent [STRING]  change the useragent");
    println!(" -v, --version             version number");
    println!(" -h, --help                this help");
    println!("\n");
    process::exit(1);
}

fn main() {
    let args : Vec<String> = env::args().collect();
    let mut opts = Options::new();

    opts.optopt("u", "useragent", "", "USERAGENT");
    opts.optflag("h", "help", "");
    opts.optflag("v", "version", "");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    
    let mut tt = Texttv { conf_dir : "".to_string(),
			  useragent: "".to_string(),
			  stream: None };
    
    if matches.opt_present("u") {
        tt.useragent = matches.opt_str("u").unwrap();
    } else {
	tt.useragent = "Text-tv-client, \
		     https://github.com/oscar-franzen/text-tv-client".to_string();
    }

    if matches.opt_present("h") {
	usage();
    }

    if matches.opt_present("v") {
	tt.help();
	process::exit(0);
    }


    tt.init();
    tt.http_connect(HOSTNAME);
    tt.show_menu();

    loop {
	println!("\nGo to page [NUMBER, m for menu, x to exit, h for help]: ");
	let mut inp_page = String::new();
	std::io::stdin().read_line(&mut inp_page).unwrap();
	let inp_page = inp_page.trim_end();

	if inp_page == "x" {
	    process::exit(0);
	}
	else if inp_page == "m" {
	    tt.show_menu();
	    continue;
	}
	else if inp_page == "h" {
	    tt.help();
	    continue;
	}
	
	let inp_page : u32 = inp_page.parse().unwrap();

	println!("Loading {}", inp_page);

	tt.http_get(inp_page);
	let d = tt.http_recv();
	tt.parse_story(&d);
    }
}
