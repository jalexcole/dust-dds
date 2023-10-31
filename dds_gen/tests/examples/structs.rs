#[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
#[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]
pub struct ChessSquare {
    #[dust_dds(key)] pub column: char,
    #[dust_dds(key)] pub line: u16,
}
#[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]
pub struct HelloWorld {
    pub message: String,
    pub id: u32,
}
#[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]
pub struct Sentence {
    pub words: Vec<String>,
    pub dependencies: Vec<[u32; 2]>,
}
#[derive(Debug, dust_dds::topic_definition::type_support::DdsType)]
pub struct User {
    pub name: String,
}