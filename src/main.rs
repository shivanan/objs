use actix_web::{web, App, http,HttpRequest,HttpServer, Responder,HttpResponse};
use std::fs::File;
use std::io::{Write, Error,Read,ErrorKind};
use clap::{Arg,ArgMatches};

async fn create_request(m:Configuration,req:HttpRequest,bytes:web::Bytes) ->  impl Responder {
    let path = req.match_info().get("path").unwrap();
    //let s = m;//Configuration{dbpath:String::from("/tmp/objdata"),peers:vec![]};
    match m.write_object(path,&bytes) {
        Ok(()) =>return HttpResponse::build(http::StatusCode::OK).body(""),
        Err(e) =>{
            return  HttpResponse::build(http::StatusCode::from_u16(400).unwrap()).body(format!("{}",e))
        }
    }
    
}


async fn read_request(m:Configuration,req:HttpRequest) ->  impl Responder {
    let path = req.match_info().get("path").unwrap();
    match m.read_object(path) {
        Ok(bytes) => return HttpResponse::build(http::StatusCode::OK).body(bytes),
        Err(e) => {
            let status = match e.kind() {
                ErrorKind::NotFound => 404,
                _ => 500,
            };
            return HttpResponse::build(http::StatusCode::from_u16(status).unwrap()).body(format!("{}",e))
        }
    }
}

#[derive(Clone)]
struct Configuration {
    dbpath: String,
    peers:Vec<String>,

}
impl Configuration {
    fn write_object(&self,file_name:&str, data:&[u8]) -> Result<(),Error> {
        let path = self.dbpath.to_owned() + file_name;
        let mut fp = File::create(path)?;
        fp.write_all(data)?;
        return Ok(());
    }
    fn read_object(&self,file_name:&str) ->  Result<Vec<u8>,Error> {
        let path = self.dbpath.to_owned() + file_name;
    
        let mut fp = File::open(path)?;
        let mut buffer = Vec::new();
        fp.read_to_end(&mut buffer)?;
       return Ok(buffer);
    }



    async fn start(&self,host:&str) -> std::io::Result<()> {
        
        
        println!("Listening to {}",host);
        let c:Configuration = Configuration{dbpath:self.dbpath.to_owned(),peers:vec![]};
        HttpServer::new(move|| {
            let x1:Configuration = c.to_owned();  
            let x2:Configuration  = c.to_owned(); 
            App::new()
                .route("/public/{path}",web::post().to(move |req:HttpRequest,bytes:web::Bytes|{create_request(x1.to_owned(),req,bytes)}))
                .route("/public/{path}",web::get().to(move |req:HttpRequest|{read_request(x2.to_owned(),req)}))

        })
        .bind(host)?
        .run()
        .await
    }

}

   


fn build_args<'a>()  -> ArgMatches<'a> {  
    clap::App::new("objs")
        .version("1.0")
        .arg(Arg::with_name("host")
            .long("host")
            .takes_value(true)
            .help("IP/name to listen to")
            .required(false)
        )
        .arg(Arg::with_name("peers")
            .long("peers")
            .takes_value(true)
            .help("Comma separated list of peer nodes (ex: 192.168.1.12:2000,192.168.1.14:2001")
            .required(false)
        )
        .arg(Arg::with_name("dbpath")
            .long("dbpath")
            .takes_value(true)
            .help("Directory where data is stored")
            .required(true)
        )
        .get_matches()
        

}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args = build_args();
    let host = args.value_of("host").unwrap_or("127.0.0.1:7000");
    let dbpath = args.value_of("dbpath").unwrap();
    let peers = args.value_of("peers").unwrap_or("");
    let c = Configuration{dbpath:String::from(dbpath),peers:vec![]};
    c.start(host).await

  
}