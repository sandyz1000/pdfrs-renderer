// #![feature(plugin)]
// #![plugin(clippy)]

#[macro_use (o, slog_log, slog_trace, slog_debug, slog_info, slog_warn, slog_error)]
extern crate slog;
extern crate slog_json;
#[macro_use]
extern crate slog_scope;
extern crate slog_stream;
extern crate slog_term;
extern crate isatty;
#[macro_use]
extern crate error_chain;

pub mod reader;
pub mod repr;
pub mod err;

// TODO Plan
// * Fix find_page()
// * Display the PDF model for debugging
// * Write back to file - that means keeping track of what has changed


// Is it possible to store it fully as intermediate repr? What are the pros/cons?
//  * takes less space
//  * throws any error only at beginning

// Later there should be an option to read directly from file

#[cfg(test)]
mod tests {
    use reader;
    use reader::PdfReader;
    use reader::lexer::Lexer;
    use repr::*;
    use err::*;

    use std;
    use slog;
    use slog::{DrainExt, Level};
    use {slog_term, slog_stream, isatty, slog_json, slog_scope};

    //#[test]
    fn sequential_read() {
        setup_logger();
        let buf = reader::read_file("edited_example.pdf").chain_err(|| "Cannot read file.").unwrap_or_else(|e| print_err(e));
        let mut lexer = Lexer::new(&buf);
        loop {
            let pos = lexer.get_pos();
            let next = match lexer.next() {
                Ok(next) => next,
                Err(Error (ErrorKind::EOF, _)) => break,
                Err(e) => print_err(e),
            };
            println!("{}\t{}", pos, next.as_string());
        }
        /*
        loop {
            let next = match lexer.back() {
                Ok(next) => next,
                Err(Error (ErrorKind::EOF, _)) => break,
                Err(e) => print_err(e),
            };
            println!("word: {}", next.as_string());
        }
        */
    }

    #[test]
    fn structured_read() {
        setup_logger();
        let reader = PdfReader::new("edited_example.pdf").chain_err(|| "Error creating PdfReader.").unwrap_or_else(|e| print_err(e));
        {
            let val = reader.trailer.dict_get(String::from("Root")).unwrap_or_else(|e| print_err(e));

            match val {
                &Object::Reference{obj_nr: 1, gen_nr: 0} => {},
                _ => error!("Wrong Trailer::Root!"),
            }

        }
        reader.read_indirect_object(3).chain_err(|| "Read ind obj 3").unwrap_or_else(|e| print_err(e));

        let n = reader.get_num_pages();
        let page = reader.get_page_contents(0).chain_err(|| "Get page 0").unwrap_or_else(|e| print_err(e));
    }

    /// Prints the error if it is an Error
    fn print_err<T>(err: Error) -> T {
        println!("\n === \nError: {}", err);
        for e in err.iter().skip(1) {
            println!("  caused by: {}", e);
        }
        println!(" === \n");

        if let Some(backtrace) = err.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }

        println!(" === \n");
        panic!("Exiting");
    }


    fn setup_logger() {
        let logger = if isatty::stderr_isatty() {
            let drain = slog_term::streamer()
                .sync()
                .stderr()
                .full()
                .use_utc_timestamp()
                .build();
            let d = slog::level_filter(Level::Trace, drain);
            slog::Logger::root(d.fuse(), o![])
        } else {
            slog::Logger::root(slog_stream::stream(std::io::stderr(), slog_json::default()).fuse(),
                               o![])
        };
        slog_scope::set_global_logger(logger);
    }
}
