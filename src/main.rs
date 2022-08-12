use electrolyte::reader::IonReader;

pub fn main() {
    let contents = std::fs::read_to_string("iontestdata/placeholder.sprite.ion").unwrap();
    let data = IonReader::read_string(&contents).unwrap();
    println!("{:?}", data);
}