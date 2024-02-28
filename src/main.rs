use std::env;
use man_prod::elf_analysis;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <elf_file_path>", args[0]);
        return;
    }
    let elf_file_path = &args[1];

    //TODO: acquire the names of the APIs provided by the developer through the JSON manifest and encapsulate them in this vector of strings.
    let api_list = vec![
        "writeOnDrive", 
        "turnLampOn", 
        "accessAudioDriver", 
        "turnLampOff", 
        "accessNetwork", 
        "accessWebcam", 
        "write_on_drive",
        "access_network",
        "access_webcam",
     ];

    match elf_analysis(elf_file_path, api_list) {
        Ok(_) => println!("Analysis performed successfully!"),
        Err(error) => eprintln!("Elf analysis failed: {}", error)
    };

}
