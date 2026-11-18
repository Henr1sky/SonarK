#![allow(dead_code)]

mod config;
mod i18n;
mod project;
mod tui;
mod actions;
mod tshark;
mod presets;
mod history;
mod log;
mod crypto;

use std::path::PathBuf;

use clap::Parser;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const BANNER: &str = r#"                                
                                     Create by: H3nr1sk                                   
          .     t#,     L.                                      G:      
         ;W    ;##W.    EW:        ,ft               j.         E#,    :
        f#E   :#L:WE    E##;       t#E            .. EW,        E#t  .GE
      .E#f   .KG  ,#D   E###t      t#E           ;W, E##j       E#t j#K;
     iWW;    EE    ;#f  E#fE#f     t#E          j##, E###D.     E#GK#f  
    L##Lffi f#.     t#i E#t D#G    t#E         G###, E#jG#W;    E##D.   
   tLLG##L  :#G     GK  E#t  f#E.  t#E       :E####, E#t t##f   E##Wi   
     ,W#i    ;#L   LW.  E#t   t#K: t#E      ;W#DG##, E#t  :K#E: E#jL#D: 
    j#E.      t#f f#:   E#t    ;#W,t#E     j###DW##, E#KDDDD###iE#t ,K#j
  .D#j         f#D#;    E#t     :K#D#E    G##i,,G##, E#f,t#Wi,,,E#t   jD
 ,WK,           G#t     E#t      .E##E  :K#K:   L##, E#t  ;#W:  j#t     
 EG.             t      ..         G#E ;##D.    L##, DWi   ,KK:  ,;     
 ,                                  fE ,,,      .,,                     
                                                                                         
                    ~ ~  E C H O   T H E   P A C K E T  ~ ~ 
"#;

const BANNER_COMPACT: &str = r#"
░█▀▀░█▀█░█▀█░█▀█░█▀▄░█░█
░▀▀█░█░█░█░█░█▀█░█▀▄░█▀▄
░▀▀▀░▀▀▀░▀░▀░▀░▀░▀░▀░▀░▀
"#;

const BANNER_MINIMAL: &str = "◢◤ SONARK ◥◣  ~ Echo the Packet ~";

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Open a local packet capture or text log.
    capture: Option<PathBuf>,

    /// Name of the project to open.
    #[arg(long, default_value = "default")]
    project: String,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    tui::run(VERSION, cli.project, cli.capture.as_deref())?;
    Ok(())
}
