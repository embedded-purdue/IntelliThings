#[derive(Debug)]
struct sensor_response([u8; 6]);

impl sensor_response {
    fn convert(&self) -> formatted_data {}
}

struct formatted_data {
    temperature: f32,
    humidity: f32
}

impl fmt::Debug for formatted_data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Temp: {}, Humidity: {}", self.temperature, self.humidity);
    }
}

pub async fn read() -> formatted_data {
    const ADDRESS: u8 = 0x44;

    let mut response: sensor_response;

    // actually do reading

    // Debugging
    println!("{}", response);

    response.convert()
}