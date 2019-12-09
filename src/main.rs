use std::io::Write;
use std::fs::read_dir;
use actix_multipart::Multipart;
use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use futures::{StreamExt};
use bio::alphabets;
use bio::data_structures::suffix_array::suffix_array;
use bio::data_structures::bwt::{bwt, less, Occ};
use bio::data_structures::fmindex::{FMIndex, FMIndexable};
use bio::io::fasta;
use std::ffi::OsStr;

async fn parse(mut payload: Multipart) -> Result<HttpResponse, Error> {  
    // iterate over multipart stream
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition().unwrap();
        let filename = content_disposition.get_filename().unwrap_or_else(|| "");
        if !filename.is_empty() {
            let content_type = field.content_disposition().unwrap();
            let filename = content_type.get_filename().unwrap();
            let filepath = format!("./tmp/{}", filename);
            // File::create is blocking operation, use threadpool
            let mut f = web::block(|| std::fs::File::create(filepath))
                .await
                .unwrap();
            // Field in turn is stream of *Bytes* object
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                // filesystem operations are blocking, we have to use threadpool
                f = web::block(move || f.write_all(&data).map(|_| f)).await?;
            }
        } else {
            while let Some(chunk) = field.next().await {

                let data = chunk.unwrap();
                let pattern = &std::str::from_utf8(&data).unwrap();
                let text = pattern.as_bytes();

                for entry in read_dir("./tmp")? {
                    let entry = entry?;
                    let path = entry.path();
                    if !path.is_dir() {
                        let ext = path.extension().unwrap();
                        if ext == OsStr::new("fa") {
                            println!("{}", path.to_str().unwrap());

                            // obtain reader or fail with error (via the unwrap method)
                            let reader = fasta::Reader::from_file(&path).unwrap();
                            for result in reader.records() {
                                // obtain record or fail with error
                                let record = result.unwrap();
                                // obtain sequence
                                let seq = record.seq();

                                // Create an FM-Index for the given text.
                                // instantiate an alphabet
                                let alphabet = alphabets::dna::iupac_alphabet();
                                // calculate a suffix array
                                let pos = suffix_array(text);
                                // calculate BWT
                                let bwt = bwt(text, &pos);
                                // calculate less and Occ
                                let less = less(&bwt, &alphabet);
                                let occ = Occ::new(&bwt, 3, &alphabet);
                                // setup FMIndex
                                let fmindex = FMIndex::new(&bwt, &less, &occ);

                                if alphabet.is_word(seq) {
                                    let interval = fmindex.backward_search(seq.iter());
                                    let positions = interval.occ(&pos);
                                    println!("{:?}", positions);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(HttpResponse::Ok().into())
}

fn search() -> HttpResponse {
    let html = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form target="/" method="post" enctype="multipart/form-data">
                <input type="text" name="text" />
                <input type="submit" value="Submit"></button>
            </form>
        </body>
    </html>"#;

    HttpResponse::Ok().body(html)
}


fn upload() -> HttpResponse {
    let html = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form target="/" method="post" enctype="multipart/form-data">
                <input type="file" multiple name="file"/>
                <input type="submit" value="Submit"></button>
            </form>
        </body>
    </html>"#;

    HttpResponse::Ok().body(html)
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    std::fs::create_dir_all("./tmp").unwrap();
    let ip = "0.0.0.0:3000";
    HttpServer::new(|| {
        App::new()
        .wrap(middleware::Logger::default())
        .service(
            web::resource("/upload")
                .route(web::get().to(upload))
                .route(web::post().to(parse)),
        )
        .service(
            web::resource("/search")
                .route(web::get().to(search))
                .route(web::post().to(parse)),
        )
    })
    .bind(ip)?
    .run()
}
