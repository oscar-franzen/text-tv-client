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
use std::net::TcpStream;
use std::sync::Arc;
use std::process;
use std::collections::HashMap;

use rustls;
use webpki;
use regex::Regex;

const MESSAGE_SIZE : usize  = 1024;
const HOSTNAME: &str = "www.svt.se";
//const HOSTNAME: &str = "www.oscarfranzen.com";

fn http_format(page: u32) -> String {
    let req_str = format!(
	"GET /svttext/web/pages/{}.html HTTP/1.1\r\n\
	 User-Agent: Mozilla/4.0 (compatible; MSIE5.01; Windows NT)\r\n\
	 Host: {}\r\n\
	 Accept-Language: *\r\n\
	 Connection: Keep-Alive\r\n\r\n", page, HOSTNAME);

    req_str
}

fn http_connect() -> Result<rustls::StreamOwned<rustls::ClientSession, TcpStream>, String> {
    let mut socket = TcpStream::connect(HOSTNAME.to_owned() + ":443").unwrap();
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
    let mut stream = rustls::StreamOwned::new(client, socket);

    Ok(stream)
}

fn http_get(stream : &mut rustls::StreamOwned<rustls::ClientSession, TcpStream>,
	    path : u32) {
    stream.write(&http_format(path).as_bytes());
}

fn http_recv(stream : &mut rustls::StreamOwned<rustls::ClientSession,
					       TcpStream>) -> String {

    let mut msg : Vec<u8> = Vec::new();  
    
    loop {
	let mut buf = [0; MESSAGE_SIZE];
	let nbytes = stream.read(&mut buf).unwrap();
	let buf_sliced = &buf[0..nbytes];
	
	msg.append(&mut buf_sliced.to_vec());

	if nbytes < MESSAGE_SIZE {
	    break;
	}
    }

    let str = &std::str::from_utf8(&msg).unwrap();

    //let mut file = std::fs::File::create("data.txt").expect("create failed");
    //file.write_all(str.as_bytes()).expect("write failed");

    str.to_string()
}

fn parse_topics(contents : &str) {
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

		println!("{} [{}]", title, index);
	    }
	}
    }
}

fn parse_story(contents : &str) {
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

fn show_menu(stream : &mut rustls::StreamOwned<rustls::ClientSession, TcpStream>) {
    let pages = [101, 102, 103, 104, 105];
    for page in &pages {
	http_get(stream, *page);
	let d = http_recv(stream);
	parse_topics(&d);
    }
}

fn help() {
    println!("This is an extremely simple tele text client for Swedish Text TV.");
    println!("Project lives at https://github.com/oscar-franzen/text-tv-client");
    println!("Feedback to: <p.oscar.franzen@gmail.com>");
}

fn main() {
    // let mut fh = std::fs::File::open("data.txt").unwrap();
    // let mut d = String::new();
    // fh.read_to_string(&mut d).unwrap();
    //process::exit(0);
    
    let mut stream = http_connect().unwrap();
    show_menu(&mut stream);

    loop {
	println!("\nGo to page [NUMBER, m for menu, x to exit, h for help]: ");
	let mut inp_page = String::new();
 	let b1 = std::io::stdin().read_line(&mut inp_page).unwrap();
	let inp_page = inp_page.trim_end();

	if inp_page == "x" {
	    process::exit(0);
	}
	else if inp_page == "m" {
	    show_menu(&mut stream);
	    continue;
	}
	else if inp_page == "h" {
	    help();
	    continue;
	}
	
	let inp_page : u32 = inp_page.parse().unwrap();

	println!("Loading {}", inp_page);

	http_get(&mut stream, inp_page);
	let d = http_recv(&mut stream);
	parse_story(&d);
    }
}
